use core::{
    mem,
    ffi::c_void,
};
use onca_core::{prelude::*, alloc::CoreMemTag};
use onca_hid as hid;
use windows::Win32::{
    UI::{
        WindowsAndMessaging::{RIM_INPUT, GIDC_ARRIVAL, GIDC_REMOVAL},
        Input::{GetRawInputData, HRAWINPUT, RAWINPUTHEADER, RID_INPUT, RAWINPUT, RIM_TYPEMOUSE, RIM_TYPEKEYBOARD, RIM_TYPEHID, RAWINPUTDEVICELIST, GetRawInputDeviceList, GetRawInputDeviceInfoA, RIDI_DEVICENAME, RID_DEVICE_INFO, RIDI_DEVICEINFO, RAWINPUTDEVICE, RAWINPUTDEVICE_FLAGS, RegisterRawInputDevices, RIDI_PREPARSEDDATA}
    },
    Foundation::{WPARAM, LPARAM, GetLastError, HANDLE, HWND}
};
use onca_logging::{log_warning, log_error};
#[cfg(feature = "raw_input_logging")]
use onca_logging::log_verbose;
use crate::{
    input_devices::DeviceInfo,
    InputManager, LOG_EVENT_CAT, LOG_INPUT_CAT,
};



pub(crate) mod keyboard;
pub(crate) use keyboard::OSKeyboard;

pub(crate) mod mouse;
pub(crate) use mouse::OSMouse;

const RIMTYPE_MOUSE    : u32 = RIM_TYPEMOUSE.0;
const RIMTYPE_KEYBOARD : u32 = RIM_TYPEKEYBOARD.0;
const RIMTYPE_HID      : u32 = RIM_TYPEHID.0;

pub(crate) struct OSInput {

}


impl OSInput {
    pub fn new() -> Self {
        Self {}
    }

    pub unsafe fn process_window_event(manager: &mut InputManager, event: &onca_window::RawInputEvent) {
        let _scope_tag = ScopedMemTag::new(CoreMemTag::input());
        
        match *event {
            onca_window::RawInputEvent::Input(raw_ptr) => {
                let (wparam, lparam) = unsafe { *(raw_ptr as *const (WPARAM, LPARAM)) };
                if wparam.0 & 0xFF != RIM_INPUT as usize {
                    return;
                }
                
                let hrawinput = HRAWINPUT(lparam.0);
                let header_size = mem::size_of::<RAWINPUTHEADER>() as u32;
                let mut size = header_size;
                let byte_count = GetRawInputData(hrawinput, RID_INPUT, None, &mut size, header_size);
                if byte_count == u32::MAX {
                    let err = GetLastError().0;
                    log_warning!(LOG_EVENT_CAT, "Failed to get raw input size (err: {err})");
                    return;
                }

                size = byte_count.max(mem::size_of::<RAWINPUT>() as u32);
                let mut buffer = DynArray::<u8>::with_capacity(size as usize);
                buffer.set_len(size as usize);
                let byte_count = GetRawInputData(hrawinput, RID_INPUT, Some(buffer.as_mut_ptr() as *mut c_void), &mut size, header_size);
                if byte_count == u32::MAX {
                    let err = GetLastError().0;
                    log_warning!(LOG_EVENT_CAT, "Failed to get raw input (err: {err})");
                    return;
                }
                let rawinput = &*(buffer.as_ptr() as *const RAWINPUT);

                match rawinput.header.dwType {
                    RIMTYPE_MOUSE => {
                        if let Some(mouse) = &mut manager.mouse {
                            OSMouse::process_window_event(mouse, &rawinput.data.mouse);
                        }
                    },
                    RIMTYPE_KEYBOARD => {
                        if let Some(kb) = &mut manager.keyboard {
                            OSKeyboard::process_window_event(kb, &rawinput.data.keyboard);
                        }
                    },
                    RIMTYPE_HID => {    
                        let handle = hid::DeviceHandle::new(rawinput.header.hDevice.0 as usize);  
                        if !manager.has_device(handle) {
                            
                            let mut dev_info = RID_DEVICE_INFO::default();
                            let mut dev_info_size = mem::size_of::<RID_DEVICE_INFO>() as u32;
                            let res = GetRawInputDeviceInfoA(rawinput.header.hDevice, RIDI_DEVICEINFO, Some(&mut dev_info as *mut _ as *mut c_void), &mut dev_info_size);
                            if res == u32::MAX {
                                log_warning!(LOG_INPUT_CAT, "Failed to get raw input device info (err: {:X})", GetLastError().0);
                                return;
                            };
                            
                            let iden = hid::Identifier {
                                vendor_device: hid::VendorProduct::from_u16(dev_info.Anonymous.hid.dwVendorId as u16, dev_info.Anonymous.hid.dwProductId as u16),
                                version: dev_info.Anonymous.hid.dwVersionNumber as u16,
                                usage: hid::Usage::from_u16(dev_info.Anonymous.hid.usUsagePage, dev_info.Anonymous.hid.usUsage),
                            };

                            if manager.can_create_device_for(iden) {
                                let mut preparse_size = 0u32;
                                GetRawInputDeviceInfoA(rawinput.header.hDevice, RIDI_PREPARSEDDATA, None, &mut preparse_size);
                                
                                let mut preparse_data = DynArray::with_capacity(preparse_size as usize);
                                preparse_data.set_len(preparse_size as usize);
                                let res = GetRawInputDeviceInfoA(rawinput.header.hDevice, RIDI_PREPARSEDDATA, Some(preparse_data.as_mut_ptr() as *mut c_void), &mut preparse_size);
                                if res == u32::MAX {
                                    log_warning!(LOG_EVENT_CAT, "Failed to retrieve HID preparse data. (error: {:X})", GetLastError().0);
                                    return;
                                }
                                
                                let preparse_data = hid::PreparseData::new_blob(preparse_data);
                                
                                let dev = hid::Device::new_raw(handle, preparse_data, iden);
                                match dev {
                                    Some(dev) => manager.add_device(dev),
                                    None => {
                                        log_error!(LOG_INPUT_CAT, OSInput::process_window_event, "Failed to create a new input HID device");
                                        return;
                                    },
                                }
                            }
                        }
                        
                        let raw_report = core::slice::from_raw_parts(rawinput.data.hid.bRawData.as_ptr(), rawinput.data.hid.dwSizeHid as usize);
                        manager.handle_hid_input(handle, raw_report);

                        #[cfg(feature = "raw_input_logging")]
                        log_verbose!(LOG_EVENT_CAT, "HID event (should be gamepad) with {} elements, data: {raw_report:?}", rawinput.data.hid.dwCount);
                    },
                    invalid => log_error!(LOG_EVENT_CAT, OSInput::process_window_event, "Received an invalid RAWINPUT type: {invalid}"),
                }
            },
            onca_window::RawInputEvent::DeviceChanged(raw_ptr) => {
                let (wparam, lparam) = unsafe { *(raw_ptr as *const (WPARAM, LPARAM)) };
                let handle = hid::DeviceHandle::new(lparam.0 as usize);

                if wparam.0 == GIDC_ARRIVAL as usize {
                    let win_handle = HANDLE(lparam.0 as isize);
                    if !manager.has_device(handle) {
                        
                        let mut dev_info = RID_DEVICE_INFO::default();
                        let mut dev_info_size = mem::size_of::<RID_DEVICE_INFO>() as u32;
                        let res = GetRawInputDeviceInfoA(win_handle, RIDI_DEVICEINFO, Some(&mut dev_info as *mut _ as *mut c_void), &mut dev_info_size);
                        if res == u32::MAX {
                            log_warning!(LOG_INPUT_CAT, "Failed to get raw input device info (err: {:X})", GetLastError().0);
                            return;
                        };
                        
                        let iden = hid::Identifier {
                            vendor_device: hid::VendorProduct::from_u16(dev_info.Anonymous.hid.dwVendorId as u16, dev_info.Anonymous.hid.dwProductId as u16),
                            version: dev_info.Anonymous.hid.dwVersionNumber as u16,
                            usage: hid::Usage::from_u16(dev_info.Anonymous.hid.usUsagePage, dev_info.Anonymous.hid.usUsage),
                        };

                        if manager.can_create_device_for(iden) {
                            let mut preparse_size = 0u32;
                            GetRawInputDeviceInfoA(win_handle, RIDI_PREPARSEDDATA, None, &mut preparse_size);
                            
                            let mut preparse_data = DynArray::with_capacity(preparse_size as usize);
                            preparse_data.set_len(preparse_size as usize);
                            let res = GetRawInputDeviceInfoA(win_handle, RIDI_PREPARSEDDATA, Some(preparse_data.as_mut_ptr() as *mut c_void), &mut preparse_size);
                            if res == u32::MAX {
                                log_warning!(LOG_EVENT_CAT, "Failed to retrieve HID preparse data. (error: {:X})", GetLastError().0);
                                return;
                            }
                            
                            let preparse_data = hid::PreparseData::new_blob(preparse_data);
                            
                            let dev = hid::Device::new_raw(handle, preparse_data, iden);
                            match dev {
                                Some(dev) => manager.add_device(dev),
                                None => {
                                    log_error!(LOG_INPUT_CAT, OSInput::process_window_event, "Failed to create a new input HID device");
                                    return;
                                },
                            }
                        }
                    }
                } else {
                    assert!(wparam.0 == GIDC_REMOVAL as usize);
                    manager.remove_device(handle);
                }
            },
        }   
    }

    pub fn register_device_usage(&self, usage: hid::Usage) -> bool {
        unsafe {
            let raw_input = RAWINPUTDEVICE {
                usUsagePage: usage.page.as_u16(),
                usUsage: usage.usage.as_u16(),
                dwFlags: RAWINPUTDEVICE_FLAGS(0),
                hwndTarget: HWND::default(),
            };

            let raw_input_devices = [raw_input];
            let res = RegisterRawInputDevices(&raw_input_devices, mem::size_of::<RAWINPUTDEVICE>() as u32).as_bool();
            if !res {
                log_error!(LOG_INPUT_CAT, Self::new, "Failed to create a raw input device for the mouse (err code: {}).", GetLastError().0);
                false
            } else {
                true
            }
        }
    }
}

unsafe fn get_raw_input_devices() -> DynArray<RAWINPUTDEVICELIST> {
    let _scoped_alloc = ScopedAlloc::new(UseAlloc::TlsTemp);

    let mut num_devices = 0;
    let res = GetRawInputDeviceList(None, &mut num_devices, mem::size_of::<RAWINPUTDEVICELIST>() as u32);
    if res == u32::MAX {
        log_error!(LOG_INPUT_CAT, get_raw_input_devices, "Failed to get number of raw input devices (err: {:X})", GetLastError().0);
        return DynArray::new();
    }

    let mut raw_devices = DynArray::with_capacity(num_devices as usize);
    raw_devices.set_len(num_devices as usize);
    let res = GetRawInputDeviceList(Some(raw_devices.as_mut_ptr()), &mut num_devices, mem::size_of::<RAWINPUTDEVICELIST>() as u32);
    if res == u32::MAX {
        log_error!(LOG_INPUT_CAT, get_raw_input_devices, "Failed to get raw input devices (err: {:X})", GetLastError().0);
        raw_devices.clear();
    }
    raw_devices
}

unsafe fn get_raw_device_name(handle: HANDLE) -> String {
    let mut device_name_len = 0;
    GetRawInputDeviceInfoA(handle, RIDI_DEVICENAME, None, &mut device_name_len);
    let mut device_name = String::with_capacity(device_name_len as usize);
    device_name.as_mut_dynarr().set_len(device_name_len as usize);

    let bytes_written = GetRawInputDeviceInfoA(handle, RIDI_DEVICENAME, Some(device_name.as_mut_ptr() as *mut c_void), &mut device_name_len);
    if bytes_written == u32::MAX {
        log_warning!(LOG_INPUT_CAT, "Failed to retrieve device name (err: {:X})", GetLastError().0);
    } else {
        // '- 1', as the last '\0' is included in the written length
        device_name.as_mut_dynarr().set_len(bytes_written as usize - 1);
    }
    device_name
}

pub(crate) unsafe fn get_device_infos() -> DynArray<DeviceInfo> {
    let raw_devices = get_raw_input_devices();

    let mut infos = DynArray::with_capacity(raw_devices.len());
    for raw_device in raw_devices {
        let mut dev_info = RID_DEVICE_INFO::default();
        let mut size = mem::size_of::<RID_DEVICE_INFO>() as u32;
        let res = GetRawInputDeviceInfoA(raw_device.hDevice, RIDI_DEVICEINFO, Some(&mut dev_info as *mut _ as *mut c_void), &mut size);
        if res == u32::MAX {
            log_warning!(LOG_INPUT_CAT, "Failed to get raw input device info (err: {:X})", GetLastError().0);
            continue;
        }


        infos.push(match raw_device.dwType {
            RIM_TYPEMOUSE => {
                DeviceInfo::Mouse {
                    name: get_raw_device_name(raw_device.hDevice),
                    buttons: dev_info.Anonymous.mouse.dwNumberOfButtons as u8,
                    sample_rate: dev_info.Anonymous.mouse.dwSampleRate as u16,
                    hor_scroll: dev_info.Anonymous.mouse.fHasHorizontalWheel.0 != 0,
                    
                }
            },
            RIM_TYPEKEYBOARD => {
                DeviceInfo::Keyboard {
                   name: get_raw_device_name(raw_device.hDevice),
                    func_keys: dev_info.Anonymous.keyboard.dwNumberOfFunctionKeys as u8,
                    num_keys: dev_info.Anonymous.keyboard.dwNumberOfKeysTotal as u16,
                }
            },
            RIM_TYPEHID => {
                DeviceInfo::Hid {
                    name: get_raw_device_name(raw_device.hDevice),
                    ident: hid::Identifier {
                        vendor_device: hid::VendorProduct::from_u16(dev_info.Anonymous.hid.dwVendorId as u16, dev_info.Anonymous.hid.dwProductId as u16),
                        version: dev_info.Anonymous.hid.dwVersionNumber as u16,
                        usage: hid::Usage::from_u16(dev_info.Anonymous.hid.usUsagePage, dev_info.Anonymous.hid.usUsage),
                    }
                }
            }
            _ => DeviceInfo::Unknown
        });
    }

    infos
}