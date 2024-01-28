use onca_common::{
    prelude::*,
    collections::BitSet,
    sync::{RwLock, Mutex},
};
use onca_common_macros::{EnumCount, EnumFromIndex, EnumDisplay};
use onca_logging::log_verbose;
use onca_math::*;
use windows::Win32::UI::Input::RAWMOUSE;

use crate::{InputDevice, os::{self, OSMouse}, LOG_INPUT_CAT, InputAxisDefinition, AxisValue, AxisType, DeviceType, InputAxisId, NativeDeviceHandle};


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
    button_timers:  [f32; MouseButton::COUNT]
}

impl Mouse {
    pub const XY_STR:            &'static str = "Mouse XY";
    pub const X_STR:             &'static str = "Mouse X";
    pub const Y_STR:             &'static str = "Mouse Y";
    pub const WHEEL_STR:         &'static str = "Mouse Wheel Axis";
    pub const WHEEL_UP_STR :     &'static str = "Mouse Wheel Up";
    pub const WHEEL_DOWN_STR:    &'static str = "Mouse Wheel Down";
    pub const HWHEEL_STR:        &'static str = "Mouse Wheel Horizontal Axis";
    pub const HWHEEL_LEFT_STR:   &'static str = "Mouse Wheel Left";
    pub const HWHEEL_RIGHT_STR:  &'static str = "Mouse Wheel Right";
    pub const LEFT_BUTTON_STR:   &'static str = "Mouse Left Button";
    pub const MIDDLE_BUTTON_STR: &'static str = "Mouse Middle Button";
    pub const RIGHT_BUTTON_STR:  &'static str = "Mouse Right Button";
    pub const SIDE0_BUTTON_STR:  &'static str = "Mouse Side Button 0";
    pub const SIDE1_BUTTON_STR:  &'static str = "Mouse Side Button 1";

    pub const XY:            InputAxisId = InputAxisId::new(Self::XY_STR           );
    pub const X:             InputAxisId = InputAxisId::new(Self::X_STR            );
    pub const Y:             InputAxisId = InputAxisId::new(Self::Y_STR            );
    pub const WHEEL:         InputAxisId = InputAxisId::new(Self::WHEEL_STR        );
    pub const WHEEL_UP:      InputAxisId = InputAxisId::new(Self::WHEEL_UP_STR     );
    pub const WHEEL_DOWN:    InputAxisId = InputAxisId::new(Self::WHEEL_DOWN_STR   );
    pub const HWHEEL:        InputAxisId = InputAxisId::new(Self::HWHEEL_STR       );
    pub const HWHEEL_LEFT:   InputAxisId = InputAxisId::new(Self::HWHEEL_LEFT_STR  );
    pub const HWHEEL_RIGHT:  InputAxisId = InputAxisId::new(Self::HWHEEL_RIGHT_STR );
    pub const LEFT_BUTTON:   InputAxisId = InputAxisId::new(Self::LEFT_BUTTON_STR  );
    pub const MIDDLE_BUTTON: InputAxisId = InputAxisId::new(Self::MIDDLE_BUTTON_STR);
    pub const RIGHT_BUTTON:  InputAxisId = InputAxisId::new(Self::RIGHT_BUTTON_STR );
    pub const SIDE0_BUTTON:  InputAxisId = InputAxisId::new(Self::SIDE0_BUTTON_STR );
    pub const SIDE1_BUTTON:  InputAxisId = InputAxisId::new(Self::SIDE1_BUTTON_STR );

    /// Create a new mouse.
    pub fn new(handle: NativeDeviceHandle) -> Result<Self, NativeDeviceHandle> {
        match os::OSMouse::new() {
            Some(os_mouse) => Ok(Self {
                _os_mouse: os_mouse,
                handle: Some(handle),
                state: RwLock::new(MouseState::new()),
                change_state: Mutex::new(MouseChangeState::new()),
                button_timers: [0f32; MouseButton::COUNT]
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

    fn tick(&mut self, dt: f32, notify_rebind: &mut dyn FnMut(InputAxisId)) {
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
                notify_rebind(Self::WHEEL);
                notify_rebind(Self::WHEEL_UP);
            } else if change_state.scroll.x < 0f32 {
                notify_rebind(Self::WHEEL);
                notify_rebind(Self::WHEEL_DOWN);
            }

            if change_state.scroll.y > 0f32 {
                notify_rebind(Self::HWHEEL);
                notify_rebind(Self::HWHEEL_LEFT);
            } else if change_state.scroll.y < 0f32 {
                notify_rebind(Self::HWHEEL);
                notify_rebind(Self::HWHEEL_RIGHT);
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
            notify_rebind(InputAxisId::new(self.get_axes()[BUTTON_AXIS_OFFSET + button_idx].path));
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

    fn get_axis_value(&self, axis_path: &InputAxisId) -> Option<AxisValue> {
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
        &[
            InputAxisDefinition { dev_type: DeviceType::Mouse, path: Self::XY_STR            , axis_type: AxisType::Axis2D , can_rebind: false },
            InputAxisDefinition { dev_type: DeviceType::Mouse, path: Self::X_STR             , axis_type: AxisType::Axis   , can_rebind: false },
            InputAxisDefinition { dev_type: DeviceType::Mouse, path: Self::Y_STR             , axis_type: AxisType::Axis   , can_rebind: false },
            InputAxisDefinition { dev_type: DeviceType::Mouse, path: Self::WHEEL_STR         , axis_type: AxisType::Axis   , can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Mouse, path: Self::WHEEL_UP_STR      , axis_type: AxisType::Axis   , can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Mouse, path: Self::WHEEL_DOWN_STR    , axis_type: AxisType::Axis   , can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Mouse, path: Self::HWHEEL_STR        , axis_type: AxisType::Axis   , can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Mouse, path: Self::HWHEEL_LEFT_STR   , axis_type: AxisType::Axis   , can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Mouse, path: Self::HWHEEL_RIGHT_STR  , axis_type: AxisType::Axis   , can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Mouse, path: Self::LEFT_BUTTON_STR   , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Mouse, path: Self::MIDDLE_BUTTON_STR , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Mouse, path: Self::RIGHT_BUTTON_STR  , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Mouse, path: Self::SIDE0_BUTTON_STR  , axis_type: AxisType::Digital, can_rebind: true },
            InputAxisDefinition { dev_type: DeviceType::Mouse, path: Self::SIDE1_BUTTON_STR  , axis_type: AxisType::Digital, can_rebind: true },
        ]
    }

    fn get_device_type(&self) -> DeviceType {
        DeviceType::Mouse
    }
    
    fn take_native_handle(&mut self) -> NativeDeviceHandle {
        core::mem::take(&mut self.handle).unwrap()
    }
}