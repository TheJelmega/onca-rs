use core::fmt::{self, Write};
use std::ffi::c_void;
use hid::{VendorProduct, HidUsage};
use onca_common::{prelude::*, fmt::Indenter};
use onca_common_macros::flags;
use onca_hid as hid;
use crate::{os, AxisValue, AxisType};

mod keyboard;
pub use keyboard::*;

mod mouse;
pub use mouse::*;

mod gamepad;
pub use gamepad::*;

mod generic;
pub use generic::*;

mod definitions;
pub use definitions::*;


/// Input device handle.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Handle {
    pub(crate) id:       u8,
    pub(crate) lifetime: u8,
}



/// Gamepad subtype.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum GamepadSubType {
    /// Generic gamepad
    Generic,
    /// Playstation dualshock gamepad (PS4).
    /// - Supports touchpad
    Dualshock4,
    /// Playstation dualsense gamepad (PS5)
    /// - Supports touchpad
    /// - Supports trigger feedback
    Dualsense,
    /// Other gamepads.
    Other(String)
}


pub enum GamepadFeatures {

}

pub enum DeviceKind {
    /// Generic pointer device
    Pointer,
    /// Mouse support
    /// 
    /// Requires `Pointer` support
    Mouse,





    /// Custom input device
    Custom,
}

/// Device type
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum DeviceType {
    /// Mouse.
    Mouse,
    /// Keyboard.
    Keyboard,
    /// Gamepad.
    Gamepad(GamepadSubType),
    //// Touch device.
    Touch,
    /// Arcade stick.
    ArcadeStick,
    /// Flight stick.
    FlightStick,
    /// Racing wheel.
    RacingWheel,
    /// Other device.
    Other(String)
}

/// Do device types match, support a common lower denominator, or neither.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum DeviceTypeMatchSupport {
    /// Device does not match.
    None,
    /// Device is not an exact match, but a device that supports the required features.
    Support,
    /// Device is an exact match.
    Match,
}

impl DeviceType {
    /// Check if a device type either matches or is supported by the requested device type.
    pub fn match_or_supports(&self, wanted_dev_type: &DeviceType) -> DeviceTypeMatchSupport {
        if self == wanted_dev_type {
            DeviceTypeMatchSupport::Match
        } else if match wanted_dev_type {
            DeviceType::Mouse => false,
            DeviceType::Keyboard => false,
            DeviceType::Gamepad(sub_type) => match sub_type {
                GamepadSubType::Generic     => matches!(self, DeviceType::Gamepad(_)),
                GamepadSubType::Dualshock4  => matches!(self, DeviceType::Gamepad(GamepadSubType::Dualshock4) | DeviceType::Gamepad(GamepadSubType::Dualsense)),
                GamepadSubType::Dualsense   => matches!(self, DeviceType::Gamepad(GamepadSubType::Dualsense)),
                GamepadSubType::Other(name) => if let DeviceType::Gamepad(GamepadSubType::Other(self_name)) = self { name == self_name } else { false },
            },
            DeviceType::Touch => matches!(self, DeviceType::Touch),
            DeviceType::ArcadeStick => matches!(self, DeviceType::ArcadeStick),
            DeviceType::FlightStick => matches!(self, DeviceType::FlightStick),
            DeviceType::RacingWheel => matches!(self, DeviceType::RacingWheel),
            DeviceType::Other(name) => if let DeviceType::Other(self_name) = self { name == self_name } else { false }, 
        } {
            DeviceTypeMatchSupport::Support
        } else {
            DeviceTypeMatchSupport::None
        }
    }
}

/// Input axis id
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct InputAxisId(StringId);

impl InputAxisId {
    /// Create a new input axis id.
    pub const fn new(path: &str) -> Self {
        Self(StringId::new(path))
    }

    /// Get the underlying string id.
    pub fn id(&self) -> StringId {
        self.0
    }

    /// Get the input axis id as a string.
    pub fn as_string(&self) -> String {
        InternedString::from_raw_id(self.0).get()
    }
}

/// Input axis definition.
#[derive(Clone, Debug)]
pub struct InputAxisDefinition {
    /// Device type.
    pub dev_type:   DeviceType,
    /// Path to the axis.
    pub path:       &'static str,
    /// Axis type.
    pub axis_type:  AxisType,
    /// Bindings can be rebound to this axis.
    pub can_rebind: bool,
}

/// Input device
pub trait InputDevice {
    /// Tick the device.
    fn tick(&mut self, dt: f32, notify_rebind: &mut dyn FnMut(InputAxisId));
    
    /// Handle an input report that was sent to the device.
    fn handle_hid_input(&mut self, input_report: &[u8]);

    /// HAndle a native input event.
    fn handle_native_input(&mut self, native_data: *const c_void);
    
    /// Get the native handle
    fn get_native_handle(&self) -> &NativeDeviceHandle;

    /// Get the axis value for a given axis.
    fn get_axis_value(&self, axis: &InputAxisId) -> Option<AxisValue>;

    /// Get all available axes.
    fn get_axes(&self) -> &[InputAxisDefinition];

    /// Get the device types, a device can represent multiple different "sub devices", e.g. keyboard with built-in touch pad.
    fn get_device_type(&self) -> DeviceType;

    /// Destroy the device and return its native handle.
    fn take_native_handle(&mut self) -> NativeDeviceHandle;

    fn get_hid_identifier(&self) -> &hid::Identifier {
        self.get_native_handle().get_hid_identifier()
    }

    fn get_unique_identifier(&self) -> &str {
        self.get_native_handle().get_unique_identifier()
    }
}

pub trait NativeDeviceHandleT {
    fn tick(&mut self);

    fn get_unique_id(&self) -> &String;
}

pub struct NativeDeviceHandle {
    pub native: os::DeviceHandle,
    pub hid_dev: Option<hid::Device>,
}

impl NativeDeviceHandle {
    pub fn get_hid_identifier(&self) -> &hid::Identifier {
        self.native.get_hid_identifier()
    }

    pub fn get_unique_identifier(&self) -> &str {
        self.native.get_unique_identifier()
    }
}

impl PartialEq for NativeDeviceHandle {
    fn eq(&self, other: &Self) -> bool {
        self.native == other.native
    }
}


pub struct DeviceInfoNew {
    pub usafe:          hid::Usage,
    pub product_vendor: VendorProduct,
    pub name:           String,
}


#[derive(Clone)]
pub enum DeviceInfo {
    Unknown,
    Mouse {
        /// Name of the device (OS-given name, not product name)
        name:        String,
        /// Number of mouse buttons
        buttons:     u8,
        /// Sample rate of the mouse
        sample_rate: u16,
        /// Has horizontal scrollwheel
        hor_scroll:  bool,
    },
    Keyboard {
        /// Name of the device (OS-given name, not product name)
        name:      String,
        /// Number of function keys
        func_keys: u8,
        /// Total number of keys
        num_keys:  u16,
    },
    Hid {
        /// Name of the device (OS-given name, not product name)
        name:  String,
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