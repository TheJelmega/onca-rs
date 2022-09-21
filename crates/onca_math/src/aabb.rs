use crate::{Numeric, Vec3, ApproxEq};

/// 2D aabbangle (can also be used as a 2D AABB)
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct AABB<T: Numeric> {
    pub min : Vec3<T>,
    pub max : Vec3<T>
}

impl<T: Numeric> AABB<T> {
    /// Get the size of the aabb
    #[inline]
    #[must_use]
    pub fn size(self) -> Vec3<T> {
        self.max - self.min
    }

    /// Get the center of the aabb
    #[inline]
    #[must_use]
    pub fn center(self) -> Vec3<T> {
        (self.min + self.max) / T::from_i32(2)
    }

    /// Resize the aabb around its center
    #[must_use]
    pub fn resize(self, size: Vec3<T>) -> Self {
        debug_assert!(size.x >= T::zero());
        debug_assert!(size.y >= T::zero());
        debug_assert!(size.z >= T::zero());

        let center = self.center();
        let half_size = size / T::from_i32(2);
        Self { min: center - half_size, max: center + half_size }
    }

    /// Recenter the aabb
    #[inline]
    #[must_use]
    pub fn recenter(self, center: Vec3<T>) -> Self {
        let half_size = self.size() / T::from_i32(2);
        Self { min: center - half_size, max: center + half_size }
    }

    /// Expand the aabb, passed as half the extend the aabb should be expanded by
    #[inline]
    #[must_use]
    pub fn expand(self, half_extend: Vec3<T>) -> Self {
        Self { min: self.min - half_extend, max: self.max + half_extend }
    }

    /// Move the aabb by the given delta
    #[inline]
    #[must_use]
    pub fn move_by(self, delta: Vec3<T>) -> Self {
        Self { min: self.min + delta, max: self.max + delta }
    }

    /// Create the smallest aabb fitting both aabbs
    #[inline]
    #[must_use]
    pub fn merge(self, other: Self) -> Self {
        Self { min: self.min.min(other.min), max: self.max.max(other.max) }
    }

    /// Calculate the area of the aabb
    #[inline]
    #[must_use]
    pub fn volume(self) -> T {
        let size = self.size();
        size.x * size.y * size.z
    }

    /// Check if the aabb fully contains another aabb
    #[inline]
    #[must_use]
    pub fn contains(self, other: Self) -> bool {
        other.min.x >= self.min.x && other.max.x <= self.max.x &&
        other.min.y >= self.min.y && other.max.y <= self.max.y &&
        other.min.z >= self.min.z && other.max.z <= self.max.z
        
    }

    /// Check if the aabb contains a point
    #[inline]
    #[must_use]
    pub fn contains_point(self, point: Vec3<T>) -> bool {
        point.x >= self.min.x && point.x >= self.min.x &&
        point.y <= self.max.y && point.y <= self.max.y &&
        point.z <= self.max.z && point.z <= self.max.z
    }

    // TODO(jel): should we distiguish between overlap and touching?
    /// Check if 2 aabbs overlap
    #[inline]
    #[must_use]
    pub fn overlaps(self, other: Self) -> bool {
        self.min.x < other.max.x && self.max.x < other.min.x &&
        self.min.y < other.max.y && self.max.y < other.min.y &&
        self.min.z < other.max.z && self.max.z < other.min.z
    }

    /// Calculate the squared distance from the aabb to a point, 0 if the point is inside the aabb
    #[must_use]
    pub fn dist_to_point_sq(self, point: Vec3<T>) -> T {
        let dist_min_x = self.min.x - point.x;
        let dist_max_x = point.x - self.max.x;
        let dist_x = dist_min_x.min(dist_max_x);

        let dist_min_y = self.min.y - point.y;
        let dist_max_y = point.y - self.max.y;
        let dist_y = dist_min_y.min(dist_max_y);

        let dist_min_z = self.min.z - point.z;
        let dist_max_z = point.z - self.max.z;
        let dist_z = dist_min_z.min(dist_max_z);

        if dist_x < T::zero() && dist_y < T::zero() && dist_z < T::zero() {
            T::zero()
        } else {
            dist_x * dist_x + dist_y * dist_y + dist_z * dist_z
        }
    }

    /// Calculate the distance from the aabb to a point, 0 if the point is inside the aabb
    #[inline]
    #[must_use]
    pub fn dist_to_point(self, point: Vec3<T>) -> T {
        self.dist_to_point_sq(point).sqrt()
    }

    /// Calculate the squared distance from the aabb to another aabb
    #[must_use]
    pub fn dist_sq(self, other: Self) -> T {
        let dist_min_x = self.min.x - other.max.x;
        let dist_max_x = other.min.x - self.max.x;
        let dist_x = dist_min_x.min(dist_max_x);

        let dist_min_y = self.min.y - other.max.y;
        let dist_max_y = other.min.y - self.max.y;
        let dist_y = dist_min_y.min(dist_max_y);

        let dist_min_z = self.min.z - other.max.z;
        let dist_max_z = other.min.z - self.max.z;
        let dist_z = dist_min_z.min(dist_max_z);

        if dist_x < T::zero() && dist_y < T::zero() && dist_z < T::zero() {
            T::zero()
        } else {
            dist_x * dist_x + dist_y * dist_y + dist_z * dist_z
        }
    }

    /// Clculate the distance from the aabb to another aabb
    #[inline]
    #[must_use]
    pub fn dist(self, other: Self) -> T {
        self.dist_sq(other).sqrt()
    }
}


impl<T: Numeric> ApproxEq for AABB<T> {
    type Epsilon = T;

    fn is_close_to(self, rhs: Self, epsilon: Self::Epsilon) -> bool {
        self.min.is_close_to(rhs.min, epsilon) &&
        self.max.is_close_to(rhs.max, epsilon)
    }
}
