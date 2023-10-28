use core::mem::take;
use std::sync::{Weak, Arc};
use onca_common::{
    prelude::*,
    sync::Mutex,
    time::DeltaTime
};
use crate::{MappingContext, ControlSet, AxisValue, DeviceHandle, ControlSchemeID, InputProcessContext, Action, TriggerResult, InputAxisId};


pub struct User {
    /// Per-user mapping contexts
    /// 
    /// These are always sorted based on priority
    mappings_contexts   : Mutex<Vec<(u16, MappingContext)>>,
    /// Control set
    control_set         : Option<ControlSet>,
    /// Trigger results for the previous frame
    prev_trigger_res    : Vec<(Weak<Mutex<Action>>, TriggerResult)>,
    /// Disconnected control set
    disconnected_scheme : ControlSchemeID,
    /// Currently held devices (re-used when reconnecting)
    cur_held_devs       : Vec<DeviceHandle>,
    /// Disconnected device that could reconnect to this user
    disconnected_devs   : Vec<DeviceHandle>,

}


impl User {
    /// Create a new user
    pub fn new() -> Self {
        Self {
            mappings_contexts: Mutex::new(Vec::new()),
            control_set: None,
            prev_trigger_res: Vec::new(),
            disconnected_scheme: ControlSchemeID::default(),
            cur_held_devs: Vec::new(),
            disconnected_devs: Vec::new()
        }
    }   

    /// Get the mapping contexts for this user.
    /// 
    /// The contexts are returned sorted by priority
    pub fn mappings(&self) -> &Mutex<Vec<(u16, MappingContext)>> {
        &self.mappings_contexts
    }

    /// Get the active control set
    pub fn control_set(&self) -> Option<&ControlSet> {
        self.control_set.as_ref()
    }

    pub(crate) fn notify_scheme_removed(&mut self, identifier: &ControlSchemeID) {
        if let Some(set) = &self.control_set && set.scheme_identifier() == identifier {
            self.control_set = None;
        }
    }

    pub(crate) fn notify_device_removed(&mut self, handle: DeviceHandle) -> bool {
        if let Some(control_set) = &self.control_set && control_set.devices().contains(&handle) {
            let mut control_set = take(&mut self.control_set).unwrap();
            self.cur_held_devs = control_set.take_devices();
            self.disconnected_devs.push(handle);
            true
        } else if !self.cur_held_devs.is_empty() && let Some(idx) = self.cur_held_devs.iter().position(|val| *val == handle) {
            self.cur_held_devs.remove(idx);
            self.disconnected_devs.push(handle);
            true
        } else {
            false
        }
    }

    pub(crate) fn add_mapping_context(&mut self, priority: u16, context: MappingContext) {
        let mut mappings = self.mappings_contexts.lock();
        match mappings.binary_search_by_key(&priority, |val| val.0) {
            Ok(mut idx) => {
                while mappings[idx].0 == priority {
                    idx += 1;
                }
                mappings.insert(idx, (priority, context))
            },
            Err(idx) => mappings.insert(idx, (priority, context)),
        }
    }

    pub(crate) fn remove_mapping_context(&mut self, identifier: &str) {
        self.mappings_contexts.lock().retain(|pair| pair.1.identifier == identifier);
    }

    pub(crate) fn remove_mapping_with_priority_below(&mut self, priority: u16) {
        let mut mappings = self.mappings_contexts.lock();
        match mappings.binary_search_by_key(&priority, |val| val.0) {
            Ok(mut idx) => {
                while mappings[idx].0 == priority {
                    idx += 1;
                }
                mappings.drain(..idx)
            },
            Err(idx) => mappings.drain(..idx),
        };
    }

    pub(crate) fn set_control_set(&mut self, control_set: ControlSet) {
        self.control_set = Some(control_set);
        self.disconnected_devs.clear();
    }

    pub(crate) fn process_input<F>(&mut self, dt: DeltaTime, user_idx: u8, get_input: F)
    where
        F : Fn(&User, &InputAxisId) -> AxisValue
    {
        let mut context = InputProcessContext::new();
        for mapping_ctx in &mut *self.mappings_contexts.lock() {
            for mapping in &mut mapping_ctx.1.mappings {
                mapping.process(dt, self, user_idx, &mut context, &get_input);
            }
        }
        self.prev_trigger_res = context.trigger_states;
    }

    pub(crate) fn get_currently_held_devices(&self) -> &Vec<DeviceHandle> {
        &self.cur_held_devs
    }

    pub(crate) fn try_reconnect_device(&mut self, handle: DeviceHandle) -> bool {
        let idx = match self.disconnected_devs.iter().position(|val| *val == handle) {
            Some(idx) => idx,
            None => return false,
        };
        self.disconnected_devs.remove(idx);

        self.cur_held_devs.push(handle);
        if self.disconnected_devs.is_empty() {
            self.control_set = Some(ControlSet{ scheme: take(&mut self.disconnected_scheme), devices: take(&mut self.cur_held_devs) });
        }
        true
    }

    pub(crate) fn get_previous_action_trigger_result(&self, action: &Arc<Mutex<Action>>) -> TriggerResult {
        match self.prev_trigger_res.iter().position(|val| Weak::ptr_eq(&val.0, &Arc::downgrade(action))) {
            Some(idx) => self.prev_trigger_res[idx].1,
            None => TriggerResult::Idle,
        }
    }

    pub(crate) fn rebind(&mut self, binding_name: &str, context_identifier: Option<&String>, input: InputAxisId) {
        for (_, mapping_ctx) in &mut *self.mappings_contexts.lock() {
            if let Some(ident) = context_identifier && mapping_ctx.identifier == *ident {
                mapping_ctx.rebind(binding_name, input.clone());
            }
        }
    }
}

