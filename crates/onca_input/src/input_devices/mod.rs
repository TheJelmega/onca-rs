use core::fmt;
use onca_core::prelude::*;
use onca_hid as hid;
use crate::{os, AxisValue, AxisType};

mod keyboard;
pub use keyboard::{KeyCode, Keyboard, KeyState};

mod mouse;
pub use mouse::{Mouse, MouseButton, MousePosition, MouseDelta, MouseScroll};

mod gamepad;
pub use gamepad::{Gamepad};

/// Input device handle
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DeviceHandle {
    Invalid,
    Mouse,
    Keyboard,
    Hid(hid::DeviceHandle)
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum GamepadSubType {
    /// Generic gamepad
    Generic,
    /// Playstation dualshock gamepad with touchpad (PS4)
    Dualshock4,
    /// Playstation dualsense gamepad (PS5)
    Dualsense,
    /// Other gamepads
    Other(String)
}

/// Device type
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum DeviceType {
    /// Mouse
    Mouse,
    /// Keyboard
    Keyboard,
    /// Gamepad device
    Gamepad(GamepadSubType),
    /// Other device
    Other(String)
}

/// Do device types match, support a common lower denominator, or neither
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum DeviceTypeMatchSupport {
    None,
    Support,
    Match,
}

impl DeviceType {
    /// Check if a device type either matches or is supported by the requested device type
    pub fn match_or_supports(&self, wanted_dev_type: &DeviceType) -> DeviceTypeMatchSupport {
        if self == wanted_dev_type {
            DeviceTypeMatchSupport::Match
        } else if match wanted_dev_type {
            DeviceType::Mouse => false,
            DeviceType::Keyboard => false,
            DeviceType::Gamepad(sub_type) => match sub_type {
                GamepadSubType::Generic => matches!(self, DeviceType::Gamepad(_)),
                GamepadSubType::Dualshock4 => matches!(self, DeviceType::Gamepad(GamepadSubType::Dualshock4) | DeviceType::Gamepad(GamepadSubType::Dualsense)),
                GamepadSubType::Dualsense => matches!(self, DeviceType::Gamepad(GamepadSubType::Dualsense)),
                GamepadSubType::Other(name) => if let DeviceType::Gamepad(GamepadSubType::Other(self_name)) = self { name == self_name } else { false },
            },
            DeviceType::Other(name) => if let DeviceType::Other(self_name) = self { name == self_name } else { false },
        } {
            DeviceTypeMatchSupport::Support
        } else {
            DeviceTypeMatchSupport::None
        }
    }
}

/// Input axis id
// TODO: Interned string
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct InputAxisId(StringId);

impl InputAxisId {
    pub const fn new(path: &str) -> Self {
        Self(StringId::new(path))
    }

    pub fn id(&self) -> StringId {
        self.0
    }

    pub fn as_string(&self) -> String {
        InternedString::from_raw_id(self.0).get()
    }
}

pub struct InputAxisDefinition {
    /// Device type
    pub dev_type  : DeviceType,
    /// Path to the axis
    pub path      : &'static str,
    /// Axis type
    pub axis_type : AxisType,
    /// Bindings can be rebound to this axis
    pub can_rebind : bool,
}

/// Input device
pub trait InputDevice {
    /// Tick the device
    fn tick(&mut self, dt: f32, notify_rebind: &mut dyn FnMut(InputAxisId));

    /// Handle an input report that was sent to the device
    fn handle_hid_input(&mut self, hid_device: &hid::Device, input_report: hid::InputReport);

    /// Get the axis value for a given axis
    fn get_axis_value(&self, axis: &InputAxisId) -> Option<AxisValue>;

    /// Get all available axes
    fn get_axes(&self) -> &[InputAxisDefinition];

    /// Get the device types, a device can represent multiple different "sub devices", e.g. keyboard with built-in touch pad
    fn get_device_type(&self) -> DeviceType;
}

#[derive(Clone)]
pub enum DeviceInfo {
    Unknown,
    Mouse {
        /// Name of the device (OS-given name, not product name)
        name        : String,
        /// Number of mouse buttons
        buttons     : u8,
        /// Sample rate of the mouse
        sample_rate : u16,
        /// Has horizontal scrollwheel
        hor_scroll  : bool,
    },
    Keyboard {
        /// Name of the device (OS-given name, not product name)
        name      : String,
        /// Number of function keys
        func_keys : u8,
        /// Total number of keys
        num_keys  : u16,
    },
    Hid {
        /// Name of the device (OS-given name, not product name)
        name : String,
        /// Vendor ID
        ident: hid::Identifier
    },
}

impl fmt::Debug for DeviceInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown => write!(f, "Unknown"),
            Self::Mouse { name, buttons, sample_rate, hor_scroll } => 
                f.debug_struct("Mouse")
                    .field("name", name)
                    .field("buttons", buttons)
                    .field("sample_rate", sample_rate)
                    .field("hor_scroll", hor_scroll)
                .finish(),
            Self::Keyboard { name, func_keys, num_keys } => 
                f.debug_struct("Keyboard")
                    .field("name", name)
                    .field("func_keys", func_keys)
                    .field("num_keys", num_keys)
                .finish(),
            Self::Hid { name, ident } => 
                f.debug_struct("Hid")
                    .field("name", name)
                    .field("identifier", &ident)
                .finish(),
        }
    }
}

impl fmt::Display for DeviceInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown => write!(f, "Unknown"),
            Self::Mouse { name, buttons, sample_rate, hor_scroll } => 
                f.debug_struct("Mouse")
                    .field("name", name)
                    .field("buttons", buttons)
                    .field("sample_rate", sample_rate)
                    .field("hor_scroll", hor_scroll)
                .finish(),
            Self::Keyboard { name, func_keys, num_keys } => 
                f.debug_struct("Keyboard")
                    .field("name", name)
                    .field("func_keys", func_keys)
                    .field("num_keys", num_keys)
                .finish(),
            Self::Hid { name, ident } => 
                f.debug_struct("Hid")
                    .field("name", name)
                    .field("identifier", &format_args!("{}", ident))
                .finish(),
        }
    }
}

pub fn get_device_infos() -> DynArray<DeviceInfo> {
    unsafe { os::get_device_infos() }
}