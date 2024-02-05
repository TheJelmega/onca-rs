use core::{
    mem,
    slice,
    ffi::c_void,
    num::NonZeroU32,
    ptr::null_mut,
};
use onca_common::{
    prelude::*,
    time::Duration,
};
use onca_logging::{log_warning, log_error};
use windows::{
    Win32::{
        Devices::HumanInterfaceDevice::*,
        Foundation::{HANDLE, GetLastError, CloseHandle, BOOL, ERROR_IO_PENDING, WAIT_OBJECT_0, BOOLEAN},
        Storage::FileSystem::{CreateFileA, ReadFile, WriteFile, FILE_FLAGS_AND_ATTRIBUTES, FILE_GENERIC_READ, FILE_GENERIC_WRITE, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING},
        System::{
            Threading::{CreateEventA, WaitForSingleObject}, IO::{OVERLAPPED, GetOverlappedResult, CancelIoEx},
        },
    },
    core::{HRESULT, PCSTR},
};

use crate::*;

pub struct OSDevice;

impl core::fmt::Debug for OSDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OSDevice").finish()
    }
}

//------------------------------------------------------------------------------------------------------------------------------
// DEVICE_CREATION
//------------------------------------------------------------------------------------------------------------------------------

pub fn open_device(path: &str) -> Option<DeviceHandle> {
    scoped_alloc!(AllocId::TlsTemp);
    let mut path = String::from(path);
    path.null_terminate();
    
    let handle = unsafe { CreateFileA(
        PCSTR(path.as_ptr()),
        (FILE_GENERIC_READ | FILE_GENERIC_WRITE).0,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        None,
        OPEN_EXISTING,
        FILE_FLAGS_AND_ATTRIBUTES(0),
        HANDLE::default()
    )};
    match handle {
        Ok(handle) => Some(DeviceHandle(handle.0 as usize)),
        Err(err) => {
            log_error!(LOG_HID_CAT, open_device, "Failed to open the HID device `{}`. ({})", path, err);
            None
        },
    }
}

pub fn close_handle(handle: DeviceHandle) {
    if let Err(err) = unsafe { CloseHandle(HANDLE(handle.0 as isize)) } {
        log_error!(LOG_HID_CAT, close_handle, "Failed to close the HID device. ({err})");
    }
}

pub fn create_os_device(_handle: &DeviceHandle) -> Option<OSDevice> {
    Some(OSDevice)
}

pub fn destroy_os_device(os_dev: &mut OSDevice) {
}

pub fn get_preparse_data(handle: DeviceHandle) -> Option<PreparseData> {
    let handle = HANDLE(handle.0 as isize);

    let mut preparse_data = unsafe { mem::zeroed() };
    let res = unsafe { HidD_GetPreparsedData(handle, &mut preparse_data) }.as_bool();
    if res {
       Some(PreparseData(PreparseDataInternal::Address(preparse_data.0 as usize))) 
    } else {
        if let Err(err) = unsafe { GetLastError() } {
            log_error!(LOG_HID_CAT, get_preparse_data, "Failed to retrieve preparse data. (error: {err})");
        }
        None
    }
}

pub fn free_preparse_data(preparse_data: &mut PreparseData) {
    if let PreparseDataInternal::Address(addr) = preparse_data.0 {
        let preparse_data = addr as isize;
        let res = unsafe { HidD_FreePreparsedData(PHIDP_PREPARSED_DATA(preparse_data as isize)) }.as_bool();
        if !res {
            if let Err(err) = unsafe { GetLastError() } {
                log_error!(LOG_HID_CAT, get_preparse_data, "Failed to free preparse data. (error: {err})");
            }
        }
    }
}

//------------------------------------------------------------------------------------------------------------------------------
// DEVICE
//------------------------------------------------------------------------------------------------------------------------------

pub fn get_identifier(handle: DeviceHandle, preparse_data: &PreparseData) -> Option<Identifier> {
    let handle = HANDLE(handle.0 as isize);

    let mut attribs = HIDD_ATTRIBUTES::default();
    attribs.Size = mem::size_of::<HIDD_ATTRIBUTES>() as u32;
    let res = unsafe { HidD_GetAttributes(handle, &mut attribs) }.as_bool();
    if !res {
        if let Err(err) = unsafe { GetLastError() } { 
            log_error!(LOG_HID_CAT, get_identifier, "Failed to retieve hid attributes. ({err})");
        }
        return None;
    }

    let mut caps = HIDP_CAPS::default();
    let res = unsafe { HidP_GetCaps(PHIDP_PREPARSED_DATA(preparse_data.get_address() as isize), &mut caps) }.ok();
    if let Err(err) = res {
        log_error!(LOG_HID_CAT, get_identifier, "Failed to retrieve hid usage page and usage. ({err})");
            return None;
    }

    Some(Identifier {
        vendor_device: VendorProduct::from_u16(attribs.VendorID,attribs.ProductID),
        version: attribs.VersionNumber,
        usage: Usage::from_u16(caps.UsagePage, caps.Usage),
    })
}

pub fn get_vendor_string(handle: DeviceHandle) -> Option<String> {
    let handle = HANDLE(handle.0 as isize);
    // +1 for null terminator
    let mut buf = [0u16; MAX_HID_STRING_LEN + 1];

    let res = unsafe { HidD_GetManufacturerString(handle, &mut buf as *mut _ as *mut c_void, MAX_HID_STRING_LEN as u32) }.as_bool();
    if res {
        Some(String::from_null_terminated_utf16_lossy(&buf))
    } else {
        if let Err(err) = unsafe { GetLastError() } { 
            log_warning!(LOG_HID_CAT, "Failed to retrieve hid vendor string. ({err})");
        }
        None
    }
}

pub fn get_product_string(handle: DeviceHandle) -> Option<String> {
    let handle = HANDLE(handle.0 as isize);
    // +1 for null terminator
    let mut buf = [0u16; MAX_HID_STRING_LEN + 1];
    let res = unsafe { HidD_GetProductString(handle, &mut buf as *mut _ as *mut c_void, MAX_HID_STRING_LEN as u32) }.as_bool();
    if res {
        Some(String::from_null_terminated_utf16_lossy(&buf))
    } else {
        if let Err(err) = unsafe { GetLastError() } { 
            log_warning!(LOG_HID_CAT, "Failed to retrieve hid vendor string. ({err})");
        }
        None
    }
}

pub fn get_serial_number_string(handle: DeviceHandle) -> Option<String> {
        let handle = HANDLE(handle.0 as isize);
        // +1 for null terminator
        let mut buf = [0u16; MAX_HID_STRING_LEN + 1];
        let res = unsafe { HidD_GetSerialNumberString(handle, &mut buf as *mut _ as *mut c_void, MAX_HID_STRING_LEN as u32) }.as_bool();
        if res {
            Some(String::from_null_terminated_utf16_lossy(&buf))
        } else {
            if let Err(err) = unsafe { GetLastError() } { 
                log_warning!(LOG_HID_CAT, "Failed to retrieve hid vendor string. ({err})");
            }
            None
        }
}

pub fn get_indexed_string(handle: DeviceHandle, index: usize) -> Option<String> {
    let handle = HANDLE(handle.0 as isize);
    // +1 for null terminator
    let mut buf = [0u16; MAX_HID_STRING_LEN + 1];

    let res = unsafe { HidD_GetIndexedString(handle, index as u32, &mut buf as *mut _ as *mut c_void, MAX_HID_STRING_LEN as u32) }.as_bool();
    if res {
        let null_term_idx = buf.iter().position(|&c| c == 0).unwrap_or(MAX_HID_STRING_LEN);
        let str_slice = unsafe { slice::from_raw_parts(buf.as_ptr(), null_term_idx) };
        Some(String::from_utf16_lossy(str_slice))
    } else {
        if let Err(err) = unsafe { GetLastError() } { 
            log_warning!(LOG_HID_CAT, "Failed to retrieve indexed string. ({err})");
        }
        None
    }
}

pub fn get_num_input_buffers(handle: DeviceHandle) -> Option<NonZeroU32> {
    let handle = HANDLE(handle.0 as isize);

    let mut num_buffers = 0;
    let res = unsafe { HidD_GetNumInputBuffers(handle, &mut num_buffers) }.as_bool();
    if res {
        NonZeroU32::new(num_buffers)
    } else {
        if let Err(err) = unsafe { GetLastError() } { 
            log_warning!(LOG_HID_CAT, "Failed to retrieve number of input buffers. ({err})");
        }
        None
    }
}

pub fn set_num_input_buffers(handle: DeviceHandle, num_buffers: u32) {
    let handle = HANDLE(handle.0 as isize);

    let res = unsafe { HidD_SetNumInputBuffers(handle, num_buffers) }.as_bool();
    if !res {
        if let Err(err) = unsafe { GetLastError() } { 
            log_error!(LOG_HID_CAT, set_num_input_buffers, "Failed to set number of hid input buffers ({err})");
        }
    }
}

pub fn flush_input_queue(handle: DeviceHandle) {
    let handle = HANDLE(handle.0 as isize);

    let res = unsafe { HidD_FlushQueue(handle) }.as_bool();
    if !res {
        if let Err(err) = unsafe { GetLastError() } { 
            log_error!(LOG_HID_CAT, flush_input_queue, "Failed to flush input queue. ({err})");
        }
    }
}

pub fn get_capabilities(preparse_data: &PreparseData) -> Option<Capabilities> {
    let mut caps = HIDP_CAPS::default();
    let res = unsafe { HidP_GetCaps(PHIDP_PREPARSED_DATA(preparse_data.get_address() as isize), &mut caps) }.ok();
    if let Err(err) = res {
        log_error!(LOG_HID_CAT, get_identifier, "Failed to retrieve hid usage page and usage. ({err})");
        return None;
    }

    Some(Capabilities {
        input_report_byte_len: caps.InputReportByteLength,
        output_report_byte_len: caps.OutputReportByteLength,
        feature_report_byte_len: caps.FeatureReportByteLength,
        num_collection_nodes: caps.NumberLinkCollectionNodes,
        num_input_button_caps: caps.NumberInputButtonCaps,
        num_input_value_caps: caps.NumberInputValueCaps,
        num_input_data_indices: caps.NumberInputDataIndices,
        num_output_button_caps: caps.NumberOutputButtonCaps,
        num_output_value_caps: caps.NumberOutputValueCaps,
        num_output_data_indices: caps.NumberOutputDataIndices,
        num_feature_button_caps: caps.NumberFeatureButtonCaps,
        num_feature_value_caps: caps.NumberFeatureValueCaps,
        num_feature_data_indices: caps.NumberFeatureDataIndices
    })
   
}

pub fn get_button_capabilities(preparse_data: &PreparseData, caps: &Capabilities) -> Option<[Vec<ButtonCaps>; ReportType::COUNT]> {
    let preparse_data = preparse_data.get_address() as isize;

    let input_caps = if caps.num_input_button_caps > 0 {
        match get_button_capabilities_for(HidP_Input, preparse_data, caps.num_input_button_caps) {
            Some(caps) => caps,
            None => return None,
        }
    } else {
        Vec::new()
    };
    
    let output_caps = if caps.num_output_button_caps > 0 {
        match get_button_capabilities_for(HidP_Output, preparse_data, caps.num_output_button_caps) {
            Some(caps) => caps,
            None => return None,
        }
    } else {
        Vec::new()
    };

    let feature_caps = if caps.num_feature_button_caps > 0 {
        match get_button_capabilities_for(HidP_Feature, preparse_data, caps.num_feature_button_caps) {
            Some(caps) => caps,
            None => return None,
        }
    } else {
        Vec::new()
    };

    Some([input_caps, output_caps, feature_caps])
}

fn get_button_capabilities_for(report_type: HIDP_REPORT_TYPE, preparse_data: isize, mut num_caps: u16) -> Option<Vec<ButtonCaps>> {
    let mut win_caps = unsafe {
        scoped_alloc!(AllocId::TlsTemp);
        let mut win_caps = Vec::with_capacity(num_caps as usize);
        win_caps.set_len(num_caps as usize);
        win_caps
    };

    let res = unsafe { HidP_GetButtonCaps(report_type, win_caps.as_mut_ptr(), &mut num_caps, PHIDP_PREPARSED_DATA(preparse_data)) }.ok();
    if let Err(err) = res {
        log_error!(LOG_HID_CAT, get_button_capabilities, "Failed to retrieve input button capabilities ({err})");
        return None;
    }

    let mut caps = Vec::with_capacity(num_caps as usize);
    for cap in win_caps {
        unsafe {
            let (usage, data_index) = if cap.IsRange.as_bool() {
                ((UsageId::new(cap.Anonymous.Range.UsageMin)..=UsageId::new(cap.Anonymous.Range.UsageMax)).into(),
                (cap.Anonymous.Range.DataIndexMin..=cap.Anonymous.Range.DataIndexMax).into())
            } else {
                let usage = cap.Anonymous.NotRange.Usage;
                let index = cap.Anonymous.NotRange.DataIndex;
                ((UsageId::new(usage)..=UsageId::new(usage)).into(), (index..=index).into())
            };
            let string_index = if cap.IsStringRange.as_bool() {
                (cap.Anonymous.Range.StringMin..=cap.Anonymous.Range.StringMax).into()
            } else {
                let index = cap.Anonymous.NotRange.StringIndex;
                (index..=index).into()
            };
            let designator = if cap.IsDesignatorRange.as_bool() {
                (cap.Anonymous.Range.DesignatorMin..=cap.Anonymous.Range.DesignatorMax).into()
            } else {
                let index = cap.Anonymous.NotRange.DesignatorIndex;
                (index..=index).into()
            };
            
            caps.push(ButtonCaps {
                usage_page: UsagePageId::new(cap.UsagePage),
                report_id: cap.ReportID,
                data_fields: cap.BitField,
                collection_id: cap.LinkCollection,
                report_count: cap.ReportCount,
                usage,
                string_index,
                designator,
                data_index,
                is_absolute: cap.IsAbsolute.as_bool(),
            })
        }
    }

    Some(caps)
}

pub fn get_value_capabilities(preparse_data: &PreparseData, caps: &Capabilities) -> Option<[Vec<ValueCaps>; ReportType::COUNT]> {
    unsafe {
        let preparse_data = preparse_data.get_address() as isize;

        let input_caps = if caps.num_input_value_caps > 0 {
            match get_value_capabilities_for(HidP_Input, preparse_data, caps.num_input_value_caps) {
                Some(caps) => caps,
                None => return None,
            }
        } else {
            Vec::new()
        };

        let output_caps = if caps.num_output_value_caps > 0 {
            match get_value_capabilities_for(HidP_Output, preparse_data, caps.num_output_value_caps) {
                Some(caps) => caps,
                None => return None,
            }
        } else {
            Vec::new()
        };

        let feature_caps = if caps.num_feature_value_caps > 0 {
            match get_value_capabilities_for(HidP_Feature, preparse_data, caps.num_feature_value_caps) {
                Some(caps) => caps,
                None => return None,
            }
        } else {
            Vec::new()
        };

        Some([input_caps, output_caps, feature_caps])
    }
}

unsafe fn get_value_capabilities_for(report_type: HIDP_REPORT_TYPE, preparse_data: isize, mut num_caps: u16) -> Option<Vec<ValueCaps>> {
    let mut win_caps = unsafe {
        scoped_alloc!(AllocId::TlsTemp);
        let mut win_caps = Vec::with_capacity(num_caps as usize);
        win_caps.set_len(num_caps as usize);
        win_caps
    };

    let res = HidP_GetValueCaps(report_type, win_caps.as_mut_ptr(), &mut num_caps, PHIDP_PREPARSED_DATA(preparse_data)).ok();
    if let Err(err) = res {
        log_error!(LOG_HID_CAT, get_button_capabilities, "Failed to retrieve input button capabilities ({err})");
        return None;
    }

    let mut caps = Vec::with_capacity(num_caps as usize);
    for cap in win_caps {
        let (usage, data_index) = if cap.IsRange.as_bool() {
            ((UsageId::new(cap.Anonymous.Range.UsageMin)..=UsageId::new(cap.Anonymous.Range.UsageMax)).into(),
             (cap.Anonymous.Range.DataIndexMin..=cap.Anonymous.Range.DataIndexMax).into())
        } else {
            let usage = cap.Anonymous.NotRange.Usage;
            let index = cap.Anonymous.NotRange.DataIndex;
            ((UsageId::new(usage)..=UsageId::new(usage)).into(), (index..=index).into())
        };
        let string_index = if cap.IsStringRange.as_bool() {
            (cap.Anonymous.Range.StringMin..=cap.Anonymous.Range.StringMax).into()
        } else {
            let index = cap.Anonymous.NotRange.StringIndex;
            (index..=index).into()
        };
        let designator = if cap.IsDesignatorRange.as_bool() {
            (cap.Anonymous.Range.DesignatorMin..=cap.Anonymous.Range.DesignatorMax).into()
        } else {
            let index = cap.Anonymous.NotRange.DesignatorIndex;
            (index..=index).into()
        };

        let bit_mask = (u32::MAX >> (32 - cap.BitSize)) as i32;

        caps.push(ValueCaps {
            usage_page: UsagePageId::new(cap.UsagePage),
            report_id: cap.ReportID,
            data_fields: cap.BitField,
            collection_id: cap.LinkCollection,
            has_null: cap.HasNull.as_bool(),
            unit_exp: cap.UnitsExp,
            units: cap.Units,
            logical_range: ((cap.LogicalMin & bit_mask)..=(cap.LogicalMax & bit_mask)).into(),
            physical_range: ((cap.PhysicalMin & bit_mask)..=(cap.PhysicalMax & bit_mask)).into(),
            bit_size: cap.BitSize,
            report_count: cap.ReportCount,
            usage,
            string_index,
            designator,
            data_index,
            is_absolute: cap.IsAbsolute.as_bool(),
            
        })
    }

    Some(caps)
}

pub fn get_top_level_collection<'a>(dev: &'a Device) -> Option<TopLevelCollection<'a>> {
    let preparse_data = dev.preparse_data.get_address() as isize;

    
    let num_collection_nodes = dev.capabilities.num_collection_nodes;
    let mut win_nodes = {
        scoped_alloc!(AllocId::TlsTemp);
        Vec::with_capacity(num_collection_nodes as usize)
    };
    unsafe { win_nodes.set_len(num_collection_nodes as usize) };

    let mut len = num_collection_nodes as u32;
    let err =  unsafe {HidP_GetLinkCollectionNodes(win_nodes.as_mut_ptr(), &mut len, PHIDP_PREPARSED_DATA(preparse_data)) }.ok();
    match err {
        Ok(_) => (),
        Err(err) => { 
            log_error!(LOG_HID_CAT, get_top_level_collection,"Failed to get collections. ({err})");
            return None;
        },
    }

    let mut nodes = Vec::new();
    let mut children = Vec::new();
    process_collection_nodes(&win_nodes, &mut nodes, &mut children, 0, None);

    Some(TopLevelCollection::new(nodes, children))
}

fn process_collection_nodes(win_nodes: &Vec<HIDP_LINK_COLLECTION_NODE>, nodes: &mut Vec<CollectionNode>, children: &mut Vec<Vec<u16>>, mut win_idx: usize, parent_idx: Option<usize>) {
    loop {
        let mut node = &win_nodes[win_idx];
        let mut usages = Vec::new();
        
        // handle aliased usages
        let range_start = win_idx as u16;
        while node._bitfield & 0x100 != 0 {
            usages.push(Usage::from_u16(node.LinkUsagePage,  node.LinkUsage));
            win_idx = node.NextSibling as usize;
            node = &win_nodes[win_idx];
        }

        usages.push(Usage::from_u16(node.LinkUsagePage,  node.LinkUsage));

        let kind_u8 = node._bitfield as u8;
        match CollectionKind::from_u8(kind_u8) {
            Some(kind) => {
                let id = win_idx as u16;
                let node_idx = nodes.len() - 1;

                if let Some(parent_idx) = parent_idx {
                    if children.len() < parent_idx + 1 {
                        children.resize_with(parent_idx + 1, || Vec::new());
                    }
                    children[parent_idx].push(node_idx as u16);
                }
            
                nodes.push(CollectionNode {
                    ids: (range_start..=id).into(),
                    usages,
                    kind: kind,
                    children: Vec::new()
                });

                if node.FirstChild != 0 {
                    process_collection_nodes(&win_nodes, nodes, children, node.FirstChild as usize, Some(node_idx));
                }
            },
            None => {
                log_error!(LOG_HID_CAT, process_collection_nodes, "Invalid collection type: '{kind_u8}'");
            },
        };

        if node.NextSibling == 0 {
            break;
        }
        win_idx = node.NextSibling as usize;
    }
}

//------------------------------------------------------------------------------------------------------------------------------
// REPORT CREATION
//------------------------------------------------------------------------------------------------------------------------------

pub fn create_report_data(dev: &Device, report_type: ReportType, report_id: u8) -> Option<Vec<u8>> {
        let preparse_data = dev.preparse_data.get_address() as isize;
        
        let report_size = match report_type {
            ReportType::Input => dev.capabilities.output_report_byte_len,
            ReportType::Output => dev.capabilities.input_report_byte_len,
            ReportType::Feature => dev.capabilities.feature_report_byte_len,
        } as usize;
        let report_type = to_native_report_type(report_type);
        
        let mut blob = vec![0; report_size as usize];
        let res = unsafe { HidP_InitializeReportForID(report_type, report_id, PHIDP_PREPARSED_DATA(preparse_data), &mut blob) }.ok();
        match res {
            Ok(_) => Some(blob),
            Err(err) => {
                log_error!(LOG_HID_CAT, get_top_level_collection,"Failed to create report data. ({err})");
                None
            },
        }
}

//------------------------------------------------------------------------------------------------------------------------------
// REPORT READ/WRITE
//------------------------------------------------------------------------------------------------------------------------------

pub fn read_input_report(dev: &mut Device) -> Result<Option<InputReport>, ()> {
    let handle = HANDLE(dev.handle.0 as isize);

    let report_len = dev.capabilities.input_report_byte_len as u32;
    let mut bytes_read = 0;

    let mut read_buffer = Vec::new();
    read_buffer.resize(report_len as usize, 0);

    match unsafe { ReadFile(handle, Some(&mut read_buffer), Some(&mut bytes_read), None) } {
        Ok(_) => (),
        Err(err) => {
            log_error!(LOG_HID_CAT, read_input_report, "Failed to read input report ({err})");
            return Err(());
        },
    };

    if bytes_read < report_len {
        log_error!(LOG_HID_CAT, read_input_report, "Failed to read full input report ({bytes_read}/{report_len} bytes read)");
    }

    unsafe { read_buffer.set_len(bytes_read as usize) };
    let report_buf = mem::replace(&mut read_buffer, Vec::with_capacity(report_len as usize));
    Ok(Some(InputReport { data: crate::ReportData::Blob(report_buf), device: dev }))
}

pub fn write_output_report<'a>(dev: &mut Device, report: OutputReport<'a>) -> Result<(), OutputReport<'a>> {
    let handle = HANDLE(dev.handle.0 as isize);

    let mut bytes_written = 0;
    let data = report.data.get_data();
    unsafe { WriteFile(handle, Some(data), Some(&mut bytes_written), None) }.map_err(|err| {
        log_error!(LOG_HID_CAT, write_output_report, "Failed to write output report (err: {err})");
        report
    })
}

pub fn get_feature_report(dev: &mut Device, report_id: u8) -> Option<FeatureReport<'_>> {
    let handle = HANDLE(dev.handle.0 as isize);
    let mut report_blob = create_report_data(dev, ReportType::Feature, report_id)?;

    let res = unsafe { HidD_GetFeature(handle, report_blob.as_mut_ptr() as *mut c_void, report_blob.len() as u32) }.as_bool();
    if res {
        Some(FeatureReport { data: ReportData::Blob(report_blob), device: dev })
    } else {
        if let Err(err) = unsafe { GetLastError() } { 
            log_error!(LOG_HID_CAT, write_output_report, "Failed to get feature report ({err})");
        }
        None
    }
}

pub fn set_feature_report<'a>(dev: &mut Device, report: FeatureReport<'a>) -> Result<(), FeatureReport<'a>> {
    let handle = HANDLE(dev.handle.0 as isize);
    let data = report.data.get_data();

    let res = unsafe { HidD_SetFeature(handle, data.as_ptr() as *const c_void, data.len() as u32) }.as_bool();
    if res {
        Ok(())
    } else {
        if let Err(err) = unsafe { GetLastError() } { 
            log_error!(LOG_HID_CAT, write_output_report, "Failed to set feature report ({err})");
        }
        Err(report)
    }
}

//------------------------------------------------------------------------------------------------------------------------------
// REPORTS GETTERS
//------------------------------------------------------------------------------------------------------------------------------

pub fn get_buttons(dev: &Device, collection_id: u16, report_type: ReportType, report: &[u8]) -> Option<Vec<Usage>> {
    let preparse_data = dev.preparse_data.get_address() as isize;
    let report_type = to_native_report_type(report_type);

    let mut button_count = 0;
    _ = unsafe { HidP_GetUsagesEx(report_type, collection_id, null_mut(), &mut button_count, PHIDP_PREPARSED_DATA(preparse_data), report) };

    let mut win_buttons = Vec::with_capacity(button_count as usize);
    
    let res = unsafe { HidP_GetUsagesEx(report_type, collection_id, win_buttons.as_mut_ptr(), &mut button_count, PHIDP_PREPARSED_DATA(preparse_data), report) }.ok();
    if let Err(err) = res {
        log_error!(LOG_HID_CAT, get_buttons, "Failed to extract buttons from report. (error: {:X})", err.code().0);
        return None;
    }
    unsafe { win_buttons.set_len(button_count as usize) };

    Some(win_buttons.into_iter()
            .map(|usage_and_page| Usage::from_u16(usage_and_page.UsagePage, usage_and_page.Usage))
            .collect()
    )
}

pub fn get_buttons_for_page(dev: &Device, page: UsagePageId, collection_id: u16, report_type: ReportType, report: &[u8]) -> Option<Vec<UsageId>> {
    let preparse_data = dev.preparse_data.get_address() as isize;
    let report_type = to_native_report_type(report_type);

    let mut button_count = 0;

    // SAFETY: HidP_GetUsages doesn't actually write to the report buffer, so this *technically* not a mutable reference
    #[allow(invalid_reference_casting)]
    let report_data = unsafe { &mut *(report as *const _ as *mut [u8]) };

    _ = unsafe { HidP_GetUsages(report_type, page.as_u16(), collection_id, null_mut(), &mut button_count, PHIDP_PREPARSED_DATA(preparse_data), report_data) };

    let mut win_buttons = Vec::with_capacity(button_count as usize);
    
    let res = unsafe { HidP_GetUsages(report_type, page.as_u16(), collection_id, win_buttons.as_mut_ptr(), &mut button_count, PHIDP_PREPARSED_DATA(preparse_data), report_data) }.ok();
    if let Err(err) = res {
        log_error!(LOG_HID_CAT, get_buttons, "Failed to extract buttons from report. ({err})");
        return None;
    }
    unsafe { win_buttons.set_len(button_count as usize) };

    Some(win_buttons.into_iter()
            .map(|usage| UsageId::new(usage))
            .collect()
    )
}

pub fn get_raw_value(dev: &Device, usage: Usage, collection_id: u16, report_type: ReportType, report: &[u8]) -> Option<RawValue> {
    let (report_count, bit_size) = get_value_report_count_and_bitsize_for_usage(dev, usage, collection_id, report_type);
    if report_count == 0 {
        log_error!(LOG_HID_CAT, get_raw_value, "Failed to find any valid report counts for the current collection id '{}'", collection_id);
        return None;
    }

    let preparse_data = dev.preparse_data.get_address() as isize;
    let report_type = to_native_report_type(report_type);

    if report_count == 1 {
        let mut value = 0;
        let res = unsafe { HidP_GetUsageValue(report_type, usage.page.as_u16(), collection_id, usage.usage.as_u16(), &mut value, PHIDP_PREPARSED_DATA(preparse_data), report) }.ok();
        match res {
            Ok(_) => Some(RawValue::Single(value, bit_size)),
            Err(err) => {
                log_error!(LOG_HID_CAT, get_raw_value, "Failed to get the value for a specific usage. ({err})");
                None
            },
        }
    } else {
        let size = (report_count as usize * bit_size as usize + 7) / 8;
        let mut values = Vec::with_capacity(size);
        unsafe { values.set_len(size) };

        let res = unsafe { HidP_GetUsageValueArray(report_type, usage.page.as_u16(), collection_id, usage.usage.as_u16(), &mut values, PHIDP_PREPARSED_DATA(preparse_data), report) }.ok();
        match res {
            Ok(_) => Some(RawValue::Array(values, bit_size)),
            Err(err) => {
                log_error!(LOG_HID_CAT, get_raw_value, "Failed to get the value for a specific usage. ({err})");
                None
            },
        }
    }
}

fn get_value_report_count_and_bitsize_for_usage(dev: &Device, usage: Usage, collection_id: u16, report_type: ReportType) -> (u16, u16) {
    let mut value = (0, 0);
    for caps in &dev.value_caps[report_type as usize] {
        if caps.usage_page == usage.page &&
           caps.usage.contains(&usage.usage)
        {
            if collection_id == 0 && caps.collection_id != 0 {
                if value.0 == 0 {
                    value = (caps.report_count, caps.bit_size);
                }
            } else if collection_id == caps.collection_id {
                return (caps.report_count, caps.bit_size);
            }
        }
    }
    value
}

pub fn get_scaled_value(dev: &Device, usage: Usage, collection_id: u16, report_type: ReportType, report: &[u8]) -> Option<i32> {
    let preparse_data = dev.preparse_data.get_address() as isize;
    let report_type = to_native_report_type(report_type);

    let mut value = 0;
    let res = unsafe { HidP_GetScaledUsageValue(report_type, usage.page.as_u16(), collection_id, usage.usage.as_u16(), &mut value, PHIDP_PREPARSED_DATA(preparse_data), report) }.ok();
    match res {
        Ok(_) => Some(value),
        Err(err) => {
            log_error!(LOG_HID_CAT, get_scaled_value, "Failed to get the scaled value for a specific usage. ({err})");
            None
        },
    }
}

pub fn get_data(dev: &Device, report_type: ReportType, report: &[u8]) -> Option<Vec<Data>> {
    let preparse_data = dev.preparse_data.get_address() as isize;
    let win_report_type = to_native_report_type(report_type);

    // SAFETY: HidP_GetUsages doesn't actually write to the report buffer, so this *technically* not a mutable reference
    #[allow(invalid_reference_casting)]
    let report_data = unsafe {  &mut *(report as *const _ as *mut [u8]) };

    let scoped_alloc = ScopedAlloc::new(AllocId::TlsTemp);

    let mut data_len = unsafe { HidP_MaxDataListLength(win_report_type, PHIDP_PREPARSED_DATA(preparse_data)) };
    let mut data = Vec::with_capacity(data_len as usize);
    
    drop(scoped_alloc);

    let res = unsafe { HidP_GetData(win_report_type, data.as_mut_ptr(), &mut data_len, PHIDP_PREPARSED_DATA(preparse_data), report_data) }.ok();
    match res {
        Ok(_) => {
            unsafe { data.set_len(data_len as usize) };
            Some(data.into_iter()
                .map(|data| 
                    Data {
                        index: data.DataIndex,
                        value: if is_data_button(dev, data.DataIndex, report_type) { 
                            DataValue::Button(unsafe { data.Anonymous.On }.as_bool())
                        } else {
                            DataValue::Value(unsafe { data.Anonymous.RawValue })
                        }
                    }
                )
                .collect()
            )
        },
        Err(err) => {
            log_error!(LOG_HID_CAT, get_scaled_value, "Failed to set the scaled value for a specific usage. ({err})");
            None
        },
    }
}

fn is_data_button(dev: &Device, data_index: u16, report_type: ReportType) -> bool {
    for caps in &dev.button_caps[report_type as usize] {
        if caps.data_index.contains(&data_index) {
            return true;
        }
    }
    false
}

//------------------------------------------------------------------------------------------------------------------------------
// REPORT SETTERS
//------------------------------------------------------------------------------------------------------------------------------

pub fn set_buttons(dev: &Device, page: UsagePageId, collection_id: u16, usages: &mut [UsageId], report_type: ReportType, report: &mut [u8]) {
    let preparse_data = dev.preparse_data.get_address() as isize;
    let report_type = to_native_report_type(report_type);

    let mut len = usages.len() as u32;
    let res = unsafe { HidP_SetUsages(report_type, page.as_u16(), collection_id, usages.as_mut_ptr() as *mut u16, &mut len, PHIDP_PREPARSED_DATA(preparse_data), report) }.ok();
    if let Err(err) = res {
        log_error!(LOG_HID_CAT, get_scaled_value, "Failed to set buttons in the report. ({err})");
    }
}

pub fn unset_buttons(dev: &Device, page: UsagePageId, collection_id: u16, usages: &mut [UsageId], report_type: ReportType, report: &mut [u8]) {
    let preparse_data = dev.preparse_data.get_address() as isize;
    let report_type = to_native_report_type(report_type);

    let mut len = usages.len() as u32;
    let res = unsafe { HidP_UnsetUsages(report_type, page.as_u16(), collection_id, usages.as_mut_ptr() as *mut u16, &mut len, PHIDP_PREPARSED_DATA(preparse_data), report) }.ok();
    if let Err(err) = res {
        log_error!(LOG_HID_CAT, get_scaled_value, "Failed to unset buttons in the report. ({err})");
    }
}

pub fn set_value(dev: &Device, usage: Usage, collection_id: u16, raw_value: u32, report_type: ReportType, report: &mut [u8]) {
    let preparse_data = dev.preparse_data.get_address() as isize;
    let report_type = to_native_report_type(report_type);

    let res = unsafe { HidP_SetUsageValue(report_type, usage.page.as_u16(), collection_id, usage.usage.as_u16(), raw_value, PHIDP_PREPARSED_DATA(preparse_data), report) }.ok();
    if let Err(err) = res {
        log_error!(LOG_HID_CAT, get_scaled_value, "Failed to unset buttons in the report. ({err})");
    }
}

pub fn set_values(dev: &Device, usage: Usage, collection_id: u16, raw_values: &[u8], report_type: ReportType, report: &mut [u8]) {
    let preparse_data = dev.preparse_data.get_address() as isize;
    let report_type = to_native_report_type(report_type);

    let res = unsafe { HidP_SetUsageValueArray(report_type, usage.page.as_u16(), collection_id, usage.usage.as_u16(), raw_values, PHIDP_PREPARSED_DATA(preparse_data), report) }.ok();
    if let Err(err) = res {
        log_error!(LOG_HID_CAT, get_scaled_value, "Failed to unset buttons in the report. ({err})");
    }
}

pub fn set_data(dev: &Device, data: &[Data], report_type: ReportType, report: &mut [u8]) {
    let preparse_data = dev.preparse_data.get_address() as isize;
    let report_type = to_native_report_type(report_type);

    let mut win_data = Vec::new();
    win_data.reserve(data.len());
    for datum in data {
        let mut hid_data = HIDP_DATA::default();
        hid_data.DataIndex = datum.index;
        match datum.value {
            DataValue::Button(on) => hid_data.Anonymous.On = BOOLEAN(on as u8),
            DataValue::Value(raw_value) => hid_data.Anonymous.RawValue = raw_value,
        }

        win_data.push(hid_data)
    }

    let mut len = win_data.len() as u32;
    let res = unsafe { HidP_SetData(report_type, win_data.as_mut_ptr(), &mut len, PHIDP_PREPARSED_DATA(preparse_data), report) }.ok();
    if let Err(err) = res {
        log_error!(LOG_HID_CAT, get_scaled_value, "Failed to set data in the report.({err})");
    }
}

//------------------------------------------------------------------------------------------------------------------------------
// MISC
//------------------------------------------------------------------------------------------------------------------------------

fn to_native_report_type(report_type: ReportType) -> HIDP_REPORT_TYPE {
    match report_type {
        ReportType::Input => HidP_Input,
        ReportType::Output => HidP_Output,
        ReportType::Feature => HidP_Feature,
    }
}


//------------------------------------------------------------------------------------------------------------------------------
// DEVICE DISCOVERY
//------------------------------------------------------------------------------------------------------------------------------
// TODO: Some of the functionality needed does not exists in windows-rs, so do this after we have a dynamic library loader