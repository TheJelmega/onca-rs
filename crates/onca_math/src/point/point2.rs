use std::{ops::*, fmt::Display};
use crate::*;


generic_point!{ doc = "A 2D point"; Point2, Vec2, 2, (T, T), x => 0, y => 1;
    i8p2  => i8
    i16p2 => i16
    i32p2 => i32
    i64p2 => i64
    u8p2  => u8
    u16p2 => u16
    u32p2 => u32
    u64p2 => u64
    f32p2 => f32
    f64p2 => f64
}

impl<T: Numeric + Display> Display for Point2<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}