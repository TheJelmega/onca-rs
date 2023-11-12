// TODO: Currently developed over parsec, which makes any controller act as the same type of device, so the current implementation (specifically trigger may not work as intended)

use core::fmt;

use crate::{InputDevice, InputAxisDefinition, AxisValue, AxisType, DeviceType, GamepadSubType, InputAxisId};
#[cfg(feature = "raw_input_logging")]
use crate::LOG_INPUT_CAT;
use onca_common::{
    prelude::*,
    collections::BitSet,
    sync::{Mutex, RwLock}
};
use onca_hid as hid;
#[cfg(feature = "raw_input_logging")]
use onca_logging::log_verbose;
use onca_math::{f32v2, Zero, MathConsts, SmoothStep};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GamepadButton {
    /// South face button e.g. A on xbox, cross on PS.
    FaceButtonBottom,
    /// East face button e.g. G on xbox, circle on PS.
    FaceButtonRight,
    /// West face button e.g. X on xbox, square on PS.
    FaceButtonLeft,
    /// North face button e.g. Y on xbox, triangle on PS.
    FaceButtonTop,
    /// Left bumper
    LeftBumper,
    /// Right bumper
    RightBumper,
    /// Left special button (e.g. menu, etc)
    LeftSpecial,
    /// Right special button (e.g. menu, etc)
    RightSpecial,
    /// Left joystick button
    LeftJoystick,
    /// Right joystick button
    RightJoystick,
}
const NUM_BUTTONS : usize = GamepadButton::RightJoystick as usize + 1;
const NUM_BUTTONS_BITS : usize = NUM_BUTTONS.next_power_of_two();

impl GamepadButton {
    pub fn from_idx(idx: u16) -> Option<GamepadButton> {
        match idx {
            0 => Some(Self::FaceButtonBottom),
            1 => Some(Self::FaceButtonRight),
            2 => Some(Self::FaceButtonLeft),
            3 => Some(Self::FaceButtonTop),
            4 => Some(Self::LeftBumper),
            5 => Some(Self::RightBumper),
            6 => Some(Self::LeftSpecial),
            7 => Some(Self::RightSpecial),
            8 => Some(Self::LeftJoystick),
            9 => Some(Self::RightJoystick),
            _ => None,
        }
    }
}

impl fmt::Display for GamepadButton {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GamepadButton::FaceButtonBottom => f.write_str("north"),
            GamepadButton::FaceButtonRight  => f.write_str("east"),
            GamepadButton::FaceButtonLeft   => f.write_str("west"),
            GamepadButton::FaceButtonTop    => f.write_str("north"),
            GamepadButton::LeftBumper       => f.write_str("left bumper"),
            GamepadButton::RightBumper      => f.write_str("right bumper"),
            GamepadButton::LeftSpecial      => f.write_str("left menu"),
            GamepadButton::RightSpecial     => f.write_str("right menu"),
            GamepadButton::LeftJoystick     => f.write_str("left stick button"),
            GamepadButton::RightJoystick    => f.write_str("right stick button"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DPadDirection {
    Neutral,
    Up,
    UpRight,
    Right,
    DownRight,
    Down,
    DownLeft,
    Left,
    UpLeft,
}

impl DPadDirection {
    pub fn from_idx(idx: u8) -> Self {
        match idx {
            1 => Self::Up,
            2 => Self::UpRight,
            3 => Self::Right,
            4 => Self::DownRight,
            5 => Self::Down,
            6 => Self::DownLeft,
            7 => Self::Left,
            8 => Self::UpLeft,
            _ => Self::Neutral,
        }
    }

    pub fn is_bottom_down(&self) -> bool {
        match self {
            DPadDirection::DownRight
            | DPadDirection::Down
            | DPadDirection::DownLeft => true,
            _ => false
        }
    }

    pub fn is_right_down(&self) -> bool {
        match self {
            DPadDirection::UpRight
            | DPadDirection::Right
            | DPadDirection::DownRight => true,
            _ => false
        }
    }

    pub fn is_left_down(&self) -> bool {
        match self {
            DPadDirection::DownLeft
            | DPadDirection::Left
            | DPadDirection::UpLeft => true,
            _ => false
        }
    }

    pub fn is_top_down(&self) -> bool {
        match self {
            DPadDirection::Up
            | DPadDirection::UpRight
            | DPadDirection::UpLeft => true,
            _ => false
        }
    }

    pub fn get_direction(&self) -> f32v2 {
        match self {
            DPadDirection::Neutral   => f32v2::new( 0f32                  ,  0f32),
            DPadDirection::Up        => f32v2::new( 0f32                  ,  1f32),
            DPadDirection::UpRight   => f32v2::new( f32::ONE_OVER_ROOT_TWO,  f32::ONE_OVER_ROOT_TWO),
            DPadDirection::Right     => f32v2::new( 1f32                  ,  0f32),
            DPadDirection::DownRight => f32v2::new( f32::ONE_OVER_ROOT_TWO, -f32::ONE_OVER_ROOT_TWO),
            DPadDirection::Down      => f32v2::new( 0f32                  , -1f32),
            DPadDirection::DownLeft  => f32v2::new(-f32::ONE_OVER_ROOT_TWO, -f32::ONE_OVER_ROOT_TWO),
            DPadDirection::Left      => f32v2::new(-1f32                  ,  0f32),
            DPadDirection::UpLeft    => f32v2::new(-f32::ONE_OVER_ROOT_TWO,  f32::ONE_OVER_ROOT_TWO),
        }
    }
}

impl fmt::Display for DPadDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DPadDirection::Neutral   => f.write_str("neutral"),
            DPadDirection::Up        => f.write_str("up"),
            DPadDirection::UpRight   => f.write_str("up-right"),
            DPadDirection::Right     => f.write_str("tight"),
            DPadDirection::DownRight => f.write_str("down-right"),
            DPadDirection::Down      => f.write_str("down"),
            DPadDirection::DownLeft  => f.write_str("down-left"),
            DPadDirection::Left      => f.write_str("left"),
            DPadDirection::UpLeft    => f.write_str("up-left"),
        }
    }
}

/// How will the stick be released if emulated
#[derive(Clone, Copy, Debug)]
pub enum GamepadReleaseCurve {
    /// The stick will instantaniously return to center.
    Instant,
    /// The stick will return to center in a linear way, over a given time (in seconds).
    Linear(f32),
    /// The stick will return to center in a smooth way (smoothstep), over a given time (in seconds).
    Smooth(f32),
}

impl GamepadReleaseCurve {
    fn interpolate_v2(&self, from: f32v2, time_passed: f32) -> f32v2 {
        match self {
            GamepadReleaseCurve::Instant => f32v2::zero(),
            // lerping from 0, is the same as * (1 - interpolant)
            GamepadReleaseCurve::Linear(max_time) => from * (1f32 - time_passed / max_time),
            GamepadReleaseCurve::Smooth(max_time) => from * (1f32 - (time_passed / max_time).smooth_step(0.0, 1.0)),
        }
    }

    fn interpolate(&self, from: f32, time_passed: f32) -> f32 {
        match self {
            GamepadReleaseCurve::Instant => 0f32,
            // lerping from 0, is the same as * (1 - interpolant)
            GamepadReleaseCurve::Linear(max_time) => from * (1f32 - time_passed / max_time),
            // lerping from 0, is the same as * (1 - interpolant)
            GamepadReleaseCurve::Smooth(max_time) => from * (1f32 - (time_passed / max_time).smooth_step(0.0, 1.0)),
        }
    }

    fn time_out(&self, time: f32) -> bool {
        match self {
            GamepadReleaseCurve::Instant => true,
            GamepadReleaseCurve::Linear(max_time) => time >= *max_time,
            GamepadReleaseCurve::Smooth(max_time) => time >= *max_time,
        }
    }
}

struct ButtonChange {
    button  : GamepadButton,
    time    : f32,
    pressed : bool,
}

struct StickMove {
    pos    : f32v2,
    time   : f32,
    curve  : GamepadReleaseCurve,
}

impl StickMove {
    fn new() -> Self {
        Self { pos: f32v2::zero(), time: 0f32, curve: GamepadReleaseCurve::Instant }
    }

    fn update(&mut self, dt: f32) -> f32v2 {
        self.time -= dt;
        if self.time > 0f32 {
            self.pos
        } else {
            let passed_time = -self.time;
            if self.curve.time_out(passed_time) {
                f32v2::zero()
            } else {
                self.curve.interpolate_v2(self.pos, passed_time)
            }
        }
    }
}

struct TriggerMove {
    val    : f32,
    time   : f32,
    curve  : GamepadReleaseCurve,
}

impl TriggerMove {
    fn new() -> Self {
        Self { val: 0f32, time: 0f32, curve: GamepadReleaseCurve::Instant }
    }

    fn update(&mut self, dt: f32) -> f32 {
        self.time -= dt;
        if self.time > 0f32 {
            self.val
        } else {
            let passed_time = -self.time;
            if self.curve.time_out(passed_time) {
                0f32
            } else {
                self.curve.interpolate(self.val, passed_time)
            }
        }
    }
}

struct GamepadState {
    buttons       : BitSet<NUM_BUTTONS_BITS>,
    dpad          : DPadDirection,
    left_stick    : f32v2,
    right_stick   : f32v2,
    left_trigger  : f32,
    right_trigger : f32,
}

impl GamepadState
{
    fn new() -> Self {
        Self {
            buttons: BitSet::new(),
            dpad: DPadDirection::Neutral,
            left_stick: f32v2::zero(),
            right_stick: f32v2::zero(),
            left_trigger: 0f32,
            right_trigger: 0f32,
        }
    }

    pub fn is_button_down(&self, button: GamepadButton) -> bool {
        let idx = button as usize;
        self.buttons.get(idx)
    }
}

/// Generic controller/gamepad
/// 
/// When making a custom gamepad that is derived from the standard gamepad, use this as the underlying input device and build additional functionality on top of this.
pub struct Gamepad {
    state           : RwLock<GamepadState>,

    button_changes  : Mutex<Vec<ButtonChange>>,
    dpad_dir        : Mutex<(DPadDirection, f32)>,
    left_stick      : Mutex<StickMove>,
    right_stick     : Mutex<StickMove>,
    left_trigger    : Mutex<TriggerMove>,
    right_trigger   : Mutex<TriggerMove>,

    button_timers   : [f32; NUM_BUTTONS],
}

impl Gamepad {
    // TODO: Make all InputAxisIds when moved to interned strings (static string ids)
    pub const LEFT_THUMB_STR             : &'static str = "Gamepad Left Thumbstick 2D-Axis";
    pub const LEFT_THUMB_X_STR           : &'static str = "Gamepad Left Thumbstick X-Axis";
    pub const LEFT_THUMB_Y_STR           : &'static str = "Gamepad Left Thumbstick Y-Axis";
    pub const LEFT_THUMB_BUTTON_STR      : &'static str = "Gamepad Left Thumbstick Button";
    pub const RIGHT_THUMB_STR            : &'static str = "Gamepad Right Thumbstick 2D-Axis";
    pub const RIGHT_THUMB_X_STR          : &'static str = "Gamepad Right Thumbstick X-Axis";
    pub const RIGHT_THUMB_Y_STR          : &'static str = "Gamepad Right Thumbstick Y-Axis";
    pub const RIGHT_THUMB_BUTTON_STR     : &'static str = "Gamepad Right Thumbstick Button";
    pub const DPAD_DIR_STR               : &'static str = "Gamepad D-Pad Direction";
    pub const DPAD_UP_STR                : &'static str = "Gamepad D-Pad Up";
    pub const DPAD_DOWN_STR              : &'static str = "Gamepad D-Pad Down";
    pub const DPAD_LEFT_STR              : &'static str = "Gamepad D-Pad Left";
    pub const DPAD_RIGHT_STR             : &'static str = "Gamepad D-Pad Right";
    pub const FACE_BOTTOM_STR            : &'static str = "Gamepad Face Button Bottom";
    pub const FACE_RIGHT_STR             : &'static str = "Gamepad Face Button Right";
    pub const FACE_LEFT_STR              : &'static str = "Gamepad Face Button Left";
    pub const FACE_UP_STR                : &'static str = "Gamepad Face Button Top";
    pub const LEFT_SPECIAL_STR           : &'static str = "Gamepad Left Special";
    pub const RIGHT_SPECIAL_STR          : &'static str = "Gamepad Right Special";
    pub const LEFT_BUMPER_STR            : &'static str = "Gamepad Left Bumper";
    pub const RIGHT_BUMPER_STR           : &'static str = "Gamepad Right Bumper";
    pub const LEFT_TRIGGER_STR           : &'static str = "Gamepad Left Trigger";
    pub const RIGHT_TRIGGER_STR          : &'static str = "Gamepad Right Trigger";

    pub const LEFT_THUMB         : InputAxisId = InputAxisId::new(Self::LEFT_THUMB_STR        );
    pub const LEFT_THUMB_X       : InputAxisId = InputAxisId::new(Self::LEFT_THUMB_X_STR      );
    pub const LEFT_THUMB_Y       : InputAxisId = InputAxisId::new(Self::LEFT_THUMB_Y_STR      );
    pub const LEFT_THUMB_BUTTON  : InputAxisId = InputAxisId::new(Self::LEFT_THUMB_BUTTON_STR );
    pub const RIGHT_THUMB        : InputAxisId = InputAxisId::new(Self::RIGHT_THUMB_STR       );
    pub const RIGHT_THUMB_X      : InputAxisId = InputAxisId::new(Self::RIGHT_THUMB_X_STR     );
    pub const RIGHT_THUMB_Y      : InputAxisId = InputAxisId::new(Self::RIGHT_THUMB_Y_STR     );
    pub const RIGHT_THUMB_BUTTON : InputAxisId = InputAxisId::new(Self::RIGHT_THUMB_BUTTON_STR);
    pub const DPAD_DIR           : InputAxisId = InputAxisId::new(Self::DPAD_DIR_STR          );
    pub const DPAD_UP            : InputAxisId = InputAxisId::new(Self::DPAD_UP_STR           );
    pub const DPAD_DOWN          : InputAxisId = InputAxisId::new(Self::DPAD_DOWN_STR         );
    pub const DPAD_LEFT          : InputAxisId = InputAxisId::new(Self::DPAD_LEFT_STR         );
    pub const DPAD_RIGHT         : InputAxisId = InputAxisId::new(Self::DPAD_RIGHT_STR        );
    pub const FACE_BOTTOM        : InputAxisId = InputAxisId::new(Self::FACE_BOTTOM_STR       );
    pub const FACE_RIGHT         : InputAxisId = InputAxisId::new(Self::FACE_RIGHT_STR        );
    pub const FACE_LEFT          : InputAxisId = InputAxisId::new(Self::FACE_LEFT_STR         );
    pub const FACE_UP            : InputAxisId = InputAxisId::new(Self::FACE_UP_STR           );
    pub const LEFT_SPECIAL       : InputAxisId = InputAxisId::new(Self::LEFT_SPECIAL_STR      );
    pub const RIGHT_SPECIAL      : InputAxisId = InputAxisId::new(Self::RIGHT_SPECIAL_STR     );
    pub const LEFT_BUMPER        : InputAxisId = InputAxisId::new(Self::LEFT_BUMPER_STR       );
    pub const RIGHT_BUMPER       : InputAxisId = InputAxisId::new(Self::RIGHT_BUMPER_STR      );
    pub const LEFT_TRIGGER       : InputAxisId = InputAxisId::new(Self::LEFT_TRIGGER_STR      );
    pub const RIGHT_TRIGGER      : InputAxisId = InputAxisId::new(Self::RIGHT_TRIGGER_STR     );

    pub fn new() -> Option<Gamepad> {
        Some(Self {
            state: RwLock::new(GamepadState::new()),
            button_changes: Mutex::new(Vec::new()),
            dpad_dir: Mutex::new((DPadDirection::Neutral, 0f32)),
            left_stick: Mutex::new(StickMove::new()),
            right_stick: Mutex::new(StickMove::new()),
            left_trigger: Mutex::new(TriggerMove::new()),
            right_trigger: Mutex::new(TriggerMove::new()),
            button_timers: [0f32; NUM_BUTTONS]
        })
    }

    /// Check if a button is down
    pub fn is_button_down(&self, button: GamepadButton) -> bool {
        self.state.read().is_button_down(button)
    }

    /// Get the dpad direction
    pub fn dpad(&self) -> DPadDirection {
        self.state.read().dpad
    }

    /// Get the left stick axis
    pub fn left_stick(&self) -> f32v2 {
        self.state.read().left_stick
    }

    /// Get the right stick axis
    pub fn right_stick(&self) -> f32v2 {
        self.state.read().right_stick
    }

    /// Get the left trigger axis
    pub fn left_trigger(&self) -> f32 {
        self.state.read().left_trigger
    }

    /// Get the right trigger axis
    pub fn right_trigger(&self) -> f32 {
        self.state.read().right_trigger
    }

    /// Emulate a button press
    pub fn press_button(&self, button: GamepadButton, time: f32) {
        self.button_changes.lock().push(ButtonChange{ button, time, pressed: true });
    }

    /// Emulate a button release
    pub fn release_button(&self, button: GamepadButton) {
        self.button_changes.lock().push(ButtonChange{ button, time: 0f32, pressed: false });
    }

    /// Emulate a dpad movement
    /// 
    /// When `time` has passed, the dpad will return to neutral
    pub fn move_dpad(&self, dir: DPadDirection, time: f32) {
        *self.dpad_dir.lock() = (dir, time);
    }

    /// Emulate a stick movement
    /// 
    /// When `time` has passed, the joystick will return to center
    // TODO: Return to center with a curve?
    pub fn move_stick(&self, right: bool, pos: f32v2, time: f32, curve: GamepadReleaseCurve) {
        if right {
            *self.right_stick.lock() = StickMove{ pos, time, curve };
        } else {
            *self.left_stick.lock() = StickMove{ pos, time, curve };
        }
    }

    pub fn move_trigger(&self, right: bool, val: f32, time: f32, curve: GamepadReleaseCurve) {
        if right {
            *self.right_trigger.lock() = TriggerMove{ val, time, curve };
        } else {
            *self.left_trigger.lock() = TriggerMove{ val, time, curve };
        }
    }
}

impl InputDevice for Gamepad {
    fn tick(&mut self, dt: f32, notify_rebind: &mut dyn FnMut(InputAxisId)) {
        let mut state = self.state.write();
        let mut button_changes = self.button_changes.lock();
        let mut dpad_dir = self.dpad_dir.lock();
        let mut left_stick = self.left_stick.lock();
        let mut right_stick = self.right_stick.lock();

        let mut left_trigger = self.left_trigger.lock();
        let mut right_trigger = self.right_trigger.lock();

        let _scoped_alloc = ScopedAlloc::new(AllocId::TlsTemp);

        // We generally only care about the last action of a button, as a press and release should not happen in a single frame.
        // While this is possible, especially at a lower framerate, it doesn't make much sense in term of the input system
        let mut processed_buttons = BitSet::<NUM_BUTTONS>::new();
        for change in button_changes.iter().rev() {
            let button_idx = change.button as usize;
            if processed_buttons.get(button_idx) {
                continue;
            }

            const BUTTON_OFFSET : usize = 13;
            notify_rebind(InputAxisId::new(self.get_axes()[BUTTON_OFFSET + button_idx].path));

            state.buttons.set(button_idx, change.pressed);
            self.button_timers[button_idx] = change.time;
            processed_buttons.enable(button_idx);
        }
        button_changes.clear();

        // Handle timers
        for (idx, timer) in self.button_timers.iter_mut().enumerate() {
            *timer = (*timer - dt).min(0f32);
            if *timer == 0f32 {
                state.buttons.disable(idx);
            }
        }
        
        state.left_stick = left_stick.update(dt);
        state.right_stick = right_stick.update(dt); 
        
        let left_trigger_val = left_trigger.update(dt);
        if left_trigger_val > 0f32 {
            notify_rebind(Self::LEFT_TRIGGER);
        }
        state.left_trigger = left_trigger_val;

        let right_trigger_val = right_trigger.update(dt);
        if left_trigger_val > 0f32 {
            notify_rebind(Self::RIGHT_TRIGGER);
        }
        state.right_trigger = right_trigger_val;
        
        // Return dpad back to neutral when it time runs out, otherwise just assign it and update the time
        let (dpad, dpad_time) = &mut *dpad_dir;
        if *dpad_time == 0f32 {
            state.dpad = DPadDirection::Neutral;
        } else {
            state.dpad = *dpad;
            *dpad_time = (*dpad_time - dt).max(0f32);

            if dpad.is_top_down() {
                notify_rebind(Self::DPAD_UP);
            }
            else if dpad.is_bottom_down() {
                notify_rebind(Self::DPAD_DOWN);
            }
            if dpad.is_left_down() {
                notify_rebind(Self::DPAD_LEFT);
            }
            else if dpad.is_right_down() {
                notify_rebind(Self::DPAD_RIGHT);
            }
        }
    }

    fn handle_hid_input(&mut self, hid_device: &onca_hid::Device, input_report: onca_hid::InputReport) {
        //Buttons
        let button_usage_page = hid::UsagePageId::new(9);
        if let Some(hid_buttons) = input_report.get_buttons() {
            if let Some(button_caps) = hid_device.get_button_capabilities_for_page(hid::ReportType::Input, button_usage_page, None) {
                let range_start = button_caps.usage.start;
                
                for hid_button in hid_buttons {
                    let button_idx = hid_button.usage.as_u16() - range_start.as_u16();
                    if let Some(button) = GamepadButton::from_idx(button_idx) {
                        self.press_button(button, f32::MAX);
                    }

                    #[cfg(feature = "raw_input_logging")]
                    if let Some(button_enum) = GamepadButton::from_idx(button_idx) {
                        log_verbose!(LOG_INPUT_CAT, "Pressed gamepad button '{button_enum:?}'");
                    }
                }

            }
        }

        const LEFT_STICK_X_USAGE : hid::Usage = hid::Usage::from_u16(1, 48);
        const LEFT_STICK_Y_USAGE : hid::Usage = hid::Usage::from_u16(1, 49);
        const RIGHT_STICK_X_USAGE : hid::Usage = hid::Usage::from_u16(1, 51);
        const RIGHT_STICK_Y_USAGE : hid::Usage = hid::Usage::from_u16(1, 52);
        const COMBINED_TRIGGERS_USAGE : hid::Usage = hid::Usage::from_u16(1, 50);
        const DPAD_USAGE : hid::Usage = hid::Usage::from_u16(1, 50);

        // Left joystick
        if let Some(x) = input_report.get_raw_value(LEFT_STICK_X_USAGE, None) &&
            let Some(x_caps) = hid_device.get_value_capabilities_for_usage(hid::ReportType::Input, LEFT_STICK_X_USAGE, None) &&
            let Some(y) = input_report.get_raw_value(LEFT_STICK_Y_USAGE, None) &&
            let Some(y_caps) = hid_device.get_value_capabilities_for_usage(hid::ReportType::Input, LEFT_STICK_Y_USAGE, None)
        {
            let normalized_x = x.first() as f32 / x_caps.physical_range.end as u32 as f32;
            let normalized_y = y.first() as f32 / y_caps.physical_range.end as u32 as f32;

            // Normalized values are in the range 0..=1, we need them in the range -1..=1
            let x = normalized_x * 2f32 - 1f32;
            let y = normalized_y * 2f32 - 1f32;

            self.move_stick(false, f32v2{ x, y }, f32::MAX, GamepadReleaseCurve::Instant);

            #[cfg(feature = "raw_input_logging")]
            log_verbose!(LOG_INPUT_CAT, "Left tick moved to {}", f32v2{ x, y });
        }

        // Right joystick
        if let Some(x) = input_report.get_raw_value(RIGHT_STICK_X_USAGE, None) &&
            let Some(x_caps) = hid_device.get_value_capabilities_for_usage(hid::ReportType::Input, RIGHT_STICK_X_USAGE, None) &&
            let Some(y) = input_report.get_raw_value(RIGHT_STICK_Y_USAGE, None) &&
            let Some(y_caps) = hid_device.get_value_capabilities_for_usage(hid::ReportType::Input, RIGHT_STICK_Y_USAGE, None)
        {
            let normalized_x = x.first() as f32 / x_caps.physical_range.end as u32 as f32;
            let normalized_y = y.first() as f32 / y_caps.physical_range.end as u32 as f32;

            // Normalized values are in the range 0..=1, we need them in the range -1..=1
            let x = normalized_x * 2f32 - 1f32;
            let y = normalized_y * 2f32 - 1f32;

            self.move_stick(true, f32v2{ x, y }, f32::MAX, GamepadReleaseCurve::Instant);

            #[cfg(feature = "raw_input_logging")]
            log_verbose!(LOG_INPUT_CAT, "Right tick moved to {}", f32v2{ x, y });
        }

        // D-pad
        if let Some(val) = input_report.get_raw_value(DPAD_USAGE, None) {
            let dir = DPadDirection::from_idx(val.first() as u8);
            self.move_dpad(dir, f32::MAX);
            #[cfg(feature = "raw_input_logging")]
            log_verbose!(LOG_INPUT_CAT, "DPad moved to {dir}");
        }

        // Triggers
        // 
        // With a standard HID trigger, we seem to currently be limited to having them on a single access, see top comment
        if let Some(trigger_axis) = input_report.get_raw_value(COMBINED_TRIGGERS_USAGE, None) &&
            let Some(caps) = hid_device.get_value_capabilities_for_usage(hid::ReportType::Input, COMBINED_TRIGGERS_USAGE, None)
        {
            let normalized = trigger_axis.first() as f32 / caps.physical_range.end as u32 as f32;

            let combined = normalized * 2f32 - 1f32;

            if combined < 0f32 {
                self.move_trigger(false, -combined, f32::MAX, GamepadReleaseCurve::Instant);
                self.move_trigger(true ,  0f32    , f32::MAX, GamepadReleaseCurve::Instant);
            } else {
                self.move_trigger(false,  0f32    , f32::MAX, GamepadReleaseCurve::Instant);
                self.move_trigger(true ,  combined, f32::MAX, GamepadReleaseCurve::Instant);
            }

        }
    }

    fn get_axis_value(&self, axis_path: &InputAxisId) -> Option<AxisValue> {
        match *axis_path {
            Self::LEFT_THUMB         => Some(AxisValue::Axis2D(self.state.read().left_stick)),
            Self::LEFT_THUMB_X       => Some(AxisValue::Axis(self.state.read().left_stick.x)),
            Self::LEFT_THUMB_Y       => Some(AxisValue::Axis(self.state.read().left_stick.y)),
            Self::RIGHT_THUMB        => Some(AxisValue::Axis2D(self.state.read().right_stick)),
            Self::RIGHT_THUMB_X      => Some(AxisValue::Axis(self.state.read().right_stick.x)),
            Self::RIGHT_THUMB_Y      => Some(AxisValue::Axis(self.state.read().right_stick.y)),
            Self::DPAD_DIR           => Some(AxisValue::Axis2D(self.state.read().dpad.get_direction())),
            Self::DPAD_UP            => Some(AxisValue::Digital(self.state.read().dpad.is_top_down())),
            Self::DPAD_DOWN          => Some(AxisValue::Digital(self.state.read().dpad.is_bottom_down())),
            Self::DPAD_LEFT          => Some(AxisValue::Digital(self.state.read().dpad.is_left_down())),
            Self::DPAD_RIGHT         => Some(AxisValue::Digital(self.state.read().dpad.is_right_down())),
            Self::FACE_BOTTOM        => Some(AxisValue::Digital(self.state.read().buttons.get(GamepadButton::FaceButtonBottom as usize))),
            Self::FACE_RIGHT         => Some(AxisValue::Digital(self.state.read().buttons.get(GamepadButton::FaceButtonRight as usize))),
            Self::FACE_LEFT          => Some(AxisValue::Digital(self.state.read().buttons.get(GamepadButton::FaceButtonLeft as usize))),
            Self::FACE_UP            => Some(AxisValue::Digital(self.state.read().buttons.get(GamepadButton::FaceButtonTop as usize))),
            Self::LEFT_BUMPER        => Some(AxisValue::Digital(self.state.read().buttons.get(GamepadButton::LeftBumper as usize))),
            Self::RIGHT_BUMPER       => Some(AxisValue::Digital(self.state.read().buttons.get(GamepadButton::RightBumper as usize))),
            Self::LEFT_SPECIAL       => Some(AxisValue::Digital(self.state.read().buttons.get(GamepadButton::LeftSpecial as usize))),
            Self::RIGHT_SPECIAL      => Some(AxisValue::Digital(self.state.read().buttons.get(GamepadButton::RightSpecial as usize))),
            Self::LEFT_THUMB_BUTTON  => Some(AxisValue::Digital(self.state.read().buttons.get(GamepadButton::LeftJoystick as usize))),
            Self::RIGHT_THUMB_BUTTON => Some(AxisValue::Digital(self.state.read().buttons.get(GamepadButton::RightJoystick as usize))),
            Self::LEFT_TRIGGER       => Some(AxisValue::Axis(self.state.read().left_trigger)),
            Self::RIGHT_TRIGGER      => Some(AxisValue::Axis(self.state.read().right_trigger)),
            _ => None
        }
    }

    fn get_axes(&self) -> &[InputAxisDefinition] {
        &[
            InputAxisDefinition { dev_type: DeviceType::Gamepad(GamepadSubType::Generic), path: Self::LEFT_THUMB_STR        , axis_type: AxisType::Axis2D , can_rebind: false },
            InputAxisDefinition { dev_type: DeviceType::Gamepad(GamepadSubType::Generic), path: Self::LEFT_THUMB_X_STR      , axis_type: AxisType::Axis   , can_rebind: false },
            InputAxisDefinition { dev_type: DeviceType::Gamepad(GamepadSubType::Generic), path: Self::LEFT_THUMB_Y_STR      , axis_type: AxisType::Axis   , can_rebind: false },
            InputAxisDefinition { dev_type: DeviceType::Gamepad(GamepadSubType::Generic), path: Self::RIGHT_THUMB_STR       , axis_type: AxisType::Axis2D , can_rebind: false },
            InputAxisDefinition { dev_type: DeviceType::Gamepad(GamepadSubType::Generic), path: Self::RIGHT_THUMB_X_STR     , axis_type: AxisType::Axis   , can_rebind: false },
            InputAxisDefinition { dev_type: DeviceType::Gamepad(GamepadSubType::Generic), path: Self::RIGHT_THUMB_Y_STR     , axis_type: AxisType::Axis   , can_rebind: false },
            InputAxisDefinition { dev_type: DeviceType::Gamepad(GamepadSubType::Generic), path: Self::DPAD_DIR_STR          , axis_type: AxisType::Axis2D , can_rebind: false },
            InputAxisDefinition { dev_type: DeviceType::Gamepad(GamepadSubType::Generic), path: Self::DPAD_UP_STR           , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Gamepad(GamepadSubType::Generic), path: Self::DPAD_DOWN_STR         , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Gamepad(GamepadSubType::Generic), path: Self::DPAD_LEFT_STR         , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Gamepad(GamepadSubType::Generic), path: Self::DPAD_RIGHT_STR        , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Gamepad(GamepadSubType::Generic), path: Self::FACE_BOTTOM_STR       , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Gamepad(GamepadSubType::Generic), path: Self::FACE_RIGHT_STR        , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Gamepad(GamepadSubType::Generic), path: Self::FACE_LEFT_STR         , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Gamepad(GamepadSubType::Generic), path: Self::FACE_UP_STR           , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Gamepad(GamepadSubType::Generic), path: Self::RIGHT_SPECIAL_STR     , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Gamepad(GamepadSubType::Generic), path: Self::RIGHT_BUMPER_STR      , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Gamepad(GamepadSubType::Generic), path: Self::LEFT_SPECIAL_STR      , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Gamepad(GamepadSubType::Generic), path: Self::LEFT_BUMPER_STR       , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Gamepad(GamepadSubType::Generic), path: Self::LEFT_THUMB_BUTTON_STR , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Gamepad(GamepadSubType::Generic), path: Self::RIGHT_THUMB_BUTTON_STR, axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Gamepad(GamepadSubType::Generic), path: Self::LEFT_TRIGGER_STR      , axis_type: AxisType::Axis   , can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Gamepad(GamepadSubType::Generic), path: Self::RIGHT_TRIGGER_STR     , axis_type: AxisType::Axis   , can_rebind: true },
        ]
    }

    fn get_device_type(&self) -> DeviceType {
        DeviceType::Gamepad(GamepadSubType::Generic)
    }
}