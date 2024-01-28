// TODO: Currently developed over parsec, which makes any controller act as the same type of device, so the current implementation (specifically trigger may not work as intended)

use crate::{InputDevice, InputAxisDefinition, AxisValue, AxisType, DeviceType, GamepadSubType, InputAxisId, NativeDeviceHandle};
#[cfg(feature = "raw_input_logging")]
use crate::LOG_INPUT_CAT;
use onca_common::{
    prelude::*,
    collections::BitSet,
    sync::{Mutex, RwLock}
};
use onca_common_macros::{EnumCount, EnumFromIndex, EnumDisplay};

#[cfg(feature = "raw_input_logging")]
use onca_logging::log_verbose;
use onca_logging::log_warning;
use onca_math::{f32v2, Zero, MathConsts, SmoothStep};

#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumCount, EnumFromIndex, EnumDisplay)]
pub enum GamepadButton {
    /// South face button e.g. A on xbox, cross on PS.
    #[display("south")]
    FaceBottom,
    /// East face button e.g. B on xbox, circle on PS.
    #[display("east")]
    FaceRight,
    /// West face button e.g. X on xbox, square on PS.
    #[display("west")]
    FaceLeft,
    /// North face button e.g. Y on xbox, triangle on PS.
    #[display("north")]
    FaceTop,
    /// Left bumper
    #[display("left bumper")]
    LeftBumper,
    /// Right bumper
    #[display("right bumper")]
    RightBumper,
    /// Left special button (e.g. menu, etc)
    #[display("left menu")]
    LeftMenu,
    /// Right special button (e.g. menu, etc)
    #[display("right menu")]
    RightMenu,
    /// Left joystick button
    #[display("left stick button")]
    LeftThumbstick,
    /// Right joystick button
    #[display("right stick button")]
    RightThumbsstick,
    /// Guide button
    #[display("guide button")]
    Guide,
}
const NUM_BUTTONS_BITS : usize = GamepadButton::COUNT.next_power_of_two();

#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumFromIndex, EnumDisplay)]
pub enum DPadDirection {
    /// DPad is in the neutral position
    #[display("neutral")]
    Neutral,
    /// Up is pressed
    #[display("up")]
    Up,
    /// Up and right are pressed
    #[display("up-right")]
    UpRight,
    /// Right is pressed
    #[display("right")]
    Right,
    /// Down and right are pressed
    #[display("down-right")]
    DownRight,
    /// Down is pressed
    #[display("down")]
    Down,
    /// Down and left are pressed
    #[display("down-left")]
    DownLeft,
    /// :eft is pressed
    #[display("left")]
    Left,
    /// Up and left are pressed
    #[display("up-left")]
    UpLeft,
}

impl DPadDirection {
    /// Check if the down button is down
    pub fn is_bottom_down(&self) -> bool {
        matches!(self, DPadDirection::DownRight | DPadDirection::Down | DPadDirection::DownLeft)
    }
    
    /// Check if the right button is down
    pub fn is_right_down(&self) -> bool {
        matches!(self, DPadDirection::UpRight | DPadDirection::Right | DPadDirection::DownRight)
    }
    
    /// Check if the left button is down
    pub fn is_left_down(&self) -> bool {
        matches!(self, DPadDirection::DownLeft | DPadDirection::Left | DPadDirection::UpLeft)
    }
    
    /// Check if the up button is down
    pub fn is_up_down(&self) -> bool {
        matches!(self, DPadDirection::UpRight | DPadDirection::Up | DPadDirection::UpLeft)
    }

    /// Get a vector representing the current DPad direction
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

    pub fn from_4_button(up: bool, down: bool, left: bool, right: bool) -> Self {
        match (up, down, left, right) {
            (false, false, false, false) => DPadDirection::Neutral,
            (true , false, false, false) => DPadDirection::Up,
            (true , false, false, true ) => DPadDirection::UpRight,
            (false, false, false, true ) => DPadDirection::Right,
            (false, true , false, true ) => DPadDirection::DownRight,
            (false, true , false, false) => DPadDirection::Down,
            (false, true , true , false) => DPadDirection::DownLeft,
            (false, false, true , false) => DPadDirection::Left,
            (true , false, true , false) => DPadDirection::UpLeft,
            _ => {
                log_warning!(LOG_INPUT_CAT, "Invalid DPAD state (up: {up}, down: {down}, left: {left}, right: {right})");
                DPadDirection::Neutral
            },
        }
    }

    pub fn from_8_button(up: bool, up_right: bool, right: bool, down_right: bool, down: bool, down_left: bool, left: bool, up_left: bool) -> DPadDirection {
        match (up, up_right, right, down_right, down, down_left, left, up_left) {
            (true , false, false, false, false, false, false, false) => DPadDirection::Up,
            (false, true , false, false, false, false, false, false) => DPadDirection::UpRight,
            (false, false, true , false, false, false, false, false) => DPadDirection::Right,
            (false, false, false, true , false, false, false, false) => DPadDirection::DownRight,
            (false, false, false, false, true , false, false, false) => DPadDirection::Down,
            (false, false, false, false, false, true , false, false) => DPadDirection::DownLeft,
            (false, false, false, false, false, false, true , false) => DPadDirection::Left,
            (false, false, false, false, false, false, false, true ) => DPadDirection::UpLeft,
            _ => {
                log_warning!(LOG_INPUT_CAT, "Only 1 button of an 8-button dpad can be pressed at any time, returning to neutral");
                DPadDirection::Neutral
            }    
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
    button:  GamepadButton,
    time:    f32,
    pressed: bool,
}

struct StickMove {
    pos:   f32v2,
    time:  f32,
    curve: GamepadReleaseCurve,
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

struct GamepadChangeState {
    buttons:  Vec<ButtonChange>,
    dpad:     (DPadDirection, f32),
    sticks:   [StickMove; 2],
    triggers: [TriggerMove; 2],
}

impl GamepadChangeState {
    pub fn new() -> Self {
        Self {
            buttons: Vec::new(),
            dpad: (DPadDirection::Neutral, 0.0),
            sticks: [StickMove::new(), StickMove::new()],
            triggers: [TriggerMove::new(), TriggerMove::new()],
        }
    }
}

/// Generic controller/gamepad
/// 
/// When making a custom gamepad that is derived from the standard gamepad, use this as the underlying input device and build additional functionality on top of this.
pub struct Gamepad {
    handle:         Option<NativeDeviceHandle>,
    state:          RwLock<GamepadState>,
    changes:        Mutex<GamepadChangeState>,

    button_timers:  [f32; GamepadButton::COUNT],
}

impl Gamepad {
    // TODO: Make all InputAxisIds when moved to interned strings (static string ids)
    pub const LEFT_THUMB_STR:         &'static str = "Gamepad Left Thumbstick 2D-Axis";
    pub const LEFT_THUMB_X_STR:       &'static str = "Gamepad Left Thumbstick X-Axis";
    pub const LEFT_THUMB_Y_STR:       &'static str = "Gamepad Left Thumbstick Y-Axis";
    pub const LEFT_THUMB_BUTTON_STR:  &'static str = "Gamepad Left Thumbstick Button";
    pub const RIGHT_THUMB_STR:        &'static str = "Gamepad Right Thumbstick 2D-Axis";
    pub const RIGHT_THUMB_X_STR:      &'static str = "Gamepad Right Thumbstick X-Axis";
    pub const RIGHT_THUMB_Y_STR:      &'static str = "Gamepad Right Thumbstick Y-Axis";
    pub const RIGHT_THUMB_BUTTON_STR: &'static str = "Gamepad Right Thumbstick Button";
    pub const DPAD_DIR_STR:           &'static str = "Gamepad D-Pad Direction";
    pub const DPAD_UP_STR:            &'static str = "Gamepad D-Pad Up";
    pub const DPAD_DOWN_STR:          &'static str = "Gamepad D-Pad Down";
    pub const DPAD_LEFT_STR:          &'static str = "Gamepad D-Pad Left";
    pub const DPAD_RIGHT_STR:         &'static str = "Gamepad D-Pad Right";
    pub const FACE_BOTTOM_STR:        &'static str = "Gamepad Face Button Bottom";
    pub const FACE_RIGHT_STR:         &'static str = "Gamepad Face Button Right";
    pub const FACE_LEFT_STR:          &'static str = "Gamepad Face Button Left";
    pub const FACE_UP_STR:            &'static str = "Gamepad Face Button Top";
    pub const LEFT_SPECIAL_STR:       &'static str = "Gamepad Left Special";
    pub const RIGHT_SPECIAL_STR:      &'static str = "Gamepad Right Special";
    pub const LEFT_BUMPER_STR:        &'static str = "Gamepad Left Bumper";
    pub const RIGHT_BUMPER_STR:       &'static str = "Gamepad Right Bumper";
    pub const LEFT_TRIGGER_STR:       &'static str = "Gamepad Left Trigger";
    pub const RIGHT_TRIGGER_STR:      &'static str = "Gamepad Right Trigger";
    pub const GUIDE_STR:              &'static str = "Gamepad Guide button";

    pub const LEFT_THUMB:         InputAxisId = InputAxisId::new(Self::LEFT_THUMB_STR        );
    pub const LEFT_THUMB_X:       InputAxisId = InputAxisId::new(Self::LEFT_THUMB_X_STR      );
    pub const LEFT_THUMB_Y:       InputAxisId = InputAxisId::new(Self::LEFT_THUMB_Y_STR      );
    pub const LEFT_THUMB_BUTTON:  InputAxisId = InputAxisId::new(Self::LEFT_THUMB_BUTTON_STR );
    pub const RIGHT_THUMB:        InputAxisId = InputAxisId::new(Self::RIGHT_THUMB_STR       );
    pub const RIGHT_THUMB_X:      InputAxisId = InputAxisId::new(Self::RIGHT_THUMB_X_STR     );
    pub const RIGHT_THUMB_Y:      InputAxisId = InputAxisId::new(Self::RIGHT_THUMB_Y_STR     );
    pub const RIGHT_THUMB_BUTTON: InputAxisId = InputAxisId::new(Self::RIGHT_THUMB_BUTTON_STR);
    pub const DPAD_DIR:           InputAxisId = InputAxisId::new(Self::DPAD_DIR_STR          );
    pub const DPAD_UP:            InputAxisId = InputAxisId::new(Self::DPAD_UP_STR           );
    pub const DPAD_DOWN:          InputAxisId = InputAxisId::new(Self::DPAD_DOWN_STR         );
    pub const DPAD_LEFT:          InputAxisId = InputAxisId::new(Self::DPAD_LEFT_STR         );
    pub const DPAD_RIGHT:         InputAxisId = InputAxisId::new(Self::DPAD_RIGHT_STR        );
    pub const FACE_BOTTOM:        InputAxisId = InputAxisId::new(Self::FACE_BOTTOM_STR       );
    pub const FACE_RIGHT:         InputAxisId = InputAxisId::new(Self::FACE_RIGHT_STR        );
    pub const FACE_LEFT:          InputAxisId = InputAxisId::new(Self::FACE_LEFT_STR         );
    pub const FACE_TOP:           InputAxisId = InputAxisId::new(Self::FACE_UP_STR           );
    pub const LEFT_SPECIAL:       InputAxisId = InputAxisId::new(Self::LEFT_SPECIAL_STR      );
    pub const RIGHT_SPECIAL:      InputAxisId = InputAxisId::new(Self::RIGHT_SPECIAL_STR     );
    pub const LEFT_BUMPER:        InputAxisId = InputAxisId::new(Self::LEFT_BUMPER_STR       );
    pub const RIGHT_BUMPER:       InputAxisId = InputAxisId::new(Self::RIGHT_BUMPER_STR      );
    pub const LEFT_TRIGGER:       InputAxisId = InputAxisId::new(Self::LEFT_TRIGGER_STR      );
    pub const RIGHT_TRIGGER:      InputAxisId = InputAxisId::new(Self::RIGHT_TRIGGER_STR     );
    pub const GUIDE:              InputAxisId = InputAxisId::new(Self::GUIDE_STR     );

    pub fn new(handle: NativeDeviceHandle) -> Result<Gamepad, NativeDeviceHandle> {
        Ok(Self {
            handle: Some(handle),
            state: RwLock::new(GamepadState::new()),
            changes: Mutex::new(GamepadChangeState::new()),
            button_timers: [0f32; GamepadButton::COUNT]
        })
    }

    pub unsafe fn new_no_handle() -> Self {
        Self {
            handle: None,
            state: RwLock::new(GamepadState::new()),
            changes: Mutex::new(GamepadChangeState::new()),
            button_timers: [0f32; GamepadButton::COUNT]
        }
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

    /// Emulate a button press or release
    pub fn set_button(&self, button: GamepadButton, time: f32, pressed: bool) {
        self.changes.lock().buttons.push(ButtonChange { button, time, pressed })
    }

    /// Emulate a dpad movement
    /// 
    /// When `time` has passed, the dpad will return to neutral
    pub fn move_dpad(&self, dir: DPadDirection, time: f32) {
        self.changes.lock().dpad = (dir, time);
    }

    /// Emulate a stick movement
    /// 
    /// When `time` has passed, the joystick will return to center
    // TODO: Return to center with a curve?
    pub fn move_stick(&self, right: bool, pos: f32v2, time: f32, curve: GamepadReleaseCurve) {
        self.changes.lock().sticks[right as usize] = StickMove{ pos, time, curve };
    }

    pub fn move_trigger(&self, right: bool, val: f32, time: f32, curve: GamepadReleaseCurve) {
        self.changes.lock().triggers[right as usize] = TriggerMove{ val, time, curve };
    }
}

impl InputDevice for Gamepad {
    fn get_native_handle(&self) -> &NativeDeviceHandle {
        self.handle.as_ref().unwrap()
    }

    fn tick(&mut self, dt: f32, notify_rebind: &mut dyn FnMut(InputAxisId)) {
        let mut state = self.state.write();
        let mut changes = self.changes.lock();

        scoped_alloc!(AllocId::TlsTemp);

        // We generally only care about the last action of a button, as a press and release should not happen in a single frame.
        // While this is possible, especially at a lower framerate, it doesn't make much sense in terms of the input system.
        let mut processed_buttons = BitSet::<{GamepadButton::COUNT}>::new();
        for change in changes.buttons.iter().rev() {
            let button_idx = change.button as usize;
            if processed_buttons.get(button_idx) {
                continue;
            }

            const BUTTON_OFFSET : usize = 13;
            if change.pressed {
                notify_rebind(InputAxisId::new(self.get_axes()[BUTTON_OFFSET + button_idx].path));
            }

            #[cfg(raw_input_logging)]
            if state.buttons.get(button_idx) != change.pressed {
                log_verbose!(LOG_INPUT_CAT, "Gamepad button {} {}", change.button, if change.pressed {"pressed"} else {"released"});
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
        
        let left_stick = changes.sticks[0].update(dt);
        let right_stick = changes.sticks[1].update(dt);

        #[cfg(raw_input_logging)]
        {
            if state.left_stick.dist_sq(left_stick) > 0.0001 {
                log_verbose!(LOG_INPUT_CAT, "Left stick moved to ({}, {})", left_stick.x, left_stick.y);
            }
            if state.right_stick.dist_sq(right_stick) > 0.0001 {
                log_verbose!(LOG_INPUT_CAT, "Right stick moved to ({}, {})", left_stick.x, left_stick.y);
            }
        }

        state.left_stick = left_stick;
        state.right_stick = right_stick;
        
        
        let left_trigger = changes.triggers[0].update(dt);
        if left_trigger > 0.5 {
            notify_rebind(Self::LEFT_TRIGGER);
        }

        #[cfg(raw_input_logging)]
        if f32::abs(state.left_trigger * state.left_trigger - left_trigger * left_trigger) > 0.0001 {
            log_verbose!(LOG_INPUT_CAT, "Left trigger moved to {left_trigger}");
        }

        state.left_trigger = left_trigger;

        let right_trigger = changes.triggers[1].update(dt);
        if right_trigger > 0.5 {
            notify_rebind(Self::RIGHT_TRIGGER);
        }

        #[cfg(raw_input_logging)]
        if f32::abs(state.right_trigger * state.right_trigger - right_trigger * right_trigger) > 0.0001 {
            log_verbose!(LOG_INPUT_CAT, "Right trigger moved to {right_trigger}");
        }

        state.right_trigger = right_trigger;
        
        // Return dpad back to neutral when it time runs out, otherwise just assign it and update the time
        let (dpad, dpad_time) = &mut changes.dpad;
        if *dpad_time == 0f32 {
            state.dpad = DPadDirection::Neutral;
        } else {
            #[cfg(raw_input_logging)]
            if state.dpad != *dpad {
                log_verbose!(LOG_INPUT_CAT, "Dpad moved to {dpad}");
            }

            state.dpad = *dpad;
            *dpad_time = (*dpad_time - dt).max(0f32);

            if dpad.is_up_down() {
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

    fn handle_hid_input(&mut self, _input_report: &[u8]) {
        // Nothing to do here
    }

    fn handle_native_input(&mut self, _native_data: *const std::ffi::c_void) {
        // Nothing to do here
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
            Self::DPAD_UP            => Some(AxisValue::Digital(self.state.read().dpad.is_up_down())),
            Self::DPAD_DOWN          => Some(AxisValue::Digital(self.state.read().dpad.is_bottom_down())),
            Self::DPAD_LEFT          => Some(AxisValue::Digital(self.state.read().dpad.is_left_down())),
            Self::DPAD_RIGHT         => Some(AxisValue::Digital(self.state.read().dpad.is_right_down())),
            Self::FACE_BOTTOM        => Some(AxisValue::Digital(self.state.read().buttons.get(GamepadButton::FaceBottom as usize))),
            Self::FACE_RIGHT         => Some(AxisValue::Digital(self.state.read().buttons.get(GamepadButton::FaceRight as usize))),
            Self::FACE_LEFT          => Some(AxisValue::Digital(self.state.read().buttons.get(GamepadButton::FaceLeft as usize))),
            Self::FACE_TOP           => Some(AxisValue::Digital(self.state.read().buttons.get(GamepadButton::FaceTop as usize))),
            Self::LEFT_BUMPER        => Some(AxisValue::Digital(self.state.read().buttons.get(GamepadButton::LeftBumper as usize))),
            Self::RIGHT_BUMPER       => Some(AxisValue::Digital(self.state.read().buttons.get(GamepadButton::RightBumper as usize))),
            Self::LEFT_SPECIAL       => Some(AxisValue::Digital(self.state.read().buttons.get(GamepadButton::LeftMenu as usize))),
            Self::RIGHT_SPECIAL      => Some(AxisValue::Digital(self.state.read().buttons.get(GamepadButton::RightMenu as usize))),
            Self::LEFT_THUMB_BUTTON  => Some(AxisValue::Digital(self.state.read().buttons.get(GamepadButton::LeftThumbstick as usize))),
            Self::RIGHT_THUMB_BUTTON => Some(AxisValue::Digital(self.state.read().buttons.get(GamepadButton::RightThumbsstick as usize))),
            Self::LEFT_TRIGGER       => Some(AxisValue::Axis(self.state.read().left_trigger)),
            Self::RIGHT_TRIGGER      => Some(AxisValue::Axis(self.state.read().right_trigger)),
            Self::GUIDE              => Some(AxisValue::Digital(self.state.read().buttons.get(GamepadButton::Guide as usize))),
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
            InputAxisDefinition { dev_type: DeviceType::Gamepad(GamepadSubType::Generic), path: Self::GUIDE_STR             , axis_type: AxisType::Digital, can_rebind: true },
        ]
    }

    fn get_device_type(&self) -> DeviceType {
        DeviceType::Gamepad(GamepadSubType::Generic)
    }
    
    fn take_native_handle(&mut self) -> NativeDeviceHandle {
        core::mem::take(&mut self.handle).unwrap()
    }
}