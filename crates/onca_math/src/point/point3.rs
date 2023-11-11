use std::{ops::*, fmt::Display};
use crate::*;


generic_point!{ doc = "A 3D point"; Point3, Vec3, 3, (T, T, T), x => 0, y => 1, z => 2;
    i8p3  => i8
    i16p3 => i16
    i32p3 => i32
    i64p3 => i64
    u8p3  => u8
    u16p3 => u16
    u32p3 => u32
    u64p3 => u64
    f32p3 => f32
    f64p3 => f64
}

impl<T: Numeric + Display> Display for Point3<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}