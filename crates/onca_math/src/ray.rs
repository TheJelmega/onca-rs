use crate::{Real, ApproxEq, Vec3, Vec2};

/// 3D Ray
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Ray<T: Real> {
    pub origin : Vec3<T>,
    pub min_t  : T,
    pub dir    : Vec3<T>,
    pub max_t  : T 
}

impl<T: Real> Ray<T> {
    /// Create a new ray
    pub fn new(orig: Vec3<T>, dir: Vec3<T>, min: T, max: T) -> Self {
        Self { origin: orig, min_t: min, dir, max_t: max }
    }

    /// Create a new ray from just an origin an a direction, implicitly sets `min_t == 0` and `max_t == T::MAX`
    pub fn from_orig_and_dir(orig: Vec3<T>, dir: Vec3<T>) -> Self {
        Self { origin: orig, min_t: T::zero(), dir: dir, max_t: T::MAX }
    }

    pub fn is_on_ray(self, ray_param: T) -> bool {
        ray_param >= T::zero() && ray_param <= T::one()
    }

    /// Get the point at a given distance on the ray
    pub fn point_at(self, ray_param: T) -> Vec3<T> {
        let len = self.max_t - self.min_t;
        self.origin + self.dir * len * ray_param
    }

    /// Get the closts point to the point at the value that is still on the ray
    pub fn closest_point(self, ray_param: T) -> Vec3<T> {
        let len = self.max_t - self.min_t;
        self.origin + self.dir * len * ray_param.clamp(T::zero(), T::one())
    }

    /// Get the ray parameter of the the closest point to the given point
    /// 
    /// The ray parameter is in the range [0;1]
    pub fn get_ray_param(self, point: Vec3<T>) -> T {
        let len = self.max_t - self.min_t;
        self.dist(point) / len
    }

    /// Get the ray parameter of the the closest point, that is on the ray, to the given point
    /// 
    /// The ray parameter is in the range [0;1]
    pub fn get_closest_ray_param(self, point: Vec3<T>) -> T {
        let len = self.max_t - self.min_t;
        self.closest_dist(point) / len
    }

    /// Clamp the given ray param so it fits on the ray
    pub fn clamp_ray_param(ray_param: T) -> T {
        ray_param.clamp(T::zero(), T::one())
    }

    /// Calculate the squared distance of the point on the ray
    pub fn dist_sq(self, point: Vec3<T>) -> T {
        let to_point = point - self.origin;
        self.dir.dot(to_point)
    }

    /// Calculate the squared distance of the point on the ray
    pub fn dist(self, point: Vec3<T>) -> T {
        self.dist_sq(point).sqrt()
    }

    /// Calculate the squared distance of the point on the ray
    pub fn closest_dist_sq(self, point: Vec3<T>) -> T {
        let to_point = point - self.origin;
        let min = self.min_t * self.min_t;
        let max = self.max_t * self.max_t; 
        self.dir.dot(to_point).clamp(min, max)
    }

    /// Calculate the squared distance of the point on the ray
    pub fn closest_dist(self, point: Vec3<T>) -> T {
        self.dist(point).clamp(self.min_t, self.max_t)
    }
}

impl<T: Real> ApproxEq for Ray<T> {
    type Epsilon = T;

    fn is_close_to(self, rhs: Self, epsilon: Self::Epsilon) -> bool {
        self.origin.is_close_to(rhs.origin, epsilon) &&
        self.min_t.is_close_to(rhs.min_t, epsilon) &&
        self.dir.is_close_to(rhs.dir, epsilon) &&
        self.max_t.is_close_to(rhs.max_t, epsilon)
    }
}

//------------------------------------------------------------------------------------------------------------------------------

/// 3D Ray
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Ray2D<T: Real> {
    pub origin : Vec2<T>,
    pub min_t  : T,
    pub dir    : Vec2<T>,
    pub max_t  : T 
}

impl<T: Real> Ray2D<T> {
    /// Create a new ray
    pub fn new(orig: Vec2<T>, dir: Vec2<T>, min: T, max: T) -> Self {
        Self { origin: orig, min_t: min, dir, max_t: max }
    }

    /// Create a new ray from just an origin an a direction, implicitly sets `min_t == 0` and `max_t == T::MAX`
    pub fn from_orig_and_dir(orig: Vec2<T>, dir: Vec2<T>) -> Self {
        Self { origin: orig, min_t: T::zero(), dir: dir, max_t: T::MAX }
    }

    pub fn is_on_ray(self, ray_param: T) -> bool {
        ray_param >= T::zero() && ray_param <= T::one()
    }

    /// Get the point at a given distance on the ray
    pub fn point_at(self, ray_param: T) -> Vec2<T> {
        let len = self.max_t - self.min_t;
        self.origin + self.dir * len * ray_param
    }

    /// Get the closts point to the point at the value that is still on the ray
    pub fn closest_point(self, ray_param: T) -> Vec2<T> {
        let len = self.max_t - self.min_t;
        self.origin + self.dir * len * ray_param.clamp(T::zero(), T::one())
    }

    /// Get the ray parameter of the the closest point to the given point
    /// 
    /// The ray parameter is in the range [0;1]
    pub fn get_ray_param(self, point: Vec2<T>) -> T {
        let len = self.max_t - self.min_t;
        self.dist(point) / len
    }

    /// Get the ray parameter of the the closest point, that is on the ray, to the given point
    /// 
    /// The ray parameter is in the range [0;1]
    pub fn get_closest_ray_param(self, point: Vec2<T>) -> T {
        let len = self.max_t - self.min_t;
        self.closest_dist(point) / len
    }

    /// Clamp the given ray param so it fits on the ray
    pub fn clamp_ray_param(ray_param: T) -> T {
        ray_param.clamp(T::zero(), T::one())
    }

    /// Calculate the squared distance of the point on the ray
    pub fn dist_sq(self, point: Vec2<T>) -> T {
        let to_point = point - self.origin;
        self.dir.dot(to_point)
    }

    /// Calculate the squared distance of the point on the ray
    pub fn dist(self, point: Vec2<T>) -> T {
        self.dist_sq(point).sqrt()
    }

    /// Calculate the squared distance of the point on the ray
    pub fn closest_dist_sq(self, point: Vec2<T>) -> T {
        let to_point = point - self.origin;
        let min = self.min_t * self.min_t;
        let max = self.max_t * self.max_t; 
        self.dir.dot(to_point).clamp(min, max)
    }

    /// Calculate the squared distance of the point on the ray
    pub fn closest_dist(self, point: Vec2<T>) -> T {
        self.dist(point).clamp(self.min_t, self.max_t)
    }
}

impl<T: Real> ApproxEq for Ray2D<T> {
    type Epsilon = T;

    fn is_close_to(self, rhs: Self, epsilon: Self::Epsilon) -> bool {
        self.origin.is_close_to(rhs.origin, epsilon) &&
        self.min_t.is_close_to(rhs.min_t, epsilon) &&
        self.dir.is_close_to(rhs.dir, epsilon) &&
        self.max_t.is_close_to(rhs.max_t, epsilon)
    }
}
