use core::mem;
use onca_common::utils::is_flag_set;
use windows::Win32::{
    UI::{ 
        Input::{
            RAWINPUTDEVICE, RegisterRawInputDevices, RAWMOUSE, RAWINPUTDEVICE_FLAGS,
        },
        WindowsAndMessaging::*,
    },
    Foundation::HWND,
    Devices::HumanInterfaceDevice::{HID_USAGE_PAGE_GENERIC, HID_USAGE_GENERIC_MOUSE, MOUSE_MOVE_ABSOLUTE, MOUSE_VIRTUAL_DESKTOP}
};

use onca_logging::log_error;
use crate::{LOG_INPUT_CAT, Mouse, MouseButton, MouseScroll, MousePosition, MouseDelta};


pub(crate) struct OSMouse;

impl OSMouse {
    pub(crate) fn new() -> Option<Self> {
        let raw_input = RAWINPUTDEVICE {
            usUsagePage: HID_USAGE_PAGE_GENERIC,
            usUsage: HID_USAGE_GENERIC_MOUSE,
            dwFlags: RAWINPUTDEVICE_FLAGS(0),
            hwndTarget: HWND::default(),
        };
        
        let raw_input_devices = [raw_input];
        if let Err(err) = unsafe { RegisterRawInputDevices(&raw_input_devices, mem::size_of::<RAWINPUTDEVICE>() as u32) } {
            log_error!(LOG_INPUT_CAT, Self::new, "Failed to create a raw input device for the mouse (err code: {err}).");
            return None;
        }
        Some(Self)
    }

    pub(crate) unsafe fn process_window_event(mouse: &mut Mouse, data: &RAWMOUSE) {
        let button_flags = data.Anonymous.Anonymous.usButtonFlags as u32;
        if is_flag_set(button_flags, RI_MOUSE_LEFT_BUTTON_DOWN) {
            mouse.press_button(MouseButton::Left, f32::MAX);
        }
        if is_flag_set(button_flags, RI_MOUSE_LEFT_BUTTON_UP) {
            mouse.release_button(MouseButton::Left);
        }
        if is_flag_set(button_flags, RI_MOUSE_RIGHT_BUTTON_DOWN) {
            mouse.press_button(MouseButton::Right, f32::MAX);
        }
        if is_flag_set(button_flags, RI_MOUSE_RIGHT_BUTTON_UP) {
            mouse.release_button(MouseButton::Right);
        }
        if is_flag_set(button_flags, RI_MOUSE_MIDDLE_BUTTON_DOWN) {
            mouse.press_button(MouseButton::Middle, f32::MAX);
        }
        if is_flag_set(button_flags, RI_MOUSE_MIDDLE_BUTTON_UP) {
            mouse.release_button(MouseButton::Middle);
        }
        if is_flag_set(button_flags, RI_MOUSE_BUTTON_4_DOWN) {
            mouse.press_button(MouseButton::Side0, f32::MAX);
        }
        if is_flag_set(button_flags, RI_MOUSE_BUTTON_4_UP) {
            mouse.release_button(MouseButton::Side0);
        }
        if is_flag_set(button_flags, RI_MOUSE_BUTTON_5_DOWN) {
            mouse.press_button(MouseButton::Side1, f32::MAX);
        }
        if is_flag_set(button_flags, RI_MOUSE_BUTTON_5_UP) {
            mouse.release_button(MouseButton::Side1);
        }

        let scroll = data.Anonymous.Anonymous.usButtonData as f32 / WHEEL_DELTA as f32;
        if is_flag_set(button_flags, RI_MOUSE_WHEEL) {
            mouse.scroll_wheel(MouseScroll::new(0f32, scroll));
        }
        if is_flag_set(button_flags, RI_MOUSE_HWHEEL) {
            mouse.scroll_wheel(MouseScroll::new(scroll, 0f32));
        }

        if is_flag_set(data.usFlags as u32, MOUSE_MOVE_ABSOLUTE) {
            let is_virtual_desktop = is_flag_set(data.usFlags as u32, MOUSE_VIRTUAL_DESKTOP);

            let width = GetSystemMetrics(if is_virtual_desktop { SM_CXVIRTUALSCREEN } else { SM_CXSCREEN });
            let height = GetSystemMetrics(if is_virtual_desktop { SM_CYVIRTUALSCREEN } else { SM_CYSCREEN });

            let abs_x = ((data.lLastX as f32 / 65535f32) * width as f32).round() as i16;
            let abs_y = ((data.lLastY as f32 / 65535f32) * height as f32).round() as i16;

            mouse.set_mouse_pos(MousePosition::new(abs_x, abs_y));
        } else if data.lLastX != 0 || data.lLastY != 0 {
            mouse.move_mouse(MouseDelta::new(data.lLastX as i16, data.lLastY as i16))
        }
    }
}