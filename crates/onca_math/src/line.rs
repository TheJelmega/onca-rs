use std::fmt::Display;

use crate::{Real, Numeric, ApproxEq, Vec2, Vec3, Ray, Ray2D};

/// 3D line
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Line<T: Real> {
    pub orig : Vec3<T>,
    pub dir  : Vec3<T>,
}

impl<T: Real> Line<T> {
    /// Create a line
    #[inline]
    #[must_use]
    pub fn new(orig: Vec3<T>, dir: Vec3<T>) -> Self {
        Self { orig, dir }
    }

    /// Create a line going through 2 points
    #[inline]
    #[must_use]
    pub fn from_points(a: Vec3<T>, b: Vec3<T>) -> Self {
        let dir = (b - a).normalize();
        Self { orig: a, dir }
    }

    /// Create a line from a segment
    #[inline]
    #[must_use]
    pub fn from_segment(segment: LineSegment<T>) -> Self {
        let dir = (segment.end - segment.begin).normalize();
        Self { orig: segment.begin, dir }
    }

    /// Create a line from a ray
    #[inline]
    #[must_use]
    pub fn from_ray(ray: Ray<T>) -> Self {
        Self { orig: ray.orig, dir: ray.dir }
    }

    /// Get a point on the line at the given distance from the origin
    #[inline]
    #[must_use]
    pub fn get_point_at(self, dist: T) -> Vec3<T> {
        self.orig + self.dir * dist
    }

    /// Get the closest point on the line to the given point
    #[must_use]
    pub fn get_closest_point_to(self, point: Vec3<T>) -> Vec3<T> {
        let to_point = point - self.orig;
        let dot = self.dir.dot(to_point);
        self.get_point_at(dot)
    }

    /// Check if a point is on the line
    #[inline]
    #[must_use]
    pub fn is_on_line(self, point: Vec3<T>) -> bool {
        let closest_point = self.get_closest_point_to(point);
        (point - closest_point).len_sq().is_zero()
    }

    /// Calculate the distance of the point on the line
    #[inline]
    #[must_use]
    pub fn dist(self, point: Vec3<T>) -> T {
        let to_point = point - self.orig;
        self.dir.dot(to_point)
    }
}

impl<T: Real> ApproxEq<T> for Line<T> {
    const EPSILON: T = T::EPSILON;

    fn is_close_to(self, rhs: Self, epsilon: T) -> bool {
        self.orig.is_close_to(rhs.orig, epsilon) &&
        self.dir.is_close_to(rhs.dir, epsilon)
    }
}

impl<T: Real + Display> Display for Line<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{{ o: {}, d: {} }}", self.orig, self.dir))
    }
}

impl<T: Real> From<LineSegment<T>> for Line<T> {
    fn from(segment: LineSegment<T>) -> Self {
        Self::from_segment(segment)
    }
}

impl<T: Real> From<Ray<T>> for Line<T> {
    fn from(ray: Ray<T>) -> Self {
        Self::from_ray(ray)
    }
}

/// 3D line segment
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct LineSegment<T: Numeric> {
    pub begin : Vec3<T>,
    pub end   : Vec3<T>
}

impl<T: Numeric> LineSegment<T> {
    /// Create a new line segment
    pub fn new(begin: Vec3<T>, end: Vec3<T>) -> Self {
        Self { begin, end }
    }
}

impl<T: Real> LineSegment<T> {
    /// Get the direction of the line segment
    #[inline]
    #[must_use]
    pub fn dir(self) -> Vec3<T> {
        (self.end - self.begin).normalize()
    }

    /// Get the closest point on the line to the given point
    #[inline]
    pub fn get_closest_point_to(self, point: Vec3<T>) -> Vec3<T> {
        let dir = self.end - self.begin;
        let len = dir.len();
        let dir = dir / len;

        let to_point = point - self.begin;
        let dot = dir.dot(to_point);

        let dist = dot.clamp(T::zero(), len);
        self.begin + dir * dist
    }

    /// Check if a point is on the line segment
    #[inline]
    #[must_use]
    pub fn is_on_line(self, point: Vec3<T>) -> bool {
        let closest_point = self.get_closest_point_to(point);
        (point - closest_point).len_sq().is_zero()
    }

    /// Convert the line segment to a line
    #[inline]
    #[must_use]
    pub fn to_line(self) -> Line<T> {
        Line::from_segment(self)
    }
}

impl<T: Numeric> ApproxEq<T> for LineSegment<T> {
    const EPSILON: T = T::EPSILON;

    fn is_close_to(self, rhs: Self, epsilon: T) -> bool {
        self.begin.is_close_to(rhs.begin, epsilon) &&
        self.end.is_close_to(rhs.end, epsilon)
    }
}

impl<T: Real + Display> Display for LineSegment<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{{ a: {}, b: {} }}", self.begin, self.end))
    }
}

//------------------------------------------------------------------------------------------------------------------------------

/// 2D line
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Line2D<T: Real> {
    pub orig : Vec2<T>,
    pub dir  : Vec2<T>,
}

impl<T: Real> Line2D<T> {
    /// Create a line
    #[inline]
    #[must_use]
    pub fn new(orig: Vec2<T>, dir: Vec2<T>) -> Self {
        Self { orig, dir }
    }

    /// Create a line going through 2 points
    #[inline]
    #[must_use]
    pub fn from_points(a: Vec2<T>, b: Vec2<T>) -> Self {
        let dir = (b - a).normalize();
        Self { orig: a, dir }
    }

    /// Create a line from a segment
    #[inline]
    #[must_use]
    pub fn from_segment(segment: LineSegment2D<T>) -> Self {
        let dir = (segment.end - segment.begin).normalize();
        Self { orig: segment.begin, dir }
    }

    /// Create a line from a ray
    #[inline]
    #[must_use]
    pub fn from_ray(ray: Ray2D<T>) -> Self {
        Self { orig: ray.orig, dir: ray.dir }
    }

    /// Get a point on the line at the given distance from the origin
    #[inline]
    #[must_use]
    pub fn get_point_at(self, dist: T) -> Vec2<T> {
        self.orig + self.dir * dist
    }

    /// Get the closest point on the line to the given point
    #[inline]
    #[must_use]
    pub fn get_closest_point_to(self, point: Vec2<T>) -> Vec2<T> {
        let to_point = point - self.orig;
        let dot = self.dir.dot(to_point);
        self.get_point_at(dot)
    }

    // Check if a point is on the line
    #[inline]
    #[must_use]
    pub fn is_on_line(self, point: Vec2<T>) -> bool {
        let closest_point = self.get_closest_point_to(point);
        (point - closest_point).len_sq().is_zero()
    }

    /// Calculate the distance of the point on the line
    #[inline]
    #[must_use]
    pub fn dist(self, point: Vec2<T>) -> T {
        let to_point = point - self.orig;
        self.dir.dot(to_point)
    }
}

impl<T: Real> ApproxEq<T> for Line2D<T> {
    const EPSILON: T = T::EPSILON;

    fn is_close_to(self, rhs: Self, epsilon: T) -> bool {
        self.orig.is_close_to(rhs.orig, epsilon) &&
        self.dir.is_close_to(rhs.dir, epsilon)
    }
}

impl<T: Real + Display> Display for Line2D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{{ o: {}, d: {} }}", self.orig, self.dir))
    }
}

impl<T: Real> From<LineSegment2D<T>> for Line2D<T> {
    fn from(segment: LineSegment2D<T>) -> Self {
        Self::from_segment(segment)
    }
}

impl<T: Real> From<Ray2D<T>> for Line2D<T> {
    fn from(ray: Ray2D<T>) -> Self {
        Self::from_ray(ray)
    }
}

/// 2D line segment
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct LineSegment2D<T: Numeric> {
    pub begin : Vec2<T>,
    pub end   : Vec2<T>
}

impl<T: Numeric> LineSegment2D<T> {
    /// Create a new line segment
    pub fn new(begin: Vec2<T>, end: Vec2<T>) -> Self {
        Self { begin, end }
    }
}

impl<T: Real> LineSegment2D<T> {
    /// Get the direction of the line segment
    #[inline]
    #[must_use]
    pub fn dir(self) -> Vec2<T> {
        (self.end - self.begin).normalize()
    }

    /// Get the closest point on the line to the given point
    #[must_use]
    pub fn get_closest_point_to(self, point: Vec2<T>) -> Vec2<T> {
        let dir = self.end - self.begin;
        let len = dir.len();
        let dir = dir / len;

        let to_point = point - self.begin;
        let dot = dir.dot(to_point);

        let dist = dot.clamp(T::zero(), len);
        self.begin + dir * dist
    }

    // Check if a point is on the line segment
    #[inline]
    #[must_use]
    pub fn is_on_line(self, point: Vec2<T>) -> bool {
        let closest_point = self.get_closest_point_to(point);
        (point - closest_point).len_sq().is_zero()
    }

    /// Convert the line segment to a line
    #[inline]
    #[must_use]
    pub fn to_line(self) -> Line2D<T> {
        Line2D::from_segment(self)
    }
}

impl<T: Numeric> ApproxEq<T> for LineSegment2D<T> {
    const EPSILON: T = T::EPSILON;

    fn is_close_to(self, rhs: Self, epsilon: T) -> bool {
        self.begin.is_close_to(rhs.begin, epsilon) &&
        self.end.is_close_to(rhs.end, epsilon)
    }
}

impl<T: Real + Display> Display for LineSegment2D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{{ a: {}, b: {} }}", self.begin, self.end))
    }
}
