use std::fmt::Display;

use crate::{Real, ApproxEq, Vec3, Vec2, Line, Line2D};

/// 3D Ray
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Ray<T: Real> {
    pub orig  : Vec3<T>,
    pub min_t : T,
    pub dir   : Vec3<T>,
    pub max_t : T 
}

impl<T: Real> Ray<T> {
    /// Create a new ray
    #[inline]
    #[must_use]
    pub fn new(orig: Vec3<T>, dir: Vec3<T>, min: T, max: T) -> Self {
        Self { orig, min_t: min, dir, max_t: max }
    }

    /// Create a new ray from just an origin an a direction, implicitly sets `min_t == 0` and `max_t == T::MAX`
    #[inline]
    #[must_use]
    pub fn from_orig_and_dir(orig: Vec3<T>, dir: Vec3<T>) -> Self {
        Self { orig, min_t: T::zero(), dir: dir, max_t: T::MAX }
    }

    /// Create a ray from a line
    #[inline]
    #[must_use]
    pub fn from_line(line: Line<T>) -> Self {
        Self { orig: line.orig, min_t: T::zero(), dir: line.dir, max_t: T::MAX }
    }

    /// Create a ray from a line
    pub fn from_line_between(line: Line<T>, min: T, max: T) -> Self {
        Self { orig: line.orig, min_t: min, dir: line.dir, max_t: max }
    }

    /// Check if a point at a given distance is on the ray
    #[inline]
    #[must_use]
    pub fn is_on_ray(self, dist: T) -> bool {
        dist >= self.min_t && dist <= self.max_t
    }

    /// Get the point at a given distance on the ray
    #[inline]
    #[must_use]
    pub fn point_at(self, dist: T) -> Vec3<T> {
        self.orig + self.dir * dist
    }

    /// Get the closts point to the point at the value that is still on the ray
    #[inline]
    #[must_use]
    pub fn closest_point(self, dist: T) -> Vec3<T> {
        self.orig + self.dir * dist.clamp(self.min_t, self.max_t)
    }

    /// Clamp the given ray param so it fits on the ray
    #[inline]
    #[must_use]
    pub fn clamp_dist(self, dist: T) -> T {
        dist.clamp(self.min_t, self.max_t)
    }

    /// Calculate the distance of the point on the ray
    #[inline]
    #[must_use]
    pub fn dist(self, point: Vec3<T>) -> T {
        let to_point = point - self.orig;
        self.dir.dot(to_point)
    }

    /// Calculate the distance of the point on the ray
    #[must_use]
    pub fn closest_dist(self, point: Vec3<T>) -> T {
        let to_point = point - self.orig;
        let min = self.min_t * self.min_t;
        let max = self.max_t * self.max_t; 
        self.dir.dot(to_point).clamp(min, max)
    }
}

impl<T: Real> ApproxEq<T> for Ray<T> {
    const EPSILON: T = T::EPSILON;

    fn is_close_to(self, rhs: Self, epsilon: T) -> bool {
        self.orig.is_close_to(rhs.orig, epsilon) &&
        self.min_t.is_close_to(rhs.min_t, epsilon) &&
        self.dir.is_close_to(rhs.dir, epsilon) &&
        self.max_t.is_close_to(rhs.max_t, epsilon)
    }
}

impl<T: Real> From<Line<T>> for Ray<T> {
    fn from(line: Line<T>) -> Self {
        Self::from_line(line)
    }
}

impl<T: Real + Display> Display for Ray<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{{ o: {}, d: {}, t: [{}, {}] }}", self.orig, self.dir, self.min_t, self.max_t))
    }
}

//------------------------------------------------------------------------------------------------------------------------------

/// 3D Ray
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Ray2D<T: Real> {
    pub orig  : Vec2<T>,
    pub min_t : T,
    pub dir   : Vec2<T>,
    pub max_t : T 
}

impl<T: Real> Ray2D<T> {
    /// Create a new ray
    #[inline]
    #[must_use]
    pub fn new(orig: Vec2<T>, dir: Vec2<T>, min: T, max: T) -> Self {
        Self { orig, min_t: min, dir, max_t: max }
    }

    /// Create a new ray from just an origin an a direction, implicitly sets `min_t == 0` and `max_t == T::MAX`
    #[inline]
    #[must_use]
    pub fn from_orig_and_dir(orig: Vec2<T>, dir: Vec2<T>) -> Self {
        Self { orig, min_t: T::zero(), dir: dir, max_t: T::MAX }
    }

    /// Create a ray from a line
    #[inline]
    #[must_use]
    pub fn from_line(line: Line2D<T>) -> Self {
        Self { orig: line.orig, min_t: T::zero(), dir: line.dir, max_t: T::MAX }
    }

    /// Create a ray from a line
    #[inline]
    #[must_use]
    pub fn from_line_between(line: Line2D<T>, min: T, max: T) -> Self {
        Self { orig: line.orig, min_t: min, dir: line.dir, max_t: max }
    }

    /// Check if a point at a given distance is on the ray
    #[inline]
    #[must_use]
    pub fn is_on_ray(self, dist: T) -> bool {
        dist >= self.min_t && dist <= self.max_t
    }

    /// Get the point at a given distance on the ray
    #[inline]
    #[must_use]
    pub fn point_at(self, dist: T) -> Vec2<T> {
        self.orig + self.dir * dist
    }

    /// Get the closts point to the point at the value that is still on the ray
    #[inline]
    #[must_use]
    pub fn closest_point(self, dist: T) -> Vec2<T> {
        self.orig + self.dir * dist.clamp(self.min_t, self.max_t)
    }

    /// Clamp the given ray param so it fits on the ray
    #[inline]
    #[must_use]
    pub fn clamp_dist(self, dist: T) -> T {
        dist.clamp(self.min_t, self.max_t)
    }

    /// Calculate the distance of the point on the ray
    #[inline]
    #[must_use]
    pub fn dist(self, point: Vec2<T>) -> T {
        let to_point = point - self.orig;
        self.dir.dot(to_point)
    }

    /// Calculate the distance of the point on the ray
    #[must_use]
    pub fn closest_dist(self, point: Vec2<T>) -> T {
        let to_point = point - self.orig;
        let min = self.min_t * self.min_t;
        let max = self.max_t * self.max_t; 
        self.dir.dot(to_point).clamp(min, max)
    }
}

impl<T: Real> ApproxEq<T> for Ray2D<T> {
    const EPSILON: T = T::EPSILON;

    fn is_close_to(self, rhs: Self, epsilon: T) -> bool {
        self.orig.is_close_to(rhs.orig, epsilon) &&
        self.min_t.is_close_to(rhs.min_t, epsilon) &&
        self.dir.is_close_to(rhs.dir, epsilon) &&
        self.max_t.is_close_to(rhs.max_t, epsilon)
    }
}


impl<T: Real> From<Line2D<T>> for Ray2D<T> {
    fn from(line: Line2D<T>) -> Self {
        Self::from_line(line)
    }
}

impl<T: Real + Display> Display for Ray2D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{{ o: {}, d: {}, t: [{}, {}] }}", self.orig, self.dir, self.min_t, self.max_t))
    }
}
