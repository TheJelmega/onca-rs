use std::fmt::Display;

use crate::{Numeric, Vec3, ApproxEq, Real};

/// 2D sphere
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Sphere<T: Numeric> {
    pub center : Vec3<T>,
    pub radius : T
}

impl<T: Numeric> Sphere<T> {
    /// Get the volume of the sphere
    #[inline]
    #[must_use]
    pub fn volume(self) -> T {
        T::from_i32(4) * self.radius * self.radius * self.radius * T::PI / T::from_i32(3)
    }

    /// Check if the sphere fully contains another sphere
    #[inline]
    #[must_use]
    pub fn contains(self, other: Self) -> bool {
        let furthest_point = self.center.dist(other.center) + other.radius;
        furthest_point <= self.radius
    }

    /// Check if the sphere contains a point
    #[inline]
    #[must_use]
    pub fn contains_point(self, point: Vec3<T>) -> bool {
        self.center.dist_sq(point) <= self.radius * self.radius
    }

    // TODO(jel): should we distiguish between overlap and touching?
    #[inline]
    #[must_use]
    /// Check if 2 spheres overlap
    pub fn overlaps(self, other: Self) -> bool {
        let max_dist = self.radius + other.radius;
        self.center.dist_sq(other.center) < max_dist * max_dist
    }

    /// Calculate the distance between the sphere and a point
    #[inline]
    #[must_use]
    pub fn dist_to_point(self, point: Vec3<T>) -> T {
        let dist = self.center.dist(point);
        if dist > self.radius { dist - self.radius } else { T::zero() }
    }

    /// Calculate the distance between the sphere and another sphere
    #[inline]
    #[must_use]
    pub fn dist_to_sphere(self, other: Self) -> T {
        let dist = self.center.dist_sq(other.center);
        let min_dist = self.radius + other.radius;

        if dist > min_dist { dist - min_dist } else { T::zero() }
    }
}

impl<T: Real> Sphere<T> {
    // TODO(jel): is there a way of doing this regardless of whether the number is an integer or real number
    /// Get the smallest sphere fitting both spheres
    #[inline]
    pub fn merge(self, other: Self) -> Self {
        let dist = self.center.dist(other.center);

        // early exit if 1 of the spheres fits into the other
        if dist + other.radius <= self.radius {
            return self;
        } else if dist + self.radius <= other.radius {
            return other;
        }

        let diam = dist + self.radius + other.radius;
        let radius = diam / T::from_i32(2);

        let theta = T::from_f32(0.5) + (other.radius - self.radius) / (dist * T::from_f32(2f32));
        let center = self.center.lerp(other.center, theta);

        Self { center, radius }
    }
}

impl<T: Numeric> ApproxEq for Sphere<T> {
    type Epsilon = T;

    fn is_close_to(self, rhs: Self, epsilon: Self::Epsilon) -> bool {
        self.center.is_close_to(rhs.center, epsilon) &&
        self.radius.is_close_to(rhs.radius, epsilon)
    }
}

impl<T: Real + Display> Display for Sphere<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{{ c: {}, r: {} }}", self.center, self.radius))
    }
}