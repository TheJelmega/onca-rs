use core:: mem;
use onca_core::utils::is_flag_set;
use onca_logging::{log_error};
use windows::Win32::{
    UI::{
        Input::{
            KeyboardAndMouse::*,
            RAWINPUTDEVICE, RIDEV_NOHOTKEYS, RegisterRawInputDevices, RAWKEYBOARD,
        },
        WindowsAndMessaging::{RI_KEY_BREAK}
    },
    Foundation::{HWND, GetLastError},
    Devices::HumanInterfaceDevice::{HID_USAGE_PAGE_GENERIC, HID_USAGE_GENERIC_KEYBOARD}
};

use crate::{LOG_INPUT_CAT, input_devices::*};

pub struct OSKeyboard; 

impl OSKeyboard {
    pub(crate) fn new() -> Option<Self> {
        unsafe {
            let raw_input = RAWINPUTDEVICE {
                usUsagePage: HID_USAGE_PAGE_GENERIC,
                usUsage: HID_USAGE_GENERIC_KEYBOARD,
                dwFlags: RIDEV_NOHOTKEYS,
                hwndTarget: HWND::default(),
            };
            
            let raw_input_devices = [raw_input];
            let res = RegisterRawInputDevices(&raw_input_devices, mem::size_of::<RAWINPUTDEVICE>() as u32).as_bool();
            if !res {
                log_error!(LOG_INPUT_CAT, Self::new, "Failed to create a raw input device for the keyboard (err code: {}).", GetLastError().0);
                return None;
            }
            Some(Self)
        }
    }

    pub(crate) unsafe fn process_window_event(keyboard: &mut Keyboard, data: &RAWKEYBOARD) {
        let up = is_flag_set(data.Flags, RI_KEY_BREAK as u16);
        let scancode = data.MakeCode;
        let vk = VIRTUAL_KEY(data.VKey);
        let keycode = match vk_to_keycode(vk) {
            Some(keycode) => keycode,
            None => return,
        };

        // Update the keyboard using emulated keys
        if up {
            keyboard.release(keycode);
        } else {
            // As we are hardware input, we need to avoid sending new pressed messages, if the key is already down.
            if keyboard.get_key_state(keycode).is_down() {
                return;
            }

            let mut utf16 = [0u16; 8];
            let mut chars = ['\0'; 4];

            let num_chars = unsafe { ToUnicode(vk.0 as u32, scancode as u32, None, &mut utf16, 0) };
            if num_chars > 0 {
                let utf16_it = (0..num_chars).map(|i| utf16[i as usize]);
                char::decode_utf16(utf16_it)
                    .zip(chars.iter_mut())
                    .for_each(|(res, elem)| *elem = res.unwrap_or_default());
            }

            // We are the hardware keyboard, so key must stay down until either the user released the key or a release is emulated for the key
            keyboard.press_with_multi_char(keycode, f32::MAX, &chars);
        }
    }
}

pub(crate) fn vk_to_keycode(virtual_key: VIRTUAL_KEY) -> Option<KeyCode> {
    match virtual_key {
        VK_BACK      => Some(KeyCode::Backspace),
        VK_TAB       => Some(KeyCode::Tab),
        VK_CLEAR     => Some(KeyCode::Clear),
        VK_RETURN    => Some(KeyCode::Enter),
        VK_SHIFT     => Some(KeyCode::Shift),
        VK_LSHIFT    => Some(KeyCode::LShift),
        VK_RSHIFT    => Some(KeyCode::RShift),
        VK_CONTROL   => Some(KeyCode::Ctrl),
        VK_LCONTROL  => Some(KeyCode::LCtr),
        VK_RCONTROL  => Some(KeyCode::RCtr),
        VK_MENU      => Some(KeyCode::Alt),
        VK_LMENU     => Some(KeyCode::LAlt),
        VK_RMENU     => Some(KeyCode::RAlt),
        VK_PAUSE     => Some(KeyCode::Break),
        VK_ESCAPE    => Some(KeyCode::Escape),
        VK_SPACE     => Some(KeyCode::Space),
        VK_PRIOR     => Some(KeyCode::PgUp),
        VK_NEXT      => Some(KeyCode::PgDown),
        VK_END       => Some(KeyCode::End),
        VK_HOME      => Some(KeyCode::Home),
        VK_LEFT      => Some(KeyCode::Left),
        VK_UP        => Some(KeyCode::Up),
        VK_RIGHT     => Some(KeyCode::Right),
        VK_DOWN      => Some(KeyCode::Down),
        VK_SNAPSHOT  => Some(KeyCode::PrintScreen),
        VK_INSERT    => Some(KeyCode::Insert),
        VK_DELETE    => Some(KeyCode::Delete),
        VK_LWIN      => Some(KeyCode::LCommand),
        VK_RWIN      => Some(KeyCode::RCommand),
        VK_APPS      => Some(KeyCode::Menu),
        VK_NUMPAD0   => Some(KeyCode::Numpad0),
        VK_NUMPAD1   => Some(KeyCode::Numpad1),
        VK_NUMPAD2   => Some(KeyCode::Numpad2),
        VK_NUMPAD3   => Some(KeyCode::Numpad3),
        VK_NUMPAD4   => Some(KeyCode::Numpad4),
        VK_NUMPAD5   => Some(KeyCode::Numpad5),
        VK_NUMPAD6   => Some(KeyCode::Numpad6),
        VK_NUMPAD7   => Some(KeyCode::Numpad7),
        VK_NUMPAD8   => Some(KeyCode::Numpad8),
        VK_NUMPAD9   => Some(KeyCode::Numpad9),
        VK_MULTIPLY  => Some(KeyCode::NumpadMultipy),
        VK_ADD       => Some(KeyCode::NumpadAdd),
        VK_SUBTRACT  => Some(KeyCode::NumpadSubtract),
        VK_DECIMAL   => Some(KeyCode::NumpadDecimal),
        VK_DIVIDE    => Some(KeyCode::NumpadDivide),
        VK_F1        => Some(KeyCode::F1),
        VK_F2        => Some(KeyCode::F2),
        VK_F3        => Some(KeyCode::F3),
        VK_F4        => Some(KeyCode::F4),
        VK_F5        => Some(KeyCode::F5),
        VK_F6        => Some(KeyCode::F6),
        VK_F7        => Some(KeyCode::F7),
        VK_F8        => Some(KeyCode::F8),
        VK_F9        => Some(KeyCode::F9),
        VK_F10       => Some(KeyCode::F10),
        VK_F11       => Some(KeyCode::F11),
        VK_F12       => Some(KeyCode::F12),
        VK_NUMLOCK   => Some(KeyCode::NumLock),
        VK_SCROLL    => Some(KeyCode::ScrollLock),
        VK_A         => Some(KeyCode::A),
        VK_B         => Some(KeyCode::B),
        VK_C         => Some(KeyCode::C),
        VK_D         => Some(KeyCode::D),
        VK_E         => Some(KeyCode::E),
        VK_F         => Some(KeyCode::F),
        VK_G         => Some(KeyCode::G),
        VK_H         => Some(KeyCode::H),
        VK_I         => Some(KeyCode::I),
        VK_J         => Some(KeyCode::J),
        VK_K         => Some(KeyCode::K),
        VK_L         => Some(KeyCode::L),
        VK_M         => Some(KeyCode::M),
        VK_N         => Some(KeyCode::N),
        VK_O         => Some(KeyCode::O),
        VK_P         => Some(KeyCode::P),
        VK_Q         => Some(KeyCode::Q),
        VK_R         => Some(KeyCode::R),
        VK_S         => Some(KeyCode::S),
        VK_T         => Some(KeyCode::T),
        VK_U         => Some(KeyCode::U),
        VK_V         => Some(KeyCode::V),
        VK_W         => Some(KeyCode::W),
        VK_X         => Some(KeyCode::X),
        VK_Y         => Some(KeyCode::Y),
        VK_Z         => Some(KeyCode::Z),
        VK_0         => Some(KeyCode::N0),
        VK_1         => Some(KeyCode::N1),
        VK_2         => Some(KeyCode::N2),
        VK_3         => Some(KeyCode::N3),
        VK_4         => Some(KeyCode::N4),
        VK_5         => Some(KeyCode::N5),
        VK_6         => Some(KeyCode::N6),
        VK_7         => Some(KeyCode::N7),
        VK_8         => Some(KeyCode::N8),
        VK_9         => Some(KeyCode::N9),
        _ => {
            let mapped_char = unsafe {
                let hkl = GetKeyboardLayout(0);
                MapVirtualKeyExA(virtual_key.0 as u32, MAPVK_VK_TO_CHAR, hkl)
            };
            match char::from_u32(mapped_char) {
                Some(ch) => match ch {
                    '`'  => Some(KeyCode::Backtick),
                    '!'  => Some(KeyCode::Exclamation),
                    '$'  => Some(KeyCode::Dollar),
                    '^'  => Some(KeyCode::Caret),
                    '&'  => Some(KeyCode::Ampersand),
                    '*'  => Some(KeyCode::Asterisk),
                    '('  => Some(KeyCode::LParen),
                    ')'  => Some(KeyCode::RParen),
                    '-'  => Some(KeyCode::Hyphen),
                    '_'  => Some(KeyCode::Underscore),
                    '='  => Some(KeyCode::Equals),
                    '['  => Some(KeyCode::LBracket),
                    ']'  => Some(KeyCode::RBracket),
                    '\\' => Some(KeyCode::Backslash),
                    ':'  => Some(KeyCode::Colon),
                    ';'  => Some(KeyCode::Semicolon),
                    '"'  => Some(KeyCode::Quote),
                    '\'' => Some(KeyCode::Apostrophe),
                    ','  => Some(KeyCode::Comma),
                    '.'  => Some(KeyCode::Period),
                    '/'  => Some(KeyCode::Slash),
                    'é'  => Some(KeyCode::EAcute),
                    'è'  => Some(KeyCode::EGrave),
                    'à'  => Some(KeyCode::AGrave),
                    'ç'  => Some(KeyCode::CCedilla),
                    '§'  => Some(KeyCode::Section),
                    _    => None,
                },
                None => None,
            }
        },
    }
}