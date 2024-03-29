// TODO: Move raw input logging to log_debug when we have a better way of getting the logging context

#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(let_chains)]

mod os;

mod input_manager;
use core::ops::Add;
use std::{collections::HashSet, sync::{Arc, Weak}};

pub use input_manager::*;

mod input_devices;
pub use input_devices::*;

mod input_mapping;
pub use input_mapping::*;

mod control_scheme;
pub use control_scheme::*;

mod user;
use onca_common::sync::Mutex;
pub use user::*;

use onca_logging::LogCategory;
use onca_math::{f32v2, f32v3};

pub const LOG_INPUT_CAT : LogCategory = LogCategory::new("Input");
pub const LOG_EVENT_CAT : LogCategory = LogCategory::new_with_sub("Input", "Event processing");

/// Input axis type
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AxisType {
    /// Digital axis (on or off)
    Digital,
    /// Integral axis
    Int,
    /// Axis (range depends on input device)
    Axis,
    /// 2D axis (range depends on input device)
    Axis2D,
    /// 3D axis (range depends on input device)
    Axis3D,
}

/// Input axis value
#[derive(Clone, Copy, Debug)]
pub enum AxisValue {
    /// Digital axis (on or off).
    Digital(bool),
    /// Integral axis.
    Int(i32),
    /// Axis (range depends on input device).
    Axis(f32),
    /// 2D axis (range depends on input device).
    Axis2D(f32v2),
    /// 3D axis (range depends on input device).
    Axis3D(f32v3),
}

impl AxisValue {
    pub fn convert_to(&self, val_type: AxisType) -> AxisValue {
        match self {
            AxisValue::Digital(val) => match val_type {
                AxisType::Digital => AxisValue::Digital(*val),
                AxisType::Int     => AxisValue::Int(*val as i32),
                AxisType::Axis    => AxisValue::Axis(*val as i32 as f32),
                AxisType::Axis2D  => AxisValue::Axis2D(f32v2::new(*val as i32 as f32, 0.0)),
                AxisType::Axis3D  => AxisValue::Axis3D(f32v3::new(*val as i32 as f32, 0.0, 0.0)),
            },
            AxisValue::Int(val) => match val_type {
                AxisType::Digital => AxisValue::Digital(*val != 0),
                AxisType::Int     => AxisValue::Int(*val),
                AxisType::Axis    => AxisValue::Axis(*val as f32),
                AxisType::Axis2D  => AxisValue::Axis2D(f32v2::new(*val as f32, 0.0)),
                AxisType::Axis3D  => AxisValue::Axis3D(f32v3::new(*val as f32, 0.0, 0.0)),
            },
            AxisValue::Axis(val) => match val_type {
                AxisType::Digital => AxisValue::Digital(*val != 0.0),
                AxisType::Int     => AxisValue::Int(*val as i32),
                AxisType::Axis    => AxisValue::Axis(*val),
                AxisType::Axis2D  => AxisValue::Axis2D(f32v2::new(*val, 0.0)),
                AxisType::Axis3D  => AxisValue::Axis3D(f32v3::new(*val, 0.0, 0.0)),
            },
            AxisValue::Axis2D(val) => match val_type {
                AxisType::Digital => AxisValue::Digital(val.x != 0.0),
                AxisType::Int     => AxisValue::Int(val.x as i32),
                AxisType::Axis    => AxisValue::Axis(val.x),
                AxisType::Axis2D  => AxisValue::Axis2D(*val),
                AxisType::Axis3D  => AxisValue::Axis3D(val.extend(0.0)),
            },
            AxisValue::Axis3D(val) => match val_type {
                AxisType::Digital => AxisValue::Digital(val.x != 0.0),
                AxisType::Int     => AxisValue::Int(val.x as i32),
                AxisType::Axis    => AxisValue::Axis(val.x),
                AxisType::Axis2D  => AxisValue::Axis2D(val.xy()),
                AxisType::Axis3D  => AxisValue::Axis3D(*val),
            },
        }
    }

    pub fn axis_type(&self) -> AxisType {
        match self {
            AxisValue::Digital(_) => AxisType::Digital,
            AxisValue::Int(_)     => AxisType::Int,
            AxisValue::Axis(_)    => AxisType::Axis,
            AxisValue::Axis2D(_)  => AxisType::Axis2D,
            AxisValue::Axis3D(_)  => AxisType::Axis3D,
        }
    }
}

impl Add for AxisValue {
    type Output = AxisValue;

    fn add(self, rhs: Self) -> Self::Output {
        match self {
            AxisValue::Digital(lhs) => match rhs {
                AxisValue::Digital(rhs) => AxisValue::Digital(lhs | rhs),
                AxisValue::Int(rhs)     => AxisValue::Int(lhs as i32 + rhs),
                AxisValue::Axis(rhs)    => AxisValue::Axis(lhs as i32 as f32 + rhs),
                AxisValue::Axis2D(rhs)  => AxisValue::Axis2D(f32v2::new(lhs as i32 as f32 + rhs.x, rhs.y)),
                AxisValue::Axis3D(rhs)  => AxisValue::Axis3D(f32v3::new(lhs as i32 as f32 + rhs.x, rhs.y, rhs.z)),
            },
            AxisValue::Int(lhs) => match rhs {
                AxisValue::Digital(rhs) => AxisValue::Int(lhs + rhs as i32),
                AxisValue::Int(rhs)     => AxisValue::Int(lhs + rhs),
                AxisValue::Axis(rhs)    => AxisValue::Axis(lhs as f32 + rhs),
                AxisValue::Axis2D(rhs)  => AxisValue::Axis2D(f32v2::new(lhs as f32 + rhs.x, rhs.y)),
                AxisValue::Axis3D(rhs)  => AxisValue::Axis3D(f32v3::new(lhs as f32 + rhs.x, rhs.y, rhs.z)),
            },
            AxisValue::Axis(lhs) => match rhs {
                AxisValue::Digital(rhs) => AxisValue::Axis(lhs + rhs as i32 as f32),
                AxisValue::Int(rhs)     => AxisValue::Axis(lhs + rhs as f32),
                AxisValue::Axis(rhs)    => AxisValue::Axis(lhs + rhs),
                AxisValue::Axis2D(rhs)  => AxisValue::Axis2D(f32v2::new(lhs + rhs.x, rhs.y)),
                AxisValue::Axis3D(rhs)  => AxisValue::Axis3D(f32v3::new(lhs + rhs.x, rhs.y, rhs.z)),
            },
            AxisValue::Axis2D(lhs) =>  match rhs {
                AxisValue::Digital(rhs) => AxisValue::Axis2D(lhs + f32v2::new(rhs as i32 as f32, 0.0)),
                AxisValue::Int(rhs)     => AxisValue::Axis2D(lhs + f32v2::new(rhs as f32, 0.0)),
                AxisValue::Axis(rhs)    => AxisValue::Axis2D(lhs + f32v2::new(rhs, 0.0)),
                AxisValue::Axis2D(rhs)  => AxisValue::Axis2D(lhs + rhs),
                AxisValue::Axis3D(rhs)  => AxisValue::Axis3D(lhs.extend(0.0) + rhs),
            },
            AxisValue::Axis3D(lhs) =>  match rhs {
                AxisValue::Digital(rhs) => AxisValue::Axis3D(lhs + f32v3::new(rhs as i32 as f32, 0.0, 0.0)),
                AxisValue::Int(rhs)     => AxisValue::Axis3D(lhs + f32v3::new(rhs as f32, 0.0, 0.0)),
                AxisValue::Axis(rhs)    => AxisValue::Axis3D(lhs + f32v3::new(rhs, 0.0, 0.0)),
                AxisValue::Axis2D(rhs)  => AxisValue::Axis3D(lhs + rhs.extend(0.0)),
                AxisValue::Axis3D(rhs)  => AxisValue::Axis3D(lhs + rhs),
            },
        }
    }
}


/// Axis definition.
/// 
/// Represents the type of the axis and the range of possible values
#[derive(Clone, Copy, Debug)]
pub enum AxisDefinition {
    /// Digital axis (on or off).
    Digital,
    /// Integral axis.
    Int(i32, i32),
    /// Axis (range depends on input device).
    Axis(f32, f32),
    /// 2D axis (range depends on input device).
    Axis2D(f32v2, f32v2),
    /// 3D axis (range depends on input device).
    Axis3D(f32v3, f32v3),
}


impl AxisDefinition {
    pub fn axis_type(&self) -> AxisType {
        match self {
            AxisDefinition::Digital      => AxisType::Digital,
            AxisDefinition::Int(_, _)    => AxisType::Int,
            AxisDefinition::Axis(_, _)   => AxisType::Axis,
            AxisDefinition::Axis2D(_, _) => AxisType::Axis2D,
            AxisDefinition::Axis3D(_, _) => AxisType::Axis3D,
        }
    }
}

pub(crate) struct InputProcessContext {
    processed_actions : Vec<Arc<Mutex<Action>>>,
    triggered_actions : Vec<Arc<Mutex<Action>>>,
    consumed_inputs   : HashSet<AxisId>,
    trigger_states    : Vec<(Weak<Mutex<Action>>, TriggerResult)>,
}

impl InputProcessContext {
    pub fn new() -> Self {
        Self { processed_actions: Vec::new(), triggered_actions: Vec::new(), consumed_inputs: HashSet::new(), trigger_states: Vec::new() }
    }
}