use core::{num::NonZeroU32, ops::BitOr};
use std::sync::{Weak, Arc};

use onca_common::{
    prelude::*,
    time::DeltaTime,
    sync::Mutex,
    event_listener::*,
};
use onca_common_macros::flags;
use onca_math::{f32v2, f32v3, SWIZZLE_X, SWIZZLE_Y, SWIZZLE_Z};

use crate::{AxisValue, AxisType, User, InputProcessContext, InputAxisId};

//------------------------------------------------------------------------------------------------------------------------------
// MODIFIERS
//------------------------------------------------------------------------------------------------------------------------------

/// Deadzone type
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DeadzoneType {
    /// Apply the deadzone to each axis individually, this results in the deadzone having sharp corners (square/cubic).
    Axial,
    /// Apply the deadzone to all axes simultaniously, this gives a smoother deadzone (circular/spherical). Works the same as `Axial` on 1D axes.
    Radial,
}

/// Input axis swizzle
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AxisSwizzle {
    XYZ,
    XZY,
    YXZ,
    YZX,
    ZXY,
    ZYX
}

impl AxisSwizzle {
    fn to_swizzle_constants(self) -> (u8, u8, u8) {
        match self {
            AxisSwizzle::XYZ => (SWIZZLE_X, SWIZZLE_Y, SWIZZLE_Z),
            AxisSwizzle::XZY => (SWIZZLE_X, SWIZZLE_Z, SWIZZLE_Y),
            AxisSwizzle::YXZ => (SWIZZLE_Y, SWIZZLE_X, SWIZZLE_Z),
            AxisSwizzle::YZX => (SWIZZLE_Y, SWIZZLE_Z, SWIZZLE_X),
            AxisSwizzle::ZXY => (SWIZZLE_Z, SWIZZLE_X, SWIZZLE_Y),
            AxisSwizzle::ZYX => (SWIZZLE_Z, SWIZZLE_Y, SWIZZLE_X),
        }
    }
}

/// Custom modifier
pub trait CustomModifier {
    /// Apply the modifier to the given value
    fn apply(&mut self, value: AxisValue) -> AxisValue;

    /// Clone the modifier
    fn clone_modifier(&self) -> Box<dyn CustomModifier>;
}

/// Input modifier
pub enum Modifier {
    /// Apply a deadzone to the value, with a given lower and upper bound
    Deadzone{ lower_bound: f32, upper_bound: f32, deadzone_type: DeadzoneType },
    /// Negate the given components
    Negate(bool, bool, bool),
    /// Scale the value
    Scale(f32, f32, f32),
    /// Scale by time
    TimeScale(bool),
    /// Swizzle the axes
    Swizzle(AxisSwizzle),
    /// Custom modifier
    Custom(Box<dyn CustomModifier>)
}

impl Modifier {
    fn apply(&mut self, value: AxisValue, dt: DeltaTime) -> AxisValue {
        match self {
            Modifier::Deadzone { lower_bound, upper_bound, deadzone_type } => Self::apply_deadzone(value, *lower_bound, *upper_bound, *deadzone_type),
            Modifier::Negate(x, y, z) => Self::apply_negate(value, *x, *y, *z),
            Modifier::Scale(x, y, z) => Self::apply_scale(value, *x, *y, *z),
            Modifier::TimeScale(use_dilation) => Self::apply_time_scale(value, dt, *use_dilation),
            Modifier::Custom(custom) => custom.apply(value),
            Modifier::Swizzle(swizzle) => Self::apply_swizzle(value, *swizzle),
        }
    }

    fn apply_deadzone(value: AxisValue, lower_bound: f32, upper_bound: f32, deadzone_type: DeadzoneType) -> AxisValue {
        let deadzone_1d = |val: f32| {
            if val < lower_bound {
                0f32
            } else {
                val.min(upper_bound)
            }
        };

        match value {
            AxisValue::Digital(val) => AxisValue::Digital(val),
            AxisValue::Axis(val) => AxisValue::Axis(deadzone_1d(val)),
            AxisValue::Axis2D(val) => {
                match deadzone_type {
                    DeadzoneType::Axial => AxisValue::Axis2D(f32v2::new(deadzone_1d(val.x), deadzone_1d(val.y))),
                    DeadzoneType::Radial => {
                        let len = val.len();
                        let deadzone_len = deadzone_1d(val.len());
                        AxisValue::Axis2D(val / len * deadzone_len)
                    },
                }
            },
            AxisValue::Axis3D(val) => {
                match deadzone_type {
                    DeadzoneType::Axial => AxisValue::Axis3D(f32v3::new(deadzone_1d(val.x), deadzone_1d(val.y), deadzone_1d(val.z))),
                    DeadzoneType::Radial => {
                        let len = val.len();
                        let deadzone_len = deadzone_1d(val.len());
                        AxisValue::Axis3D(val / len * deadzone_len)
                    },
                }
            },
        }
    }

    fn apply_negate(value: AxisValue, x: bool, y: bool, z: bool) -> AxisValue {
        match value {
            AxisValue::Digital(val) => AxisValue::Digital(if x { !val } else {val}),
            AxisValue::Axis(val)    => AxisValue::Axis(if x { -val } else { val }),
            AxisValue::Axis2D(val)  => AxisValue::Axis2D(f32v2::new(if x { -val.x } else { val.x }, if y { -val.y } else { val.y })),
            AxisValue::Axis3D(val)  => AxisValue::Axis3D(f32v3::new(if x { -val.x } else { val.x }, if y { -val.y } else { val.y }, if z { -val.z } else { val.z })),
        }
    }

    fn apply_scale(value: AxisValue, x: f32, y: f32, z: f32) -> AxisValue {
        match value {
            AxisValue::Digital(val) => AxisValue::Digital(val),
            AxisValue::Axis(val)    => AxisValue::Axis(val * x),
            AxisValue::Axis2D(val)  => AxisValue::Axis2D(val * f32v2{ x, y }),
            AxisValue::Axis3D(val)  => AxisValue::Axis3D(val * f32v3{ x, y, z }),
        }
    }

    fn apply_time_scale(value: AxisValue, dt: DeltaTime, dilation: bool) -> AxisValue {
        match value {
            AxisValue::Digital(val) => AxisValue::Digital(val),
            AxisValue::Axis(val)    => AxisValue::Axis(val * dt.get(dilation)),
            AxisValue::Axis2D(val)  => AxisValue::Axis2D(val * dt.get(dilation)),
            AxisValue::Axis3D(val)  => AxisValue::Axis3D(val * dt.get(dilation)),
        }
    }

    fn apply_swizzle(value: AxisValue, swizzle: AxisSwizzle) -> AxisValue {
        let (x, y, z) = swizzle.to_swizzle_constants();
        match value {
            AxisValue::Digital(val) => AxisValue::Digital(val),
            AxisValue::Axis(val) => AxisValue::Axis(if x == SWIZZLE_X { val } else { 0f32 }),
            AxisValue::Axis2D(val) => {
                AxisValue::Axis2D(f32v2::new(
                    if x == SWIZZLE_Z { 0f32 } else { val[x as usize] },
                    if y == SWIZZLE_Z { 0f32 } else { val[y as usize] }
                ))
            },
            AxisValue::Axis3D(val) => AxisValue::Axis3D(val.swizzle(x, y, z)),
        }
    }
}

impl Clone for Modifier {
    fn clone(&self) -> Self {
        match self {
            Self::Deadzone { lower_bound, upper_bound, deadzone_type } => Self::Deadzone { lower_bound: lower_bound.clone(), upper_bound: upper_bound.clone(), deadzone_type: deadzone_type.clone() },
            Self::Negate(arg0, arg1, arg2) => Self::Negate(arg0.clone(), arg1.clone(), arg2.clone()),
            Self::Scale(arg0, arg1, arg2) => Self::Scale(arg0.clone(), arg1.clone(), arg2.clone()),
            Self::TimeScale(arg0) => Self::TimeScale(arg0.clone()),
            Self::Swizzle(arg0) => Self::Swizzle(arg0.clone()),
            Self::Custom(arg0) => Self::Custom(arg0.clone_modifier()),
        }
    }
}

//------------------------------------------------------------------------------------------------------------------------------
// TRIGGERS
//------------------------------------------------------------------------------------------------------------------------------

/// Custom trigger
pub trait CustomTrigger {
    /// Check the trigger based on the current value
    fn check(&mut self, value: AxisValue, ctx: &mut TriggerContext) -> TriggerResult;

    /// Get the trigger type
    fn trigger_type(&self) -> TriggerType;

    /// Clone the trigger
    fn clone_trigger(&self) -> Box<dyn CustomTrigger>;
}

/// Trigger type
pub enum Trigger {
    /// Trigger when the value passes the given threshold.
    Down(f32),
    /// Trigger when the value passes the given threshold for the first time.
    Pressed(f32),
    /// Trigger when the value was over the given threshold and moves below it.
    Released(f32),
    /// Triggers when the value stays over a given value for a given period of time, can trigger once or continuously after that.
    Hold {
        /// Period of time the input needs to be actuated to trigger.
        hold_time     : f32,
        /// Whether this trigger should only fire once after it is triggered.
        one_shot      : bool,
        /// Whether the trigger time is affected by time dialation.
        time_dilation : bool,
        /// Actuation treshold.
        threshold     : f32
    },
    /// Triggers when the value stays over a given value for a given period of time, and it then released.
    HoldAndRelease {
        /// Period of time the input needs to be actuated to triggered on release.
        hold_time     : f32,
        /// Whether the hold trigger is affected by time dialation.
        time_dilation : bool,
        /// Actuation treshold.
        threshold     : f32
    },
    /// Pulse the trigger when the given values stays over a given value.
    Pulse {
        /// Should the trigger pulse on the initial actuation.
        trigger_on_start : bool,
        /// Time between pulses.
        interval         : f32,
        /// Maximum number of pulses, `None` is no limit.
        trigger_limit    : Option<NonZeroU32>,
        /// Whether the hold trigger is affected by time dialation.
        time_dilation    : bool,
        /// Actuation treshold.
        threshold        : f32
    },
    /// Triggers if the value passes the given value and moves below within a given limit
    Tap {
        /// Maximum amount of time the values can be above the given actuation threshold
        release_time_threshold : f32,
        /// Whether the hold trigger is affected by time dialation.
        time_dilation          : bool,
        /// Actuation treshold.
        threshold              : f32
    },
    /// Chorded trigger (other action needs to be triggered)
    Chord(Weak<Mutex<Action>>),
    /// Custom trigger
    Custom(Box<dyn CustomTrigger>)
}

impl Clone for Trigger {
    fn clone(&self) -> Self {
        match self {
            Self::Down(arg0) => Self::Down(arg0.clone()),
            Self::Pressed(arg0) => Self::Pressed(arg0.clone()),
            Self::Released(arg0) => Self::Released(arg0.clone()),
            Self::Hold { hold_time, one_shot, time_dilation, threshold } => Self::Hold { hold_time: hold_time.clone(), one_shot: one_shot.clone(), time_dilation: time_dilation.clone(), threshold: threshold.clone() },
            Self::HoldAndRelease { hold_time, time_dilation, threshold } => Self::HoldAndRelease { hold_time: hold_time.clone(), time_dilation: time_dilation.clone(), threshold: threshold.clone() },
            Self::Pulse { trigger_on_start, interval, trigger_limit, time_dilation, threshold } => Self::Pulse { trigger_on_start: trigger_on_start.clone(), interval: interval.clone(), trigger_limit: trigger_limit.clone(), time_dilation: time_dilation.clone(), threshold: threshold.clone() },
            Self::Tap { release_time_threshold, time_dilation, threshold } => Self::Tap { release_time_threshold: release_time_threshold.clone(), time_dilation: time_dilation.clone(), threshold: threshold.clone() },
            Self::Chord(action) => Self::Chord(action.clone()),
            Self::Custom(arg0) => Self::Custom(arg0.clone_trigger()),
        }
    }
}

/// Current trigger state
/// 
/// The trigger state is calculated from the last and current trigger result
/// 
/// prev result | new result | state
/// ------------|------------|------- 
/// idle        | idle       | idle
/// idle        | ongoing    | started
/// idle        | triggered  | started + triggered
/// ongoing     | idle       | cancelled
/// ongoing     | ongoing    | ongoing
/// ongoing     | triggered  | triggered
/// triggered   | idle       | completed
/// triggered   | ongoing    | ongoing
/// triggered   | triggered  | triggered
/// 
/// Only Started and Triggered can ever be set at the same times, other combos are invalid
#[flags]
pub enum TriggerState {
    /// The trigger is currently idle
    Idle = 0,
    /// The trigger has started processing, e.g. key is down and hold trigger is starting
    Started,
    /// The trigger is currently ongoing, e.g. hold trigger is waiting for enough time having passed
    Ongoing,
    /// The trigger is cancelled, e.g. key is up and hold trigger has not triggered yet
    Cancelled,
    /// The trigger has been triggered
    Triggered,
    /// The trigger has completed, e.g. hold trigger was triggered and has ended
    Completed,
}

impl TriggerState {
    pub fn from_results(prev: TriggerResult, current: TriggerResult) -> TriggerState {
        match prev {
            TriggerResult::Idle => match current {
                TriggerResult::Idle => TriggerState::Idle,
                TriggerResult::Ongoing => TriggerState::Started,
                TriggerResult::Triggered => TriggerState::Started | TriggerState::Triggered,
            },
            TriggerResult::Ongoing => match current {
                TriggerResult::Idle => TriggerState::Cancelled,
                TriggerResult::Ongoing => TriggerState::Ongoing,
                TriggerResult::Triggered => TriggerState::Triggered,
            },
            TriggerResult::Triggered => match current {
                TriggerResult::Idle => TriggerState::Completed,
                TriggerResult::Ongoing => TriggerState::Ongoing,
                TriggerResult::Triggered => TriggerState::Triggered,
            },
        }
    }
}

/// Trigger result
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TriggerResult {
    /// The trigger is idle
    Idle,
    /// The trigger is being processed
    /// 
    /// This result is not useful for `Implicit` and `Blocker` triggers
    Ongoing,
    /// The trigger has triggered
    Triggered,
}

impl BitOr for TriggerResult {
    type Output = TriggerResult;

    fn bitor(self, rhs: Self) -> Self::Output {
        match self {
            TriggerResult::Idle => match rhs {
                TriggerResult::Idle => TriggerResult::Idle,
                TriggerResult::Ongoing => TriggerResult::Ongoing,
                TriggerResult::Triggered => TriggerResult::Triggered,
            },
            TriggerResult::Ongoing => match rhs {
                TriggerResult::Idle => TriggerResult::Ongoing,
                TriggerResult::Ongoing => TriggerResult::Ongoing,
                TriggerResult::Triggered => TriggerResult::Triggered,
            },
            TriggerResult::Triggered => TriggerResult::Triggered,
        }
    }
}

pub(crate) struct FinalTriggerResult {
    result       : TriggerResult,
    all_implicit : bool,
    blocked      : bool,
}

impl FinalTriggerResult {
    pub fn new() -> Self {
        Self { result: TriggerResult::Idle, all_implicit: true, blocked: false }
    }

    pub fn update(&mut self, ty: TriggerType, res: TriggerResult) {
        match ty {
            TriggerType::Explicit => self.result = self.result | res,
            TriggerType::Implicit => self.all_implicit &= res == TriggerResult::Triggered,
            TriggerType::Blocker => self.blocked |= res == TriggerResult::Triggered,
        }
    }

    pub fn to_result(self) -> TriggerResult {
        if self.blocked || !self.all_implicit {
            TriggerResult::Idle
        } else {
            self.result
        }
    }
}

/// Default trigger context
#[derive(Clone)]
pub struct TriggerContext {
    prev_value  : AxisValue,
    prev_result : TriggerResult,
    timer       : f32,
    misc        : u32,
}

impl Default for TriggerContext {
    fn default() -> Self {
        Self {
            prev_value: AxisValue::Digital(false),
            prev_result: TriggerResult::Idle,
            timer: 0f32,
            misc: 0
        }
    }
}

/// Trigger kind
pub enum TriggerType {
    /// Any explicit trigger needs to be triggered for the action to happen
    Explicit,
    /// All implicit triggers need to be triggered for the action to happen
    Implicit,
    /// If triggered, the action cannot happen
    Blocker,
}

/// Trigger
#[derive(Clone)]
pub struct TriggerData {
    trigger : Trigger,
    context      : TriggerContext,
}

impl TriggerData {
    /// Checks if the trigger has been triggered
    fn check(&mut self, value: AxisValue, dt: DeltaTime, context: &mut InputProcessContext) -> TriggerResult {
        let res = match &mut self.trigger {
            Trigger::Down(threshold) => Self::check_down(value, *threshold),
            Trigger::Pressed(threshold) => Self::check_pressed(self.context.prev_value, value, *threshold),
            Trigger::Released(threshold) => Self::check_released(self.context.prev_value, value, *threshold),
            Trigger::Hold { hold_time, one_shot, time_dilation: time_dialation, threshold } => Self::check_hold(&mut self.context, value, dt, *hold_time, *one_shot, *time_dialation, *threshold),
            Trigger::HoldAndRelease { hold_time, time_dilation: time_dialation, threshold } => Self::check_hold_and_release(&mut self.context, value, dt, *hold_time, *time_dialation, *threshold),
            Trigger::Pulse { trigger_on_start, interval, trigger_limit, time_dilation, threshold } => 
                Self::check_pulse(&mut self.context, value, dt, *trigger_on_start, *interval, *trigger_limit, *time_dilation, *threshold),
            Trigger::Tap { release_time_threshold, time_dilation, threshold } => Self::check_tap(&mut self.context, value, dt, *release_time_threshold, *time_dilation, *threshold),
            Trigger::Chord(chorded_action) => if context.triggered_actions.iter().find(|action| Weak::ptr_eq(&Arc::downgrade(action), chorded_action)).is_some() { TriggerResult::Triggered } else { TriggerResult::Idle },
            Trigger::Custom(custom) => custom.check(value, &mut self.context),
        };
        self.context.prev_value = value;
        self.context.prev_result = res;
        res
    }

    fn trigger_type(&self) -> TriggerType {
        match &self.trigger {
            Trigger::Down(_)               => TriggerType::Explicit,
            Trigger::Pressed(_)            => TriggerType::Explicit,
            Trigger::Released(_)           => TriggerType::Explicit,
            Trigger::Hold { .. }           => TriggerType::Explicit,
            Trigger::HoldAndRelease { .. } => TriggerType::Explicit,
            Trigger::Pulse { .. }          => TriggerType::Explicit,
            Trigger::Tap { .. }            => TriggerType::Explicit,
            Trigger::Chord(_)              => TriggerType::Implicit,
            Trigger::Custom(custom)        => custom.trigger_type(),
        }
    }

    fn check_down(value: AxisValue, threshold: f32) -> TriggerResult {
        match value {
            AxisValue::Digital(val) => if val { TriggerResult::Triggered } else { TriggerResult::Idle },
            AxisValue::Axis(val)    => if val > threshold { TriggerResult::Triggered } else { TriggerResult::Idle },
            AxisValue::Axis2D(val)  => if val.x > threshold || val.y > threshold { TriggerResult::Triggered } else { TriggerResult::Idle },
            AxisValue::Axis3D(val)  => if val.x > threshold || val.y > threshold || val.z > threshold { TriggerResult::Triggered } else { TriggerResult::Idle },
        }
    }

    fn is_down(value: AxisValue, threshold: f32) -> bool {
        Self::check_down(value, threshold) == TriggerResult::Triggered
    }

    fn check_pressed(prev_value: AxisValue, value: AxisValue, threshold: f32) -> TriggerResult {
        if !Self::is_down(prev_value, threshold) && Self::is_down(value, threshold) {
            TriggerResult::Triggered
        } else {
            TriggerResult::Idle
        }
    }

    fn check_released(prev_value: AxisValue, value: AxisValue, threshold: f32) -> TriggerResult {
        if Self::is_down(prev_value, threshold) && !Self::is_down(value, threshold) {
            TriggerResult::Triggered
        } else {
            TriggerResult::Idle
        }
    }

    fn check_hold(ctx: &mut TriggerContext, value: AxisValue, dt: DeltaTime, hold_time: f32, one_shot: bool, time_dilation: bool, threshold: f32) -> TriggerResult {
        if Self::is_down(value, threshold) {
            if ctx.timer < 0f32 {
                if one_shot {
                    TriggerResult::Ongoing
                } else {
                    TriggerResult::Triggered
                }
            } else {
                ctx.timer += dt.get(time_dilation);
                if ctx.timer > hold_time {
                    ctx.timer = -1f32;
                    TriggerResult::Triggered
                } else {
                    TriggerResult::Ongoing
                }
            }
        } else {
            ctx.timer = 0f32;
            TriggerResult::Idle
        }
    }

    fn check_hold_and_release(ctx: &mut TriggerContext, value: AxisValue, dt: DeltaTime, hold_time: f32, time_dilation: bool, threshold: f32) -> TriggerResult {
        if Self::is_down(value, threshold) {
            ctx.timer += dt.get(time_dilation);
            TriggerResult::Ongoing
        } else {
            let res = if ctx.timer > hold_time {
                TriggerResult::Triggered
            } else {
                TriggerResult::Idle
            };
            ctx.timer = 0f32;
            res
        }
    }

    fn check_pulse(ctx: &mut TriggerContext, value: AxisValue, dt: DeltaTime, trigger_on_start: bool, interval: f32, trigger_limit: Option<NonZeroU32>, time_dilation: bool, threshold: f32) -> TriggerResult {
        if Self::is_down(value, threshold) {
            if let Some(limit) = trigger_limit && ctx.misc >= limit.get() {
                return TriggerResult::Ongoing;
            }
       
            ctx.timer += dt.get(time_dilation);
            if ctx.timer > interval {
                ctx.timer -= interval;
                ctx.misc += 1;
                return TriggerResult::Triggered;
            }

            if ctx.prev_result == TriggerResult::Idle && trigger_on_start {
                ctx.misc += 1;
                TriggerResult::Triggered
            } else {
                TriggerResult::Ongoing
            }
        } else {
            ctx.timer = 0f32;
            ctx.misc = 0;
            TriggerResult::Idle
        }
    }

    fn check_tap(ctx: &mut TriggerContext, value: AxisValue, dt: DeltaTime, release_time_threshold: f32, time_dilation: bool, threshold: f32) -> TriggerResult {
        if Self::is_down(value, threshold) {
            ctx.timer += dt.get(time_dilation);
            ctx.misc = 1;
            TriggerResult::Ongoing
        } else {
            let res = if ctx.misc == 1 && ctx.timer <= release_time_threshold {
                TriggerResult::Triggered
            } else {
                TriggerResult::Idle
            };
            ctx.timer = 0f32;
            ctx.misc = 0;
            res
        }
    }
}

impl From<Trigger> for TriggerData {
    fn from(trigger: Trigger) -> Self {
        Self { trigger, context: TriggerContext::default() }
    }
}

//------------------------------------------------------------------------------------------------------------------------------
// MAPPING
//------------------------------------------------------------------------------------------------------------------------------

/// Input action
/// 
/// If the action is present twice for a user, only the first occurance of the action will be processed
pub struct Action {
    /// Localized description of this action
    // TODO: Localized string
    pub description         : (),
    /// Should the input action consumes the input? (input will not be processed by any triggers that occur after this action)
    pub consume_input       : bool,
    /// Should the input action trigger when the game is paused?
    pub trigger_when_paused : bool,
    /// Axis type that will be used by this action
    pub axis_type           : AxisType,
    /// Triggers that apply to all mappings for this action.
    /// 
    /// These triggers are used to instantiate the mapping specific triggers and are not use themselves
    // TODO: Have these associated to the action only when in its asset?
    pub triggers            : Vec<TriggerData>,
    /// Modifiers that apply to all mappings for this action. (Modifiers will be applied after the mapping specific modifiers have been applied)
    /// 
    /// These modifiers are used to instantiate the mapping specific triggers and are not use themselves
    // TODO: Have these associated to the action only when in its asset?
    pub modifiers           : Vec<Modifier>,
    /// Action event listeners
    pub listeners           : DynEventListenerArray<(TriggerState, AxisValue, u8)>,
}

impl Action {
    pub fn new(consume_input: bool, trigger_when_paused: bool, axis_type: AxisType) -> Self {
        Self {
            description: (),
            consume_input,
            trigger_when_paused,
            axis_type,
            triggers: Vec::new(),
            modifiers: Vec::new(),
            listeners: DynEventListenerArray::new()
        }
    }

    pub fn add_trigger(&mut self, trigger: Trigger) {
        self.triggers.push(trigger.into());
    }

    pub fn add_modifier(&mut self, modifier: Modifier) {
        self.modifiers.push(modifier);
    }

    pub fn add_listener(&mut self, listener: DynEventListenerRef<(TriggerState, AxisValue, u8)>) {
        self.listeners.push(listener);
    }

    pub fn dispatch(&mut self, trigger_res: TriggerState, value: AxisValue, user_idx: u8) {
        self.listeners.notify(&(trigger_res, value.convert_to(self.axis_type), user_idx));
    }
}

/// Options to rebind a binding.
#[derive(Clone)]
pub struct RebindOptions {
    /// Localized description of this action
    // TODO: Localized string
    pub display_name     : (),
    /// Localized category of this action
    // TODO: Localized string
    pub display_category : (),
    /// Name used to remap
    pub name             : String,
}

#[derive(Clone)]
pub struct Binding {
    /// Input axis that the binding is bound to.
    pub input_axis     : InputAxisId,
    /// Binding specific triggers
    pub triggers       : Vec<TriggerData>,
    /// Binding specific modifiers
    pub modifiers      : Vec<Modifier>,
    /// Binding rebind options
    pub rebind_options : Option<RebindOptions>
}

impl Binding {
    pub fn new(input_axis: InputAxisId) -> Self {
        Self {
            input_axis,
            triggers: Vec::new(),
            modifiers: Vec::new(),
            rebind_options: None
        }
    }

    pub fn add_trigger(&mut self, trigger: Trigger) {
        self.triggers.push(trigger.into());
    }

    pub fn add_modifier(&mut self, modifier: Modifier) {
        self.modifiers.push(modifier);
    }

    pub(crate) fn apply_modifiers(&mut self, value: &mut AxisValue, dt: DeltaTime) {
        for modifier in &mut self.modifiers {
            *value = modifier.apply(*value, dt);
        }
    }

    pub(crate) fn process_triggers(&mut self, value: AxisValue, dt: DeltaTime, context: &mut InputProcessContext, final_res: &mut FinalTriggerResult) {
        for trigger in &mut self.triggers {
            let trigger_type = trigger.trigger_type();
            let res = trigger.check(value, dt, context);
            final_res.update(trigger_type, res);
        }
    }
}

#[derive(Clone)]
pub struct Mapping {
    /// Input action associated with this binding
    pub action    : Arc<Mutex<Action>>,
    /// Instance of the action triggers that will actually be used
    pub triggers  : Vec<TriggerData>,
    /// Instance of the action triggers that will actually be used
    pub modifiers : Vec<Modifier>,
    /// Input 
    pub bindings  : Vec<Binding>,
}

impl Mapping {
    pub fn new(action: Arc<Mutex<Action>>) -> Self {

        let (triggers, modifiers) = {
            let action = action.lock();
            (action.triggers.clone(), action.modifiers.clone())
        };

        Self { action, triggers, modifiers, bindings: Vec::new() }
    }

    pub fn set_action(&mut self, action: Arc<Mutex<Action>>) {
        self.action = action;

        let action = self.action.lock();
        self.triggers = action.triggers.clone();
        self.modifiers = action.modifiers.clone();
    }

    pub fn add_binding(&mut self, binding: Binding) {
        self.bindings.push(binding);
    }

    pub(crate) fn apply_modifiers(&mut self, value: &mut AxisValue, dt: DeltaTime) {
        for modifier in &mut self.modifiers {
            *value = modifier.apply(*value, dt);
        }
    }

    pub(crate) fn process_triggers(&mut self, value: AxisValue, dt: DeltaTime, context: &mut InputProcessContext, final_res: &mut FinalTriggerResult) {
        for trigger in &mut self.triggers {
            let trigger_type = trigger.trigger_type();
            let res = trigger.check(value, dt, context);
            final_res.update(trigger_type, res);
        }
    }

    pub(crate) fn process<F>(&mut self, dt: DeltaTime, user: &User, user_idx: u8, context: &mut InputProcessContext, get_input: &F) 
    where
        F : Fn(&User, &InputAxisId) -> AxisValue
    {
        // Only the first occurance of an action is processed, so check that first
        if context.processed_actions.iter().any(|action| Arc::ptr_eq(action, &self.action)) {
            return;
        }

        let mut value = AxisValue::Digital(false);
        let mut trigger_res = FinalTriggerResult::new();
        let consume_inputs = self.action.lock().consume_input;

        for binding in &mut self.bindings {
            // Skip bindings with consumed inputs
            if context.consumed_inputs.contains(&binding.input_axis) {
                continue;
            }

            let mut binding_value = get_input(user, &binding.input_axis);
            binding.apply_modifiers(&mut binding_value, dt);
            binding.process_triggers(binding_value, dt, context, &mut trigger_res);

            value = value + binding_value;

            if consume_inputs {
                context.consumed_inputs.insert(binding.input_axis.clone());
            }
        }

        self.apply_modifiers(&mut value, dt);
        self.process_triggers(value, dt, context, &mut trigger_res);

        let result = trigger_res.to_result();
        let prev_res = user.get_previous_action_trigger_result(&self.action);
        context.trigger_states.push((Arc::downgrade(&self.action), result));
        
        let trigger_state = TriggerState::from_results(prev_res, result);
        // Only notify if the state is not idle
        if trigger_state != TriggerState::Idle {
            self.action.lock().dispatch(trigger_state, value, user_idx);
        }
        context.processed_actions.push(self.action.clone());

        if result == TriggerResult::Triggered {
            context.triggered_actions.push(self.action.clone());
        }   
    }

    pub(crate) fn rebind(&mut self, binding_name: &str, input: InputAxisId) {
        for binding in &mut self.bindings {
            if let Some(rebind_options) = &binding.rebind_options && rebind_options.name == binding_name {
                binding.input_axis = input.clone();
            }
        }
    }
}

#[derive(Clone)]
pub struct MappingContext {
    /// Localized description of this mapping context
    // TODO: Localized string
    pub description : (),
    /// Identifier
    // TODO: Id should be something associated to the asset
    pub identifier  : String,
    /// Input mappings
    pub mappings    : Vec<Mapping>,
}

impl MappingContext {
    pub fn new(identifier: String) -> Self {
        Self { description: (), identifier, mappings: Vec::new() }
    }

    pub fn add_mapping(&mut self, mapping: Mapping) {
        self.mappings.push(mapping);
    }

    pub(crate) fn rebind(&mut self, binding_name: &str, input: InputAxisId) {
        for mapping in &mut self.mappings {
            mapping.rebind(binding_name, input.clone());
        }
    }
}