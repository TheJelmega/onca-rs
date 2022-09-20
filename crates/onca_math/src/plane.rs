use crate::{Vec3, ApproxEq, Real};

/// Representation of a plane, represented by its normal and its distance from the origin
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Plane<T: Real> {
    pub normal : Vec3<T>,
    pub dist   : T
}

impl<T: Real> Plane<T> {
    /// Calculate the distance from a point to the plane, distance is signed, positive when above the plane, negative when below```
    pub fn distance(self, point: Vec3<T>) -> T {
        self.normal.dot(point) - self.dist
    }

    /// Check if a point is above the plane
    pub fn is_above(self, point: Vec3<T>) -> bool {
        self.normal.dot(point) > self.dist
    }
}

impl<T: Real> ApproxEq for Plane<T> {
    type Epsilon = T;

    fn is_close_to(self, rhs: Self, epsilon: Self::Epsilon) -> bool {
        self.normal.is_close_to(rhs.normal, epsilon) &&
        self.dist.is_close_to(rhs.dist, epsilon)
    }
}