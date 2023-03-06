//! Currently just re-exports std::time, will eventually also include timers and delta-time
pub use std::time::*;

use core::fmt::Display;
use crate::os;

#[derive(Clone, Copy, Debug)]
pub struct TimeStamp {
    pub year        : u16,
    pub month       : u8,
    pub day_of_week : u8,
    pub day         : u8,
    pub hour        : u8,
    pub minute      : u8,
    pub second      : u8,
    pub millisecond : u16,
}

// TODO: customizable formatter
impl Display for TimeStamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}", self.year, self.month, self.day, self.hour, self.minute, self.second, self.millisecond))
    }
}

pub fn get_timestamp() -> TimeStamp {
    os::time::get_timestamp()
}

#[derive(Clone, Copy, Debug)]
pub struct DeltaTime {
    dt       : f32,
    dilation : f32,
}

impl DeltaTime {
    pub fn new(dt: f32) -> Self {
        Self { dt, dilation: 1f32 }
    }

    pub fn with_dilation(dt: f32, dilation: f32) -> Self {
        DeltaTime { dt, dilation }
    }

    pub fn get_dt(&self) -> f32 {
        self.dt
    }

    pub fn get_dilation(&self) -> f32 {
        self.dilation
    }

    pub fn get(&self, use_dilation: bool) -> f32 {
        if use_dilation {
            self.dt * self.dilation
        } else {
            self.dt
        }
    }
}