use crate::{Real, Numeric, ApproxEq, Vec2, Vec3};

/// 3D line
pub struct Line<T: Real> {
    pub orig : Vec3<T>,
    pub dir  : Vec3<T>,
}

impl<T: Real> Line<T> {
    /// Create a line
    pub fn new(orig: Vec3<T>, dir: Vec3<T>) -> Self {
        Self { orig, dir }
    }

    /// Create a line going through 2 points
    pub fn from_points(a: Vec3<T>, b: Vec3<T>) -> Self {
        let dir = (b - a).normalize();
        Self { orig: a, dir }
    }

    /// Get a point on the line at the given distance from the origin
    pub fn get_point_at(self, dist: T) -> Vec3<T> {
        self.orig + self.dir * dist
    }

    /// Get the closest point on the line to the given point
    pub fn get_closest_point_to(self, point: Vec3<T>) -> Vec3<T> {
        let to_point = point - self.orig;
        let dot = self.dir.dot(to_point);
        self.get_point_at(dot)
    }

    pub fn is_on_line(self, point: Vec3<T>) -> bool {
        let closest_point = self.get_closest_point_to(point);
        (point - closest_point).len_sq().is_zero()
    }
}

impl<T: Real> ApproxEq for Line<T> {
    type Epsilon = T;

    fn is_close_to(self, rhs: Self, epsilon: Self::Epsilon) -> bool {
        self.orig.is_close_to(rhs.orig, epsilon) &&
        self.dir.is_close_to(rhs.dir, epsilon)
    }
}

/// 3D line segment
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
    pub fn dir(self) -> Vec3<T> {
        (self.end - self.begin).normalize()
    }

    /// Get the closest point on the line to the given point
    pub fn get_closest_point_to(self, point: Vec3<T>) -> Vec3<T> {
        let dir = self.end - self.begin;
        let len = dir.len();
        let dir = dir / len;

        let to_point = point - self.begin;
        let dot = dir.dot(to_point);

        let dist = dot.clamp(T::zero(), len);
        self.begin + dir * dist
    }

    pub fn is_on_line(self, point: Vec3<T>) -> bool {
        let closest_point = self.get_closest_point_to(point);
        (point - closest_point).len_sq().is_zero()
    }

    pub fn to_line(self) -> Line<T> {
        let dir = (self.end - self.begin).normalize();
        Line { orig: self.begin, dir }
    }
}

impl<T: Numeric> ApproxEq for LineSegment<T> {
    type Epsilon = T;

    fn is_close_to(self, rhs: Self, epsilon: Self::Epsilon) -> bool {
        self.begin.is_close_to(rhs.begin, epsilon) &&
        self.end.is_close_to(rhs.end, epsilon)
    }
}

//------------------------------------------------------------------------------------------------------------------------------

/// 2D line
pub struct Line2D<T: Real> {
    pub orig : Vec2<T>,
    pub dir  : Vec2<T>,
}

impl<T: Real> Line2D<T> {
    /// Create a line
    pub fn new(orig: Vec2<T>, dir: Vec2<T>) -> Self {
        Self { orig, dir }
    }

    /// Create a line going through 2 points
    pub fn from_points(a: Vec2<T>, b: Vec2<T>) -> Self {
        let dir = (b - a).normalize();
        Self { orig: a, dir }
    }

    /// Get a point on the line at the given distance from the origin
    pub fn get_point_at(self, dist: T) -> Vec2<T> {
        self.orig + self.dir * dist
    }

    /// Get the closest point on the line to the given point
    pub fn get_closest_point_to(self, point: Vec2<T>) -> Vec2<T> {
        let to_point = point - self.orig;
        let dot = self.dir.dot(to_point);
        self.get_point_at(dot)
    }

    pub fn is_on_line(self, point: Vec2<T>) -> bool {
        let closest_point = self.get_closest_point_to(point);
        (point - closest_point).len_sq().is_zero()
    }
}

impl<T: Real> ApproxEq for Line2D<T> {
    type Epsilon = T;

    fn is_close_to(self, rhs: Self, epsilon: Self::Epsilon) -> bool {
        self.orig.is_close_to(rhs.orig, epsilon) &&
        self.dir.is_close_to(rhs.dir, epsilon)
    }
}

/// 2D line segment
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
    pub fn dir(self) -> Vec2<T> {
        (self.end - self.begin).normalize()
    }

    /// Get the closest point on the line to the given point
    pub fn get_closest_point_to(self, point: Vec2<T>) -> Vec2<T> {
        let dir = self.end - self.begin;
        let len = dir.len();
        let dir = dir / len;

        let to_point = point - self.begin;
        let dot = dir.dot(to_point);

        let dist = dot.clamp(T::zero(), len);
        self.begin + dir * dist
    }

    pub fn is_on_line(self, point: Vec2<T>) -> bool {
        let closest_point = self.get_closest_point_to(point);
        (point - closest_point).len_sq().is_zero()
    }

    pub fn to_line(self) -> Line2D<T> {
        let dir = (self.end - self.begin).normalize();
        Line2D { orig: self.begin, dir }
    }
}

impl<T: Numeric> ApproxEq for LineSegment2D<T> {
    type Epsilon = T;

    fn is_close_to(self, rhs: Self, epsilon: Self::Epsilon) -> bool {
        self.begin.is_close_to(rhs.begin, epsilon) &&
        self.end.is_close_to(rhs.end, epsilon)
    }
}
