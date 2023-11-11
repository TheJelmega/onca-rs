use std::{ops::*, fmt::Display};
use crate::*;

generic_normal!{ doc = "A 2D normal."; Normal2, Vec2, 2, (T, T), x => 0, y => 1; }

impl<T: Numeric + Display> Display for Normal2<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}