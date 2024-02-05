use onca_base::{EnumFromIndexT, EnumCountT};
use onca_hid as hid;
use onca_math::f32v2;
use crate::{AxisId, AxisValue, DefinitionAxis, DefinitionDPad, DefinitionKind, DeviceType, Gamepad, GamepadButton, GamepadFeatures, HatSwitch, InputAxisDefinition, InputDevice, InputDeviceDefinition, NativeDeviceHandle, OutputInfo, Rebinder, ReleaseCurve, RumbleSupport, UsageDef};


struct GamepadMapping {
    buttons: [Option<hid::Usage>; GamepadButton::COUNT],
    thumbsticks: [Option<(UsageDef, UsageDef)>; 2],
    triggers: [Option<UsageDef>; 2],

    dpad: DefinitionDPad,
}

const GAMEPAD_BUTTON_DEF_MAPPING: [&'static str; GamepadButton::COUNT] = [
    "face_bottom",
    "face_right",
    "face_left",
    "face_up",
    "left_bumper",
    "right_bumper",
    "left_trigger_button",
    "right_trigger_button",
    "left_menu",
    "right_menu",
    "left_thumbstick",
    "right_thumbstick",
    "guide",
];

const GAMEPAD_THUMBSTICK_DEF_MAPPING: [&'static str; 2] = [
    "left_thumbstick",
    "right_thumbstick",
];

const GAMEPAD_TRIGGER_DEF_MAPPING: [&'static str; 2] = [
    "left_trigger",
    "right_trigger",
];

pub struct GenericDevice {
    pub(crate) handle: Option<NativeDeviceHandle>,
    dev_type: DeviceType,
    gamepad: Option<(Gamepad, GamepadMapping)>,

    axis_info: Vec<InputAxisDefinition>
}

impl GenericDevice {
    pub fn new(handle: NativeDeviceHandle, definition: &InputDeviceDefinition) -> Result<Self, NativeDeviceHandle> {
        let mut axis_info = Vec::new();
        let mut dev_type = DeviceType::Other(String::new());

        let gamepad = if definition.kind.contains(DefinitionKind::Gamepad) {
            let mut buttons = [None; GamepadButton::COUNT];
            for i in 0..GamepadButton::COUNT {
                buttons[i] = definition.buttons.get(GAMEPAD_BUTTON_DEF_MAPPING[i]).map(|usage| usage.usage);
            }

            let mut thumbsticks = [None; 2];
            for i in 0..2 {
                if let Some(DefinitionAxis { x, y: Some(y_val), .. }) = definition.axes.get(GAMEPAD_THUMBSTICK_DEF_MAPPING[i]) {
                    thumbsticks[i] = Some((*x, *y_val));
                }
            }

            let mut triggers = [None; 2];
            for i in 0..2 {
                if let Some(DefinitionAxis{ x, .. }) = definition.axes.get(GAMEPAD_TRIGGER_DEF_MAPPING[i]) {
                    triggers[i] = Some(*x);
                }
            }

            let gamepad = unsafe { Gamepad::new_no_handle() };
            axis_info.extend(gamepad.get_axes().iter().map(|axis| axis.clone()));

            dev_type = DeviceType::Gamepad(GamepadFeatures::None);

            Some((
                unsafe { Gamepad::new_no_handle() },
                GamepadMapping {
                    buttons,
                    thumbsticks,
                    triggers,
                    dpad: definition.dpad.unwrap(),
                }
            ))
        } else {
            None
        };

        Ok(Self {
            handle: Some(handle),
            dev_type,
            gamepad,
            axis_info,
        })
    }

    fn get_raw_value(input_report: &hid::InputReport, usage: UsageDef) -> Option<u32> {
        let val = input_report.get_raw_value(usage.usage, None)?;
        Some(val.get_value(usage.report))
    }
    
    fn calculate_axis_value(hid_dev: &hid::Device, input_report: &hid::InputReport, usage: UsageDef) -> Option<f32> {
        let val = Self::get_raw_value(input_report, usage)?;
        let props = hid_dev.get_value_capabilities_for_usage(hid::ReportType::Input, usage.usage, None)?;

        let range = props.logical_range.end - props.logical_range.start;
        let val = (val as i32 - props.logical_range.start) as f32 / range as f32;
        Some(val)
    }

}

impl InputDevice for GenericDevice {
    fn tick(&mut self, dt: f32, rebinder: &mut Rebinder) {
        
        
        if let Some(gamepad) = &mut self.gamepad {
            gamepad.0.tick(dt, rebinder);
        }
 
    }

    fn handle_hid_input(&mut self, input_report: &[u8]) {
        let hid_dev = match unsafe { &self.handle.as_ref().unwrap_unchecked().hid_dev } {
            Some(hid_dev) => hid_dev,
            None => return,
        };
        let input_report = unsafe { hid::InputReport::from_raw_slice(input_report, hid_dev) };

        let buttons = input_report.get_buttons().unwrap_or(Vec::new());
        if let Some((gamepad, mapping)) = &mut self.gamepad {
            // Buttons
            for i in 0..GamepadButton::COUNT {
                if let Some(button_mapping) = mapping.buttons[i] {
                    let button = unsafe { GamepadButton::from_idx_unchecked(i) };
                    gamepad.set_button(button, f32::MAX, buttons.contains(&button_mapping));
                }
            }

            // DPad
            match mapping.dpad {
                DefinitionDPad::Hat{ usage, neutral } => {
                    let value = Self::get_raw_value(&input_report, usage).map_or(neutral as usize, |val| val as usize);
                    if neutral == 0 {
                        gamepad.move_dpad(unsafe { HatSwitch::from_idx_unchecked(value) }, f32::MAX);
                    } else if value == 8 {
                        gamepad.move_dpad(HatSwitch::Neutral, f32::MAX);
                    } else {
                        gamepad.move_dpad(unsafe { HatSwitch::from_idx_unchecked(value + 1) }, f32::MAX);
                    }

                },
                DefinitionDPad::Buttons { up, down, left, right, diags } => if let Some(diag) = diags {
                    let up = buttons.contains(&up.usage);
                    let down = buttons.contains(&down.usage);
                    let left = buttons.contains(&left.usage);
                    let right = buttons.contains(&right.usage);
                    let up_left = buttons.contains(&diag.up_left.usage);
                    let up_right = buttons.contains(&diag.up_right.usage);
                    let down_left = buttons.contains(&diag.down_left.usage);
                    let down_right = buttons.contains(&diag.down_right.usage);

                    gamepad.move_dpad(HatSwitch::from_8_button(up, up_right, right, down_right, down, down_left, left, up_left), f32::MAX);
                } else {
                    let up = buttons.contains(&up.usage);
                    let down = buttons.contains(&down.usage);
                    let left = buttons.contains(&left.usage);
                    let right = buttons.contains(&right.usage);

                    gamepad.move_dpad(HatSwitch::from_4_button(up, down, left, right), f32::MAX);
                },
            }
            
            // Values
            for (idx, thumbstick) in mapping.thumbsticks.iter().enumerate() {
                if let Some(thumbstick) = thumbstick {
                    let x = Self::calculate_axis_value(hid_dev, &input_report, thumbstick.0).map_or(0.0, |val| val * 2.0 - 1.0);
                    let y = Self::calculate_axis_value(hid_dev, &input_report, thumbstick.1).map_or(0.0, |val| val * 2.0 - 1.0);
                    gamepad.move_stick(idx == 1, f32v2::new(x, y), f32::MAX, ReleaseCurve::Instant);
                }
            }

            for (idx, trigger) in mapping.triggers.iter().enumerate() {
                if let Some(trigger) = trigger {
                    let val = Self::calculate_axis_value(hid_dev, &input_report, *trigger).unwrap_or(0.0);
                    gamepad.move_trigger(idx == 1, val, f32::MAX, ReleaseCurve::Instant);
                }
            }
        }
    }

    fn handle_native_input(&mut self, _native_data: *const std::ffi::c_void) {
        // Nothing to do here
    }

    fn get_native_handle(&self) -> &crate::NativeDeviceHandle {
        self.handle.as_ref().unwrap()
    }

    fn get_axis_value(&self, axis: &crate::AxisId) -> Option<crate::AxisValue> {
        if let Some(axis) = self.gamepad.as_ref().map(|gamepad| gamepad.0.get_axis_value(axis)) {
            return axis;
        }
        None
    }

    fn get_axes(&self) -> &[InputAxisDefinition] {
        &self.axis_info
    }

    fn get_device_type(&self) -> DeviceType {
        self.dev_type.clone()
    }

    fn take_native_handle(&mut self) -> crate::NativeDeviceHandle {
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