// TODO: Currently developed over parsec, which makes any controller act as the same type of device, so the current implementation (specifically trigger may not work as intended)

use crate::{AxisDefinition, AxisId, AxisMove, AxisValue, ButtonChange, DeviceType, GamepadFeatures, HatSwitch, InputAxisDefinition, InputDevice, NativeDeviceHandle, OutputInfo, Rebinder, ReleaseCurve, RumbleSupport};
#[cfg(feature = "raw_input_logging")]
use crate::LOG_INPUT_CAT;
use onca_common::{
    prelude::*,
    collections::BitSet,
    sync::{Mutex, RwLock},
};
use onca_common_macros::{EnumCount, EnumFromIndex, EnumDisplay};

#[cfg(feature = "raw_input_logging")]
use onca_logging::log_verbose;
use onca_math::{f32v2, Zero};

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
    /// Left trigger button
    #[display("left trigger button")]
    LeftTrigger,
    /// Right trigger button
    #[display("right trigger button")]
    RightTrigger,
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
const NUM_BUTTONS_BITS: usize = GamepadButton::COUNT.next_power_of_two();


const TRIGGER_AXIS_MAPPING: [&[AxisId]; 2] = [
    &[Gamepad::LEFT_THUMB ],
    &[Gamepad::RIGHT_THUMB],
];


struct GamepadState {
    buttons:  BitSet<NUM_BUTTONS_BITS>,
    dpad:     HatSwitch,
    sticks:    [f32v2; 2],
    triggers: [f32; 2],
}

impl GamepadState {
    fn new() -> Self {
        Self {
            buttons: BitSet::new(),
            dpad: HatSwitch::Neutral,
            sticks: Default::default(),
            triggers: Default::default(),
        }
    }
}

struct GamepadChangeState {
    buttons:  Vec<ButtonChange<GamepadButton>>,
    dpad:     (HatSwitch, f32),
    sticks:   [AxisMove<f32v2>; 2],
    triggers: [AxisMove<f32>; 2],
}

impl GamepadChangeState {
    pub fn new() -> Self {
        Self {
            buttons: Vec::new(),
            dpad: (HatSwitch::Neutral, 0.0),
            sticks: [AxisMove::default(), AxisMove::default()],
            triggers: [AxisMove::default(), AxisMove::default()],
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
    // Input
    pub const LEFT_THUMB:           AxisId = AxisId::new("Gamepad Left Thumbstick 2D-Axis");
    pub const LEFT_THUMB_X:         AxisId = AxisId::new("Gamepad Left Thumbstick X-Axis");
    pub const LEFT_THUMB_Y:         AxisId = AxisId::new("Gamepad Left Thumbstick Y-Axis");
    pub const LEFT_THUMB_BUTTON:    AxisId = AxisId::new("Gamepad Left Thumbstick Button");
    pub const RIGHT_THUMB:          AxisId = AxisId::new("Gamepad Right Thumbstick 2D-Axis");
    pub const RIGHT_THUMB_X:        AxisId = AxisId::new("Gamepad Right Thumbstick X-Axis");
    pub const RIGHT_THUMB_Y:        AxisId = AxisId::new("Gamepad Right Thumbstick Y-Axis");
    pub const RIGHT_THUMB_BUTTON:   AxisId = AxisId::new("Gamepad Right Thumbstick Button");
    pub const DPAD_DIR:             AxisId = AxisId::new("Gamepad D-Pad Direction");
    pub const DPAD_UP:              AxisId = AxisId::new("Gamepad D-Pad Up");
    pub const DPAD_DOWN:            AxisId = AxisId::new("Gamepad D-Pad Down");
    pub const DPAD_LEFT:            AxisId = AxisId::new("Gamepad D-Pad Left");
    pub const DPAD_RIGHT:           AxisId = AxisId::new("Gamepad D-Pad Right");
    pub const FACE_BOTTOM:          AxisId = AxisId::new("Gamepad Face Button Bottom");
    pub const FACE_RIGHT:           AxisId = AxisId::new("Gamepad Face Button Right");
    pub const FACE_LEFT:            AxisId = AxisId::new("Gamepad Face Button Left");
    pub const FACE_TOP:             AxisId = AxisId::new("Gamepad Face Button Top");
    pub const LEFT_MENU:            AxisId = AxisId::new("Gamepad Left Menu");
    pub const RIGHT_MENU:           AxisId = AxisId::new("Gamepad Right Menu");
    pub const LEFT_BUMPER:          AxisId = AxisId::new("Gamepad Left Bumper");
    pub const RIGHT_BUMPER:         AxisId = AxisId::new("Gamepad Right Bumper");
    pub const LEFT_TRIGGER:         AxisId = AxisId::new("Gamepad Left Trigger");
    pub const LEFT_TRIGGER_BUTTON:  AxisId = AxisId::new("Gamepad Left Trigger Button");
    pub const RIGHT_TRIGGER:        AxisId = AxisId::new("Gamepad Right Trigger");
    pub const RIGHT_TRIGGER_BUTTON: AxisId = AxisId::new("Gamepad Right Trigger Button");
    pub const GUIDE:                AxisId = AxisId::new("Gamepad Guide button");

    // Output
    pub const OUT_PLAYER_INDICATOR: AxisId = AxisId::new("Gamepad Player Indicator");

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

    /// Emulate a button press or release
    pub fn set_button(&self, button: GamepadButton, time: f32, pressed: bool) {
        self.changes.lock().buttons.push(ButtonChange { button, time, pressed })
    }

    /// Emulate a dpad movement
    /// 
    /// When `time` has passed, the dpad will return to neutral
    pub fn move_dpad(&self, dir: HatSwitch, time: f32) {
        self.changes.lock().dpad = (dir, time);
    }

    /// Emulate a stick movement
    /// 
    /// When `time` has passed, the joystick will return to center
    // TODO: Return to center with a curve?
    pub fn move_stick(&self, right: bool, pos: f32v2, time: f32, curve: ReleaseCurve) {
        self.changes.lock().sticks[right as usize] = AxisMove::new(pos, time, curve);
    }

    pub fn move_trigger(&self, right: bool, val: f32, time: f32, curve: ReleaseCurve) {
        self.changes.lock().triggers[right as usize] = AxisMove::new(val, time, curve);
    }
}

impl InputDevice for Gamepad {
    fn get_native_handle(&self) -> &NativeDeviceHandle {
        self.handle.as_ref().unwrap()
    }

    fn tick(&mut self, dt: f32, rebinder: &mut Rebinder) {
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
                rebinder.notify(self.get_axes()[BUTTON_OFFSET + button_idx].ids);
            }

            #[cfg(feature = "raw_input_logging")]
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
        
        for i in 0..state.sticks.len() {
            let stick = changes.sticks[i].update(dt, f32v2::zero());
            #[cfg(feature = "raw_input_logging")]
            if state.sticks[i].dist_sq(stick) > 0.0001 {
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
                rebinder.notify(&[Self::DPAD_UP]);
            }
            else if dpad.is_bottom_down() {
                rebinder.notify(&[Self::DPAD_DOWN]);
            }
            if dpad.is_left_down() {
                rebinder.notify(&[Self::DPAD_LEFT]);
            }
            else if dpad.is_right_down() {
                rebinder.notify(&[Self::DPAD_RIGHT]);
            }
        }
    }

    fn handle_hid_input(&mut self, _input_report: &[u8]) {
        // Nothing to do here
    }

    fn handle_native_input(&mut self, _native_data: *const std::ffi::c_void) {
        // Nothing to do here
    }

    fn get_axis_value(&self, axis_path: &AxisId) -> Option<AxisValue> {
        match *axis_path {
            Self::LEFT_THUMB           => Some(AxisValue::Axis2D(self.state.read().sticks[0])),
            Self::LEFT_THUMB_X         => Some(AxisValue::Axis(self.state.read().sticks[0].x)),
            Self::LEFT_THUMB_Y         => Some(AxisValue::Axis(self.state.read().sticks[0].y)),
            Self::RIGHT_THUMB          => Some(AxisValue::Axis2D(self.state.read().sticks[1])),
            Self::RIGHT_THUMB_X        => Some(AxisValue::Axis(self.state.read().sticks[1].x)),
            Self::RIGHT_THUMB_Y        => Some(AxisValue::Axis(self.state.read().sticks[1].y)),
            Self::DPAD_DIR             => Some(AxisValue::Axis2D(self.state.read().dpad.get_direction(true))),
            Self::DPAD_UP              => Some(AxisValue::Digital(self.state.read().dpad.is_up_down())),
            Self::DPAD_DOWN            => Some(AxisValue::Digital(self.state.read().dpad.is_bottom_down())),
            Self::DPAD_LEFT            => Some(AxisValue::Digital(self.state.read().dpad.is_left_down())),
            Self::DPAD_RIGHT           => Some(AxisValue::Digital(self.state.read().dpad.is_right_down())),
            Self::FACE_BOTTOM          => Some(AxisValue::Digital(self.state.read().buttons.get(GamepadButton::FaceBottom as usize))),
            Self::FACE_RIGHT           => Some(AxisValue::Digital(self.state.read().buttons.get(GamepadButton::FaceRight as usize))),
            Self::FACE_LEFT            => Some(AxisValue::Digital(self.state.read().buttons.get(GamepadButton::FaceLeft as usize))),
            Self::FACE_TOP             => Some(AxisValue::Digital(self.state.read().buttons.get(GamepadButton::FaceTop as usize))),
            Self::LEFT_BUMPER          => Some(AxisValue::Digital(self.state.read().buttons.get(GamepadButton::LeftBumper as usize))),
            Self::RIGHT_BUMPER         => Some(AxisValue::Digital(self.state.read().buttons.get(GamepadButton::RightBumper as usize))),
            Self::LEFT_TRIGGER_BUTTON  => Some(AxisValue::Digital(self.state.read().buttons.get(GamepadButton::LeftTrigger as usize))),
            Self::RIGHT_TRIGGER_BUTTON => Some(AxisValue::Digital(self.state.read().buttons.get(GamepadButton::RightTrigger as usize))),
            Self::LEFT_MENU            => Some(AxisValue::Digital(self.state.read().buttons.get(GamepadButton::LeftMenu as usize))),
            Self::RIGHT_MENU           => Some(AxisValue::Digital(self.state.read().buttons.get(GamepadButton::RightMenu as usize))),
            Self::LEFT_THUMB_BUTTON    => Some(AxisValue::Digital(self.state.read().buttons.get(GamepadButton::LeftThumbstick as usize))),
            Self::RIGHT_THUMB_BUTTON   => Some(AxisValue::Digital(self.state.read().buttons.get(GamepadButton::RightThumbsstick as usize))),
            Self::LEFT_TRIGGER         => Some(AxisValue::Axis(self.state.read().triggers[0])),
            Self::RIGHT_TRIGGER        => Some(AxisValue::Axis(self.state.read().triggers[0])),
            Self::GUIDE                => Some(AxisValue::Digital(self.state.read().buttons.get(GamepadButton::Guide as usize))),
            _ => None
        }
    }

    fn get_axes(&self) -> &[InputAxisDefinition] {
        const ONE_V2:  f32v2 = f32v2{ x: 1.0, y: 1.0 };
        const MONE_V2: f32v2 = f32v2{ x: -1.0, y: -1.0 };

        &[
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Self::LEFT_THUMB]        , axis: AxisDefinition::Axis2D(MONE_V2, ONE_V2), can_rebind: false},
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Self::LEFT_THUMB_X]      , axis: AxisDefinition::Axis  (-1.0   , 1.0)   , can_rebind: false},
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Self::LEFT_THUMB_Y]      , axis: AxisDefinition::Axis  (-1.0   , 1.0)   , can_rebind: false},
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Self::RIGHT_THUMB]       , axis: AxisDefinition::Axis2D(MONE_V2, ONE_V2), can_rebind: false},
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Self::RIGHT_THUMB_X]     , axis: AxisDefinition::Axis  (-1.0   , 1.0)   , can_rebind: false},
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Self::RIGHT_THUMB_Y]     , axis: AxisDefinition::Axis  (-1.0   , 1.0)   , can_rebind: false},
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Self::DPAD_DIR]          , axis: AxisDefinition::Axis2D(MONE_V2, ONE_V2), can_rebind: false},
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Self::DPAD_UP]           , axis: AxisDefinition::Digital                , can_rebind: true},
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Self::DPAD_DOWN]         , axis: AxisDefinition::Digital                , can_rebind: true},
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Self::DPAD_LEFT]         , axis: AxisDefinition::Digital                , can_rebind: true},
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Self::DPAD_RIGHT]        , axis: AxisDefinition::Digital                , can_rebind: true},
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Self::FACE_BOTTOM]       , axis: AxisDefinition::Digital                , can_rebind: true},
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Self::FACE_RIGHT]        , axis: AxisDefinition::Digital                , can_rebind: true},
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Self::FACE_LEFT]         , axis: AxisDefinition::Digital                , can_rebind: true},
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Self::FACE_TOP]          , axis: AxisDefinition::Digital                , can_rebind: true},
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Self::RIGHT_MENU]        , axis: AxisDefinition::Digital                , can_rebind: true},
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Self::RIGHT_BUMPER]      , axis: AxisDefinition::Digital                , can_rebind: true},
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Self::LEFT_MENU]         , axis: AxisDefinition::Digital                , can_rebind: true},
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Self::LEFT_BUMPER]       , axis: AxisDefinition::Digital                , can_rebind: true},
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Self::LEFT_THUMB_BUTTON] , axis: AxisDefinition::Digital                , can_rebind: true},
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Self::RIGHT_THUMB_BUTTON], axis: AxisDefinition::Digital                , can_rebind: true},
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Self::LEFT_TRIGGER]      , axis: AxisDefinition::Axis  (0.0    , 1.0)   , can_rebind: true},
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Self::RIGHT_TRIGGER]     , axis: AxisDefinition::Axis  (0.0    , 1.0)   , can_rebind: true},
            InputAxisDefinition{ dev_type: DeviceType::Gamepad(GamepadFeatures::None), ids: &[Self::GUIDE]             , axis: AxisDefinition::Digital                , can_rebind: true},
        ]
    }

    fn get_device_type(&self) -> DeviceType {
        DeviceType::Gamepad(GamepadFeatures::None)
    }
    
    fn take_native_handle(&mut self) -> NativeDeviceHandle {
        core::mem::take(&mut self.handle).unwrap()
    }

    fn get_battery_info(&self) -> Option<crate::BatteryInfo> {
        None
    }

    fn get_output_info<'a>(&'a self) -> &'a OutputInfo<'a> {
        &OutputInfo {
            rumble: RumbleSupport::None,
            trigger_feedback: None,
            led_support: &[],
            output_axes: &[]
        }
    }

    fn set_rumble(&self, _rumble: crate::RumbleState) {
        // Nothing to do here, as we don't support output yes
    }

    fn set_trigger_feedback(&self, _right_trigger: bool, _trigger_feedback: crate::TriggerFeedback) {
        // Nothing to do here, as we don't support output yes
    }

    fn set_led_state(&self, _index: u16, _state: crate::LedState) {
        // Nothing to do here, as we don't support output yes
    }

    fn set_output_axis(&self, _axis: AxisId, _value: AxisValue) {
        // Nothing to do here, as we don't support output yes
    }
}