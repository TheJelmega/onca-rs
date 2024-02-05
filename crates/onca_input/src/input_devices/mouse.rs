use onca_base::{EnumCountT, EnumFromIndexT};
use onca_common::{
    collections::BitSet,
    sync::{RwLock, Mutex},
};
use onca_common_macros::{EnumCount, EnumFromIndex, EnumDisplay};
use onca_logging::log_verbose;
use onca_math::*;
use windows::Win32::UI::Input::RAWMOUSE;

use crate::{os::{self, OSMouse}, AxisDefinition, AxisId, AxisValue, DeviceType, InputAxisDefinition, InputDevice, NativeDeviceHandle, OutputInfo, Rebinder, RumbleSupport, LOG_INPUT_CAT};


/// Mouse button
#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumCount, EnumFromIndex, EnumDisplay)]
#[allow(unused)]
pub enum MouseButton {
    /// Left mouse button
    #[display("left button")]
    Left,
    /// Middle mouse button
    #[display("middle button")]
    Middle,
    /// Right mouse button
    #[display("right button")]
    Right,
    /// Side mouse button 0
    #[display("side button 0")]
    Side0,
    /// Side mouse button 1
    #[display("side button 1")]
    Side1,
}
const MOUSE_BUTTON_BITS: usize = MouseButton::COUNT.next_power_of_two();

/// Button state
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ButtonState {
    /// Button is up.
    Up,
    /// Button is pressed (implies that the key is down).
    Pressed,
    /// Button is down, but was pressed in a previous frame.
    Down,
    /// Button was released (implies that the key is up).
    Released
}

impl ButtonState {
    pub fn is_down(self) -> bool {
        matches!(self, Self::Pressed | Self::Down)
    }

    pub fn is_up(self) -> bool {
        matches!(self, Self::Released | Self::Down)
    }
}

pub type MousePosition = i16v2;
pub type MouseDelta = i16v2;
pub type MouseScroll = f32v2;


struct ButtonChange {
    button:  MouseButton,
    time:    f32,
    pressed: bool,
}

struct MouseState {
    pressed:  BitSet<MOUSE_BUTTON_BITS>,
    down:     BitSet<MOUSE_BUTTON_BITS>,
    released: BitSet<MOUSE_BUTTON_BITS>,
    position: MousePosition,
    delta:    MouseDelta,
    scroll:   MouseScroll
}

impl MouseState {
    pub fn new() -> Self {
        Self {
            pressed: BitSet::new(),
            down: BitSet::new(),
            released: BitSet::new(),
            position: MousePosition::zero(),
            delta: MouseDelta::zero(),
            scroll: MouseScroll::zero(),
        }
    }

    pub fn prepare_for_update(&mut self) {
        self.pressed.clear();
        self.released.clear();
    }

    pub fn press(&mut self, idx: usize) {
        self.pressed.enable(idx);
        self.down.enable(idx);
    }

    pub fn release(&mut self, idx: usize) {
        self.pressed.disable(idx);
        self.down.disable(idx);
        self.released.enable(idx);
    }

    pub fn get_button_state(&self, button: MouseButton) -> ButtonState {
        let idx = button as usize;
        if self.pressed.get(idx) {
            ButtonState::Pressed
        } else if self.down.get(idx) {
            ButtonState::Down
        } else if self.released.get(idx) {
            ButtonState::Released
        } else {
            ButtonState::Up
        }
    }

    pub fn is_released(&self, button: MouseButton) -> bool {
        let idx = button as usize;
        self.released.get(idx)
    }
}

struct MouseChangeState {
    buttons:  Vec<ButtonChange>,
    mouse_pos: MousePosition,
    scroll:   MouseScroll,
}

impl MouseChangeState {
    pub fn new() -> Self {
        Self {
            buttons: Vec::new(),
            mouse_pos: MousePosition::zero(),
            scroll: MouseScroll::zero(),
        }
    }
}

pub struct Mouse {
    _os_mouse:      os::OSMouse,
    handle:         Option<NativeDeviceHandle>,
    state:          RwLock<MouseState>,
    change_state:   Mutex<MouseChangeState>,
    button_timers:  [f32; MouseButton::COUNT],
}

impl Mouse {
    pub const XY:            AxisId = AxisId::new("Mouse XY");
    pub const X:             AxisId = AxisId::new("Mouse X");
    pub const Y:             AxisId = AxisId::new("Mouse Y");
    pub const WHEEL:         AxisId = AxisId::new("Mouse Wheel Axis");
    pub const WHEEL_UP:      AxisId = AxisId::new("Mouse Wheel Up");
    pub const WHEEL_DOWN:    AxisId = AxisId::new("Mouse Wheel Down");
    pub const HWHEEL:        AxisId = AxisId::new("Mouse Wheel Horizontal Axis");
    pub const HWHEEL_LEFT:   AxisId = AxisId::new("Mouse Wheel Left");
    pub const HWHEEL_RIGHT:  AxisId = AxisId::new("Mouse Wheel Right");
    pub const LEFT_BUTTON:   AxisId = AxisId::new("Mouse Left Button");
    pub const MIDDLE_BUTTON: AxisId = AxisId::new("Mouse Middle Button");
    pub const RIGHT_BUTTON:  AxisId = AxisId::new("Mouse Right Button");
    pub const SIDE0_BUTTON:  AxisId = AxisId::new("Mouse Side Button 0");
    pub const SIDE1_BUTTON:  AxisId = AxisId::new("Mouse Side Button 1");

    /// Create a new mouse.
    pub fn new(handle: NativeDeviceHandle) -> Result<Self, NativeDeviceHandle> {
        match os::OSMouse::new() {
            Some(os_mouse) => Ok(Self {
                _os_mouse: os_mouse,
                handle: Some(handle),
                state: RwLock::new(MouseState::new()),
                change_state: Mutex::new(MouseChangeState::new()),
                button_timers: [0f32; MouseButton::COUNT],
             }),
            None => Err(handle),
        }
    }

    /// Emulate a mouse button press.
    pub fn press_button(&self, button: MouseButton, time: f32) {
        self.change_state.lock().buttons.push(ButtonChange { button, time, pressed: true });
    }

    /// Emulate a mouse button release.
    pub fn release_button(&self, button: MouseButton) {
        self.change_state.lock().buttons.push(ButtonChange { button, time: 0f32, pressed: false });
    }

    pub fn set_mouse_pos(&self, pos: MousePosition) {
        self.change_state.lock().mouse_pos = pos;
    }

    /// Emulate a mouse movement.
    pub fn move_mouse(&self, delta: MouseDelta) {
        self.change_state.lock().mouse_pos += delta;
    }

    /// Emulate a mouse wheel scroll
    pub fn scroll_wheel(&self, delta: MouseScroll) {
        self.change_state.lock().scroll += delta;
    }

    /// Get the button state for a given button
    pub fn get_button_state(&self, button: MouseButton) -> ButtonState {
        self.state.read().get_button_state(button)
    }

    /// Get the mouse position
    pub fn get_mouse_position(&self) -> MousePosition {
        self.state.read().position
    }
    
    /// Get the mouse delta
    pub fn get_mouse_delta(&self) -> MouseDelta {
        self.state.read().delta
    }
    
    pub fn get_mouse_wheel(&self) -> MouseScroll {
        self.state.read().scroll
    }
}

impl InputDevice for Mouse {
    fn get_native_handle(&self) -> &crate::NativeDeviceHandle {
        self.handle.as_ref().unwrap()
    }

    fn tick(&mut self, dt: f32, rebinder: &mut Rebinder) {
        let mut change_state = self.change_state.lock();
        let mut state = self.state.write();

        state.prepare_for_update();

        // Update mouse position, etc
        state.delta = change_state.mouse_pos - state.position;
        state.position = change_state.mouse_pos;

        #[cfg(feature = "mouse_pos_logging")]
        if !state.delta.is_zero() {
            log_verbose!(LOG_INPUT_CAT, "Mouse moved to position {} and delta {}", state.position, state.delta);
        }
        #[cfg(feature = "mouse_scroll_logging")]
        {
            if !change_state.scroll.x.is_zero() {
                log_verbose!(LOG_INPUT_CAT, "Horizontal scroll with delta of {}", change_state.scroll.x);
            }
            if !change_state.scroll.y.is_zero() {
                log_verbose!(LOG_INPUT_CAT, "Scroll with delta of {}", change_state.scroll.y);
            }
        }

        state.scroll += change_state.scroll;
        if !change_state.scroll.is_zero() {
            if change_state.scroll.x > 0f32 {
                rebinder.notify(&[Self::WHEEL, Self::WHEEL_UP]);
            } else if change_state.scroll.x < 0f32 {
                rebinder.notify(&[Self::WHEEL, Self::WHEEL_DOWN]);
            }
            
            if change_state.scroll.y > 0f32 {
                rebinder.notify(&[Self::HWHEEL, Self::HWHEEL_LEFT]);
            } else if change_state.scroll.y < 0f32 {
                rebinder.notify(&[Self::HWHEEL, Self::HWHEEL_RIGHT]);
            }
        }

        change_state.scroll = MouseScroll::zero();

        // We generally only care about the last action of a button, as a press and release should not happen in a single frame.
        // While this is possible, especially at a lower framerate, it doesn't make much sense in terms of the input system.
        let mut processed_buttons = BitSet::<{MouseButton::COUNT}>::new();
        for change in change_state.buttons.iter().rev() {
            let button_idx = change.button as usize;
            if processed_buttons.get(button_idx) {
                continue;
            }

            if change.pressed {
                state.press(button_idx);
                self.button_timers[button_idx] = change.time;
                log_verbose!(LOG_INPUT_CAT, "{} has been pressed", change.button);
            } else {
                state.release(button_idx);
                log_verbose!(LOG_INPUT_CAT, "{} has been released", change.button);
            }

            processed_buttons.enable(button_idx);

            const BUTTON_AXIS_OFFSET : usize = 9;
            rebinder.notify(self.get_axes()[BUTTON_AXIS_OFFSET + button_idx].ids);
        }
        change_state.buttons.clear();

        // Handle timers
        for (idx, timer) in self.button_timers.iter_mut().enumerate() {
            *timer = (*timer - dt).min(0f32);

            // SAFETY: `idx` is guaranteed to represent a valid mouse button
            let button = unsafe { MouseButton::from_idx(idx).unwrap_unchecked() };
            if !state.is_released(button) && *timer == 0f32 {
                state.release(idx);
                #[cfg(feature = "raw_input_logging")]
                log_verbose!(LOG_INPUT_CAT, "{} has been released", button);
            }
        }
        
    }

    fn handle_hid_input(&mut self, _input_report: &[u8]) {
        // We don't do anything here, as the mouse is special and gets input in a different way
    }

    fn handle_native_input(&mut self, native_data: *const std::ffi::c_void) {
        unsafe {
            let raw_mouse = &*(native_data as *const RAWMOUSE);
            OSMouse::process_window_event(self, raw_mouse);
        }
    }

    fn get_axis_value(&self, axis_path: &AxisId) -> Option<AxisValue> {
        match *axis_path {
            Self::XY            => Some(AxisValue::Axis2D( self.get_mouse_position().cast())),
            Self::X             => Some(AxisValue::Axis(   self.get_mouse_position().x as f32)),
            Self::Y             => Some(AxisValue::Axis(   self.get_mouse_position().y as f32)),
            Self::WHEEL         => Some(AxisValue::Axis(   self.get_mouse_wheel().y)),
            Self::WHEEL_UP      => Some(AxisValue::Axis(   self.get_mouse_wheel().y.max(0f32))),
            Self::WHEEL_DOWN    => Some(AxisValue::Axis(  -self.get_mouse_wheel().y.min(0f32))),
            Self::HWHEEL        => Some(AxisValue::Axis(   self.get_mouse_wheel().x)),
            Self::HWHEEL_LEFT   => Some(AxisValue::Axis(   self.get_mouse_wheel().x.max(0f32))),
            Self::HWHEEL_RIGHT  => Some(AxisValue::Axis(  -self.get_mouse_wheel().x.min(0f32))),
            Self::LEFT_BUTTON   => Some(AxisValue::Digital(self.get_button_state(MouseButton::Left).is_down())),
            Self::MIDDLE_BUTTON => Some(AxisValue::Digital(self.get_button_state(MouseButton::Middle).is_down())),
            Self::RIGHT_BUTTON  => Some(AxisValue::Digital(self.get_button_state(MouseButton::Right).is_down())),
            Self::SIDE0_BUTTON  => Some(AxisValue::Digital(self.get_button_state(MouseButton::Side0).is_down())),
            Self::SIDE1_BUTTON  => Some(AxisValue::Digital(self.get_button_state(MouseButton::Side1).is_down())),
            _ => None
        }
    }

    fn get_axes(&self) -> &[InputAxisDefinition] {
        const MIN_V2:  f32v2 = f32v2{ x: f32::MIN, y: f32::MAX };
        const MAX_V2:  f32v2 = f32v2{ x: f32::MAX, y: f32::MAX };

        &[        
            InputAxisDefinition { dev_type: DeviceType::Mouse, ids: &[Self::XY]           , axis: AxisDefinition::Axis2D(MIN_V2  , MAX_V2)  , can_rebind: false},
            InputAxisDefinition { dev_type: DeviceType::Mouse, ids: &[Self::X]            , axis: AxisDefinition::Axis  (f32::MIN, f32::MAX), can_rebind: false},
            InputAxisDefinition { dev_type: DeviceType::Mouse, ids: &[Self::Y]            , axis: AxisDefinition::Axis  (f32::MIN, f32::MAX), can_rebind: false},
            InputAxisDefinition { dev_type: DeviceType::Mouse, ids: &[Self::WHEEL]        , axis: AxisDefinition::Axis  (f32::MIN, f32::MAX), can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Mouse, ids: &[Self::WHEEL_UP]     , axis: AxisDefinition::Axis  (f32::MIN, f32::MAX), can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Mouse, ids: &[Self::WHEEL_DOWN]   , axis: AxisDefinition::Axis  (f32::MIN, f32::MAX), can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Mouse, ids: &[Self::HWHEEL]       , axis: AxisDefinition::Axis  (f32::MIN, f32::MAX), can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Mouse, ids: &[Self::HWHEEL_LEFT]  , axis: AxisDefinition::Axis  (f32::MIN, f32::MAX), can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Mouse, ids: &[Self::HWHEEL_RIGHT] , axis: AxisDefinition::Axis  (f32::MIN, f32::MAX), can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Mouse, ids: &[Self::LEFT_BUTTON]  , axis: AxisDefinition::Digital                   , can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Mouse, ids: &[Self::MIDDLE_BUTTON], axis: AxisDefinition::Digital                   , can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Mouse, ids: &[Self::RIGHT_BUTTON] , axis: AxisDefinition::Digital                   , can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Mouse, ids: &[Self::SIDE0_BUTTON] , axis: AxisDefinition::Digital                   , can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Mouse, ids: &[Self::SIDE1_BUTTON] , axis: AxisDefinition::Digital                   , can_rebind: true },
        ]
    }

    fn get_device_type(&self) -> DeviceType {
        DeviceType::Mouse
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
        // Nothing to do here, as we don't support output
    }

    fn set_trigger_feedback(&self, _right_trigger: bool, _trigger_feedback: crate::TriggerFeedback) {
        // Nothing to do here, as we don't support output
    }

    fn set_led_state(&self, _index: u16, _state: crate::LedState) {
        // Nothing to do here, as we don't support output
    }

    fn set_output_axis(&self, _axis: AxisId, _value: AxisValue) {
        // Nothing to do here, as we don't support output
    }
}