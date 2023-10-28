use std::{
    mem,
    fmt::Write, collections::HashMap,
};
use onca_common::{
    prelude::*,
    sys::get_app_handle,
};

use onca_logging::{log_error, log_debug};
use windows::{
    Win32::{
        UI::WindowsAndMessaging::{
                RegisterClassExA, PeekMessageA, TranslateMessage, DispatchMessageA,
                WNDCLASSEXA, HICON, HCURSOR, MSG,
                CS_HREDRAW, CS_VREDRAW,
                PM_REMOVE, 
            },
        Foundation::{WPARAM, LPARAM, HWND, LRESULT, GetLastError},
        Graphics::Gdi::{COLOR_BACKGROUND, HBRUSH}
    },
    core::PCSTR
};

use crate::{WindowSettings, LOG_CAT};


#[derive(PartialEq, Eq, Hash)]
pub(crate) struct WndClassExKey {
    icon    : Option<isize>,
    icon_sm : Option<isize>,
}

pub(crate) struct WindowManagerData {
    wnd_classes : HashMap<WndClassExKey, (u16, u16)>,
}

impl WindowManagerData {
    pub(crate) fn new() -> Self {
        Self { wnd_classes: HashMap::new() }
    }

    pub(crate) fn register_wndclassex(&mut self, settings: &WindowSettings, wnd_proc: unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT) -> Option<u16> {
        unsafe {
            let search_key = WndClassExKey {
                icon: settings.icon().map(|ico| ico.get_os_icon().hicon().0),
                icon_sm: settings.small_icon().map(|ico| ico.get_os_icon().hicon().0),
            };
            let entry = self.wnd_classes.get_mut(&search_key);
            if let Some((class_atom, ref_count)) = entry {
                *ref_count += 1;
                return Some(*class_atom);
            }

            let mut class_name = String::new();
            let class_id = self.wnd_classes.values().len();
            let _ = write!(&mut class_name, "Win32 Class {class_id}");

            let hinstance = get_app_handle().hmodule().into();

            let hicon = settings.icon().map(|ico| ico.get_os_icon().hicon()).unwrap_or(HICON(0));
            let hicon_sm = settings.small_icon().map(|ico| ico.get_os_icon().hicon()).unwrap_or(HICON(0));

            let wndclassex =  WNDCLASSEXA {
                cbSize: mem::size_of::<WNDCLASSEXA>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(wnd_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: hinstance,
                hIcon: hicon,
                hCursor: HCURSOR(0),
                hbrBackground: HBRUSH(COLOR_BACKGROUND.0 as isize),
                lpszMenuName: PCSTR(core::ptr::null()),
                lpszClassName: PCSTR(class_name.as_ptr()),
                hIconSm: hicon_sm,
            };

            let atom = RegisterClassExA(&wndclassex);
            if atom == 0 {
                let err_code = GetLastError().map_or_else(|err| err.code().0, |_| 0);
                log_error!(LOG_CAT, Self::register_wndclassex, "Failed to create WNDCLASSEX (err: {err_code:x})");
                None
            } else {
                self.wnd_classes.insert(search_key, (atom, 1));
                log_debug!(LOG_CAT, Self::register_wndclassex, "Registered new WNDCLASSEX `{atom}`");
                Some(atom)
            }
        }
    }

    pub(crate) fn tick(&mut self) {
        unsafe {
            let mut msg = MSG::default();
            while PeekMessageA(&mut msg, HWND(0), 0, 0, PM_REMOVE).as_bool() {
                TranslateMessage(&msg);
                DispatchMessageA(&msg);
            }
        }
    }
}