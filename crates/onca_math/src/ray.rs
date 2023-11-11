use std::fmt::Display;

use crate::*;

/// 2D Ray
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Ray2D<T: Real> {
    pub orig  : Point2<T>,
    pub dir   : Vec2<T>,
}

impl<T: Real> Ray2D<T> {
    /// Create a new ray
    #[inline]
    #[must_use]
    pub fn new(orig: Point2<T>, dir: Vec2<T>) -> Self {
        Self { orig, dir }
    }

    /// Create a ray from a line
    #[inline]
    #[must_use]
    pub fn from_line(line: Line2D<T>) -> Self {
        Self { orig: line.orig, dir: line.dir }
    }

    
    /// Create a ray from a line segment, starting at the segment's start
    #[inline(always)]
    #[must_use]
    pub fn from_line_segment(segment: LineSegment2D<T>) -> Self {
        let ab = segment.end - segment.begin;
        let dir = ab.normalize();
        Self { orig: segment.begin, dir }
    }

    
    /// Convert the ray to a bounded ray with a min and max value
    #[inline(always)]
    #[must_use]
    pub fn to_bounded_ray(self, min: T, max: T) -> BoundedRay2D<T> {
        BoundedRay2D::new(self.orig, self.dir, min, max)
    }

    // Convert the ray to a bounded ray with `min` and `max` set to `0`` and `T::MAX` respectively
    #[inline(always)]
    #[must_use]
    pub fn to_unbounded_bounded_ray(self) -> BoundedRay2D<T> {
        BoundedRay2D::new_orig_dir(self.orig, self.dir)
    }


    /// Check if a `t` will map to a valid point on the ray
    #[must_use]
    pub fn is_on_ray(&self, t: T) -> bool {
        t >= T::zero()
    }

    /// Get the point at a given `t`` on the ray
    #[must_use]
    pub fn point_at(&self, t: T) -> Point2<T> {
        self.orig + self.dir * t
    }

    /// Get the closts point to the point at the value that is still on the ray
    /// 
    /// This function is the same as `point_at`, but exists to keep a consistent API between a bounded and unbounded ray
    #[inline(always)]
    #[must_use]
    pub fn closest_point(&self, t: T) -> Point2<T> {
        self.point_at(t)
    }

    /// Clamp a given `t` to the allowed range on the ray
    /// 
    /// This function just returns `t`, but exists to keep a consistent API between a bounded and unbounded ray
    #[inline(always)]
    #[must_use]
    pub fn clamp_t(&self, t: T) -> T {
        t
    }

    /// Calculate the `t` of the closest point on the ray to a given `point`
    #[must_use]
    pub fn get_t(&self, point: Point2<T>) -> T {
        let to_point = point - self.orig;
        self.dir.dot(to_point)
    }

    /// Calculate the `t` of the closest point on the ray to a given `point`
    /// 
    /// This function is the same as `get_t`, but exists to keep a consisten API between a bounded and unbounded ray
    #[inline(always)]
    #[must_use]
    pub fn get_bounded_t(&self, point: Point2<T>) -> T {
        self.get_t(point)
    }

    /// Calculate the squae distance between a `point`` and the closest point on a ray
    pub fn dist_sq(&self, point: Point2<T>) -> T {
        self.dist(point).sqr()
    }

    /// Calculate the distance between a `point`` and the closest point on a ray
    pub fn dist(&self, point: Point2<T>) -> T {
        let to_point = point - self.orig;
        self.dir.cross(to_point)
    }

    /// Calculate the square distance between a `point`` and the closest point on a ray
    /// 
    /// This function is the same as `get_t`, but exists to keep a consisten API between a bounded and unbounded ray
    pub fn bounded_dist_sq(&self, point: Point2<T>) -> T {
        self.dist_sq(point)
    }

    /// Calculate the distance between a `point`` and the closest point on a ray
    /// 
    /// This function is the same as `get_t`, but exists to keep a consisten API between a bounded and unbounded ray
    pub fn bounded_dist(&self, point: Point2<T>) -> T {
        self.dist(point)
    }

    /// Calculate the distance to the intersection between the ray and an object
    pub fn intersect<U>(&self, obj: &U) -> Option<T> where
        U: IntersectWithRay<T, Self>
    {
        obj.intersect_ray(self)
    }
}

impl<T: Real> ApproxEq<T> for Ray2D<T> {
    const EPSILON: T = T::EPSILON;

    fn is_close_to(self, rhs: Self, epsilon: T) -> bool {
        self.orig.is_close_to(rhs.orig, epsilon) &&
        self.dir.is_close_to(rhs.dir, epsilon)
    }
}

impl<T: Real> From<Line2D<T>> for Ray2D<T> {
    fn from(line: Line2D<T>) -> Self {
        Self::from_line(line)
    }
}

impl<T: Real> From<LineSegment2D<T>> for Ray2D<T> {
    fn from(line: LineSegment2D<T>) -> Self {
        Self::from_line_segment(line)
    }
}

impl<T: Real + Display> Display for Ray2D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{{ o: {}, d: {} }}", self.orig, self.dir))
    }
}

//------------------------------------------------------------------------------------------------------------------------------

/// 3D Ray
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Ray<T: Real> {
    pub orig  : Point3<T>,
    pub dir   : Vec3<T>,
}

impl<T: Real> Ray<T> {
    /// Create a new ray
    #[inline]
    #[must_use]
    pub fn new(orig: Point3<T>, dir: Vec3<T>) -> Self {
        Self { orig, dir }
    }

    /// Create a ray from a line
    #[inline]
    #[must_use]
    pub fn from_line(line: Line<T>) -> Self {
        Self { orig: line.orig, dir: line.dir }
    }

    
    /// Create a ray from a line segment, starting at the segment's start
    #[inline(always)]
    #[must_use]
    pub fn from_line_segment(segment: LineSegment<T>) -> Self {
        let ab = segment.end - segment.begin;
        let dir = ab.normalize();
        Self { orig: segment.begin, dir }
    }

    
    /// Convert the ray to a bounded ray with a min and max value
    #[inline(always)]
    #[must_use]
    pub fn to_bounded_ray(self, min: T, max: T) -> BoundedRay<T> {
        BoundedRay::new(self.orig, self.dir, min, max)
    }

    // Convert the ray to a bounded ray with `min` and `max` set to `0`` and `T::MAX` respectively
    #[inline(always)]
    #[must_use]
    pub fn to_unbounded_bounded_ray(self) -> BoundedRay<T> {
        BoundedRay::new_orig_dir(self.orig, self.dir)
    }


    /// Check if a `t` will map to a valid point on the ray
    #[inline(always)]
    #[must_use]
    pub fn is_on_ray(&self, t: T) -> bool {
        t >= T::zero()
    }

    /// Get the point at a given distance on the ray
    #[inline]
    #[must_use]
    pub fn point_at(&self, t: T) -> Point3<T> {
        self.orig + self.dir * t
    }

    /// Get the closts point to the point at the value that is still on the ray
    /// 
    /// This function is the same as `point_at`, but exists to keep a consistent API between a bounded and unbounded ray
    #[inline(always)]
    #[must_use]
    pub fn closest_point(&self, t: T) -> Point3<T> {
        self.point_at(t)
    }

    /// Clamp a given `t` to the allowed range on the ray
    /// 
    /// This function just returns `t`, but exists to keep a consistent API between a bounded and unbounded ray
    #[inline(always)]
    #[must_use]
    pub fn clamp_t(&self, t: T) -> T {
        t
    }

    // Calculate the `t` of the closest point on the ray to a given `point`
    #[must_use]
    pub fn get_t(&self, point: Point3<T>) -> T {
        let to_point = point - self.orig;
        self.dir.dot(to_point)
    }

    /// Calculate the `t` of the closest point on the ray to a given `point`
    /// 
    /// This function is the same as `get_t`, but exists to keep a consisten API between a bounded and unbounded ray
    #[inline(always)]
    #[must_use]
    pub fn get_bounded_t(&self, point: Point3<T>) -> T {
        self.get_t(point)
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
        self.dist_sq(point)
    }

    /// Calculate the distance between a `point`` and the closest point on a ray
    /// 
    /// This function is the same as `get_t`, but exists to keep a consisten API between a bounded and unbounded ray
    pub fn bounded_dist(&self, point: Point3<T>) -> T {
        self.dist(point)
    }

    /// Calculate the distance to the intersection between the ray and an object
    pub fn intersect<U>(&self, obj: &U) -> Option<T> where
        U: IntersectWithRay<T, Self>
    {
        obj.intersect_ray(self)
    }
}

impl<T: Real> ApproxEq<T> for Ray<T> {
    const EPSILON: T = T::EPSILON;

    fn is_close_to(self, rhs: Self, epsilon: T) -> bool {
        self.orig.is_close_to(rhs.orig, epsilon) &&
        self.dir.is_close_to(rhs.dir, epsilon)
    }
}

impl<T: Real> From<Line<T>> for Ray<T> {
    fn from(line: Line<T>) -> Self {
        Self::from_line(line)
    }
}

impl<T: Real> From<LineSegment<T>> for Ray<T> {
    fn from(line: LineSegment<T>) -> Self {
        Self::from_line_segment(line)
    }
}

impl<T: Real + Display> Display for Ray<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{{ o: {}, d: {} }}", self.orig, self.dir))
    }
}
