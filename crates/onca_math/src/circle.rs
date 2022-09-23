use std::fmt::Display;

use crate::{Numeric, Vec2, ApproxEq, Real};

/// 2D circle
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Circle<T: Numeric> {
    pub center : Vec2<T>,
    pub radius : T
}

impl<T: Numeric> Circle<T> {
    /// Get the area of the circle
    #[inline]
    #[must_use]
    pub fn area(self) -> T {
        self.radius * self.radius * T::PI
    }

    /// Check if the circle fully contains another circle
    #[inline]
    #[must_use]
    pub fn contains(self, other: Self) -> bool {
        let furthest_point = self.center.dist(other.center) + other.radius;
        furthest_point <= self.radius
    }

    /// Check if the circle contains a point
    #[inline]
    #[must_use]
    pub fn contains_point(self, point: Vec2<T>) -> bool {
        self.center.dist_sq(point) <= self.radius * self.radius
    }

    // TODO(jel): should we distiguish between overlap and touching?
    /// Check if 2 circles overlap
    #[inline]
    #[must_use]
    pub fn overlaps(self, other: Self) -> bool {
        let max_dist = self.radius + other.radius;
        self.center.dist_sq(other.center) < max_dist * max_dist
    }

    /// Calculate the distance between the circle and a point
    #[inline]
    #[must_use]
    pub fn dist_to_point(self, point: Vec2<T>) -> T {
        let dist = self.center.dist(point);
        if dist > self.radius { dist - self.radius } else { T::zero() }
    }

    /// Calculate the distance between the circle and another circle
    #[inline]
    #[must_use]
    pub fn dist_to_circle(self, other: Self) -> T {
        let dist = self.center.dist_sq(other.center);
        let min_dist = self.radius + other.radius;

        if dist > min_dist { dist - min_dist } else { T::zero() }
    }
}

impl<T: Real> Circle<T> {
    // TODO(jel): is there a way of doing this regardless of whether the number is an integer or real number
    /// Get the smallest circle fitting both circles
    #[must_use]
    pub fn merge(self, other: Self) -> Self {
        let dist = self.center.dist(other.center);

        // early exit if 1 of the circles fits into the other
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

impl<T: Numeric> ApproxEq for Circle<T> {
    type Epsilon = T;

    fn is_close_to(self, rhs: Self, epsilon: Self::Epsilon) -> bool {
        self.center.is_close_to(rhs.center, epsilon) &&
        self.radius.is_close_to(rhs.radius, epsilon)
    }
}

impl<T: Real + Display> Display for Circle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{{ c: {}, r: {} }}", self.center, self.radius))
    }
}