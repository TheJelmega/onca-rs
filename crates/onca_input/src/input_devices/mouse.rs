use core::fmt;
use onca_core::{
    prelude::*,
    collections::BitSet,
    sync::{RwLock, Mutex},
};
use onca_logging::log_verbose;
use onca_math::*;

use crate::{InputDevice, os, LOG_INPUT_CAT, InputAxisDefinition, AxisValue, AxisType, DeviceType, InputAxisId};


/// Mouse button
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(unused)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    Side0,
    Side1,
}
const NUM_BUTTONS : usize = MouseButton::Side1 as usize + 1;
const MOUSE_BUTTON_BITS : usize = NUM_BUTTONS.next_power_of_two();

impl MouseButton {
    pub fn from_idx(idx: usize) -> Option<MouseButton> {
        match idx {
            0 => Some(Self::Left),
            1 => Some(Self::Middle),
            2 => Some(Self::Right),
            3 => Some(Self::Side0),
            4 => Some(Self::Side1),
            _ => None,
        }
    }
}

impl fmt::Display for MouseButton {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MouseButton::Left   => f.write_str("left button"),
            MouseButton::Middle => f.write_str("middle button"),
            MouseButton::Right  => f.write_str("right button"),
            MouseButton::Side0  => f.write_str("side button 0"),
            MouseButton::Side1  => f.write_str("side button 1"),
        }
    }
}

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
    button  : MouseButton,
    time    : f32,
    pressed : bool,
}


struct MouseState {
    pressed  : BitSet<MOUSE_BUTTON_BITS>,
    down     : BitSet<MOUSE_BUTTON_BITS>,
    released : BitSet<MOUSE_BUTTON_BITS>,
    position : MousePosition,
    delta    : MouseDelta,
    scroll   : MouseScroll
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
        self.pressed.set(idx, true);
        self.down.set(idx, true);
    }

    pub fn release(&mut self, idx: usize) {
        self.pressed.set(idx, false);
        self.down.set(idx, false);
        self.released.set(idx, true);
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

pub struct Mouse {
    _os_mouse        : os::OSMouse,
    state            : RwLock<MouseState>,
    button_changes   : Mutex<DynArray<ButtonChange>>,
    move_pos         : Mutex<MousePosition>,
    scroll           : Mutex<MouseScroll>,
    button_timers    : [f32; NUM_BUTTONS]
}

impl Mouse {
    // TODO: Make all InputAxisIds when moved to interned strings (static string ids)
    pub const XY_STR            : &str = "Mouse XY 2D-Axis";
    pub const X_STR             : &str = "Mouse X";
    pub const Y_STR             : &str = "Mouse Y";
    pub const WHEEL_STR         : &str = "Mouse Wheel Axis";
    pub const WHEEL_UP_STR      : &str = "Mouse Wheel Up";
    pub const WHEEL_DOWN_STR    : &str = "Mouse Wheel Down";
    pub const HWHEEL_STR        : &str = "Mouse Wheel Horizontal Axis";
    pub const HWHEEL_LEFT_STR   : &str = "Mouse Mouse Wheel Left";
    pub const HWHEEL_RIGHT_STR  : &str = "Mouse Wheel Right";
    pub const LEFT_BUTTON_STR   : &str = "Mouse Left Button";
    pub const MIDDLE_BUTTON_STR : &str = "Mouse Middle Button";
    pub const RIGHT_BUTTON_STR  : &str = "Mouse Right Button";
    pub const SIDE0_BUTTON_STR  : &str = "Mouse Side Button0";
    pub const SIDE1_BUTTON_STR  : &str = "Mouse Side Button1";

    pub const XY            : InputAxisId = InputAxisId::new(Self::XY_STR           );
    pub const X             : InputAxisId = InputAxisId::new(Self::X_STR            );
    pub const Y             : InputAxisId = InputAxisId::new(Self::Y_STR            );
    pub const WHEEL         : InputAxisId = InputAxisId::new(Self::WHEEL_STR        );
    pub const WHEEL_UP      : InputAxisId = InputAxisId::new(Self::WHEEL_UP_STR     );
    pub const WHEEL_DOWN    : InputAxisId = InputAxisId::new(Self::WHEEL_DOWN_STR   );
    pub const HWHEEL        : InputAxisId = InputAxisId::new(Self::HWHEEL_STR       );
    pub const HWHEEL_LEFT   : InputAxisId = InputAxisId::new(Self::HWHEEL_LEFT_STR  );
    pub const HWHEEL_RIGHT  : InputAxisId = InputAxisId::new(Self::HWHEEL_RIGHT_STR );
    pub const LEFT_BUTTON   : InputAxisId = InputAxisId::new(Self::LEFT_BUTTON_STR  );
    pub const MIDDLE_BUTTON : InputAxisId = InputAxisId::new(Self::MIDDLE_BUTTON_STR);
    pub const RIGHT_BUTTON  : InputAxisId = InputAxisId::new(Self::RIGHT_BUTTON_STR );
    pub const SIDE0_BUTTON  : InputAxisId = InputAxisId::new(Self::SIDE0_BUTTON_STR );
    pub const SIDE1_BUTTON  : InputAxisId = InputAxisId::new(Self::SIDE1_BUTTON_STR );
    /// Create a new mouse.
    pub fn new() -> Option<Self> {
        os::OSMouse::new().map(|os_mouse| Self {
            _os_mouse: os_mouse,
            state: RwLock::new(MouseState::new()),
            button_changes: Mutex::new(DynArray::new()),
            move_pos: Mutex::new(MousePosition::zero()),
            scroll: Mutex::new(MouseScroll::zero()),
            button_timers: [0f32; NUM_BUTTONS]
         })
    }

    /// Emulate a mouse button press.
    pub fn press_button(&self, button: MouseButton, time: f32) {
        self.button_changes.lock().push(ButtonChange { button, time, pressed: true });
    }

    /// Emulate a mouse button release.
    pub fn release_button(&self, button: MouseButton) {
        self.button_changes.lock().push(ButtonChange { button, time: 0f32, pressed: false });
    }

    pub fn set_mouse_pos(&self, pos: MousePosition) {
        *self.move_pos.lock() = pos;
    }

    /// Emulate a mouse movement.
    pub fn move_mouse(&self, delta: MouseDelta) {
        *self.move_pos.lock() += delta;
    }

    /// Emulate a mouse wheel scroll
    pub fn scroll_wheel(&self, delta: MouseScroll) {
        *self.scroll.lock() += delta;
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
    fn tick(&mut self, dt: f32, notify_rebind: &mut dyn FnMut(InputAxisId)) {

        let mouse_pos = self.move_pos.lock();
        let mut scroll = self.scroll.lock();

        let mut button_changes = self.button_changes.lock();
        let mut state = self.state.write();

        state.prepare_for_update();

        // Update mouse position, etc
        state.delta = *mouse_pos - state.position;
        state.position = *mouse_pos;

        #[cfg(feature = "mouse_pos_logging")]
        if !state.delta.is_zero() {
            log_verbose!(LOG_INPUT_CAT, "Mouse moved to position {} and delta {}", state.position, state.delta);
        }
        #[cfg(feature = "raw_input_logging")]
        {
            if !scroll.x.is_zero() {
                log_verbose!(LOG_INPUT_CAT, "Horizontal scroll with delta of {}", scroll.x);
            }
            if !scroll.y.is_zero() {
                log_verbose!(LOG_INPUT_CAT, "Scroll with delta of {}", scroll.y);
            }
        }

        state.scroll += *scroll;
        if !scroll.is_zero() {
            if scroll.x > 0f32 {
                notify_rebind(Self::WHEEL);
                notify_rebind(Self::WHEEL_UP);
            } else if scroll.x < 0f32 {
                notify_rebind(Self::WHEEL);
                notify_rebind(Self::WHEEL_DOWN);
            }

            if scroll.y > 0f32 {
                notify_rebind(Self::HWHEEL);
                notify_rebind(Self::HWHEEL_LEFT);
            } else if scroll.y < 0f32 {
                notify_rebind(Self::HWHEEL);
                notify_rebind(Self::HWHEEL_RIGHT);
            }
        }

        *scroll = MouseScroll::zero();

        // We generally only care about the last action of a button, as a press and release should not happen in a single frame.
        // While this is possible, especially at a lower framerate, it doesn't make much sense in term of the input system
        let mut processed_buttons = BitSet::<NUM_BUTTONS>::new();
        for change in button_changes.iter().rev() {
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
        button_changes.clear();

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

    fn handle_hid_input(&mut self, _hid_device: &onca_hid::Device, _input_report: onca_hid::InputReport) {
        // We don't do anything here, as the mouse is special and gets input in a different way
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
}