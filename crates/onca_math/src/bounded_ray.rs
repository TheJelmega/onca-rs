use std::fmt::Display;

use crate::*;

/// A 2D ray with bounds on the minimum and maximum distance
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct BoundedRay2D<T: Real> {
    pub orig: Point2<T>,
    pub dir:  Vec2<T>,
    pub min:  T,
    pub max:  T,
}

impl<T: Real> BoundedRay2D<T> {
    /// Create a new bounded ray.
    #[inline(always)]
    #[must_use]
    pub fn new(orig: Point2<T>, dir: Vec2<T>, min: T, max: T) -> Self {
        Self { orig, dir, min, max }
    }

    /// Create a new bounded ray from just an origin and direction.
    /// 
    /// Min and max will be set to 0 and the maximum possible value respectively.
    #[inline(always)]
    #[must_use]
    pub fn new_orig_dir(orig: Point2<T>, dir: Vec2<T>) -> Self {
        Self { orig, dir, min: T::zero(), max: T::MAX }
    }

    /// Create a new bounded ray from just a line.
    /// 
    /// Min and max will be set to 0 and the maximum possible value respectively.
    #[inline(always)]
    #[must_use]
    pub fn from_line(line: Line2D<T>) -> Self {
        Self { orig: line.orig, dir: line.dir, min: T::zero(), max: T::MAX }
    }

    /// Create a new bounded ray from just a line, with a min and max bound.
    #[inline(always)]
    #[must_use]
    pub fn from_line_between(line: Line2D<T>, min: T, max: T) -> Self {
        Self { orig: line.orig, dir: line.dir, min, max }
    }

    /// Create a ray from a line segment, which is bounded by the segment.
    #[inline(always)]
    #[must_use]
    pub fn from_line_segment(segment: LineSegment2D<T>) -> Self {
        let ab = segment.end - segment.begin;
        let (dir, len) = ab.dir_and_len();
        Self { orig: segment.begin, dir, min: T::zero(), max: len }
    }

    /// Convert a bounded ray into a regular ray.
    #[inline(always)]
    #[must_use]
    pub fn to_ray(self) -> Ray2D<T> {
        Ray2D { orig: self.orig, dir: self.dir }
    }
    
    /// Check if a `t` will map to a valid point on the ray.
    #[inline]
    #[must_use]
    pub fn is_on_ray(&self, t: T) -> bool {
        t >= self.min && t <= self.max
    }

    /// Caculate the point at location `t` on the ray.
    #[inline]
    #[must_use]
    pub fn point_at(&self, t: T) -> Point2<T> {
        self.orig + self.dir * t
    }

    /// Calculate the closets point to given a `t` on the ray.
    #[inline]
    #[must_use]
    pub fn closest_point_at(&self, t: T) -> Point2<T> {
        self.orig + self.dir * t.clamp(self.min, self.max)
    }

    /// Clamp a given `t` to the allowed range on the ray.
    #[inline]
    #[must_use]
    pub fn clamp_t(&self, t: T) -> T {
        t.clamp(self.min, self.max)
    }

    /// Calculate the `t` of the closest point on the ray to a given `point`, when ignoring the `min` and `max` bounds.
    pub fn get_t(&self, point: Point2<T>) -> T {
        let ab = point - self.orig;
        self.dir.dot(ab)
    }

    /// Calculate the `t` of the closest point on the ray to a given `point`, within the bounds of the ray.
    pub fn get_bounded_t(&self, point: Point2<T>) -> T {
        self.get_t(point).clamp(self.min, self.max)
    }

    /// Calculate the distance between a `point`` and the closest point on a ray, when ignoring the `min` and `max` bounds.
    pub fn dist_sq(&self, point: Point2<T>) -> T {
        self.dist(point).sqr()
    }

    /// Calculate the distance between a `point`` and the closest point on a ray, when ignoring the `min` and `max` bounds.
    pub fn dist(&self, point: Point2<T>) -> T {
        let to_point = point - self.orig;
        self.dir.cross(to_point)
    }

    /// Calculate the square distance between a `point`` and the closest point on a ray, with the closest point within the bounds of the ray.
    pub fn bounded_dist_sq(&self, point: Point2<T>) -> T {
        self.dist_sq(point).clamp(self.min.sqr(), self.max.sqr())
    }
    
    /// Calculate the distance between a `point`` and the closest point on a ray, with the closest point within the bounds of the ray.
    pub fn bounded_dist(&self, point: Point2<T>) -> T {
        self.dist(point).clamp(self.min, self.max)
    }
    
    /// Calculate the distance to the intersection between the ray and an object
    pub fn intersect<U>(&self, obj: &U) -> Option<T> where
        U: IntersectWithRay<T, Self>
    {
        obj.intersect_ray(self)
    }
}


impl<T: Real> ApproxEq<T> for BoundedRay2D<T> {
    const EPSILON: T = T::EPSILON;

    fn is_close_to(self, rhs: Self, epsilon: T) -> bool {
        self.orig.is_close_to(rhs.orig, epsilon) &&
        self.min.is_close_to(rhs.min, epsilon) &&
        self.dir.is_close_to(rhs.dir, epsilon) &&
        self.max.is_close_to(rhs.max, epsilon)
    }
}

impl<T: Real> From<Line2D<T>> for BoundedRay2D<T> {
    fn from(line: Line2D<T>) -> Self {
        Self::from_line(line)
    }
}

impl<T: Real> From<LineSegment2D<T>> for BoundedRay2D<T> {
    fn from(line: LineSegment2D<T>) -> Self {
        Self::from_line_segment(line)
    }
}

impl<T: Real + Display> Display for BoundedRay2D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{{ o: {}, d: {}, t: [{}, {}] }}", self.orig, self.dir, self.min, self.max))
    }
}

//------------------------------------------------------------------------------------------------------------------------------


/// A 2D ray with bounds on the minimum and maximum distance
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct BoundedRay<T: Real> {
    pub orig: Point3<T>,
    pub dir:  Vec3<T>,
    pub min:  T,
    pub max:  T,
}

impl<T: Real> BoundedRay<T> {
    /// Create a new bounded ray.
    #[inline(always)]
    #[must_use]
    pub fn new(orig: Point3<T>, dir: Vec3<T>, min: T, max: T) -> Self {
        Self { orig, dir, min, max }
    }

    /// Create a new bounded ray from just an origin and direction.
    /// 
    /// Min and max will be set to 0 and the maximum possible value respectively.
    #[inline(always)]
    #[must_use]
    pub fn new_orig_dir(orig: Point3<T>, dir: Vec3<T>) -> Self {
        Self { orig, dir, min: T::zero(), max: T::MAX }
    }

    /// Create a new bounded ray from just a line.
    /// 
    /// Min and max will be set to 0 and the maximum possible value respectively.
    #[inline(always)]
    #[must_use]
    pub fn from_line(line: Line<T>) -> Self {
        Self { orig: line.orig, dir: line.dir, min: T::zero(), max: T::MAX }
    }

    /// Create a new bounded ray from just a line, with a min and max bound.
    #[inline(always)]
    #[must_use]
    pub fn from_line_between(line: Line<T>, min: T, max: T) -> Self {
        Self { orig: line.orig, dir: line.dir, min, max }
    }

    /// Create a ray from a line segment, which is bounded by the segment.
    #[inline(always)]
    #[must_use]
    pub fn from_line_segment(segment: LineSegment<T>) -> Self {
        let ab = segment.end - segment.begin;
        let (dir, len) = ab.dir_and_len();
        Self { orig: segment.begin, dir, min: T::zero(), max: len }
    }

    /// Convert a bounded ray into a regular ray.
    #[inline(always)]
    #[must_use]
    pub fn to_ray(self) -> Ray<T> {
        Ray { orig: self.orig, dir: self.dir }
    }
    
    /// Check if a `t` will map to a valid point on the ray.
    #[inline]
    #[must_use]
    pub fn is_on_ray(&self, t: T) -> bool {
        t >= self.min && t <= self.max
    }

    /// Caculate the point at location `t` on the ray.
    #[inline]
    #[must_use]
    pub fn point_at(&self, t: T) -> Point3<T> {
        self.orig + self.dir * t
    }

    /// Calculate the closets point to given a `t` on the ray.
    #[inline]
    #[must_use]
    pub fn closest_point_at(&self, t: T) -> Point3<T> {
        self.orig + self.dir * t.clamp(self.min, self.max)
    }

    /// Clamp a given `t` to the allowed range on the ray.
    #[inline]
    #[must_use]
    pub fn clamp_t(&self, t: T) -> T {
        t.clamp(self.min, self.max)
    }

    /// Calculate the `t` of the closest point on the ray to a given `point`, when ignoring the `min` and `max` bounds.
    pub fn get_t(&self, point: Point3<T>) -> T {
        let ab = point - self.orig;
        self.dir.dot(ab)
    }

    /// Calculate the `t` of the closest point on the ray to a given `point`, within the bounds of the ray.
    pub fn get_bounded_t(&self, point: Point3<T>) -> T {
        self.get_t(point).clamp(self.min, self.max)
    }

    /// Calculate the distance between a `point`` and the closest point on a ray
    pub fn dist_sq(&self, point: Point3<T>) -> T {
        let t = self.get_t(point);
        let closest_point = self.point_at(t);
        point.dist_sq(closest_point)
    }

    /// Calculate the distance between a `point`` and the closest point on a ray
    pub fn dist(&self, point: Point3<T>) -> T {
        self.dist_sq(point).sqrt()
    }

    /// Calculate the square distance between a `point`` and the closest point on a ray
    /// 
    /// This function is the same as `get_t`, but exists to keep a consisten API between a bounded and unbounded ray
    pub fn bounded_dist_sq(&self, point: Point3<T>) -> T {
        let t = self.get_t(point);
        let closest_point = self.closest_point_at(t);
        point.dist_sq(closest_point)
    }

    /// Calculate the distance between a `point`` and the closest point on a ray
    /// 
    /// This function is the same as `get_t`, but exists to keep a consisten API between a bounded and unbounded ray
    pub fn bounded_dist(&self, point: Point3<T>) -> T {
        self.bounded_dist_sq(point).sqrt()
    }

    /// Calculate the distance to the intersection between the ray and an object
    pub fn intersect<U>(&self, obj: &U) -> Option<T> where
        U: IntersectWithRay<T, Self>
    {
        obj.intersect_ray(self)
    }
}


impl<T: Real> ApproxEq<T> for BoundedRay<T> {
    const EPSILON: T = T::EPSILON;

    fn is_close_to(self, rhs: Self, epsilon: T) -> bool {
        self.orig.is_close_to(rhs.orig, epsilon) &&
        self.min.is_close_to(rhs.min, epsilon) &&
        self.dir.is_close_to(rhs.dir, epsilon) &&
        self.max.is_close_to(rhs.max, epsilon)
    }
}

impl<T: Real> From<Line<T>> for BoundedRay<T> {
    fn from(line: Line<T>) -> Self {
        Self::from_line(line)
    }
}

impl<T: Real> From<LineSegment<T>> for BoundedRay<T> {
    fn from(line: LineSegment<T>) -> Self {
        Self::from_line_segment(line)
    }
}

impl<T: Real + Display> Display for BoundedRay<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{{ o: {}, d: {}, t: [{}, {}] }}", self.orig, self.dir, self.min, self.max))
    }
}
