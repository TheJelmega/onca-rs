use std::{ops::*, fmt::Display};
use crate::*;

generic_normal!{ doc = "A 3D normal."; Normal3, Vec3, 3, (T, T, T), x => 0, y => 1, z => 2; }

impl<T: Numeric + Display> Display for Normal3<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}