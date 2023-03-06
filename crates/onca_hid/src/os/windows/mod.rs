use core::{
    mem,
    slice,
    ffi::c_void,
    num::NonZeroU32,
    ptr::null_mut,
};
use onca_core::{
    prelude::*,
    time::Duration,
};
use onca_logging::{log_warning, log_error};
use windows::{
    Win32::{
        Devices::HumanInterfaceDevice::*,
        Foundation::{HANDLE, GetLastError, CloseHandle, BOOL, ERROR_IO_PENDING, WAIT_OBJECT_0, BOOLEAN},
        Storage::FileSystem::{CreateFileA, FILE_ACCESS_FLAGS, FILE_SHARE_WRITE, FILE_SHARE_READ, FILE_FLAG_OVERLAPPED, OPEN_EXISTING, ReadFile, WriteFile},
        System::{
            SystemServices::{GENERIC_READ, GENERIC_WRITE},
            Threading::{CreateEventA, WaitForSingleObject}, IO::{OVERLAPPED, GetOverlappedResult, CancelIoEx},
        },
    },
    core::PCSTR,
};

use crate::*;

pub struct OSDevice {
    read_overlapped  : HeapPtr<OVERLAPPED>,
    write_overlapped : HeapPtr<OVERLAPPED>
}

//------------------------------------------------------------------------------------------------------------------------------
// DEVICE_CREATION
//------------------------------------------------------------------------------------------------------------------------------

pub fn open_device(path: &str) -> Option<DeviceHandle> {
    unsafe {
        let name = PCSTR(path.as_ptr());
        
        let handle = CreateFileA(
            name,
            FILE_ACCESS_FLAGS(GENERIC_READ | GENERIC_WRITE),
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_FLAG_OVERLAPPED,
            HANDLE::default()
        );
        match handle {
            Ok(handle) => Some(DeviceHandle(handle.0 as usize)),
            Err(err) => {
                log_error!(LOG_HID_CAT, open_device, "Failed to open the HID device. (error: {:X})", err.code().0);
                None
            },
        }
    }
}

pub fn close_handle(handle: DeviceHandle) {
    unsafe {
        let res = CloseHandle(HANDLE(handle.0 as isize)).as_bool();
        if !res {
            log_error!(LOG_HID_CAT, close_handle, "Failed to close the HID device. (error: {:X})", GetLastError().0);
        }
    }
}

pub fn create_os_device(_handle: &DeviceHandle) -> Option<OSDevice> {
    unsafe {
        let read_overlapped = create_overlapped("read")?;
        let write_overlapped = create_overlapped("write")?;
        Some(OSDevice { read_overlapped, write_overlapped })
    }
}

unsafe fn create_overlapped(err_kind: &str) -> Option<HeapPtr<OVERLAPPED>> {
    let event = CreateEventA(None, BOOL(0), BOOL(0), PCSTR(null_mut()));
   let read_event = match event {
       Ok(event) => event,
       Err(err) => {
           log_error!(LOG_HID_CAT, create_os_device, "Failed to create a {} event for the HID device. (error: {:X})", err_kind, err.code().0);
           return None
       },
   };

   let mut overlapped = HeapPtr::new(OVERLAPPED::default());
   overlapped.hEvent = read_event;

   Some(overlapped)
}

pub fn destroy_os_device(os_dev: &mut OSDevice) {
    unsafe {
        let res = CloseHandle(os_dev.read_overlapped.hEvent).as_bool();
        if !res {
            log_error!(LOG_HID_CAT, destroy_os_device, "Failed to destroy an event for the HID device. (error: {:X})", GetLastError().0);
        }
    }
}

pub fn get_preparse_data(handle: DeviceHandle) -> Option<PreparseData> {
    unsafe {
        let handle = HANDLE(handle.0 as isize);

        let mut preparse_data = 0isize;
        let res = HidD_GetPreparsedData(handle, &mut preparse_data).as_bool();
        if res {
           Some(PreparseData(PreparseDataInternal::Address(preparse_data as usize))) 
        } else {
            log_error!(LOG_HID_CAT, get_preparse_data, "Failed to retrieve preparse data. (error: {:X})", GetLastError().0);
            None
        }
    }
}

pub fn free_preparse_data(preparse_data: &mut PreparseData) {
    unsafe {
        if let PreparseDataInternal::Address(addr) = preparse_data.0 {
            let preparse_data = addr as isize;
            let res = HidD_FreePreparsedData(preparse_data).as_bool();
            if !res {
                log_error!(LOG_HID_CAT, get_preparse_data, "Failed to free preparse data. (error: {:X})", GetLastError().0);
            }
        }
    }
}

//------------------------------------------------------------------------------------------------------------------------------
// DEVICE
//------------------------------------------------------------------------------------------------------------------------------

pub fn get_identifier(handle: DeviceHandle, preparse_data: &PreparseData) -> Option<Identifier> {
    unsafe {
        let handle = HANDLE(handle.0 as isize);

        let mut attribs = HIDD_ATTRIBUTES::default();
        attribs.Size = mem::size_of::<HIDD_ATTRIBUTES>() as u32;
        let res = HidD_GetAttributes(handle, &mut attribs).as_bool();
        if !res {
            log_error!(LOG_HID_CAT, get_identifier, "Failed to retieve hid attributes. (error: {:X})", GetLastError().0);
        }

        let mut caps = HIDP_CAPS::default();
        let res = HidP_GetCaps(preparse_data.get_address() as isize, &mut caps);
        match res {
            Ok(_) => (),
            Err(_) => {
                log_error!(LOG_HID_CAT, get_identifier, "Failed to retrieve hid usage page and usage. (error: {:X})", GetLastError().0);
                return None;
            },
        }

        Some(Identifier {
            vendor_device: VendorProduct::from_u16(attribs.VendorID,attribs.ProductID),
            version: attribs.VersionNumber,
            usage: Usage::from_u16(caps.UsagePage, caps.Usage),
        })
    }
}

pub fn get_vendor_string(handle: DeviceHandle) -> Option<String> {
    unsafe {
        let handle = HANDLE(handle.0 as isize);
        // +1 for null terminator
        let mut buf = [0u16; MAX_HID_STRING_LEN + 1];

        let res = HidD_GetManufacturerString(handle, &mut buf as *mut _ as *mut c_void, MAX_HID_STRING_LEN as u32).as_bool();
        if res {
            let null_term_idx = buf.iter().position(|&c| c == 0).unwrap_or(MAX_HID_STRING_LEN);
            let str_slice = slice::from_raw_parts(buf.as_ptr(), null_term_idx);
            Some(String::from_utf16_lossy(str_slice))
        } else {
            log_warning!(LOG_HID_CAT, "Failed to retrieve hid vendor string. (error:{:X})", GetLastError().0);
            None
        }
    }
}

pub fn get_product_string(handle: DeviceHandle) -> Option<String> {
    unsafe {
        let handle = HANDLE(handle.0 as isize);
        // +1 for null terminator
        let mut buf = [0u16; MAX_HID_STRING_LEN + 1];
        let res = HidD_GetProductString(handle, &mut buf as *mut _ as *mut c_void, MAX_HID_STRING_LEN as u32).as_bool();
        if res {
            let null_term_idx = buf.iter().position(|&c| c == 0).unwrap_or(MAX_HID_STRING_LEN);
            let str_slice = slice::from_raw_parts(buf.as_ptr(), null_term_idx);
            Some(String::from_utf16_lossy(str_slice))
        } else {
            log_warning!(LOG_HID_CAT, "Failed to retrieve hid vendor string. (error:{:X})", GetLastError().0);
            None
        }
    }
}

pub fn get_serial_number_string(handle: DeviceHandle) -> Option<String> {
    unsafe {
        let handle = HANDLE(handle.0 as isize);
        // +1 for null terminator
        let mut buf = [0u16; MAX_HID_STRING_LEN + 1];
        let res = HidD_GetSerialNumberString(handle, &mut buf as *mut _ as *mut c_void, MAX_HID_STRING_LEN as u32).as_bool();
        if res {
            let null_term_idx = buf.iter().position(|&c| c == 0).unwrap_or(MAX_HID_STRING_LEN);
            let str_slice = slice::from_raw_parts(buf.as_ptr(), null_term_idx);
            Some(String::from_utf16_lossy(str_slice))
        } else {
            log_warning!(LOG_HID_CAT, "Failed to retrieve hid vendor string. (error:{:X})", GetLastError().0);
            None
        }
    }
}

pub fn get_indexed_string(handle: DeviceHandle, index: usize) -> Option<String> {
    unsafe {
        let handle = HANDLE(handle.0 as isize);
        // +1 for null terminator
        let mut buf = [0u16; MAX_HID_STRING_LEN + 1];

        let res = HidD_GetIndexedString(handle, index as u32, &mut buf as *mut _ as *mut c_void, MAX_HID_STRING_LEN as u32).as_bool();
        if res {
            let null_term_idx = buf.iter().position(|&c| c == 0).unwrap_or(MAX_HID_STRING_LEN);
            let str_slice = slice::from_raw_parts(buf.as_ptr(), null_term_idx);
            Some(String::from_utf16_lossy(str_slice))
        } else {
            log_warning!(LOG_HID_CAT, "Failed to retrieve indexed string. (error:{:X})", GetLastError().0);
            None
        }
    }
}

pub fn get_num_input_buffers(handle: DeviceHandle) -> Option<NonZeroU32> {
    unsafe {
        let handle = HANDLE(handle.0 as isize);

        let mut num_buffers = 0;
        let res = HidD_GetNumInputBuffers(handle, &mut num_buffers).as_bool();
        if res {
            NonZeroU32::new(num_buffers)
        } else {
            log_warning!(LOG_HID_CAT, "Failed to retrieve number of input buffers. (error: {:X})", GetLastError().0);
            None
        }
    }
}

pub fn set_num_input_buffers(handle: DeviceHandle, num_buffers: u32) {
    unsafe {
        let handle = HANDLE(handle.0 as isize);

        let res = HidD_SetNumInputBuffers(handle, num_buffers).as_bool();
        if !res {
            log_error!(LOG_HID_CAT, set_num_input_buffers, "Failed to set number of hid input buffers (error: {:X})", GetLastError().0);
        }
    }
}

pub fn flush_input_queue(handle: DeviceHandle) {
    unsafe {
        let handle = HANDLE(handle.0 as isize);

        let res = HidD_FlushQueue(handle).as_bool();
        if !res {
            log_error!(LOG_HID_CAT, flush_input_queue, "Failed to flush input queue. (error: {:X})", GetLastError().0);
        }
    }
}

pub fn get_capabilities(preparse_data: &PreparseData) -> Option<Capabilities> {
    unsafe {
        let mut caps = HIDP_CAPS::default();
        let res = HidP_GetCaps(preparse_data.get_address() as isize, &mut caps);
        match res {
            Ok(_) => (),
            Err(_) => {
                log_error!(LOG_HID_CAT, get_identifier, "Failed to retrieve hid usage page and usage. (error: {:X})", GetLastError().0);
                return None;
            },
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
}

pub fn get_button_capabilities(preparse_data: &PreparseData, caps: &Capabilities) -> Option<[DynArray<ButtonCaps>; NUM_REPORT_TYPES]> {
    unsafe {
        let preparse_data = preparse_data.get_address() as isize;

        let input_caps = if caps.num_input_button_caps > 0 {
            match get_button_capabilities_for(HidP_Input, preparse_data, caps.num_input_button_caps) {
                Some(caps) => caps,
                None => return None,
            }
        } else {
            DynArray::new()
        };
        
        let output_caps = if caps.num_output_button_caps > 0 {
            match get_button_capabilities_for(HidP_Output, preparse_data, caps.num_output_button_caps) {
                Some(caps) => caps,
                None => return None,
            }
        } else {
            DynArray::new()
        };

        let feature_caps = if caps.num_feature_button_caps > 0 {
            match get_button_capabilities_for(HidP_Feature, preparse_data, caps.num_feature_button_caps) {
                Some(caps) => caps,
                None => return None,
            }
        } else {
            DynArray::new()
        };

        Some([input_caps, output_caps, feature_caps])
    }
}

unsafe fn get_button_capabilities_for(report_type: HIDP_REPORT_TYPE, preparse_data: isize, mut num_caps: u16) -> Option<DynArray<ButtonCaps>> {
    let scoped_alloc = ScopedAlloc::new(UseAlloc::TlsTemp);

    let mut win_caps = DynArray::with_capacity(num_caps as usize);
    win_caps.set_len(num_caps as usize);
    
    drop(scoped_alloc);

    let res = HidP_GetButtonCaps(report_type, win_caps.as_mut_ptr(), &mut num_caps, preparse_data);
    if let Err(err) = res {
        log_error!(LOG_HID_CAT, get_button_capabilities, "Failed to retrieve input button capabilities (error: {:X})", err.code().0);
        return None;
    }

    let mut caps = DynArray::with_capacity(num_caps as usize);
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

    Some(caps)
}

pub fn get_value_capabilities(preparse_data: &PreparseData, caps: &Capabilities) -> Option<[DynArray<ValueCaps>; NUM_REPORT_TYPES]> {
    unsafe {
        let preparse_data = preparse_data.get_address() as isize;

        let input_caps = if caps.num_input_value_caps > 0 {
            match get_value_capabilities_for(HidP_Input, preparse_data, caps.num_input_value_caps) {
                Some(caps) => caps,
                None => return None,
            }
        } else {
            DynArray::new()
        };

        let output_caps = if caps.num_output_value_caps > 0 {
            match get_value_capabilities_for(HidP_Output, preparse_data, caps.num_output_value_caps) {
                Some(caps) => caps,
                None => return None,
            }
        } else {
            DynArray::new()
        };

        let feature_caps = if caps.num_feature_value_caps > 0 {
            match get_value_capabilities_for(HidP_Feature, preparse_data, caps.num_feature_value_caps) {
                Some(caps) => caps,
                None => return None,
            }
        } else {
            DynArray::new()
        };

        Some([input_caps, output_caps, feature_caps])
    }
}

unsafe fn get_value_capabilities_for(report_type: HIDP_REPORT_TYPE, preparse_data: isize, mut num_caps: u16) -> Option<DynArray<ValueCaps>> {
    let scoped_alloc = ScopedAlloc::new(UseAlloc::TlsTemp);

    let mut win_caps = DynArray::with_capacity(num_caps as usize);
    win_caps.set_len(num_caps as usize);

    drop(scoped_alloc);

    let res = HidP_GetValueCaps(report_type, win_caps.as_mut_ptr(), &mut num_caps, preparse_data);
    if let Err(err) = res {
        log_error!(LOG_HID_CAT, get_button_capabilities, "Failed to retrieve input button capabilities (error: {:X})", err.code().0);
        return None;
    }

    let mut caps = DynArray::with_capacity(num_caps as usize);
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
    unsafe {
        let preparse_data = dev.preparse_data.get_address() as isize;

        let scoped_alloc = ScopedAlloc::new(UseAlloc::TlsTemp);

        let num_collection_nodes = dev.capabilities.num_collection_nodes;
        let mut win_nodes = DynArray::with_capacity(num_collection_nodes as usize);
        win_nodes.set_len(num_collection_nodes as usize);

        drop(scoped_alloc);

        let mut len = num_collection_nodes as u32;
        let err = HidP_GetLinkCollectionNodes(win_nodes.as_mut_ptr(), &mut len, preparse_data);
        match err {
            Ok(_) => (),
            Err(err) => { 
                log_error!(LOG_HID_CAT, get_top_level_collection,"Failed to get collections. (error: {:X})", err.code().0);
                return None;
            },
        }

        let mut nodes = DynArray::new();
        let mut children = DynArray::new();
        process_collection_nodes(&win_nodes, &mut nodes, &mut children, 0, None);

        Some(TopLevelCollection::new(nodes, children))
    }
}

fn process_collection_nodes(win_nodes: &DynArray<HIDP_LINK_COLLECTION_NODE>, nodes: &mut DynArray<CollectionNode>, children: &mut DynArray<DynArray<u16>>, mut win_idx: usize, parent_idx: Option<usize>) {
    loop {
        let mut node = &win_nodes[win_idx];
        let mut usages = DynArray::new();
        
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
                    children.resize_with(parent_idx + 1, || DynArray::new());
                    children[parent_idx].push(node_idx as u16);
                }

                if node.FirstChild != 0 {
                    process_collection_nodes(&win_nodes, nodes, children, node.FirstChild as usize, Some(node_idx));
                }
            
                nodes.push(CollectionNode {
                    ids: (range_start..=id).into(),
                    usages,
                    kind: kind,
                    children: DynArray::new()
                });
            },
            None => {
                log_error!(LOG_HID_CAT, process_collection_nodes, "Invalid collection type: '{kind_u8}'");
            },
        };

        // TODO: what about `node.UserContext`

        if node.NextSibling == 0 {
            break;
        }
        win_idx = node.NextSibling as usize;
    }
}

//------------------------------------------------------------------------------------------------------------------------------
// REPORT CREATION
//------------------------------------------------------------------------------------------------------------------------------

pub fn create_report_data(dev: &Device, report_type: ReportType, report_id: u8) -> Option<DynArray<u8>> {
    unsafe {
        let preparse_data = dev.preparse_data.get_address() as isize;
        let report_type = to_native_report_type(report_type);

        let report_size = dev.capabilities.output_report_byte_len as usize;
        let mut blob = DynArray::with_capacity(report_size);
        blob.set_len(report_size);

        let res = HidP_InitializeReportForID(report_type, report_id, preparse_data, &mut blob);
        match res {
            Ok(_) => Some(blob),
            Err(_) => None,
        }
    }
}

//------------------------------------------------------------------------------------------------------------------------------
// REPORT READ/WRITE
//------------------------------------------------------------------------------------------------------------------------------

pub fn read_input_report(dev: &mut Device, timeout: Duration) -> Result<Option<InputReport>, ()> {
    unsafe {
        let handle = HANDLE(dev.handle.0 as isize);
        let event = dev.os_dev.read_overlapped.hEvent;
        let overlapped = dev.os_dev.write_overlapped.ptr_mut();

        let report_len = dev.capabilities.input_report_byte_len as u32;
        let mut bytes_read = 0;

        if !dev.read_pending {
            dev.read_buffer.reserve(report_len as usize);
            dev.read_buffer.set_len(report_len as usize);

            let res = ReadFile(handle, Some(dev.read_buffer.as_mut_ptr() as *mut c_void), report_len, Some(&mut bytes_read), Some(overlapped)).as_bool();
            
            dev.read_pending = if !res {
                let err = GetLastError().0;
                if err == ERROR_IO_PENDING.0 {
                    true
                } else {
                    CancelIoEx(handle, Some(overlapped));
                    log_error!(LOG_HID_CAT, read_input_report, "Failed to read input report (err: {:X})", GetLastError().0);
                    return Err(());
                }
            } else {
                false
            };
        }

        if !timeout.is_zero() {
            let res = WaitForSingleObject(event, timeout.as_millis() as u32);
            if res != WAIT_OBJECT_0 {
                return Ok(None);
            }
        }

        let res = GetOverlappedResult(handle, overlapped, &mut bytes_read, true).as_bool();
        dev.read_pending = true;

        if res && bytes_read > 0 {
            dev.read_buffer.set_len(bytes_read as usize);
            let report_buf = mem::replace(&mut dev.read_buffer, DynArray::with_capacity(report_len as usize));
            Ok(Some(InputReport { data: crate::ReportData::Blob(report_buf), device: dev }))
        } else {
            Err(())
        }
    }
}

pub fn write_output_report<'a>(dev: &mut Device, report: OutputReport<'a>) -> Result<(), OutputReport<'a>> {
    unsafe {
        let handle = HANDLE(dev.handle.0 as isize);
        let event = dev.os_dev.read_overlapped.hEvent;
        let overlapped = dev.os_dev.write_overlapped.ptr_mut();

        let mut bytes_written = 0;
        let data = report.data.get_data();
        let res = WriteFile(handle, Some(data.as_ptr() as *const c_void), data.len() as u32, Some(&mut bytes_written), Some(overlapped)).as_bool();

        let write_pending = if !res {
            let err = GetLastError().0;
            if err == ERROR_IO_PENDING.0 {
                true
            } else {
                log_error!(LOG_HID_CAT, write_output_report, "Failed to write output report (err: {:X})", GetLastError().0);
                return Err(report);
            }
        } else {
            false
        };
        

        if write_pending {
            // Wait for about a second, if we failed writing at that point, we fail
            let res = WaitForSingleObject(event, 1000);
            if res != WAIT_OBJECT_0 {
                CancelIoEx(handle, Some(overlapped));
                log_error!(LOG_HID_CAT, write_output_report, "Timout while writing output report (error: {:X})", res.0);
                return Err(report);
            }

            let res = GetOverlappedResult(handle, overlapped, &mut bytes_written, BOOL(1)).as_bool();
            if !res {
                log_error!(LOG_HID_CAT, write_output_report, "Failed to get overlapped result for output report. (error: {:X})", GetLastError().0);
                return Err(report);
            }
        }
        Ok(())
    }
}

pub fn get_feature_report(dev: &mut Device) -> Option<FeatureReport<'_>> {
    unsafe {
        let handle = HANDLE(dev.handle.0 as isize);
        let report_len = dev.capabilities.feature_report_byte_len as u32;

        let mut report_blob = DynArray::with_capacity(report_len as usize);
        report_blob.set_len(report_len as usize);

        let res = HidD_GetFeature(handle, report_blob.as_mut_ptr() as *mut c_void, report_len).as_bool();
        if res {
            Some(FeatureReport { data: ReportData::Blob(report_blob), device: dev })
        } else {
            log_error!(LOG_HID_CAT, write_output_report, "Failed to get feature report (err: {:X})", GetLastError().0);
            None
        }
    }
}

pub fn set_feature_report<'a>(dev: &mut Device, report: FeatureReport<'a>) -> Result<(), FeatureReport<'a>> {
    unsafe {
        let handle = HANDLE(dev.handle.0 as isize);
        let data = report.data.get_data();

        let res = HidD_SetFeature(handle, data.as_ptr() as *const c_void, data.len() as u32).as_bool();
        if res {
            Ok(())
        } else {
            log_error!(LOG_HID_CAT, write_output_report, "Failed to get feature report (err: {:X})", GetLastError().0);
            Err(report)
        }
    }
}

//------------------------------------------------------------------------------------------------------------------------------
// REPORTS GETTERS
//------------------------------------------------------------------------------------------------------------------------------

pub fn get_buttons(dev: &Device, collection_id: u16, report_type: ReportType, report: &[u8]) -> Option<DynArray<Usage>> {
    unsafe {
        let preparse_data = dev.preparse_data.get_address() as isize;
        let report_type = to_native_report_type(report_type);

        let mut button_count = 0;
        _ = HidP_GetUsagesEx(report_type, collection_id, null_mut(), &mut button_count, preparse_data, report);

        let mut win_buttons = DynArray::with_capacity(button_count as usize);
        
        let res = HidP_GetUsagesEx(report_type, collection_id, win_buttons.as_mut_ptr(), &mut button_count, preparse_data, report);
        if let Err(err) = res {
            log_error!(LOG_HID_CAT, get_buttons, "Failed to extract buttons from report. (error: {:X})", err.code().0);
            return None;
        }
        win_buttons.set_len(button_count as usize);

        Some(win_buttons.into_iter()
                .map(|usage_and_page| Usage::from_u16(usage_and_page.UsagePage, usage_and_page.Usage))
                .collect()
        )
    }
}

pub fn get_buttons_for_page(dev: &Device, page: UsagePageId, collection_id: u16, report_type: ReportType, report: &[u8]) -> Option<DynArray<UsageId>> {
    unsafe {
        let preparse_data = dev.preparse_data.get_address() as isize;
        let report_type = to_native_report_type(report_type);

        let mut button_count = 0;

        // Safety: HidP_GetUsages doesn't actually write to the report buffer, so this *technically* not a mutable reference
        let report_data = &mut *(report as *const _ as *mut [u8]);

        _ = HidP_GetUsages(report_type, page.as_u16(), collection_id, null_mut(), &mut button_count, preparse_data, report_data);

        let mut win_buttons = DynArray::with_capacity(button_count as usize);
        
        let res = HidP_GetUsages(report_type, page.as_u16(), collection_id, win_buttons.as_mut_ptr(), &mut button_count, preparse_data, report_data);
        if let Err(err) = res {
            log_error!(LOG_HID_CAT, get_buttons, "Failed to extract buttons from report. (error: {:X})", err.code().0);
            return None;
        }
        win_buttons.set_len(button_count as usize);

        Some(win_buttons.into_iter()
                .map(|usage| UsageId::new(usage))
                .collect()
        )
    }
}

pub fn get_raw_value(dev: &Device, usage: Usage, collection_id: u16, report_type: ReportType, report: &[u8]) -> Option<RawValue> {
    unsafe {
        let (report_count, bit_size) = get_value_report_count_and_bitsize_for_usage(dev, usage, collection_id, report_type);
        if report_count == 0 {
            log_error!(LOG_HID_CAT, get_raw_value, "Failed to find any valid report counts for the current collection id '{}'", collection_id);
            return None;
        }

        let preparse_data = dev.preparse_data.get_address() as isize;
        let report_type = to_native_report_type(report_type);

        if report_count == 1 {
            let mut value = 0;
            let res = HidP_GetUsageValue(report_type, usage.page.as_u16(), collection_id, usage.usage.as_u16(), &mut value, preparse_data, report);
            match res {
                Ok(_) => Some(RawValue::Single(value, bit_size)),
                Err(err) => {
                    log_error!(LOG_HID_CAT, get_raw_value, "Failed to get the value for a specific usage. (error: {:X})", err.code().0);
                    None
                },
            }
        } else {
            let mut values = DynArray::with_capacity(report_count as usize);
            values.set_len(report_count as usize);

            let slice = slice::from_raw_parts_mut(values.as_mut_ptr() as *mut u8, report_count as usize * mem::size_of::<u32>());

            let res = HidP_GetUsageValueArray(report_type, usage.page.as_u16(), collection_id, usage.page.as_u16(), slice, preparse_data, report);
            match res {
                Ok(_) => Some(RawValue::Array(values, bit_size)),
                Err(err) => {
                    log_error!(LOG_HID_CAT, get_raw_value, "Failed to get the value for a specific usage. (error: {:X})", err.code().0);
                    None
                },
            }
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
    unsafe {
        let preparse_data = dev.preparse_data.get_address() as isize;
        let report_type = to_native_report_type(report_type);

        let mut value = 0;
        let res = HidP_GetScaledUsageValue(report_type, usage.page.as_u16(), collection_id, usage.usage.as_u16(), &mut value, preparse_data, report);
        match res {
            Ok(_) => Some(value),
            Err(err) => {
                log_error!(LOG_HID_CAT, get_scaled_value, "Failed to get the scaled value for a specific usage. (error: {:X})", err.code().0);
                None
            },
        }
    }
}

pub fn get_data(dev: &Device, report_type: ReportType, report: &[u8]) -> Option<DynArray<Data>> {
    unsafe {
        let preparse_data = dev.preparse_data.get_address() as isize;
        let win_report_type = to_native_report_type(report_type);

        // Safety: HidP_GetUsages doesn't actually write to the report buffer, so this *technically* not a mutable reference
        let report_data = &mut *(report as *const _ as *mut [u8]);

        let scoped_alloc = ScopedAlloc::new(UseAlloc::TlsTemp);

        let mut data_len = HidP_MaxDataListLength(win_report_type, preparse_data);
        let mut data = DynArray::with_capacity(data_len as usize);
        
        drop(scoped_alloc);

        let res = HidP_GetData(win_report_type, data.as_mut_ptr(), &mut data_len, preparse_data, report_data);
        match res {
            Ok(_) => {
                data.set_len(data_len as usize);
                Some(data.into_iter()
                    .map(|data| 
                        Data {
                            index: data.DataIndex,
                            value: if is_data_button(dev, data.DataIndex, report_type) { 
                                DataValue::Button(data.Anonymous.On.as_bool())
                            } else {
                                DataValue::Value(data.Anonymous.RawValue)
                            }
                        }
                    )
                    .collect()
                )
            },
            Err(err) => {
                log_error!(LOG_HID_CAT, get_scaled_value, "Failed to set the scaled value for a specific usage. (error: {:X})", err.code().0);
                None
            },
        }
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
    unsafe {
        let preparse_data = dev.preparse_data.get_address() as isize;
        let report_type = to_native_report_type(report_type);

        let mut len = usages.len() as u32;
        let res = HidP_SetUsages(report_type, page.as_u16(), collection_id, usages.as_mut_ptr() as *mut u16, &mut len, preparse_data, report);
        if let Err(err) = res {
            log_error!(LOG_HID_CAT, get_scaled_value, "Failed to set buttons in the report. (error: {:X})", err.code().0);
        }

    }
}

pub fn unset_buttons(dev: &Device, page: UsagePageId, collection_id: u16, usages: &mut [UsageId], report_type: ReportType, report: &mut [u8]) {
    unsafe {
        let preparse_data = dev.preparse_data.get_address() as isize;
        let report_type = to_native_report_type(report_type);

        let mut len = usages.len() as u32;
        let res = HidP_UnsetUsages(report_type, page.as_u16(), collection_id, usages.as_mut_ptr() as *mut u16, &mut len, preparse_data, report);
        if let Err(err) = res {
            log_error!(LOG_HID_CAT, get_scaled_value, "Failed to unset buttons in the report. (error: {:X})", err.code().0);
        }
    }
}

pub fn set_value(dev: &Device, usage: Usage, collection_id: u16, raw_value: u32, report_type: ReportType, report: &mut [u8]) {
    unsafe {
        let preparse_data = dev.preparse_data.get_address() as isize;
        let report_type = to_native_report_type(report_type);

        let res = HidP_SetUsageValue(report_type, usage.page.as_u16(), collection_id, usage.usage.as_u16(), raw_value, preparse_data, report);
        if let Err(err) = res {
            log_error!(LOG_HID_CAT, get_scaled_value, "Failed to unset buttons in the report. (error: {:X})", err.code().0);
        }
    }
}

pub fn set_values(dev: &Device, usage: Usage, collection_id: u16, raw_values: &[u8], report_type: ReportType, report: &mut [u8]) {
    unsafe {
        let preparse_data = dev.preparse_data.get_address() as isize;
        let report_type = to_native_report_type(report_type);

        let res = HidP_SetUsageValueArray(report_type, usage.page.as_u16(), collection_id, usage.usage.as_u16(), raw_values, preparse_data, report);
        if let Err(err) = res {
            log_error!(LOG_HID_CAT, get_scaled_value, "Failed to unset buttons in the report. (error: {:X})", err.code().0);
        }
    }
}

pub fn set_data(dev: &Device, data: &[Data], report_type: ReportType, report: &mut [u8]) {
    unsafe {
        let preparse_data = dev.preparse_data.get_address() as isize;
        let report_type = to_native_report_type(report_type);

        let mut win_data = DynArray::new();
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
        let res = HidP_SetData(report_type, win_data.as_mut_ptr(), &mut len, preparse_data, report);
        if let Err(err) = res {
            log_error!(LOG_HID_CAT, get_scaled_value, "Failed to set data in the report. (error: {:X})", err.code().0);
        }
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