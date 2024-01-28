use std::ffi::c_void;

use onca_common::dynlib::DynLib;
use onca_hid as hid;
use onca_logging::log_warning;
use onca_math::Vec2;
use windows::Win32::UI::Input::XboxController::*;

use crate::{Gamepad, NativeDeviceHandle, InputDevice, GamepadReleaseCurve, LOG_INPUT_CAT, DPadDirection, GamepadButton};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct XInputCapabilitiesEx {
    pub capabilities: XINPUT_CAPABILITIES,
    pub vendor_id: u16,
    pub product_id: u16,
    pub product_version: u16,
    pub unk1: u16,
    pub unk2: u32,
}



pub(crate) struct XInputContext {
    pub xinput_get_capabilities_ex: fn(u32, u32, u32, *mut c_void),
    pub attached_devices: [bool; 4],
}

impl XInputContext {
    pub fn new() -> Self {
        let dynlib = DynLib::load("XInput1_4.dll").unwrap();

        Self {
            // 108 is the ordinal for _XInputGetCapabilitiesEx, which additionally returns VID/PID of the controller
            xinput_get_capabilities_ex: dynlib.get_indexed(108).unwrap(),
            attached_devices: [false; 4],
        }
    }

    fn xinput_get_capabilities_ex(&self, user_id: u32, flags: u32) -> XInputCapabilitiesEx {
        let mut result = unsafe { core::mem::zeroed() };
        (self.xinput_get_capabilities_ex)(1, user_id, flags, &mut result as *mut _ as *mut _);
        result
    }

    /// Try to get the best device id for a given xinput device
    pub fn try_get_id_for_device(&mut self, iden: &hid::Identifier) -> Option<u32> {
        for i in 0..XUSER_MAX_COUNT {
            let caps = self.xinput_get_capabilities_ex(i, 0);
            if caps.vendor_id == iden.vendor_device.vendor.as_u16() &&
               caps.product_id == iden.vendor_device.device.as_u16() &&
               !self.attached_devices[i as usize]
            {
                self.attached_devices[i as usize] = true;
                return Some(i);
            }
        }
        None
    }
}

pub struct XInputGamepad {
    gamepad:       Gamepad,
    xinput_idx:    u32,
    cur_packet_id: u32,
}

impl XInputGamepad {
    pub fn new(ctx: &mut XInputContext, handle: NativeDeviceHandle) -> Result<Self, NativeDeviceHandle> {
        let xinput_idx = match ctx.try_get_id_for_device(handle.get_hid_identifier()) {
            Some(idx) => idx,
            None => return Err(handle),
        };
        
        Ok(Self {
            gamepad: Gamepad::new(handle)?,
            xinput_idx,
            cur_packet_id: 0,
        })
    }
}

impl InputDevice for XInputGamepad {
    fn tick(&mut self, _dt: f32, notify_rebind: &mut dyn FnMut(crate::InputAxisId)) {
        let mut state = XINPUT_STATE::default();
        unsafe { XInputGetState(self.xinput_idx, &mut state) };

        if state.dwPacketNumber != self.cur_packet_id {
            // DPad
            let up = state.Gamepad.wButtons.contains(XINPUT_GAMEPAD_DPAD_UP);
            let down = state.Gamepad.wButtons.contains(XINPUT_GAMEPAD_DPAD_DOWN);
            let left = state.Gamepad.wButtons.contains(XINPUT_GAMEPAD_DPAD_LEFT);
            let right = state.Gamepad.wButtons.contains(XINPUT_GAMEPAD_DPAD_RIGHT);

            match (up, down, left, right) {
                (false, false, false, false) => self.gamepad.move_dpad(DPadDirection::Neutral, f32::MAX),
                (true , false, false, false) => self.gamepad.move_dpad(DPadDirection::Up, f32::MAX),
                (true , false, false, true ) => self.gamepad.move_dpad(DPadDirection::UpRight, f32::MAX),
                (false, false, false, true ) => self.gamepad.move_dpad(DPadDirection::Right, f32::MAX),
                (false, true , false, true ) => self.gamepad.move_dpad(DPadDirection::DownRight, f32::MAX),
                (false, true , false, false) => self.gamepad.move_dpad(DPadDirection::Down, f32::MAX),
                (false, true , true , false) => self.gamepad.move_dpad(DPadDirection::DownLeft, f32::MAX),
                (false, false, true , false) => self.gamepad.move_dpad(DPadDirection::Left, f32::MAX),
                (true , false, true , false) => self.gamepad.move_dpad(DPadDirection::UpLeft, f32::MAX),
                _ => log_warning!(LOG_INPUT_CAT, "Invalid DPAD state (up: {up}, down: {down}, left: {left}, right: {right})"),
            }

            if up { notify_rebind(Gamepad::DPAD_UP); }
            if down { notify_rebind(Gamepad::DPAD_DOWN); }
            if left { notify_rebind(Gamepad::DPAD_LEFT); }
            if right { notify_rebind(Gamepad::DPAD_RIGHT); }

            // Buttons
            self.gamepad.set_button(GamepadButton::FaceBottom, f32::MAX, state.Gamepad.wButtons.contains(XINPUT_GAMEPAD_A));
            self.gamepad.set_button(GamepadButton::FaceRight, f32::MAX, state.Gamepad.wButtons.contains(XINPUT_GAMEPAD_B));
            self.gamepad.set_button(GamepadButton::FaceLeft, f32::MAX, state.Gamepad.wButtons.contains(XINPUT_GAMEPAD_X));
            self.gamepad.set_button(GamepadButton::FaceTop, f32::MAX, state.Gamepad.wButtons.contains(XINPUT_GAMEPAD_Y));
            self.gamepad.set_button(GamepadButton::LeftBumper, f32::MAX, state.Gamepad.wButtons.contains(XINPUT_GAMEPAD_LEFT_SHOULDER));
            self.gamepad.set_button(GamepadButton::RightBumper, f32::MAX, state.Gamepad.wButtons.contains(XINPUT_GAMEPAD_RIGHT_SHOULDER));
            self.gamepad.set_button(GamepadButton::LeftMenu, f32::MAX, state.Gamepad.wButtons.contains(XINPUT_GAMEPAD_BACK));
            self.gamepad.set_button(GamepadButton::RightMenu, f32::MAX, state.Gamepad.wButtons.contains(XINPUT_GAMEPAD_START));
            self.gamepad.set_button(GamepadButton::LeftThumbstick, f32::MAX, state.Gamepad.wButtons.contains(XINPUT_GAMEPAD_LEFT_THUMB));
            self.gamepad.set_button(GamepadButton::RightThumbsstick, f32::MAX, state.Gamepad.wButtons.contains(XINPUT_GAMEPAD_RIGHT_THUMB));

            if state.Gamepad.wButtons.contains(XINPUT_GAMEPAD_A) { notify_rebind(Gamepad::FACE_BOTTOM); }
            if state.Gamepad.wButtons.contains(XINPUT_GAMEPAD_B) { notify_rebind(Gamepad::FACE_RIGHT); }
            if state.Gamepad.wButtons.contains(XINPUT_GAMEPAD_X) { notify_rebind(Gamepad::FACE_LEFT); }
            if state.Gamepad.wButtons.contains(XINPUT_GAMEPAD_Y) { notify_rebind(Gamepad::FACE_TOP); }
            if state.Gamepad.wButtons.contains(XINPUT_GAMEPAD_LEFT_SHOULDER) { notify_rebind(Gamepad::LEFT_BUMPER); }
            if state.Gamepad.wButtons.contains(XINPUT_GAMEPAD_RIGHT_SHOULDER) { notify_rebind(Gamepad::RIGHT_BUMPER); }
            if state.Gamepad.wButtons.contains(XINPUT_GAMEPAD_BACK) { notify_rebind(Gamepad::LEFT_SPECIAL); }
            if state.Gamepad.wButtons.contains(XINPUT_GAMEPAD_START) { notify_rebind(Gamepad::RIGHT_SPECIAL); }
            if state.Gamepad.wButtons.contains(XINPUT_GAMEPAD_LEFT_THUMB) { notify_rebind(Gamepad::LEFT_THUMB_BUTTON); }
            if state.Gamepad.wButtons.contains(XINPUT_GAMEPAD_RIGHT_THUMB) { notify_rebind(Gamepad::RIGHT_THUMB_BUTTON); }

            // Thumbsticks
            let lx = (state.Gamepad.sThumbLX as i32 - i16::MIN as i32) as f32 / (u16::MAX / 2) as f32 - 1.0;
            let ly = (state.Gamepad.sThumbLY as i32 - i16::MIN as i32) as f32 / (u16::MAX / 2) as f32 - 1.0;
            self.gamepad.move_stick(false, Vec2::new(lx, ly), f32::MAX, GamepadReleaseCurve::Instant);

            if lx.abs() > 0.5 { notify_rebind(Gamepad::LEFT_THUMB_X); }
            if ly.abs() > 0.5 { notify_rebind(Gamepad::LEFT_THUMB_Y); }

            let rx = (state.Gamepad.sThumbRX as i32 - i16::MIN as i32) as f32 / (i16::MAX / 2) as f32 - 1.0;
            let ry = (state.Gamepad.sThumbRY as i32 - i16::MIN as i32) as f32 / (i16::MAX / 2) as f32 - 1.0;
            self.gamepad.move_stick(true, Vec2::new(rx, ry), f32::MAX, GamepadReleaseCurve::Instant);

            if rx.abs() > 0.5 { notify_rebind(Gamepad::RIGHT_THUMB_X); }
            if ry.abs() > 0.5 { notify_rebind(Gamepad::RIGHT_THUMB_Y); }

            // Triggers
            let lt = state.Gamepad.bLeftTrigger as f32 / 255.0;
            self.gamepad.move_trigger(false, lt, f32::MAX, GamepadReleaseCurve::Instant);
            let rt = state.Gamepad.bRightTrigger as f32 / 255.0;
            self.gamepad.move_trigger(true, rt, f32::MAX, GamepadReleaseCurve::Instant);

            if lt > 0.5 { notify_rebind(Gamepad::LEFT_TRIGGER) };
            if rt > 0.5 { notify_rebind(Gamepad::RIGHT_TRIGGER) };


            self.cur_packet_id = state.dwPacketNumber;
        }
    }

    fn handle_hid_input(&mut self, _input_report: &[u8]) {
        // Nothing to do here
    }

    fn handle_native_input(&mut self, _native_data: *const c_void) {
        // Nothing to do here
    }

    fn get_native_handle(&self) -> &NativeDeviceHandle {
        self.gamepad.get_native_handle()
    }

    fn get_axis_value(&self, axis: &crate::InputAxisId) -> Option<crate::AxisValue> {
        self.gamepad.get_axis_value(axis)
    }

    fn get_axes(&self) -> &[crate::InputAxisDefinition] {
        self.gamepad.get_axes()
    }

    fn get_device_type(&self) -> crate::DeviceType {
        self.gamepad.get_device_type()
    }

    fn take_native_handle(&mut self) -> NativeDeviceHandle {
        self.gamepad.take_native_handle()
    }
}