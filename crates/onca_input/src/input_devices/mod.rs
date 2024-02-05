use std::{ffi::c_void, ops::{Add, Sub, Mul}};
use onca_common::prelude::*;

use onca_common_macros::{flags, EnumFromIndex, EnumDisplay};
use onca_hid as hid;
use onca_logging::log_warning;
use onca_math::{SmoothStep, f32v2, MathConsts};
use crate::{os, AxisDefinition, AxisValue, Rebinder, LOG_INPUT_CAT};

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

// TODO: Move into plugin once plugin system is added
mod dualsense;
pub use dualsense::*;


//==============================================================================================================================
// COMMON
//==============================================================================================================================


/// Input device handle.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Handle {
    pub(crate) id:       u8,
    pub(crate) lifetime: u8,
}

#[flags]
pub enum GamepadFeatures {
    /// The gamepad has a touchpad
    Touch,
    /// The gamepad has a keyboard (often an accessory)
    Keyboard,
    /// The gamepad has a gyroscope
    Gyro,
    /// The gamepad has an accelerometer
    Accel
}

/// Device type
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum DeviceType {
    /// Mouse.
    Mouse,
    /// Keyboard.
    Keyboard,
    /// Gamepad.
    Gamepad(GamepadFeatures),
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
            DeviceType::Mouse => matches!(self, DeviceType::Mouse),
            DeviceType::Keyboard => matches!(self, DeviceType::Keyboard),
            DeviceType::Gamepad(features) => match self {
                DeviceType::Gamepad(self_features) => self_features.contains(*features),
                _ => false
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
pub struct AxisId(StringId);

impl AxisId {
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
    /// Ids for the axis.
    pub ids:      &'static [AxisId],
    /// Axis definition.
    pub axis: AxisDefinition,
    /// Bindings can be rebound to this axis.
    pub can_rebind: bool,
}

pub struct OutputAxisDefinition {
    /// Ids for the axis.
    pub ids:       &'static [AxisId],
    /// Axis definition
    pub axis: AxisDefinition,
}

//==============================================================================================================================
// INPUT DEVICE
//==============================================================================================================================


/// Input device
pub trait InputDevice {
    /// Tick the device, and returns the axes that need to be notified for rebinding this frame.
    fn tick(&mut self, dt: f32, rebinder: &mut Rebinder);
    
    /// Handle an input report that was sent to the device.
    fn handle_hid_input(&mut self, input_report: &[u8]);

    /// HAndle a native input event.
    fn handle_native_input(&mut self, native_data: *const c_void);
    
    /// Get the native handle
    fn get_native_handle(&self) -> &NativeDeviceHandle;

    /// Get the axis value for a given axis.
    fn get_axis_value(&self, axis: &AxisId) -> Option<AxisValue>;

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

    /// Get the device's battery info.
    /// 
    /// None is returned if the device has no battery.
    fn get_battery_info(&self) -> Option<BatteryInfo>;

    /// Get the info for the device's supported output.
    fn get_output_info<'a>(&'a self) -> &'a OutputInfo<'a>;

    /// Set the device's rumble state
    fn set_rumble(&self, rumble: RumbleState);

    /// Set the device's trigger feedback.
    fn set_trigger_feedback(&self, right_trigger: bool, trigger_feedback: TriggerFeedback);

    /// Set the state for a given LED.
    fn set_led_state(&self, index: u16, state: LedState);

    /// Set the value of an output axis.
    fn set_output_axis(&self, axis: AxisId, value: AxisValue);
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


//==============================================================================================================================
// OUTPUT
//==============================================================================================================================

/// Supported Rrumble motors.
#[flags]
pub enum RumbleSupport {
    LowFrequecy,
    HighFrequency,
    LeftTrigger,
    RightTrigger,
}

/// Supported trigger feedback modes
#[flags]
pub enum TriggerFeedbackSupport {
    Feedback,
    Weapon,
    Vibration,
}

/// LED support for the specifc led.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LedModeSupport {
    /// Led only support a binary on/off mode.
    OnOff,
    /// Led support LED intensity.
    IntensityOnly,
    /// Led support RGB colors.
    Color,
}

/// LED support info
#[derive(Clone, Copy, Debug)]
pub struct LedSupport {
    pub name: &'static str,
    pub mode: LedModeSupport,
}

/// Supported output for device
pub struct OutputInfo<'a> {
    /// Input device's rumble support
    pub rumble: RumbleSupport,
    /// Supported trigger feedback
    pub trigger_feedback: Option<TriggerFeedbackSupport>,
    /// Supported LEDs.
    pub led_support: &'a [LedSupport],
    /// Output axes.
    pub output_axes: &'a [OutputAxisDefinition],
}

/// Active rumble state
/// 
/// Unsupported 'axes' will be ignored.
/// 
/// All values are in the range (0-1) representing the percentage of full power.
#[derive(Clone, Copy, Debug, Default)]
pub struct RumbleState {
    /// Low frequency intensity.
    pub low_frequency: f32,
    /// High frequency intensity.
    pub high_frequency: f32,
    /// Left trigger intensity.
    pub left_trigger: f32,
    /// Right trigger intensity.
    pub right_trigger: f32,
}

/// Trigger feedback
/// 
/// This is not complete yet and will be continued to be worked on in the future
// TODO
#[derive(Clone, Copy, Debug, Default)]
pub enum TriggerFeedback {
    /// No trigger feedback.
    #[default]
    Off,
    /// Provide continueous feedback when the trigger is greater or equal than a given position.
    Continuous {
        /// Start trigger position for feedback.
        start:    f32,
        /// Feedback resistive strength.
        strength: f32,
    },
    /// Provide feedback when the trigger is between a given start and end position.
    Sectioned {
        /// Start trigger position for feedback.
        start:    f32,
        /// End trigger position for feedback.
        end:      f32,
        /// Feedback resistive strength.
        strength: f32,
    },
    /// Vibrate when the trigger is greater or equal than a given position.
    Vibration {
         /// Start trigger position for feedback.
         start:    f32,
        /// Vibration frequency in Hz.
        frequency: f32,
        /// Vibration strength.
        strength:  f32,
    },
}

#[derive(Clone, Copy, Debug, Default)]
/// Led state
pub enum LedState {
    /// Led is turned off.
    #[default]
    Off,
    /// Led is turned on.
    On,
    /// Led intensity.
    Intensity(f32),
    /// Color output.
    Color(f32, f32, f32),
}

//==============================================================================================================================
// CONTROLS
//==============================================================================================================================


/// How will the value (with a neutral position as 0.0) be released if emulated
#[derive(Clone, Copy, Debug, Default)]
pub enum ReleaseCurve {
    /// The stick will instantaniously return to center.
    #[default]
    Instant,
    /// The value will return to neutral in a linear way, over a given time (in seconds).
    Linear(f32),
    /// The value will return to neutral in a smooth way (smoothstep), over a given time (in seconds).
    Smooth(f32),
}

impl ReleaseCurve {
    fn interpolate<T>(&self, from: T, neutral: T, time_passed: f32) -> T where
        T: Copy + Add<Output = T> + Sub<Output = T> + Mul<f32, Output = T>
    {
        match self {
            ReleaseCurve::Instant => neutral,
            // lerping from 0, is the same as * (1 - interpolant)
            ReleaseCurve::Linear(max_time) => {
                let interp = 1f32 - time_passed / max_time;
                neutral + ((from - neutral) * interp)
            },
            // lerping from 0, is the same as * (1 - interpolant)
            ReleaseCurve::Smooth(max_time) => {
                let interp = (1f32 - time_passed / max_time).smooth_step(0.0, 1.0);
                neutral + ((from - neutral) * interp)
            }
        }
    }

    fn time_out(&self, time: f32) -> bool {
        match self {
            ReleaseCurve::Instant => true,
            ReleaseCurve::Linear(max_time) => time >= *max_time,
            ReleaseCurve::Smooth(max_time) => time >= *max_time,
        }
    }
}

#[derive(Clone, Copy, Debug)]
/// Defines a button input change
pub struct ButtonChange<T: Copy> {
    pub button:  T,
    pub time:    f32,
    pub pressed: bool,
}

impl<T: Copy> ButtonChange<T> {
    pub fn new(button: T, time: f32, pressed: bool) -> Self {
        Self { button, time, pressed }
    }
}


#[derive(Clone, Copy, Debug)]
/// Defines an axis value change
pub struct AxisMove<T: Copy>{
    pub value: T,
    pub time:  f32,
    pub curve: ReleaseCurve,
}

impl<T: Copy> AxisMove<T> {
    pub fn new(value: T, time: f32, curve: ReleaseCurve) -> Self {
        Self { value, time, curve }
    }
}

impl<T> AxisMove<T> where
    T: Copy + Add<Output = T> + Sub<Output = T> + Mul<f32, Output = T>
{
    pub fn update(&mut self, dt: f32, neutral: T) -> T {
        self.time -= dt;
        if self.time > 0f32 {
            self.value
        } else {
            let passed_time = -self.time;
            if self.curve.time_out(self.time) {
                self.value
            } else {
                self.curve.interpolate(self.value, neutral, passed_time)
            }
        }
    }
}

impl<T: Copy + Default> Default for AxisMove<T> {
    fn default() -> Self {
        Self { value: Default::default(), time: Default::default(), curve: Default::default() }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumFromIndex, EnumDisplay)]
pub enum HatSwitch {
    /// Hat switch is in the neutral position
    #[display("neutral")]
    Neutral,
    /// Hat switch points up
    #[display("up")]
    Up,
    /// Hat switch points up and right
    #[display("up-right")]
    UpRight,
    /// Hat switch points right
    #[display("right")]
    Right,
    /// Hat switch points down and right
    #[display("down-right")]
    DownRight,
    /// Hat switch points down
    #[display("down")]
    Down,
    /// Down and left are pressed
    #[display("down-left")]
    DownLeft,
    /// Hat switch points left
    #[display("left")]
    Left,
    /// Hat switch points up and left
    #[display("up-left")]
    UpLeft,
}

impl HatSwitch {
    /// Check if the down direction is down.
    /// 
    /// This is most useful in case of a 4-direction hat switch.
    pub fn is_bottom_down(&self) -> bool {
        matches!(self, HatSwitch::DownRight | HatSwitch::Down | HatSwitch::DownLeft)
    }
    
    /// Check if the right direction is down.
    /// 
    /// This is most useful in case of a 4-direction hat switch.
    pub fn is_right_down(&self) -> bool {
        matches!(self, HatSwitch::UpRight | HatSwitch::Right | HatSwitch::DownRight)
    }
    
    /// Check if the left direction is down.
    /// 
    /// This is most useful in case of a 4-direction hat switch.
    pub fn is_left_down(&self) -> bool {
        matches!(self, HatSwitch::DownLeft | HatSwitch::Left | HatSwitch::UpLeft)
    }
    
    /// Check if the up direction is down.
    /// 
    /// This is most useful in case of a 4-direction hat switch.
    pub fn is_up_down(&self) -> bool {
        matches!(self, HatSwitch::UpRight | HatSwitch::Up | HatSwitch::UpLeft)
    }

    /// Get a vector representing the current DPad direction
    pub fn get_direction(&self, normalized: bool) -> f32v2 {
        let diag = if normalized { f32::ONE_OVER_ROOT_TWO } else { 1.0 };
        match self { 
            Self::Neutral   => f32v2::new( 0f32,  0f32),
            Self::Up        => f32v2::new( 0f32,  1f32),
            Self::UpRight   => f32v2::new( diag,  diag),
            Self::Right     => f32v2::new( 1f32,  0f32),
            Self::DownRight => f32v2::new( diag, -diag),
            Self::Down      => f32v2::new( 0f32, -1f32),
            Self::DownLeft  => f32v2::new(-diag, -diag),
            Self::Left      => f32v2::new(-1f32,  0f32),
            Self::UpLeft    => f32v2::new(-diag,  diag),
        }
    }

    pub fn from_4_button(up: bool, down: bool, left: bool, right: bool) -> Self {
        match (up, down, left, right) {
            (false, false, false, false) => Self::Neutral,
            (true , false, false, false) => Self::Up,
            (true , false, false, true ) => Self::UpRight,
            (false, false, false, true ) => Self::Right,
            (false, true , false, true ) => Self::DownRight,
            (false, true , false, false) => Self::Down,
            (false, true , true , false) => Self::DownLeft,
            (false, false, true , false) => Self::Left,
            (true , false, true , false) => Self::UpLeft,
            _ => {
                log_warning!(LOG_INPUT_CAT, "Invalid 4-button hat switch state (up: {up}, down: {down}, left: {left}, right: {right})");
                Self::Neutral
            },
        }
    }

    pub fn from_8_button(up: bool, up_right: bool, right: bool, down_right: bool, down: bool, down_left: bool, left: bool, up_left: bool) -> Self {
        match (up, up_right, right, down_right, down, down_left, left, up_left) {
            (true , false, false, false, false, false, false, false) => Self::Up,
            (false, true , false, false, false, false, false, false) => Self::UpRight,
            (false, false, true , false, false, false, false, false) => Self::Right,
            (false, false, false, true , false, false, false, false) => Self::DownRight,
            (false, false, false, false, true , false, false, false) => Self::Down,
            (false, false, false, false, false, true , false, false) => Self::DownLeft,
            (false, false, false, false, false, false, true , false) => Self::Left,
            (false, false, false, false, false, false, false, true ) => Self::UpLeft,
            _ => {
                log_warning!(LOG_INPUT_CAT, "Only 1 button of an 8-button hat switch can be pressed at any time, returning to neutral");
                Self::Neutral
            }    
        }
    }
}

// ===============================================================================================================
// DEVICE INFO
// ===============================================================================================================

#[derive(Clone, Copy, Debug, EnumDisplay)]
pub enum BatteryState {
    /// The battery is at an unknown state or has an error.
    #[display("error")]
    Error,
    /// The battery is currently discharging.
    #[display("discharging")]
    Discharging,
    /// The battery is currently charging.
    #[display("charging")]
    Charging,
    /// The battery is currently not discharging or charging, but staying at a constant level.
    #[display("neutral")]
    Neutral,
    /// The battery is at an abnormal voltage.
    #[display("abnormal voltage")]
    AbnormalVoltage,
    /// The battery is at an abnormal temperature.
    #[display("abnormal temperature")]
    AbnomralTemperature,
    
}

#[derive(Clone, Copy, Debug, EnumDisplay)]
pub enum BatteryLevel {
    /// The battery is full (70%-100% capacity).
    #[display("full")]
    Full,
    /// The battery is medium (40%-70% capacity).
    #[display("medium")]
    Medium,
    /// The battery is low (10%-40% capacity).
    #[display("low")]
    Low,
    /// The battery is critical (<10% capacity).
    #[display("critical")]
    Critical
}

pub struct BatteryInfo {
    /// Current charge rate in watt-hour
    pub charge_rate:     f32,
    /// Maximum safe charge rate in watt-hour
    pub max_charge_rate: f32,
    /// Remaining battery capacity in %
    pub remaining_cap:   f32,
    /// Full battery capacity in watt-hour
    pub full_capacity:   f32,
    /// Battery state
    pub state: BatteryState,
}

impl BatteryInfo {
    /// Returns the battery state as a granular 4-state level
    pub fn level(&self) -> BatteryLevel {
        if self.remaining_cap <= 0.1 {
            BatteryLevel::Critical
        } else if self.remaining_cap <= 0.4 {
            BatteryLevel::Low
        } else if self.remaining_cap <= 0.7 {
            BatteryLevel::Medium
        } else {
            BatteryLevel::Full
        }
    }
}