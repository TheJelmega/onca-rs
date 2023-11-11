use crate::*;

/// 2D local coordinate space used to represent a space that is not the global space.
/// 
/// The global space is defined as:
/// - Origin at (0, 0)
/// - X-axis of (1, 0)
/// - Y-axis of (0, 1)
pub struct LocalSpace2D<T: Real> {
    /// Origin
    pub orig: Point2<T>,
    /// X-axis
    pub x:    Vec2<T>,
    /// Y-axis
    pub y:    Vec2<T>,
}

impl<T: Real> LocalSpace2D<T> {
    /// Construct a local space from given axis.
    /// 
    /// If the axes aren't orhogonal, they will be made orthogonal using x as a base.
    #[must_use]
    pub fn new(orig: Point2<T>, x: Vec2<T>, y: Vec2<T>) -> Self {
        let x = x.normalize();
        let y = y.normalize();

        // Check for parallel axes
        debug_assert!(!x.is_approx_eq(y));

        let y = y.gram_schmidt(x);
        Self { orig, x, y }
    }

    /// Create a local space from an x-axis.
    #[must_use]
    pub fn from_x_axis(orig: Point2<T>, x: Vec2<T>) -> Self {
        let x = x.normalize();
        let y = x.perpendicular_ccw();
        Self{ orig, x, y }
    }

    /// Create a local space from a y-axis.
    #[must_use]
    pub fn from_y_axis(orig: Point2<T>, y: Vec2<T>) -> Self {
        let y = y.normalize();
        let x = y.perpendicular_cw();
        Self{ orig, x, y }
    }

    /// Convert a point in local space to a point in global space.
    #[must_use]
    pub fn point_to_global(&self, point: Point2<T>) -> Point2<T> {
        (self.orig + self.x * point.x + self.y * point.y).into()
    }

    /// Convert a point in global space to a point in local space.
    #[must_use]
    pub fn point_from_global(&self, point: Point2<T>) -> Point2<T> {
        let point = point - self.orig;
        let x = self.x.dot(point);
        let y = self.y.dot(point);
        Point2 { x, y }
    }

    /// Convert a point from an old local space to the current local space.
    #[must_use]
    pub fn convert_point(&self, point: Point2<T>, prev_system: LocalSpace2D<T>) -> Point2<T> {
        let global = prev_system.point_to_global(point);
        self.point_from_global(global)
    }

    /// Convert a point in local space to a point in global space.
    /// 
    /// # Note
    /// 
    /// The vector will not be translated, only rotated and scaled to fit in the current coordinate system.
    #[must_use]
    pub fn vec_to_global(&self, vec: Vec2<T>) -> Vec2<T> {
        self.x * vec.x + self.y * vec.y
    }

    /// Convert a point in global space to a point in local space.
    #[must_use]
    pub fn vec_from_global(&self, vec: Vec2<T>) -> Vec2<T> {
        let x = self.x.dot(vec);
        let y = self.y.dot(vec);
        Vec2 { x, y }
    }

    /// Convert a vector from an old space to the current space.
    /// 
    /// The resulting vector will ***not*** be normalized.
    #[must_use]
    pub fn convert_vector(&self, vec: Vec2<T>, prev_system: LocalSpace2D<T>) -> Vec2<T> {
        let global = prev_system.vec_to_global(vec);
        self.vec_from_global(global)
    }

    /// Get the 3x3 tranform represention.
    #[must_use]
    pub fn transform3(&self) -> Mat3<T> {
        Mat3::new(
            self.x.x   , self.x.y   , T::zero(),
            self.y.x   , self.y.y   , T::zero(),
            self.orig.x, self.orig.y, T::one(),
        )
    }

    /// Get the 4x4 tranform represention.
    #[must_use]
    pub fn transform4(&self) -> Mat4<T> {
        Mat4::new(
            self.x.x   , self.x.y   , T::zero(), T::zero(),
            self.y.x   , self.y.y   , T::zero(), T::zero(),
            T::zero()  , T::zero()  , T::one() , T::zero(),
            self.orig.x, self.orig.y, T::zero(), T::one(),
        )
    }

    /// Get the 4x3 tranform represention.
    #[must_use]
    pub fn transform4x3(&self) -> Mat4x3<T> {
        Mat4x3::new(
            self.x.x   , self.x.y   , T::zero(),
            self.y.x   , self.y.y   , T::zero(),
            T::zero()  , T::zero()  , T::one() ,
            self.orig.x, self.orig.y, T::zero() ,
        )
    }
}


//------------------------------------------------------------------------------------------------------------------------------

/// 3D local space (LH) used to represent a space that is not the global space.
/// 
/// The global space is defined as:
/// - Origin at (0, 0, 0)
/// - X-axis of (1, 0, 0)
/// - Y-axis of (0, 1, 0)
/// - Z-axis of (0, 0, 1)
pub struct LocalSpace3D<T: Real> {
    /// Origin
    pub orig: Point3<T>,
    /// X-axis
    pub x:    Vec3<T>,
    /// Y-axis
    pub y:    Vec3<T>,
    /// Z-axis
    pub z:    Vec3<T>,
}

impl<T: Real> LocalSpace3D<T> {
    /// Construct a local coordinate space from given axie.
    /// 
    /// If the axes aren't orhogonal, they will be made orthogonal using x as a base.
    #[must_use]
    pub fn new(orig: Point3<T>, x: Vec3<T>, y: Vec3<T>, z: Vec3<T>) -> Self {
        debug_assert!(!x.is_zero());
        debug_assert!(!y.is_zero());
        debug_assert!(!z.is_zero());

        // Check for parallel axes
        debug_assert!(!x.is_approx_eq(y));
        debug_assert!(!x.is_approx_eq(z));
        debug_assert!(!y.is_approx_eq(z));

        let x = x.normalize();
        let y = y.gram_schmidt(x);
        let z = z.gram_schmidt(x).gram_schmidt(y);
        Self { orig, x, y, z }
    }

    /// Create a local space from an x and y axes.
    #[must_use]
    pub fn from_x_y_axes(orig: Point3<T>, x: Vec3<T>, z: Vec3<T>) -> Self {
        let x = x.normalize();
        let z = z.normalize();

        // Check for parallel axes
        debug_assert!(!x.is_approx_eq(z));

        let y = x.cross(z);
        Self { orig, x, y, z }
    }

    /// Create a local space from an x and z axes.
    #[must_use]
    pub fn from_x_z_axes(orig: Point3<T>, x: Vec3<T>, z: Vec3<T>) -> Self {
        let x = x.normalize();
        let z = z.normalize();

        // Check for parallel axes
        debug_assert!(!x.is_approx_eq(z));

        let y = z.cross(x);
        Self { orig, x, y, z }
    }

    /// Create a local space from an y and z axes.
    #[must_use]
    pub fn from_y_z_axes(orig: Point3<T>, y: Vec3<T>, z: Vec3<T>) -> Self {
        let y = y.normalize();
        let z = z.normalize();

        // Check for parallel axes
        debug_assert!(!y.is_approx_eq(z));

        let x = y.cross(z);
        Self { orig, x, y, z }
    }

    /// Create a local space from an x-axis.
    /// 
    /// For info on how this coordinate system is defined, see the non-code documentation.
    #[must_use]
    pub fn from_x_axis(orig: Point3<T>, x: Vec3<T>) -> Self {
        let x = x.normalize();
        let (y, z) = Self::calc_2_axes(x);
        Self{ orig, x, y, z }
    }

    /// Create a local space from a y-axis
    /// 
    /// For info on how this coordinate system is defined, see the non-code documentation.
    #[must_use]
    pub fn from_y_axis(orig: Point3<T>, y: Vec3<T>) -> Self {
        let y = y.normalize();
        let (x, z) = Self::calc_2_axes(y);
        Self{ orig, x, y, z }
    }

    /// Create a local space from a z-axis.
    /// 
    /// For info on how this coordinate system is defined, see the non-code documentation.
    #[must_use]
    pub fn from_z_axis(orig: Point3<T>, z: Vec3<T>) -> Self {
        let z = z.normalize();
        let (x, y) = Self::calc_2_axes(z);
        Self{ orig, x, y, z }
    }

    fn calc_2_axes(axis: Vec3<T>) -> (Vec3<T>, Vec3<T>) {
        let sign = axis.z.sign();
        let a = -T::one() / (sign + axis.z);
        let b = axis.x * axis.y * a;
        let axis0 = Vec3 { x: T::one() + sign * axis.x.sqr(), y: sign * b, z: -sign * axis.x };
        let axis1 = Vec3 { x: b, y: sign + axis.y.sqr() * a, z: -axis.y };
        (axis0, axis1)
    }

    /// Convert a point in local space to a point in global space.
    #[must_use]
    pub fn point_to_global(&self, point: Point3<T>) -> Point3<T> {
        (self.orig + self.x * point.x + self.y * point.y).into()
    }

    /// Convert a point in global space to a point in local space.
    #[must_use]
    pub fn point_from_global(&self, point: Point3<T>) -> Point3<T> {
        let point = point - self.orig;
        let x = self.x.dot(point);
        let y = self.y.dot(point);
        let z = self.z.dot(point);
        Point3 { x, y, z }
    }

    /// Convert a point from an old space to the current local space.
    #[must_use]
    pub fn convert_point(&self, point: Point3<T>, prev_system: &LocalSpace3D<T>) -> Point3<T> {
        let global = prev_system.point_to_global(point);
        self.point_from_global(global)
    }

    /// Convert a point in local space to a point in global space.
    /// 
    /// # Note
    /// 
    /// The vector will not be translated, only rotated and scaled to fit in the current space.
    #[must_use]
    pub fn vec_to_global(&self, vec: Vec3<T>) -> Vec3<T> {
        self.x * vec.x + self.y * vec.y
    }

    /// Convert a point in global space to a point in local space.
    #[must_use]
    pub fn vec_from_global(&self, vec: Vec3<T>) -> Vec3<T> {
        let x = self.x.dot(vec);
        let y = self.y.dot(vec);
        let z = self.z.dot(vec);
        Vec3 { x, y, z }
    }

    /// Convert a vector from an old space to the current space.
    /// 
    /// The resulting vector will ***not*** be normalized.
    #[must_use]
    pub fn convert_vector(&self, vec: Vec3<T>, prev_system: LocalSpace3D<T>) -> Vec3<T> {
        let global = prev_system.vec_to_global(vec);
        self.vec_from_global(global)
    }

    /// Get the 4x4 tranform represention.
    #[must_use]
    pub fn transform4(&self) -> Mat4<T> {
        Mat4::new(
            self.x.x   , self.x.y   , self.x.z   , T::zero(),
            self.y.x   , self.y.y   , self.y.z   , T::zero(),
            self.z.x   , self.z.y   , self.z.z   , T::zero(),
            self.orig.x, self.orig.y, self.orig.z, T::one(),
        )
    }

    /// Get the 4x3 tranform represention.
    #[must_use]
    pub fn transform4x3(&self) -> Mat4x3<T> {
        Mat4x3::new(
            self.x.x   , self.x.y   , self.x.z   ,
            self.y.x   , self.y.y   , self.y.z   ,
            self.z.x   , self.z.y   , self.z.z   ,
            self.orig.x, self.orig.y, self.orig.z,
        )
    }
}
