/// Implementation for the Dualsense controller (PS5)
/// 
/// The controller has generic controller data as expected, but also a 52 byte block with additional data, this data seems to be Litte-Endian (LE)
// Info:
// https://controllers.fandom.com/wiki/Sony_DualSense

use static_assertions as sa;

use onca_common::{prelude::*, sync::{Mutex, RwLock}, collections::BitSet};
use onca_common_macros::{flags, EnumCount, EnumFromIndex};
use onca_hid as hid;
use onca_logging::log_warning;
#[cfg(feature = "raw_input_logging")]
use onca_logging::log_verbose;
use onca_math::{f32v2, f32v3, MathConsts, Zero};
use crate::*;

const BUTTON_AXIS_MAPPING: [&[AxisId]; 15] = [
    &[Gamepad::FACE_RIGHT        , DualSense::SQUARE],
    &[Gamepad::FACE_BOTTOM       , DualSense::CROSS],
    &[Gamepad::FACE_LEFT         , DualSense::CIRCLE],
    &[Gamepad::FACE_TOP          , DualSense::TRIANGLE],
    &[Gamepad::LEFT_BUMPER       , DualSense::L1],
    &[Gamepad::RIGHT_BUMPER      , DualSense::R1],
    &[],
    &[],
    &[Gamepad::LEFT_MENU         , DualSense::CREATE],
    &[Gamepad::RIGHT_MENU        , DualSense::OPTIONS],
    &[Gamepad::LEFT_THUMB_BUTTON , DualSense::L3],
    &[Gamepad::RIGHT_THUMB_BUTTON, DualSense::R3],
    &[Gamepad::GUIDE             , DualSense::PS_BUTTON],
    &[DualSense::TOUCH_BUTTON],
    &[DualSense::MUTE],
];

const TRIGGER_AXIS_MAPPING: [&[AxisId]; 2] = [
    &[Gamepad::LEFT_THUMB , DualSense::L2],
    &[Gamepad::RIGHT_THUMB, DualSense::R2],
];


#[repr(C)]
struct RawTouchState {
    touch_and_id: u8,
    locs: [u8; 3],
}

impl RawTouchState {
    /// Touchpad represents a 1920x1080 resolution touchpad
    const TOUCHPAD_DIMENSIONS: f32v2 = f32v2 { x: 1920.0, y: 1080.0 };
     
    pub fn is_touched(&self) -> bool {
        self.touch_and_id & 0x80 == 0
    }

    pub fn id(&self) -> u8 {
        self.touch_and_id & 0x7F
    }

    // Stored in upper 12 bits of `locs`
    pub fn x(&self) -> f32 {
        let val = u16::from_le_bytes([self.locs[0], self.locs[1]])& 0x0FFF;
        val as f32 / Self::TOUCHPAD_DIMENSIONS.x
    }
    
    // Stored in lower 12 bits of `locs`
    pub fn y(&self) -> f32 {
        let val = u16::from_le_bytes([self.locs[1], self.locs[2]])>> 4;
        val as f32 / Self::TOUCHPAD_DIMENSIONS.y
    }
}



#[flags(u16)]
enum RawControllerState {
    PluggedHeadphones,
    PluggedMic,
    MicMuted,
    PluggedUsbData,
    PluggedUsbPower,
    ExternalMic = 0x0100,
    HapticLowPassFilter
}


#[repr(C, packed(1))]
struct RawInputState {
    hid_report_id:    u8,
    left_stick_x:     u8,
    left_stick_y:     u8,
    right_stick_x:    u8,
    right_stick_y:    u8,
    left_trigger:     u8,
    right_trigger:    u8,
    counter:          u8,
    button_and_hat:   [u8; 3],
    padding:          u8,
    packet_sequence:  u32,
    gyro_pitch:       i16,
    gyro_yaw:         i16,
    gyro_roll:        i16,
    accel_x:          i16,
    accel_y:          i16,
    accel_z:          i16,
    sensor_timestamp: u32,
    battery_temp:     u8,
    touch:            [RawTouchState; 2],
    unknown:          [u8; 3],
    host_timestamp:   u32,
    unknown2:         u8,
    timer2:           u32,
    battery_level:    u8,
    controller_state: RawControllerState,
    aes_cmac:         [u8; 8],
}
sa::assert_eq_size!(RawInputState, [u8; 64]);

impl RawInputState {
    unsafe fn from_raw_report(report: &[u8]) -> Self {
        assert!(report.len() == 64);
        let mut res = core::mem::zeroed();
        core::ptr::copy_nonoverlapping(report.as_ptr(), &mut res as *mut _ as *mut _, core::mem::size_of::<RawInputState>());
        res
    }
}

#[flags(u32)]
enum RawButtons {
    Square = 0x10,
    Cross,
    Circle,
    Triangle,
    L1,
    R1,
    L2,
    R2,
    Create,
    Options,
    L3,
    R3,
    Home,
    Pad,
    Mute,
    // Dualsense edge pads
    LeftFunc,
    RightFunc,
    LeftPaddle,
    RightPaddle
}

pub struct RawCalibrationData {
    gyro_pitch_bias:  i16,
    gyro_yaw_bias:    i16,
    gyro_roll_bias:   i16,
    gyro_pitch_plus:  i16,
    gyro_pitch_minus: i16,
    gyro_yaw_plus:    i16,
    gyro_yaw_minus:   i16,
    gyro_roll_plus:   i16,
    gyro_roll_minus:  i16,
    gyro_speed_plus:  i16,
    gyro_speed_min:   i16,
    accel_x_plus:     i16,
    accel_x_minus:    i16,
    accel_y_plus:     i16,
    accel_y_minus:    i16,
    accel_z_plus:     i16,
    accel_z_minus:    i16,
}

impl RawCalibrationData {
    pub fn from_report(report: &hid::FeatureReport) -> Self {
        let raw_data = report.get_raw_data();

        Self {
            gyro_pitch_bias:   i16::from_le_bytes([raw_data[1] , raw_data[2]]),
            gyro_yaw_bias:     i16::from_le_bytes([raw_data[3] , raw_data[4]]),
            gyro_roll_bias:    i16::from_le_bytes([raw_data[5] , raw_data[6]]),
            gyro_pitch_plus:   i16::from_le_bytes([raw_data[7] , raw_data[8]]),
            gyro_pitch_minus:  i16::from_le_bytes([raw_data[9] , raw_data[10]]),
            gyro_yaw_plus:     i16::from_le_bytes([raw_data[11], raw_data[12]]),
            gyro_yaw_minus:    i16::from_le_bytes([raw_data[13], raw_data[14]]),
            gyro_roll_plus:    i16::from_le_bytes([raw_data[15], raw_data[16]]),
            gyro_roll_minus:   i16::from_le_bytes([raw_data[17], raw_data[18]]),
            gyro_speed_plus:   i16::from_le_bytes([raw_data[19], raw_data[20]]),
            gyro_speed_min:    i16::from_le_bytes([raw_data[21], raw_data[22]]),
            accel_x_plus:      i16::from_le_bytes([raw_data[23], raw_data[24]]),
            accel_x_minus:     i16::from_le_bytes([raw_data[25], raw_data[26]]),
            accel_y_plus:      i16::from_le_bytes([raw_data[27], raw_data[28]]),
            accel_y_minus:     i16::from_le_bytes([raw_data[29], raw_data[30]]),
            accel_z_plus:      i16::from_le_bytes([raw_data[31], raw_data[32]]),
            accel_z_minus:     i16::from_le_bytes([raw_data[33], raw_data[34]]),
        }
    }
}

#[derive(Clone, Copy)]
pub struct IMUCalibrationData {
    bias:       i16,
    sensitivity: f32,
}

#[derive(Clone, Copy)]
pub enum CalibrationMode {
    GyroPitch,
    GyroYaw,
    GyroRoll,
    AccelX,
    AccelY,
    AccelZ,
}

pub struct CalibrartionData {
    imu: [IMUCalibrationData; 6],
}

impl CalibrartionData {
    const GYRO_RES_PER_DEGREE: f32 = 1024.0;
    const ACCEL_RES_PER_G: f32 = 8192.0;

    // Standard gravity
    const STANDARD_GRAVITY: f32 = 9.80665;

    pub fn from_raw_data(raw: RawCalibrationData) -> Self {
        let numerator = (raw.gyro_speed_plus + raw.gyro_speed_min) as f32 * Self::GYRO_RES_PER_DEGREE;

        let range_2g_x = raw.accel_x_plus - raw.accel_x_minus;
        let range_2g_y = raw.accel_y_plus - raw.accel_y_minus;
        let range_2g_z = raw.accel_z_plus - raw.accel_z_minus;

        Self {
            imu: [IMUCalibrationData {
                bias: raw.gyro_pitch_bias,
                sensitivity: numerator / (raw.gyro_pitch_plus - raw.gyro_pitch_minus) as f32,
            },
            IMUCalibrationData {
                bias: raw.gyro_yaw_bias,
                sensitivity: numerator / (raw.gyro_yaw_plus - raw.gyro_yaw_minus) as f32,
            },
            IMUCalibrationData {
                bias: raw.gyro_roll_bias,
                sensitivity: numerator / (raw.gyro_roll_plus - raw.gyro_roll_minus) as f32,
            },
            IMUCalibrationData {
                bias: raw.accel_x_plus - range_2g_x / 2,
                sensitivity: 2.0 * Self::ACCEL_RES_PER_G / range_2g_x as f32,
            },
            IMUCalibrationData {
                bias: raw.accel_y_plus - range_2g_y / 2,
                sensitivity: 2.0 * Self::ACCEL_RES_PER_G / range_2g_y as f32,
            },
            IMUCalibrationData {
                bias: raw.accel_z_plus - range_2g_z / 2,
                sensitivity: 2.0 * Self::ACCEL_RES_PER_G / range_2g_z as f32,
            }],
        }
    }

    pub fn apply(&self, mode: CalibrationMode, value: i16) -> f32 {
        let data = self.imu[mode as usize];
        let value = (value - data.bias) as f32 * data.sensitivity;

        // Convert to correct units
        match mode {
            // Convert to radians
            CalibrationMode::GyroPitch |
            CalibrationMode::GyroYaw |
            CalibrationMode::GyroRoll => value / Self::GYRO_RES_PER_DEGREE * f32::DEG_TO_RAD,
            // Convert to newtons
            CalibrationMode::AccelX |
            CalibrationMode::AccelY |
            CalibrationMode::AccelZ => value / Self::ACCEL_RES_PER_G * Self::STANDARD_GRAVITY,
        }
    }
}

// Known feature report ids
/// Calibration feature report id
const FEATURE_REPORT_ID_CALIBRATION: u8 = 5;

#[flags(u16)]
enum RawOutputFlags {
    /// Output strength should be halved
    EnableRumbleEmulation,
    /// Disabled haptics, but enabled rumble
    DisableAudioHaptics,
    AllowRightTriggerForceFeedback,
    AllowLeftTriggerForceFeedback,
    AllowHeadphoneVolume,
    AllowSpeakerVolume,
    AllowMicVolume,
    AllowAudioControl,
    AllowMuteLight,
    AllowAudioMute,
    AllowLedColor,
    /// Release the LEDs from wireless firmware control, when in wireless mode, this must be signalled to control.
    /// 
    /// This cannot be applied during BT pairing operation.
    /// SDL seems to waits for sensor timestamp to be >= 10_200_000 before pulsing this bit once
    ResetLights,
    AllowPlayerIndicators,
    AllowHapticLowPassFilter,
    AllowMotorPowerLevel,
    AllowAudioControl2,
}

#[derive(Clone, Copy, Debug)]
pub enum RawMicSelect {
    Auto,
    InternalOnly,
    ExternalOnly,
    Both,
}

// Not sure what any of these actually are
pub enum RawOutputPathSelect {
    LRX,
    LLX,
    LLR,
    XXR,
}

/// Asr is likely "Automatic Speech Recognition"
pub enum RawInputPathSelect {
    ChatAsr,
    ChatChat,
    AsrAsr,
}

#[derive(Default)]
pub enum RawMuteLight {
    #[default]
    Off,
    On,
    Breathing,
}

#[derive(Default)]
#[repr(transparent)]
pub struct RawAudioControl(u8);

#[allow(unused)]
impl RawAudioControl {
    fn mic_select(&self) -> RawMicSelect {
        unsafe { core::mem::transmute(self.0 & 0x03) }
    }

    fn set_mic_select(&mut self, val: RawMicSelect) {
        self.0 &= 0xFC;
        self.0 |= val as u8;
    }

    fn echo_cancel(&self) -> bool {
        self.0 & 0x04 != 0
    }

    fn set_echo_select(&mut self, enabled: bool) {
        self.0 &= 0xFB;
        self.0 |= (enabled as u8) << 2;
    }

    fn noise_cancel(&self) -> bool {
        self.0 & 0x08 != 0
    }

    fn set_noise_cancel(&mut self, enabled: bool) {
        self.0 &= 0xF7;
        self.0 |= (enabled as u8) << 3;
    }

    fn output_path_select(&self) -> RawOutputPathSelect {
        let val = (self.0 >> 4) & 0x3;
        unsafe { core::mem::transmute(val) }
    }

    fn set_output_path_select(&mut self, path_select: RawOutputPathSelect) {
        self.0 &= 0xCF;
        self.0 = unsafe { core::mem::transmute::<_, u8>(path_select) } << 4;
    }

    fn input_path_select(&self) -> RawInputPathSelect {
        let val = (self.0 >> 6) & 0x3;
        unsafe { core::mem::transmute(val) }
    }

    fn set_input_path_select(&mut self, path_select: RawOutputPathSelect) {
        self.0 &= 0x3F;
        self.0 = unsafe { core::mem::transmute::<_, u8>(path_select) } << 6;
    }
}

#[flags]
pub enum MuteControl {
    TouchPowerSave,
    MotionPowerSave,
    HapticPowerSave,
    AudioPowerSave,
    MicMute,
    SpeakerMute,
    HeadphoneMute,
    HapticMute,
}

#[derive(Default)]
#[repr(transparent)]
pub struct RawMotorPowerLevel(u8);

impl RawMotorPowerLevel {
    pub fn trigger_motor_power_reduction(&self) -> f32 {
        let ival = self.0 & 0xF;
        0.125 + ival as f32 * 0.125
    }

    pub fn set_trigger_motor_power_reduction(&mut self, val: f32) {
        debug_assert!(val >= 0.0 && val <= 1.0);
        self.0 &= 0xF0;
        let ival = (val - 0.125).max(0.0) / 0.125;
        self.0 |= ival as u8;
    }
    
    pub fn rumble_motor_power_reduction(&self) -> f32 {
        let ival = self.0 & 0xF;
        0.125 + ival as f32 * 0.125
    }

    pub fn set_rumble_motor_power_reduction(&mut self, val: f32) {
        debug_assert!(val >= 0.0 && val <= 1.0);
        self.0 &= 0xF0;
        let ival = (val - 0.125).max(0.0) / 0.125;
        self.0 |= ival as u8;
    }
}

#[derive(Default)]
struct RawTriggerForceFeedback([u8; 11]);

#[derive(Default)]
struct RawAudioControl2(u8);

#[allow(unused)]
impl RawAudioControl2 {
    fn speaker_comp_pre_gain(&self) -> u8 {
        self.0 & 0x07
    }

    fn set_speaker_comp_pre_gain(&mut self, gain: u8) {
        self.0 &= 0xF8;
        self.0 |= gain &0x07;
    }

    fn beamforming_enabled(&self) -> bool {
        self.0 & 0x08 != 0
    }

    fn set_beamforming_enabled(&mut self, enabled: bool) {
        self.0 &= 0xF7;
        self.0 |= (enabled as u8) << 3;
    }
}

#[flags]
pub enum RawLedFlags {
    AllowLightBrightnessChange,
    AllowCOlorLightFadeAnimation,
    EnableImprovedRumbleEmulation,
}

#[flags]
pub enum RawPassFilter {
    HapticLowPassFilter,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum LightBrightness {
    #[default]
    Bright,
    Dim,
    Mid
}

#[derive(Default)]
pub enum LightFadeAnimation {
    #[default]
    Nothing,
    FadeIn,
    FadeOut,
}

#[flags]
pub enum PlayerLight {
    // x---- or x---x (depending on hardware revision)
    Light1,
    // -x--- or -x-x-
    Light2,
    // --x--
    Light3,
    // ---x- or -x-x-
    Light4,
    // ----x or x---x
    Light5,
    DontFade,
}

#[derive(Default)]
#[repr(C, packed(1))]
struct RawOutputState {
    report_id:            u8,
    flags:                RawOutputFlags,
    /// Emulated right rumble
    rumble_right:         u8,
    /// Emulated left rumble
    rumble_left:          u8,
    /// Maximum of 0x1F
    volume_headphone:     u8,
    /// Seems to be in range 0x3D-0x64
    volume_speaker:       u8,
    /// Not linear, range [0-64], with 0 not muted.
    volume_mic:           u8,
    audio_control:        RawAudioControl,
    mute_light_mode:      RawMuteLight,
    mute_control:         MuteControl,
    right_trigger_ffb:    RawTriggerForceFeedback,
    left_trigger_ffb:     RawTriggerForceFeedback,
    host_timestamp:       u32,
    power_reduction:      RawMotorPowerLevel,
    audio_control2:       RawAudioControl2,
    led_flags:            RawLedFlags,
    haptic_filter:        RawPassFilter,
    unknown:              u8,
    light_fade_animation: LightFadeAnimation,
    light_brighness:      LightBrightness,
    player_light:         PlayerLight,
    led_red:              u8,
    led_green:            u8,
    led_blue:             u8,
}
sa::assert_eq_size!(RawOutputState, [u8; 48]);

impl RawOutputState {
    const OUTPUT_REPORT_ID: u8 = 2;

    pub fn new() -> Self {
        Self {
            report_id: Self::OUTPUT_REPORT_ID,
            ..Default::default()
        }
    }
}


#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumCount, EnumFromIndex)]
pub enum DualsenseButton {
    Square,
    Cross,
    Circle,
    Triangle,
    L1,
    R1,
    L2,
    R2,
    Create,
    Options,
    L3,
    R3,
    PsButton,
    Touchpad,
    Mute,
    LeftFunc,
    RightFunc,
    LeftPaddle,
    RightPaddle,
}
const NUM_BUTTONS_BITS: usize = DualsenseButton::COUNT.next_power_of_two();

struct DualsenseChangeState {
    buttons:  Vec<ButtonChange<DualsenseButton>>,
    dpad:     (HatSwitch, f32),
    sticks:   [AxisMove<f32v2>; 2],
    triggers: [AxisMove<f32>; 2],
    touch:    [Option<TouchState>; 2],
    angular:  f32v3,
    accel:    f32v3,
    battery:  u8,
}

impl DualsenseChangeState {
    pub fn new() -> Self {
        Self {
            buttons: Vec::new(),
            dpad: (HatSwitch::Neutral, 0.0),
            sticks: Default::default(),
            triggers: Default::default(),
            touch: Default::default(),
            angular: Default::default(),
            accel: Default::default(),
            battery: Default::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct TouchState {
    pos: f32v2,
    _id:  u8,
}

pub struct DualsenseInputState {
    buttons:  BitSet<NUM_BUTTONS_BITS>,
    dpad:     HatSwitch,
    sticks:   [f32v2; 2],
    triggers: [f32; 2],
    touch:    [Option<TouchState>; 2],
    angular:  f32v3,
    accel:    f32v3,
    battery:  u8,
}

impl DualsenseInputState {
    pub fn new() -> Self {
        Self {
            buttons: BitSet::new(),
            dpad: HatSwitch::Neutral,
            sticks: Default::default(),
            triggers: Default::default(),
            touch: Default::default(),
            angular: Default::default(),
            accel: Default::default(),
            battery: Default::default(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct DualsenseOutputState {
    rumble:    RumbleState,
    trigger_feedback: [TriggerFeedback; 2],

    ring_led:  LedState,
    mute_led:  LedState,
    player_id: u8,
}

impl DualsenseOutputState {
    pub fn new() -> Self {
        Self {
            rumble: Default::default(),
            trigger_feedback: Default::default(),
            ring_led: LedState::default(),
            mute_led: LedState::default(),
            player_id: 0
        }
    }
}

/// Dualsense controller (PS5)
/// 
/// this will likely be moved into a plugin in the future
pub struct DualSense {
    handle:        Option<NativeDeviceHandle>,

    // Input
    state:         RwLock<DualsenseInputState>,
    changes:       Mutex<DualsenseChangeState>,
    button_timers: [f32; DualsenseButton::COUNT],
    
    // Output
    out_state:     Mutex<DualsenseOutputState>,

    // Info
    calibration:   CalibrartionData,
}

impl DualSense {
    // Input
    pub const LEFT_THUMB:    AxisId = AxisId::new("Dualsense Left Thumbstick 2D-Axis");
    pub const LEFT_THUMB_X:  AxisId = AxisId::new("Dualsense Left Thumbstick X-Axis");
    pub const LEFT_THUMB_Y:  AxisId = AxisId::new("Dualsense Left Thumbstick Y-Axis");
    pub const RIGHT_THUMB:   AxisId = AxisId::new("Dualsense Right Thumbstick 2D-Axis");
    pub const RIGHT_THUMB_X: AxisId = AxisId::new("Dualsense Right Thumbstick X-Axis");
    pub const RIGHT_THUMB_Y: AxisId = AxisId::new("Dualsense Right Thumbstick Y-Axis");

    pub const DPAD_DIR:      AxisId = AxisId::new("Dualsense D-pad Direction");
    pub const DPAD_UP:       AxisId = AxisId::new("Dualsense D-pad Up");
    pub const DPAD_DOWN:     AxisId = AxisId::new("Dualsense D-pad Down");
    pub const DPAD_LEFT:     AxisId = AxisId::new("Dualsense D-pad Left");
    pub const DPAD_RIGHT:    AxisId = AxisId::new("Dualsense D-pad Right");

    pub const SQUARE:        AxisId = AxisId::new("Dualsense Square");
    pub const CROSS:         AxisId = AxisId::new("Dualsense Cross");
    pub const CIRCLE:        AxisId = AxisId::new("Dualsense Circle");
    pub const TRIANGLE:      AxisId = AxisId::new("Dualsense Triangle");
    pub const L1:            AxisId = AxisId::new("Dualsense L1");
    pub const R1:            AxisId = AxisId::new("Dualsense R1");
    pub const L2:            AxisId = AxisId::new("Dualsense L2");
    pub const R2:            AxisId = AxisId::new("Dualsense R2");
    pub const CREATE:        AxisId = AxisId::new("Dualsense Create");
    pub const OPTIONS:       AxisId = AxisId::new("Dualsense Options");
    pub const L3:            AxisId = AxisId::new("Dualsense L3");
    pub const R3:            AxisId = AxisId::new("Dualsense R3");
    pub const PS_BUTTON:     AxisId = AxisId::new("Dualsense Playstation Button");
    pub const TOUCH_BUTTON:  AxisId = AxisId::new("Dualsense Touch Button");
    pub const MUTE:          AxisId = AxisId::new("Dualsense Mute");

    pub const LEFT_TRIGGER:  AxisId = AxisId::new("Dualsense Left Trigger");
    pub const RIGHT_TRIGGER: AxisId = AxisId::new("Dualsense Right Trigger");

    pub const TOUCH_0:       AxisId = AxisId::new("Dualsense Touch 0");
    pub const TOUCH_0_X:     AxisId = AxisId::new("Dualsense Touch 0 X");
    pub const TOUCH_0_Y:     AxisId = AxisId::new("Dualsense Touch 0 Y");

    pub const TOUCH_1:       AxisId = AxisId::new("Dualsense Touch 1");
    pub const TOUCH_1_X:     AxisId = AxisId::new("Dualsense Touch 1 X");
    pub const TOUCH_1_Y:     AxisId = AxisId::new("Dualsense Touch 1 Y");

    pub const GYRO:          AxisId = AxisId::new("Dualsense Gyro");
    pub const GYRO_PITCH:    AxisId = AxisId::new("Dualsense Gyro Pitch");
    pub const GYRO_YAW:      AxisId = AxisId::new("Dualsense Gyro Yaw");
    pub const GYRO_ROLL:     AxisId = AxisId::new("Dualsense Gyro Roll");

    pub const ACCEL:         AxisId = AxisId::new("Dualsense Accel");
    pub const ACCEL_X:       AxisId = AxisId::new("Dualsense Accel X");
    pub const ACCEL_Y:       AxisId = AxisId::new("Dualsense Accel Y");
    pub const ACCEL_Z:       AxisId = AxisId::new("Dualsense Accel Z");

    // Output
    pub const OUT_PLAYER_INDICATOR: AxisId = AxisId::new("Dualsense Player Indicator");

    pub fn new(mut handle: NativeDeviceHandle) -> Result<Self, NativeDeviceHandle> {
        let hid_dev = match handle.hid_dev.as_mut() {
            Some(hid_dev) => hid_dev,
            None => return Err(handle),
        };
        let calibration_report = match hid_dev.get_feature_report(FEATURE_REPORT_ID_CALIBRATION) {
            Some(report) => report,
            None => return Err(handle),
        };
        let raw_calibration = RawCalibrationData::from_report(&calibration_report);
        let calibration = CalibrartionData::from_raw_data(raw_calibration);


        Ok(Self {
            handle: Some(handle),
            state: RwLock::new(DualsenseInputState::new()),
            changes: Mutex::new(DualsenseChangeState::new()),
            button_timers: [0.0; DualsenseButton::COUNT],
            out_state: Mutex::new(DualsenseOutputState::new()),
            calibration,
        })
    }

    fn  set_button(&self, button: DualsenseButton, time: f32, pressed: bool) {
        self.changes.lock().buttons.push(ButtonChange { button, time, pressed })
    }

    fn move_dpad(&self, dir: HatSwitch, time: f32) {
        self.changes.lock().dpad = (dir, time);
    }

    pub fn move_stick(&self, right: bool, pos: f32v2, time: f32, curve: ReleaseCurve) {
        self.changes.lock().sticks[right as usize] = AxisMove::new(pos, time, curve);
    }

    pub fn move_trigger(&self, right: bool, val: f32, time: f32, curve: ReleaseCurve) {
        self.changes.lock().triggers[right as usize] = AxisMove::new(val, time, curve);
    }

    pub fn move_touch(&self, is_touch_2: bool, pos: f32v2, id: u8) {
        self.changes.lock().touch[is_touch_2 as usize] = Some(TouchState { pos, _id: id });
    }

    pub fn set_angular_velocity(&self, vel: f32v3) {
        self.changes.lock().angular = vel;
    }

    pub fn set_acceleration(&self, vel: f32v3) {
        self.changes.lock().accel = vel;
    }

    fn update_input(&mut self, dt: f32, rebinder: &mut Rebinder) {
        let mut state = self.state.write();
        let mut changes = self.changes.lock();

        scoped_alloc!(AllocId::TlsTemp);

        // We generally only care about the last action of a button, as a press and release should not happen in a single frame.
        // While this is possible, especially at a lower framerate, it doesn't make much sense in terms of the input system.
        let mut processed_buttons = BitSet::<{DualsenseButton::COUNT}>::new();
        for change in changes.buttons.iter().rev() {
            let button_idx = change.button as usize;
            if processed_buttons.get(button_idx) {
                continue;
            }

            if change.pressed {
                rebinder.notify(BUTTON_AXIS_MAPPING[button_idx]);
            }

            #[cfg(feature = "raw_input_logging")]
            if state.buttons.get(button_idx) != change.pressed {
                log_verbose!(LOG_INPUT_CAT, "Gamepad button {:?} {}", change.button, if change.pressed {"pressed"} else {"released"});
            }

            state.buttons.set(button_idx, change.pressed);
            self.button_timers[button_idx] = change.time;
            processed_buttons.enable(button_idx);
        }
        changes.buttons.clear();

        // Handle timers
        for (idx, timer) in self.button_timers.iter_mut().enumerate() {
            *timer = (*timer - dt).max(0f32);
            if *timer == 0f32 {
                state.buttons.disable(idx);
            }
        }

        for i in 0..state.sticks.len() {
            let stick = changes.sticks[i].update(dt, f32v2::zero());
            #[cfg(feature = "raw_input_logging")]
            if state.sticks[i].dist_sq(stick) > 0.0001 && (state.sticks[i].x.abs() > 0.1 || state.sticks[i].y.abs() > 0.1) {
                log_verbose!(LOG_INPUT_CAT, "{} stick moved to ({}, {})", if i == 0 { "Left" } else { "Right" }, stick.x, stick.y);
            }
            state.sticks[i] = stick;
        }
        
        for i in 0..state.triggers.len() {
            let trigger = changes.triggers[i].update(dt, 0.0);
            if trigger > 0.5 {
                rebinder.notify(TRIGGER_AXIS_MAPPING[i]);
            }

            #[cfg(feature = "raw_input_logging")]
            if f32::abs(state.triggers[i] * state.triggers[i] - trigger * trigger) > 0.0001 {
                log_verbose!(LOG_INPUT_CAT, "{} trigger moved to {trigger}", if i == 0 { "Left" } else { "Right" });
            }
            state.triggers[i] = trigger;
        }

        // Return dpad back to neutral when it time runs out, otherwise just assign it and update the time
        let (dpad, dpad_time) = &mut changes.dpad;
        if *dpad_time == 0f32 {
            state.dpad = HatSwitch::Neutral;
        } else {
            #[cfg(feature = "raw_input_logging")]
            if state.dpad != *dpad {
                log_verbose!(LOG_INPUT_CAT, "Dpad moved to {dpad}");
            }

            state.dpad = *dpad;
            *dpad_time = (*dpad_time - dt).max(0f32);

            if dpad.is_up_down() {
                rebinder.notify(&[Gamepad::DPAD_UP, Self::DPAD_UP]);
            }
            else if dpad.is_bottom_down() {
                rebinder.notify(&[Gamepad::DPAD_DOWN, Self::DPAD_DOWN]);
            }
            if dpad.is_left_down() {
                rebinder.notify(&[Gamepad::DPAD_LEFT, Self::DPAD_LEFT]);
            }
            else if dpad.is_right_down() {
                rebinder.notify(&[Gamepad::DPAD_RIGHT, Self::DPAD_RIGHT]);
            }
        }

        // Touch
        let mut touch_id = 0;
        for i in 0..state.touch.len() {
            let touch = changes.touch[i];

            #[cfg(feature = "raw_input_logging")]
            if let Some(touch) = touch {
                if let Some(cur) = state.touch[touch_id] {
                    if touch.id != cur.id || (touch.pos.x - cur.pos.x).abs() > 0.01 || (touch.pos.y - cur.pos.y).abs() > 0.01 {
                        log_verbose!(LOG_INPUT_CAT, "Touch {} id: {}, pos: ({}, {})", touch_id + 1, touch.id, touch.pos.x, touch.pos.y);
                    }
                }
            }

            state.touch[touch_id] = touch;
            touch_id += touch.is_some() as usize;
        }

        // Sensors
        #[cfg(feature = "sensor_logging")]
        if state.angular.dist_sq(changes.angular) > 0.0001 {
            log_verbose!(LOG_INPUT_CAT, "Angular velocity: ({}, {}, {})", changes.angular.x, changes.angular.y, changes.angular.z);
        }
        state.angular = changes.angular;

        #[cfg(feature = "sensor_logging")]
        if state.accel.dist_sq(changes.accel) > 0.01 {
            log_verbose!(LOG_INPUT_CAT, "Acceleration: ({:+07.3}, {:+07.3}, {:+07.3})", changes.accel.x, changes.accel.y, changes.accel.z);
        }
        state.accel = changes.accel;
    }

    fn update_ouput(&mut self) {
        let mut state = self.out_state.lock();

        state.ring_led = LedState::Color(0.5, 0.0, 1.0);

        // Update

        // Write to controller
        let mut raw_state = RawOutputState::new();

        let mut flags = RawOutputFlags::AllowMuteLight | RawOutputFlags::AllowLedColor;

        // We don't support trigger rumble, so ignore them
        if state.rumble.low_frequency != 0.0 || state.rumble.high_frequency != 0.0 {
            flags |= RawOutputFlags::EnableRumbleEmulation | RawOutputFlags::DisableAudioHaptics;
            
            // Halve to match xbox
            raw_state.rumble_left = (state.rumble.low_frequency * 128.0) as u8;
            raw_state.rumble_right = (state.rumble.high_frequency * 128.0) as u8;
        }

        // Trigger feedback
        raw_state.left_trigger_ffb = Self::get_trigger_feedback(state.trigger_feedback[0], &mut flags);
        raw_state.right_trigger_ffb = Self::get_trigger_feedback(state.trigger_feedback[1], &mut flags);

        match state.mute_led {
            LedState::Off => raw_state.mute_light_mode = RawMuteLight::Off,
            LedState::On => raw_state.mute_light_mode = RawMuteLight::On,
            LedState::Intensity(val) => raw_state.mute_light_mode = if val > 0.5 { RawMuteLight::On } else { RawMuteLight::Off },
            LedState::Color(_, _, _) => (),
        }

        match state.ring_led {
            LedState::Off => {
                raw_state.led_red = 0;
                raw_state.led_green = 0;
                raw_state.led_blue = 0;
            },
            LedState::On => {
                raw_state.led_red = 255;
                raw_state.led_green = 255;
                raw_state.led_blue = 255;
            },
            LedState::Intensity(intensity) => {
                raw_state.led_red = (intensity * 255.0) as u8;
                raw_state.led_green = (intensity * 255.0) as u8;
                raw_state.led_blue = (intensity * 255.0) as u8;
            },
            LedState::Color(r, g, b) => {
                raw_state.led_red = (r * 255.0) as u8;
                raw_state.led_green = (g * 255.0) as u8;
                raw_state.led_blue = (b * 255.0) as u8;
            },
        }

        raw_state.player_light = match state.player_id {
            1 => PlayerLight::Light3,
            2 => PlayerLight::Light2 | PlayerLight::Light4,
            3 => PlayerLight::Light1 | PlayerLight::Light3 | PlayerLight::Light5,
            4 => PlayerLight::Light1 | PlayerLight::Light2 | PlayerLight::Light4 | PlayerLight::Light5,
            _ => PlayerLight::None,
        };
        flags |= RawOutputFlags::AllowPlayerIndicators;


        raw_state.flags = flags;

        let handle = self.handle.as_mut().unwrap();
        let hid_dev = handle.hid_dev.as_mut().unwrap();
        let raw_state_slice = unsafe { core::slice::from_raw_parts(&raw_state as *const _ as *const u8, core::mem::size_of::<RawOutputState>()) };
        let report = unsafe { hid::OutputReport::from_raw_slice(raw_state_slice, hid_dev) };
        _ = hid_dev.write_output_report(report);
    }

    // TODO: not all modes are implemented yet
    fn get_trigger_feedback(trigger_feedback: TriggerFeedback, flags: &mut RawOutputFlags) -> RawTriggerForceFeedback {
        match trigger_feedback {
            TriggerFeedback::Off => RawTriggerForceFeedback([0; 11]),
            TriggerFeedback::Continuous { start, strength } => {
                *flags |= RawOutputFlags::AllowLeftTriggerForceFeedback;
                RawTriggerForceFeedback([
                    1,
                    (start * 255.0) as u8,
                    (strength * 255.0) as u8,
                    0, 0, 0, 0, 0, 0, 0, 0
                ])
            },
            TriggerFeedback::Sectioned { start, end, strength } => {
                *flags |= RawOutputFlags::AllowLeftTriggerForceFeedback;

                let start = start.clamp(0.0, 1.0);
                let end = end.clamp(start, 1.0);
                let strength = strength.clamp(0.0, 1.0);
                RawTriggerForceFeedback([
                    1,
                    (start * 255.0) as u8,
                    (end * 255.0) as u8,
                    (strength * 255.0) as u8,
                    0, 0, 0, 0, 0, 0, 0
                ])
            },
            TriggerFeedback::Vibration { start, frequency, strength } => {
                *flags |= RawOutputFlags::AllowLeftTriggerForceFeedback;

                let start = start.clamp(0.0, 1.0);
                let frequency = frequency.clamp(0.0, 255.0);
                let strength = strength.clamp(0.0, 1.0);

                RawTriggerForceFeedback([
                    6,
                    frequency as u8,
                    (strength * 128.0) as u8, // Strength above 128 seems to be the same no matter the value
                    (start * 255.0) as u8,
                    0, 0, 0, 0, 0, 0, 0
                ])
            },
        }

    }
}

impl InputDevice for DualSense {
    fn tick(&mut self, dt: f32, rebinder: &mut Rebinder) {
        self.update_input(dt, rebinder);
        self.update_ouput();
    }

    fn handle_hid_input(&mut self, input_report: &[u8]) {
        scoped_alloc!(AllocId::TlsTemp);

        let raw_state = unsafe { RawInputState::from_raw_report(input_report) };

        let hid_dev = self.get_native_handle().hid_dev.as_ref().unwrap();
        let input_report = unsafe { hid::InputReport::from_raw_slice(input_report, hid_dev) };

        let buttons = u32::from_le_bytes([raw_state.button_and_hat[0], raw_state.button_and_hat[1], raw_state.button_and_hat[2], 0]);
        let buttons = RawButtons::new(buttons);
        self.set_button(DualsenseButton::Square      , f32::MAX, buttons.contains(RawButtons::Square));
        self.set_button(DualsenseButton::Cross       , f32::MAX, buttons.contains(RawButtons::Cross));
        self.set_button(DualsenseButton::Circle      , f32::MAX, buttons.contains(RawButtons::Circle));
        self.set_button(DualsenseButton::Triangle    , f32::MAX, buttons.contains(RawButtons::Triangle));
        self.set_button(DualsenseButton::L1          , f32::MAX, buttons.contains(RawButtons::L1));
        self.set_button(DualsenseButton::R1          , f32::MAX, buttons.contains(RawButtons::R1));
        self.set_button(DualsenseButton::L2          , f32::MAX, buttons.contains(RawButtons::L2));
        self.set_button(DualsenseButton::R2          , f32::MAX, buttons.contains(RawButtons::R2));
        self.set_button(DualsenseButton::Create      , f32::MAX, buttons.contains(RawButtons::Create));
        self.set_button(DualsenseButton::Options     , f32::MAX, buttons.contains(RawButtons::Options));
        self.set_button(DualsenseButton::L3          , f32::MAX, buttons.contains(RawButtons::L3));
        self.set_button(DualsenseButton::R3          , f32::MAX, buttons.contains(RawButtons::R3));
        self.set_button(DualsenseButton::PsButton    , f32::MAX, buttons.contains(RawButtons::Home));
        self.set_button(DualsenseButton::Touchpad    , f32::MAX, buttons.contains(RawButtons::Pad));
        self.set_button(DualsenseButton::Mute        , f32::MAX, buttons.contains(RawButtons::Mute));
        self.set_button(DualsenseButton::LeftFunc    , f32::MAX, buttons.contains(RawButtons::LeftFunc));
        self.set_button(DualsenseButton::RightFunc   , f32::MAX, buttons.contains(RawButtons::RightFunc));
        self.set_button(DualsenseButton::LeftPaddle  , f32::MAX, buttons.contains(RawButtons::LeftPaddle));
        self.set_button(DualsenseButton::RightPaddle , f32::MAX, buttons.contains(RawButtons::RightPaddle));
        
        let hat = raw_state.button_and_hat[0] & 0xF;
        assert!(hat <= 8);
        if hat == 8 {
            self.move_dpad(HatSwitch::Neutral, f32::MAX);
        } else {
            self.move_dpad(unsafe { HatSwitch::from_idx_unchecked(hat as usize + 1) }, f32::MAX);
        }

        let x = (raw_state.left_stick_x as f32 / 255.0) * 2.0 - 1.0;
        let y = (raw_state.left_stick_y as f32 / 255.0) * 2.0 - 1.0;
        self.move_stick(false, f32v2 { x, y }, f32::MAX, ReleaseCurve::Instant);

        let x = (raw_state.right_stick_x as f32 / 255.0) * 2.0 - 1.0;
        let y = (raw_state.right_stick_y as f32 / 255.0) * 2.0 - 1.0;
        self.move_stick(true, f32v2 { x, y }, f32::MAX, ReleaseCurve::Instant);

        let trigger = raw_state.left_trigger as f32 / 255.0;
        self.move_trigger(false, trigger, f32::MAX, ReleaseCurve::Instant);
        let trigger = raw_state.right_trigger as f32 / 255.0;
        self.move_trigger(true, trigger, f32::MAX, ReleaseCurve::Instant);

        // "Propriatary" data
        let raw = if let Some(raw) = input_report.get_raw_value(hid::Usage::from_u16(0xFF00, 0x22), None) {
            raw
        } else {
            return;
        };

        let prop_data = if let Some(arr) = raw.get_arr() {
            arr
        } else {
            return;
        };
        
        let touch_1 = unsafe { &*prop_data[21..=24].as_ptr().cast::<RawTouchState>() };
        if touch_1.is_touched() {
            self.move_touch(false, f32v2::new(touch_1.x(), touch_1.y()), touch_1.id());
        }
        
        let touch_2 = unsafe { &*prop_data[25..=28].as_ptr().cast::<RawTouchState>() };
        if touch_2.is_touched() {
            self.move_touch(true, f32v2::new(touch_2.x(), touch_2.y()), touch_2.id());
        }

        let angular_x = self.calibration.apply(CalibrationMode::GyroPitch, raw_state.gyro_pitch);
        let angular_y = self.calibration.apply(CalibrationMode::GyroPitch, raw_state.gyro_yaw);
        let angular_z = self.calibration.apply(CalibrationMode::GyroPitch, raw_state.gyro_roll);

        self.set_angular_velocity(f32v3::new(angular_x, angular_y, angular_z));
        
        let accel_x = self.calibration.apply(CalibrationMode::AccelX, raw_state.accel_x);
        let accel_y = self.calibration.apply(CalibrationMode::AccelY, raw_state.accel_y);
        let accel_z = self.calibration.apply(CalibrationMode::AccelZ, raw_state.accel_z);
        self.set_acceleration(f32v3::new(accel_x, accel_y, accel_z));

        // 3000000 diff per ms
        // let sensor_timestamp = u32::from_le_bytes([prop_data[16], prop_data[17], prop_data[18], prop_data[19]]);
        // log_verbose!(LOG_INPUT_CAT, "Sensor timestamp: {}", sensor_timestamp);


        self.changes.lock().battery = raw_state.battery_level;
    }

    fn handle_native_input(&mut self, _native_data: *const std::ffi::c_void) {
        // Nothing to do here 
    }

    fn get_native_handle(&self) -> &crate::NativeDeviceHandle {
        self.handle.as_ref().unwrap()
    }

    fn get_axis_value(&self, axis: &crate::AxisId) -> Option<AxisValue> {
        match *axis {
            Gamepad::DPAD_DIR             | Self::DPAD_DIR       => Some(AxisValue::Axis2D (self.state.read().dpad.get_direction(true))),
            Gamepad::DPAD_UP              | Self::DPAD_UP        => Some(AxisValue::Digital(self.state.read().dpad.is_up_down())),
            Gamepad::DPAD_DOWN            | Self::DPAD_DOWN      => Some(AxisValue::Digital(self.state.read().dpad.is_bottom_down())),
            Gamepad::DPAD_LEFT            | Self::DPAD_LEFT      => Some(AxisValue::Digital(self.state.read().dpad.is_left_down())),
            Gamepad::DPAD_RIGHT           | Self::DPAD_RIGHT     => Some(AxisValue::Digital(self.state.read().dpad.is_right_down())),
            Gamepad::FACE_LEFT            | Self::CIRCLE         => Some(AxisValue::Digital(self.state.read().buttons.get(DualsenseButton::Circle   as usize))),
            Gamepad::FACE_BOTTOM          | Self::CROSS          => Some(AxisValue::Digital(self.state.read().buttons.get(DualsenseButton::Cross    as usize))),
            Gamepad::FACE_RIGHT           | Self::SQUARE         => Some(AxisValue::Digital(self.state.read().buttons.get(DualsenseButton::Square   as usize))),
            Gamepad::FACE_TOP             | Self::TRIANGLE       => Some(AxisValue::Digital(self.state.read().buttons.get(DualsenseButton::Triangle as usize))),
            Gamepad::LEFT_BUMPER          | Self::L1             => Some(AxisValue::Digital(self.state.read().buttons.get(DualsenseButton::L1       as usize))),
            Gamepad::RIGHT_BUMPER         | Self::R1             => Some(AxisValue::Digital(self.state.read().buttons.get(DualsenseButton::R1       as usize))),
            Gamepad::LEFT_TRIGGER_BUTTON  | Self::L2             => Some(AxisValue::Digital(self.state.read().buttons.get(DualsenseButton::L2       as usize))),
            Gamepad::RIGHT_TRIGGER_BUTTON | Self::R2             => Some(AxisValue::Digital(self.state.read().buttons.get(DualsenseButton::R2       as usize))),
            Gamepad::LEFT_MENU            | Self::CREATE         => Some(AxisValue::Digital(self.state.read().buttons.get(DualsenseButton::Create   as usize))),
            Gamepad::RIGHT_MENU           | Self::OPTIONS        => Some(AxisValue::Digital(self.state.read().buttons.get(DualsenseButton::Options  as usize))),
            Gamepad::LEFT_THUMB_BUTTON    | Self::L3             => Some(AxisValue::Digital(self.state.read().buttons.get(DualsenseButton::L3       as usize))),
            Gamepad::RIGHT_THUMB_BUTTON   | Self::R3             => Some(AxisValue::Digital(self.state.read().buttons.get(DualsenseButton::R3       as usize))),
            Gamepad::GUIDE                | Self::PS_BUTTON      => Some(AxisValue::Digital(self.state.read().buttons.get(DualsenseButton::PsButton as usize))),
            Self::TOUCH_BUTTON                                   => Some(AxisValue::Digital(self.state.read().buttons.get(DualsenseButton::Touchpad as usize))),
            Self::MUTE                                           => Some(AxisValue::Digital(self.state.read().buttons.get(DualsenseButton::Mute     as usize))),

            Gamepad::LEFT_THUMB           | Self::LEFT_THUMB     => Some(AxisValue::Axis2D (self.state.read().sticks[0])),
            Gamepad::LEFT_THUMB_X         | Self::LEFT_THUMB_X   => Some(AxisValue::Axis   (self.state.read().sticks[0].x)),
            Gamepad::LEFT_THUMB_Y         | Self::LEFT_THUMB_Y   => Some(AxisValue::Axis   (self.state.read().sticks[0].y)),
            Gamepad::RIGHT_THUMB          | Self::RIGHT_THUMB    => Some(AxisValue::Axis2D (self.state.read().sticks[1])),
            Gamepad::RIGHT_THUMB_X        | Self::RIGHT_THUMB_X  => Some(AxisValue::Axis   (self.state.read().sticks[0].x)),
            Gamepad::RIGHT_THUMB_Y        | Self::RIGHT_THUMB_Y  => Some(AxisValue::Axis   (self.state.read().sticks[0].y)),

            Gamepad::LEFT_TRIGGER         | Self::LEFT_TRIGGER   => Some(AxisValue::Axis   (self.state.read().triggers[0])),
            Gamepad::RIGHT_TRIGGER        | Self::RIGHT_TRIGGER  => Some(AxisValue::Axis   (self.state.read().triggers[1])),

            /*Touch::TOUCH_0   |*/ Self::TOUCH_0   => self.state.read().touch[0].map(|val| AxisValue::Axis2D(val.pos)),
            /*Touch::TOUCH_0_X |*/ Self::TOUCH_0_X => self.state.read().touch[0].map(|val| AxisValue::Axis(val.pos.x)),
            /*Touch::TOUCH_0_Y |*/ Self::TOUCH_0_Y => self.state.read().touch[0].map(|val| AxisValue::Axis(val.pos.y)),

            /*Touch::TOUCH_1   |*/ Self::TOUCH_1   => self.state.read().touch[1].map(|val| AxisValue::Axis2D(val.pos)),
            /*Touch::TOUCH_1_X |*/ Self::TOUCH_1_X => self.state.read().touch[1].map(|val| AxisValue::Axis(val.pos.x)),
            /*Touch::TOUCH_1_Y |*/ Self::TOUCH_1_Y => self.state.read().touch[1].map(|val| AxisValue::Axis(val.pos.y)),

            _ => None
        }
    }

    fn get_axes(&self) -> &[crate::InputAxisDefinition] {
        const ZERO_V2: f32v2 = f32v2{ x: 0.0, y: 0.0 };
        const ONE_V2:  f32v2 = f32v2{ x: 1.0, y: 1.0 };
        const MONE_V2: f32v2 = f32v2{ x: -1.0, y: -1.0 };
        const MIN_V3:  f32v3 = f32v3{ x: f32::MIN, y: f32::MAX, z: f32::MAX };
        const MAX_V3:  f32v3 = f32v3{ x: f32::MAX, y: f32::MAX, z: f32::MAX };

        &[
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Gamepad::DPAD_DIR            , Self::DPAD_DIR]     , axis: AxisDefinition::Axis2D(MONE_V2, ONE_V2)   , can_rebind: false},
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Gamepad::DPAD_UP             , Self::DPAD_UP]      , axis: AxisDefinition::Digital                   , can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Gamepad::DPAD_DOWN           , Self::DPAD_DOWN]    , axis: AxisDefinition::Digital                   , can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Gamepad::DPAD_LEFT           , Self::DPAD_LEFT]    , axis: AxisDefinition::Digital                   , can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Gamepad::DPAD_RIGHT          , Self::DPAD_RIGHT]   , axis: AxisDefinition::Digital                   , can_rebind: true },

            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Gamepad::FACE_LEFT           , Self::CIRCLE]       , axis: AxisDefinition::Digital                   , can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Gamepad::FACE_BOTTOM         , Self::CROSS]        , axis: AxisDefinition::Digital                   , can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Gamepad::FACE_RIGHT          , Self::SQUARE]       , axis: AxisDefinition::Digital                   , can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Gamepad::FACE_TOP            , Self::TRIANGLE]     , axis: AxisDefinition::Digital                   , can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Gamepad::LEFT_BUMPER         , Self::L1]           , axis: AxisDefinition::Digital                   , can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Gamepad::RIGHT_BUMPER        , Self::R1]           , axis: AxisDefinition::Digital                   , can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Gamepad::LEFT_TRIGGER_BUTTON , Self::L2]           , axis: AxisDefinition::Digital                   , can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Gamepad::RIGHT_TRIGGER_BUTTON, Self::R2]           , axis: AxisDefinition::Digital                   , can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Gamepad::LEFT_MENU           , Self::CREATE]       , axis: AxisDefinition::Digital                   , can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Gamepad::RIGHT_MENU          , Self::OPTIONS]      , axis: AxisDefinition::Digital                   , can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Gamepad::LEFT_THUMB_BUTTON   , Self::L3]           , axis: AxisDefinition::Digital                   , can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Gamepad::RIGHT_THUMB_BUTTON  , Self::R3]           , axis: AxisDefinition::Digital                   , can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Gamepad::GUIDE               , Self::PS_BUTTON]    , axis: AxisDefinition::Digital                   , can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[                               Self::TOUCH_BUTTON] , axis: AxisDefinition::Digital                   , can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[                               Self::MUTE]         , axis: AxisDefinition::Digital                   , can_rebind: true },

            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Gamepad::LEFT_THUMB          , Self::LEFT_THUMB]   , axis: AxisDefinition::Axis2D(MONE_V2 , ONE_V2)  , can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Gamepad::LEFT_THUMB_X        , Self::LEFT_THUMB_X] , axis: AxisDefinition::Axis  (0.0     , 1.0   )  , can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Gamepad::LEFT_THUMB_Y        , Self::LEFT_THUMB_Y] , axis: AxisDefinition::Axis  (0.0     , 1.0   )  , can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Gamepad::RIGHT_THUMB         , Self::RIGHT_THUMB]  , axis: AxisDefinition::Axis2D(MONE_V2 , ONE_V2)  , can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Gamepad::RIGHT_THUMB_X       , Self::RIGHT_THUMB_X], axis: AxisDefinition::Axis  (0.0     , 1.0)     , can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Gamepad::RIGHT_THUMB_Y       , Self::RIGHT_THUMB_Y], axis: AxisDefinition::Axis  (0.0     , 1.0)     , can_rebind: true },

            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Gamepad::LEFT_TRIGGER        , Self::LEFT_TRIGGER] , axis: AxisDefinition::Axis  (0.0     , 1.0)     , can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Gamepad::RIGHT_TRIGGER       , Self::RIGHT_TRIGGER], axis: AxisDefinition::Axis  (0.0     , 1.0)     , can_rebind: true },

            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::Touch), ids: &[/*Touch::TOUCH_0  ,*/         Self::TOUCH_0]      , axis: AxisDefinition::Axis2D(ZERO_V2 , ONE_V2)  , can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::Touch), ids: &[/*Touch::TOUCH_0_X,*/         Self::TOUCH_0_X]    , axis: AxisDefinition::Axis  (0.0     , 1.0)     , can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::Touch), ids: &[/*Touch::TOUCH_0_Y,*/         Self::TOUCH_0_Y]    , axis: AxisDefinition::Axis  (0.0     , 1.0)     , can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::Touch), ids: &[/*Touch::TOUCH_1  ,*/         Self::TOUCH_1]      , axis: AxisDefinition::Axis2D(ZERO_V2 , ONE_V2)  , can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::Touch), ids: &[/*Touch::TOUCH_1_X,*/         Self::TOUCH_1_X]    , axis: AxisDefinition::Axis  (0.0     , 1.0)     , can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::Touch), ids: &[/*Touch::TOUCH_1_Y,*/         Self::TOUCH_1_Y]    , axis: AxisDefinition::Axis  (0.0     , 1.0)     , can_rebind: true },

            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::Gyro), ids: &[/*Gryo::GYRO,*/                Self::GYRO]         , axis: AxisDefinition::Axis3D(MIN_V3  , MAX_V3)  , can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::Gyro), ids: &[/*Gryo::GYRO_PITCH,*/          Self::GYRO_PITCH]   , axis: AxisDefinition::Axis  (f32::MIN, f32::MAX), can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::Gyro), ids: &[/*Gryo::GYRO_YAW,*/            Self::GYRO_YAW]     , axis: AxisDefinition::Axis  (f32::MIN, f32::MAX), can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::Gyro), ids: &[/*Gryo::GYRO_YROLL*/           Self::GYRO_ROLL]    , axis: AxisDefinition::Axis  (f32::MIN, f32::MAX), can_rebind: true },

            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::Accel), ids: &[/*Accel::ACCEL,*/             Self::ACCEL]        , axis: AxisDefinition::Axis3D(MIN_V3  , MAX_V3)  , can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::Accel), ids: &[/*Accel::ACCEL_X,*/           Self::ACCEL_X]      , axis: AxisDefinition::Axis  (f32::MIN, f32::MAX), can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::Accel), ids: &[/*Accel::ACCEL_Y,*/           Self::ACCEL_Y]      , axis: AxisDefinition::Axis  (f32::MIN, f32::MAX), can_rebind: true },
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::Accel), ids: &[/*Accel::ACCEL_Z*/            Self::ACCEL_Z]      , axis: AxisDefinition::Axis  (f32::MIN, f32::MAX), can_rebind: true },
        ]
    }

    fn get_device_type(&self) -> crate::DeviceType {
        crate::DeviceType::Gamepad(GamepadFeatures::Touch | GamepadFeatures::Gyro | GamepadFeatures::Accel)
    }

    fn take_native_handle(&mut self) -> crate::NativeDeviceHandle {
        core::mem::take(&mut self.handle).unwrap()
    }

    fn get_battery_info(&self) -> Option<crate::BatteryInfo> {
        const DISCHARGING:   u8 = 0x0;
        const CHARGING:      u8 = 0x1;
        const COMPLETE:      u8 = 0x2;
        const ABNORMAL_VOLT: u8 = 0xA;
        const ABNORMAL_TEMP: u8 = 0xB;
        const ERROR:         u8 = 0xF;

        let state = self.state.read();

        let battery_state = match state.battery >> 4 {
            DISCHARGING   => BatteryState::Discharging,
            CHARGING      => BatteryState::Charging,
            COMPLETE      => BatteryState::Neutral,
            ABNORMAL_VOLT => BatteryState::AbnormalVoltage,
            ABNORMAL_TEMP => BatteryState::AbnomralTemperature,
            ERROR         => BatteryState::Error,
            _             => unreachable!(),
        };

        let remaining_cap = (state.battery & 0xFF) as f32 / 10.0;

        Some(BatteryInfo {
            charge_rate: 0.0, // unknown
            max_charge_rate: 0.0, // unknown
            remaining_cap,
            full_capacity: 0.0, // unknown
            state: battery_state,
        })
    }

    fn get_output_info<'a>(&'a self) -> &'a OutputInfo<'a> {
        const INFO: OutputInfo = OutputInfo {
            rumble: RumbleSupport::LowFrequecy.bitor(RumbleSupport::HighFrequency),
            trigger_feedback: Some(TriggerFeedbackSupport::all()),
            led_support: &[
                LedSupport{ name: "Touchpad ring", mode: LedModeSupport::Color },
                LedSupport{ name: "Mute"         , mode: LedModeSupport::OnOff },
            ],
            output_axes: &[
                OutputAxisDefinition { ids: &[Gamepad::OUT_PLAYER_INDICATOR, DualSense::OUT_PLAYER_INDICATOR], axis: AxisDefinition::Int(0, 4) },
            ]
        };
         &INFO
    }

    fn set_rumble(&self, rumble: RumbleState) {
        self.out_state.lock().rumble = rumble;
    }

    fn set_trigger_feedback(&self, right_trigger: bool, trigger_feedback: TriggerFeedback) {
        self.out_state.lock().trigger_feedback[right_trigger as usize] = trigger_feedback;
    }

    fn set_led_state(&self, index: u16, state: LedState) {
        match index {
            0 => self.out_state.lock().ring_led = state,
            1 => self.out_state.lock().mute_led = state,
            _ => log_warning!(LOG_INPUT_CAT ,"Trying to set an invalid led on a dualsense controller"),
        }
    }

    fn set_output_axis(&self, axis: AxisId, value: AxisValue) {
        match axis {
            Gamepad::OUT_PLAYER_INDICATOR |
            Self::OUT_PLAYER_INDICATOR    => if let AxisValue::Int(id) = value {
                self.out_state.lock().player_id = id.clamp(0, 4) as u8;
            }

            _ => (),
        }
    }
}