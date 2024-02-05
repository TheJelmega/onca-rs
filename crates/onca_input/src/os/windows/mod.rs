use core::{
    mem,
    ffi::c_void,
};
use std::{mem::size_of, collections::HashMap, sync::Arc};
use onca_common::prelude::*;
use onca_hid as hid;
use onca_window::Window;
use windows::{Win32::{
    UI::{
        WindowsAndMessaging::{RIM_INPUT, GIDC_ARRIVAL, GIDC_REMOVAL},
        Input::{GetRawInputData, HRAWINPUT, RAWINPUTHEADER, RID_INPUT, RAWINPUT, RIM_TYPEMOUSE, RIM_TYPEKEYBOARD, RIM_TYPEHID, GetRawInputDeviceInfoA, RIDI_DEVICENAME, RID_DEVICE_INFO, RIDI_DEVICEINFO, RAWINPUTDEVICE, RegisterRawInputDevices, RIDEV_DEVNOTIFY}
    },
    Foundation::{WPARAM, LPARAM, GetLastError, HANDLE, HWND}
}, core::HRESULT};
use onca_logging::{log_warning, log_error};
#[cfg(feature = "raw_input_logging")]
use onca_logging::log_verbose;
use crate::{
    InputManager, LOG_EVENT_CAT, LOG_INPUT_CAT, NativeDeviceHandleT, NativeDeviceHandle, Handle, InputDevice,
};

pub(crate) mod keyboard;
pub(crate) use keyboard::OSKeyboard;

pub(crate) mod mouse;
pub(crate) use mouse::OSMouse;

use self::xinput::{XInputContext, XInputGamepad};

pub(crate) mod xinput;

const RIMTYPE_MOUSE    : u32 = RIM_TYPEMOUSE.0;
const RIMTYPE_KEYBOARD : u32 = RIM_TYPEKEYBOARD.0;
const RIMTYPE_HID      : u32 = RIM_TYPEHID.0;

pub fn register_input_devices(manager: &Arc<InputManager>) {
    let manager_ptr = manager.clone();

    manager.register_custom_create_device(
        |_, unique_id| unique_id.contains("IG_"), // XInput device identifier
        &[],
        move |handle| {
            XInputGamepad::new(&mut manager_ptr.get_os_input().xinput_ctx, handle).map(|x| {
                // We need to get around rust not realizing that `Box` could `CoerseUnsized` directly in a return statement
                // This could be one of those "std::boxed::Box is special" cases, as the first line clearly shows that it works
                let res: Box<dyn InputDevice> = Box::new(x);
                res
            })

        }
    );



}

pub(crate) struct OSInput {
    handle_mapping: HashMap<isize, Handle>,
    main_window:    HWND,
    xinput_ctx:     XInputContext,
}

impl OSInput {
    pub fn new(main_window: &Window) -> Result<Self, i32> {
        Ok(Self {
            handle_mapping: HashMap::new(),
            main_window: main_window.os_handle().hwnd(),
            xinput_ctx: XInputContext::new(),
        })
    }

    pub fn tick(&mut self) {
    }

    pub fn notify_device_added(&mut self, handle: Handle, native_handle: &NativeDeviceHandle) {
        self.handle_mapping.insert(native_handle.native.handle.0, handle);
    }

    pub unsafe fn process_window_event(manager: &InputManager, event: &onca_window::RawInputEvent) {
        match *event {
            onca_window::RawInputEvent::Input(raw_ptr) => {
                let (wparam, lparam) = unsafe { *(raw_ptr as *const (WPARAM, LPARAM)) };
                if wparam.0 & 0xFF != RIM_INPUT as usize {
                    return;
                }
                
                let hrawinput = HRAWINPUT(lparam.0);
                let header_size = mem::size_of::<RAWINPUTHEADER>() as u32;
                let mut size = header_size;
                let res = GetRawInputData(hrawinput, RID_INPUT, None, &mut size, header_size);
                if res == u32::MAX {
                    if let Err(err) = GetLastError() {
                        log_warning!(LOG_EVENT_CAT, "Failed to get raw input size ({err})");
                    }
                    return;
                }

                size = size.max(mem::size_of::<RAWINPUT>() as u32);
                let mut buffer = Vec::<u8>::with_capacity(size as usize);
                buffer.set_len(size as usize);
                let byte_count = GetRawInputData(hrawinput, RID_INPUT, Some(buffer.as_mut_ptr() as *mut c_void), &mut size, header_size);
                if byte_count == u32::MAX {
                    if let Err(err) = GetLastError() {
                        log_warning!(LOG_EVENT_CAT, "Failed to get raw input ({err})");
                    }
                    return;
                }
                let rawinput = &*(buffer.as_ptr() as *const RAWINPUT);

                // Just ignore hdevice 0
                if rawinput.header.hDevice == HANDLE::default() {
                    return;
                }

                let handle = match manager.get_os_input().handle_mapping.get(&rawinput.header.hDevice.0) {
                    Some(handle) => *handle,
                    None => {
                        log_warning!(LOG_EVENT_CAT, "Cannot handle input for unmapped device");
                        return;
                    },
                };

                if !manager.has_device(handle) {
                    log_warning!(LOG_EVENT_CAT, "Cannot handle input for removed device");
                    return;
                }
                
                match rawinput.header.dwType {
                    RIMTYPE_MOUSE    => manager.handle_native_input(handle, &rawinput.data.mouse as *const _ as *const _),
                    RIMTYPE_KEYBOARD => manager.handle_native_input(handle, &rawinput.data.keyboard as *const _ as *const _),
                    RIMTYPE_HID => {
                        let raw_report = core::slice::from_raw_parts(rawinput.data.hid.bRawData.as_ptr(), rawinput.data.hid.dwSizeHid as usize);
                        manager.handle_hid_input(handle, raw_report);
                    },
                    invalid => log_error!(LOG_EVENT_CAT, "Received an invalid RAWINPUT type: {invalid}"),
                }
            },
            onca_window::RawInputEvent::DeviceChanged(raw_ptr) => {
                let (wparam, lparam) = unsafe { *(raw_ptr as *const (WPARAM, LPARAM)) };
                
                if wparam.0 == GIDC_ARRIVAL as usize {
                    let native_handle = HANDLE(lparam.0);
                    let native_handle = match DeviceHandle::new(native_handle) {
                        Ok(handle) => handle,
                        Err(_) => {
                            log_error!(LOG_EVENT_CAT, "Failed to create native handle for device");
                            return;
                        },
                    };

                    let iden = native_handle.native.get_hid_identifier();
                    #[cfg(feature = "raw_input_logging")]
                    log_verbose!(LOG_EVENT_CAT, "Devices connected with id: {iden}");

                    if manager.can_create_device_for(*iden) {
                        if let Some(handle) = manager.add_device(*native_handle.native.get_hid_identifier(), native_handle) {
                            manager.get_os_input().handle_mapping.insert(lparam.0, handle);
                        }
                    }
                } else {
                    assert!(wparam.0 == GIDC_REMOVAL as usize);

                    let handle = match manager.get_os_input().handle_mapping.get(&lparam.0) {
                        Some(handle) => *handle,
                        None => {
                            log_warning!(LOG_EVENT_CAT, "Cannot remove a device that was already removed");
                            return;
                        },
                    };
                    manager.remove_device(handle);
                }
            },
        }   
    }

    pub fn register_device_usages(&self, usages: &[hid::Usage]) -> bool {
        if usages.is_empty() {
            return true;
        }

        scoped_alloc!(AllocId::TlsTemp);
        let mut raw_input_devices = Vec::new();
        for usage in usages {
            raw_input_devices.push(RAWINPUTDEVICE {
                usUsagePage: usage.page.as_u16(),
                usUsage: usage.usage.as_u16(),
                dwFlags: RIDEV_DEVNOTIFY,
                hwndTarget: self.main_window,
            });
        }

        if let Err(err) = unsafe { RegisterRawInputDevices(&raw_input_devices, mem::size_of::<RAWINPUTDEVICE>() as u32) } {
            log_error!(LOG_INPUT_CAT, "Failed to create a raw input device for the usages ({err}).");
            false
        } else {
            true
        }
    }
}

pub struct DeviceHandle {
    pub handle:   HANDLE,
    pub dev_name: String,
    pub info:     RID_DEVICE_INFO,
    pub iden:     hid::Identifier,
}

impl DeviceHandle {
    pub fn new(handle: HANDLE) -> Result<NativeDeviceHandle, HRESULT> {

        let mut name_len = 0;
        let res = unsafe { GetRawInputDeviceInfoA(handle, RIDI_DEVICENAME, None, &mut name_len) };
        if res == u32::MAX {
            // Should always return an error
            return Err(unsafe { GetLastError() }.unwrap_err().code());
        }

        let mut dev_name = String::with_capacity(name_len as usize);
        let name_len = unsafe { GetRawInputDeviceInfoA(handle, RIDI_DEVICENAME, Some(dev_name.as_mut_ptr() as *mut _), &mut name_len) };
        if name_len == u32::MAX {
            // Should always return an error
            return Err(unsafe { GetLastError() }.unwrap_err().code());
        }
        unsafe { dev_name.as_mut_vec().set_len(name_len as usize) };


        let mut info = RID_DEVICE_INFO::default();
        let mut size = size_of::<RID_DEVICE_INFO>() as u32;
        let res = unsafe { GetRawInputDeviceInfoA(handle, RIDI_DEVICEINFO, Some(&mut info as *mut _ as *mut _), &mut size) };
        if res == u32::MAX {
            // Should always return an error
            return Err(unsafe { GetLastError() }.unwrap_err().code());
        }

        let hid_dev = if info.dwType == RIM_TYPEHID {
            hid::Device::new_path(&dev_name)
        } else {
            None
        };

        let iden = match info.dwType {
            RIM_TYPEMOUSE |
            RIM_TYPEKEYBOARD => {
                // VID_ & PID_

                let vid = match dev_name.find("VID_") {
                    Some(idx) => {
                        let sub_str = &dev_name[idx + 4..idx + 8];
                        u16::from_str_radix(sub_str, 16)
                    },
                    None => return Err(HRESULT(0)),
                }.map_err(|_| HRESULT(0))?;

                let pid = match dev_name.find("PID_") {
                    Some(idx) => {
                        let sub_str = &dev_name[idx + 4..idx + 8];
                        u16::from_str_radix(sub_str, 16)
                    },
                    None => return Err(HRESULT(0)),
                }.map_err(|_| HRESULT(0))?;

                let usage = if info.dwType == RIM_TYPEMOUSE {
                    2
                } else {
                    6
                };

                hid::Identifier {
                    vendor_device: hid::VendorProduct::from_u16(vid, pid),
                    version: 0,
                    usage: hid::Usage::from_u16(1, usage),
                }
            },
            RIM_TYPEHID => match &hid_dev {
                Some(hid_dev) => *hid_dev.identifier(),
                None => {
                    let hid_info = unsafe { &info.Anonymous.hid };
                    hid::Identifier {
                        vendor_device: hid::VendorProduct::from_u16(hid_info.dwVendorId as u16, hid_info.dwProductId as u16),
                        version: hid_info.dwVersionNumber as u16,
                        usage: hid::Usage::from_u16(hid_info.usUsagePage, hid_info.usUsage),
                    }
                },
            }
            _ => unreachable!()
        };


        Ok(NativeDeviceHandle {
            native: Self {
                handle,
                dev_name,
                info,
                iden,
            },
            hid_dev,
        })
    }

    pub(crate) fn get_hid_identifier(&self) -> &hid::Identifier {
        &self.iden
    }

    pub(crate) fn get_unique_identifier(&self) -> &str {
        &self.dev_name
    }
}

impl NativeDeviceHandleT for DeviceHandle {
    fn tick(&mut self) {
        todo!()
    }

    fn get_unique_id(&self) -> &String {
        &self.dev_name
    }
}

impl PartialEq for DeviceHandle {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}