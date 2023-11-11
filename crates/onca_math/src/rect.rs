use std::fmt::Display;

use onca_common_macros::flags;

use crate::*;

/// Quadrants of the a rect
/// ```text
///  TopLeft   |  Top   |    TopRight
/// -----------|--------|------------
///  Left      | Inside |       Right
/// -----------|--------|------------
/// BottomLeft | Bottom | BottomRight
/// ```
#[flags]
pub enum RectQuadrant {
    Inside = 0,
    Left,
    Right,
    Top,
    Bottom
}

/// 2D rectangle (can also be used as a 2D AABB)
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Rect<T: Numeric> {
    pub min : Point2<T>,
    pub max : Point2<T>
}

impl<T: Numeric> Rect<T> {
    /// Create a new rect
    #[must_use]
    pub fn new(min: Point2<T>, max: Point2<T>) -> Self {
        Self { min, max }
    }

    /// Get the size of the rect
    #[inline]
    #[must_use]
    pub fn size(self) -> Vec2<T> {
        self.max - self.min
    }

    /// Get the center of the rect
    #[inline]
    #[must_use]
    pub fn center(self) -> Point2<T> {
        ((self.min.to_vec() + self.max.to_vec()) / T::from_i32(2)).into()
    }

    /// Resize the rect around its center
    #[must_use]
    pub fn resize(self, size: Vec2<T>) -> Self {
        debug_assert!(size.x >= T::zero());
        debug_assert!(size.y >= T::zero());

        let center = self.center();
        let half_size = size / T::from_i32(2);
        Self { min: center - half_size, max: center + half_size }
    }

    /// Recenter the rect
    #[inline]
    #[must_use]
    pub fn recenter(self, center: Point2<T>) -> Self {
        let half_size = self.size() / T::from_i32(2);
        Self { min: center - half_size, max: center + half_size }
    }

    /// Expand the rect, passed as half the extend the rect should be expanded by
    #[inline]
    #[must_use]
    pub fn expand(self, half_extend: Vec2<T>) -> Self {
        Self { min: self.min - half_extend, max: self.max + half_extend }
    }

    /// Move the rect by the given delta
    #[inline]
    #[must_use]
    pub fn move_by(self, delta: Vec2<T>) -> Self {
        Self { min: self.min + delta, max: self.max + delta }
    }

    /// Create the smallest rect fitting both rects
    #[inline]
    #[must_use]
    pub fn merge(self, other: Self) -> Self {
        Self { min: self.min.min(other.min), max: self.max.max(other.max) }
    }

    /// Calculate the area of the rect
    #[inline]
    #[must_use]
    pub fn area(self) -> T {
        let size = self.size();
        size.x * size.y
    }

    /// Check if the rect fully contains another rect
    #[inline]
    #[must_use]
    pub fn contains(self, other: Self) -> bool {
        other.min.x >= self.min.x && other.max.x <= self.max.x &&
        other.min.y >= self.min.y && other.max.y <= self.max.y
    }

    /// Check if the rect contains a point
    #[inline]
    #[must_use]
    pub fn contains_point(self, point: Vec2<T>) -> bool {
        point.x >= self.min.x && point.x >= self.min.x &&
        point.y <= self.max.y && point.y <= self.max.y
    }

    // TODO: should we distiguish between overlap and touching?
    /// Check if 2 rects overlap
    #[inline]
    #[must_use]
    pub fn overlaps(self, other: Self) -> bool {
        self.min.x < other.max.x && self.max.x < other.min.x &&
        self.min.y < other.max.y && self.max.y < other.min.y
    }

    /// Calculate the squared distance from the rect to a point, 0 if the point is inside the rect
    #[must_use]
    pub fn dist_to_point_sq(self, point: Vec2<T>) -> T {
        let dist_min_x = self.min.x - point.x;
        let dist_max_x = point.x - self.max.x;
        let dist_x = dist_min_x.min(dist_max_x);

        let dist_min_y = self.min.y - point.y;
        let dist_max_y = point.y - self.max.y;
        let dist_y = dist_min_y.min(dist_max_y);

        if dist_x < T::zero() && dist_y < T::zero() {
            T::zero()
        } else {
            dist_x * dist_x + dist_y * dist_y
        }
    }

    /// Calculate the distance from the rect to a point, 0 if the point is inside the rect
    #[inline]
    #[must_use]
    pub fn dist_to_point(self, point: Vec2<T>) -> T {
        self.dist_to_point_sq(point).sqrt()
    }

    /// Clculate the squared distance from the rect to another rect
    #[must_use]
    pub fn dist_sq(self, other: Self) -> T {
        let dist_min_x = self.min.x - other.max.x;
        let dist_max_x = other.min.x - self.max.x;
        let dist_x = dist_min_x.min(dist_max_x);

        let dist_min_y = self.min.y - other.max.y;
        let dist_max_y = other.min.y - self.max.y;
        let dist_y = dist_min_y.min(dist_max_y);

        if dist_x < T::zero() && dist_y < T::zero() {
            T::zero()
        } else {
            dist_x * dist_x + dist_y * dist_y
        }
    }

    /// Clculate the distance from the rect to another rect
    #[inline]
    #[must_use]
    pub fn dist(self, other: Self) -> T {
        self.dist_sq(other).sqrt()
    }

    /// Get the [`RectQuadrant`] of the rectangle in which the point lies
    #[must_use]
    pub fn quadrant(self, point: Point2<T>) -> RectQuadrant {
        let mut quadrant = RectQuadrant::Inside;
        // If statements seems to create branched code, even with opt-level=3, so might not be optimal, but good enough for now
        if point.x < self.min.x {
            quadrant |= RectQuadrant::Left;
        } else if point.x > self.max.x {
            quadrant |= RectQuadrant::Right;
        }
        if point.y < self.min.y {
            quadrant |= RectQuadrant::Bottom;
        } else if point.y > self.max.y {
            quadrant  |= RectQuadrant::Top;
        }

        // This branchless version could be used if needed:
        // Comparison: https://godbolt.org/z/bYvKWKdPr
        //quadrant |= if point.x < self.min.x { RectQuadrant::Left } else { RectQuadrant::Inside };
        //quadrant |= if point.x > self.max.x { RectQuadrant::Right } else { RectQuadrant::Inside };
        //quadrant |= if point.y < self.min.y { RectQuadrant::Bottom } else { RectQuadrant::Inside };
        //quadrant |= if point.y > self.max.y { RectQuadrant::Top } else { RectQuadrant::Inside };
        quadrant
    }
}


impl<T: Numeric> ApproxEq<T> for Rect<T> {
    const EPSILON: T = T::EPSILON;

    fn is_close_to(self, rhs: Self, epsilon: T) -> bool {
        self.min.is_close_to(rhs.min, epsilon) &&
        self.max.is_close_to(rhs.max, epsilon)
    }
}

impl<T: Numeric + Display> Display for Rect<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{{ min: {}, max: {} }}", self.min, self.max))
    }
}
