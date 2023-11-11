use core::{
    ops::*,
    mem
};
use std::fmt::Display;
use crate::*;

/// Quaternion
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Quat<T: Real> {
    pub w : T,
    pub x : T,
    pub y : T,
    pub z : T
}

impl<T: Real> Quat<T> {
    /// Create a new quaternion
    #[inline]
    #[must_use]
    pub fn new(w: T, x: T, y: T, z: T) -> Self {
        Self { w, x, y, z }
    }

    /// Create a quaternion from an array
    #[inline(always)]
    #[must_use]
    pub fn from_array(arr: [T; 4]) -> Self {
        unsafe { mem::transmute_copy(&arr) }
    }

    /// Interpret a reference to an array as a reference to a quaternion
    #[inline(always)]
    #[must_use]
    pub fn ref_from_array(arr: &[T; 4]) -> &Self {
        unsafe { mem::transmute(arr) }
    }

    /// Interpret a mutable reference to an array as a mutable reference to a quaternion
    #[inline(always)]
    #[must_use]
    pub fn mut_from_array(arr: &mut [T; 4]) -> &mut Self {
        unsafe { mem::transmute(arr) }
    }

    /// Get the content of the quaternion as an array
    #[inline(always)]
    #[must_use]
    pub fn to_array(self) -> [T; 4] {
        unsafe{ mem::transmute_copy(&self) }
    }
    
    /// Interpret a reference to an quaternion as a reference to a array
    #[inline(always)]
    #[must_use]
    pub fn as_array(&self) -> &[T; 4] {
        unsafe{ mem::transmute(self) }
    }
    
    /// Interpret a mutable reference to an quaternion as a mutable reference to a array
    #[inline(always)]
    #[must_use]
    pub fn as_mut_array(&mut self) -> &mut [T; 4] {
        unsafe{ mem::transmute(self) }
    }

    /// Create an identity quaternion
    #[inline]
    #[must_use]
    pub fn identity() -> Self {
        let zero = T::zero();
        Self { w: T::one(), x: zero, y: zero, z: zero }
    }

    /// Create a new quaternion from a `real` and `imaginary` part
    #[inline]
    #[must_use]
    pub fn from_real_and_imaginary(real: T, imaginary: Vec3<T>) -> Self {
        Self { w: real, x: imaginary.x, y: imaginary.y, z: imaginary.z }
    }

    /// Create a quaternion from a euler rotation
    pub fn from_euler(euler: EulerAngles<T>, order: EulerOrder) -> Self where
        Radians<T>: Trig<Output = T>
    {
        let half = T::from_f32(0.5);

        let (sp, cp) = (euler.pitch * half).sin_cos();
        let (sy, cy) = (euler.yaw * half).sin_cos();
        let (sr, cr) = (euler.roll * half).sin_cos();

        match order {
            EulerOrder::XYZ => Quat { 
                w: cp * cy * cr - sp * sy * sr,
                x: sp * cy * cr + cp * sy * sr,
                y: cp * sy * cr - sp * cy * sr,
                z: cp * cy * sr + sp * sy * cr
            },
            EulerOrder::XZY => Quat { 
                w: cp * cy * cr + sp * sy * sr,
                x: sp * cy * cr - cp * sy * sr,
                y: cp * sy * cr - sp * cy * sr,
                z: cp * cy * sr + sp * sy * cr
            },
            EulerOrder::YXZ => Quat { 
                w: cp * cy * cr + sp * sy * sr,
                x: sp * cy * cr + cp * sy * sr,
                y: cp * sy * cr - sp * cy * sr,
                z: cp * cy * sr - sp * sy * cr
            },
            EulerOrder::YZX => Quat { 
                w: cp * cy * cr - sp * sy * sr,
                x: sp * cy * cr + cp * sy * sr,
                y: cp * sy * cr + sp * cy * sr,
                z: cp * cy * sr - sp * sy * cr
            },
            EulerOrder::ZXY => Quat { 
                w: cp * cy * cr - sp * sy * sr,
                x: sp * cy * cr - cp * sy * sr,
                y: cp * sy * cr + sp * cy * sr,
                z: cp * cy * sr + sp * sy * cr
            },
            EulerOrder::ZYX => Quat { 
                w: cp * cy * cr + sp * sy * sr,
                x: sp * cy * cr - cp * sy * sr,
                y: cp * sy * cr + sp * cy * sr,
                z: cp * cy * sr - sp * sy * cr
            },
        }
    }

    /// Create a qauternion from an `axis` to rotate around and an `angle` to rotate
    #[inline]
    #[must_use]
    pub fn from_axis_angle(axis: Vec3<T>, angle: Radians<T>) -> Self where
       Radians<T>: Trig<Output = T>
    {
        let (sin, cos) = (angle / T::from_i32(2)).sin_cos();
        Quat::from_real_and_imaginary(cos, axis * sin).normalize()
    }

    /// Create a quaternion from a 3x3 rotation matrix
    #[must_use]
    pub fn from_matrix(mat: Mat3<T>) -> Self where {
        debug_assert!(mat.column(0).is_normalized());
        debug_assert!(mat.column(1).is_normalized());
        debug_assert!(mat.column(2).is_normalized());

        let trace = mat.trace();
        if trace > T::zero() {
            let s = (T::one() + trace).sqrt() * T::from_i32(2); // 4 * w
            Self { w: s * T::from_f32(0.25), 
                   x: (mat[7] - mat[5]) / s, 
                   y: (mat[2] - mat[6]) / s,
                    z: (mat[3] - mat[1]) / s }
        } else if mat[0] > mat[4] && mat[0] > mat[8] {
            let s = (T::one() + mat[0] - mat[4] - mat[8]).sqrt() * T::from_i32(2);
            Self { w: (mat[7] - mat[5]) / s, 
                  x: s * T::from_f32(0.25), 
                  y: (mat[1] + mat[3]) / s, 
                  z: (mat[2] + mat[6]) / s }
        } else if mat[4] > mat[8] {
            let s = (T::one() + mat[4] - mat[0] - mat[8]).sqrt() * T::from_i32(2);
            Self { w: (mat[2] - mat[6]) / s,
                   x: (mat[1] + mat[3]) / s, 
                   y: s * T::from_f32(0.25), 
                   z: (mat[5] + mat[7]) / s }
        } else {
            let s = (T::one() + mat[8] - mat[0] - mat[4]).sqrt() * T::from_i32(2);
            Self { w: (mat[3] - mat[1]) / s, 
                   x: (mat[2] + mat[6]) / s, 
                   y: (mat[5] + mat[7]) / s, 
                   z: s * T::from_f32(0.25) }
        }
    }

    /// Get the real part of the quaternion
    #[inline]
    #[must_use]
    pub fn real(self) -> T {
        self.w
    }

    /// Get the imaginary part of the quaternion
    #[inline]
    #[must_use]
    pub fn imaginary(self) -> Vec3<T> {
        Vec3 { x: self.x, y: self.y, z: self.z }
    }

    /// Get the square norm of the quaternion
    #[inline]
    #[must_use]
    pub fn norm_sq(self) -> T {
        self.dot(self)
    }
    
    /// Get the norm of the quaternion
    #[inline]
    #[must_use]
    pub fn norm(self) -> T {
        self.norm_sq().sqrt()
    }

    /// Normalize the vector
    #[must_use]
    pub fn normalize(self) -> Self {
        if self.is_zero() {
            self
        } else {
            unsafe{ self.normalize_unsafe() }
        }
    }

    /// Normalize the vector (no check for a length of 0)
    #[must_use]
    pub unsafe fn normalize_unsafe(self) -> Self {
        let scale = self.norm_sq().rsqrt();
        self.scale(scale)
    }

    /// Check if the vector is close to being normalized, using a given epsilon, which defines the max difference `len` can be relative to 1
    #[inline]
    #[must_use]
    pub fn is_close_to_normalized(self, epsilon: T) -> bool {
        self.norm_sq().is_close_to(T::one(), epsilon)
    }

    /// Ckeck if the vector is normalized, using the machine epsilon
    #[inline]
    #[must_use]
    pub fn is_normalized(self) -> bool {
        self.norm_sq().is_approx_eq(T::one())
    }

    /// Calculate the dot product of 2 quaternions
    #[inline]
    #[must_use]
    pub fn dot(self, other: Self) -> T {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }

    /// Calculate the cross product of 2 quaternions
    #[must_use]
    pub fn cross(self, other: Self) -> Self {
        Self::from_real_and_imaginary(T::zero(), self.imaginary().cross(other.imaginary()))
    }

    /// Get the inverse of the quaternion
    #[must_use]
    pub fn inverse(self) -> Self {
        let scale = self.norm_sq().recip();
        Self { w: self.w * scale, x: -self.x * scale, y: -self.y * scale, z: -self.z * scale }
    }

    /// Get the conjugate of the quaternion
    #[inline]
    #[must_use]
    pub fn conjugate(self) -> Self {
        Self { w: self.w, x: -self.x, y: -self.y, z: -self.z }
    }

    /// Interpolate between 2 quaternions according to the given order, with i being clamped between 0 and 1
    #[must_use]
    pub fn lerp_full_path(self, other: Self, i: T) -> Self {
        debug_assert!(self.is_normalized());
        debug_assert!(other.is_normalized());

        let i = i.clamp(T::zero(), T::one());
        Self { w: self.w + (other.w - self.w) * i, 
               x: self.x + (other.x - self.x) * i, 
               y: self.y + (other.y - self.y) * i, 
               z: self.z + (other.z - self.z) * i }
    }

    /// Linear nterpolation between 2 quaternions along the shortest path, with i being clamped between 0 and 1
    #[must_use]
    pub fn lerp(self, other: Self, i: T) -> Self {
        debug_assert!(self.is_normalized());
        debug_assert!(other.is_normalized());

        let cos = self.dot(other);
        if cos < T::zero() { (-self).lerp_full_path(other, i) } else { self.lerp_full_path(other, i) }
    }

    /// Spherical linear interpolation between 2 quaternions according to the given order, with i being clamped between 0 and 1
    #[must_use]
    pub fn slerp_full_path(self, other: Self, i: T) -> Self where
        Radians<T>: Trig<Output = T> + InvTrig<T>
    {
        debug_assert!(self.is_normalized());
        debug_assert!(other.is_normalized());

        let i = i.clamp(T::zero(), T::one());
        let cos = self.dot(other);

        // If angles are the same or the opposite, linearly interpolate in between them
		// Go via the shortest path to try and avoid the case there t = 0.5 results in a zero quaternion
        if cos.abs().is_approx_eq(T::one()) {
            return if cos < T::zero() { (-self).lerp_full_path(other, i) } else { self.lerp_full_path(other, i) }
        }

        let angle = Radians::acos(cos);
        let sin = (T::one() - cos * cos).sqrt();
        let sin_i0 = (angle * i).sin();
        let sin_i1 = (angle * (T::one() - i)).sin();

        let rcp_sin = sin.recip();
        Self { w: (self.w * sin_i0 + other.w * sin_i1) * rcp_sin,
               x: (self.x * sin_i0 + other.x * sin_i1) * rcp_sin,
               y: (self.y * sin_i0 + other.y * sin_i1) * rcp_sin,
               z: (self.z * sin_i0 + other.z * sin_i1) * rcp_sin }
    }

    /// Spherical linear interpolation between 2 quaternions along the shortest path, with i being clamped between 0 and 1
    #[must_use]
    pub fn slerp(self, other: Self, i: T) -> Self {
        debug_assert!(self.is_normalized());
        debug_assert!(other.is_normalized());
        
        let cos = self.dot(other);
        if cos < T::zero() { (-self).lerp_full_path(other, i) } else { self.lerp_full_path(other, i) }
    }

    /// Spherical quadratic interpolation between 2 quaternions according to the given order, with i being clamped between 0 and 1
    /// 
    /// `a` represent the tangent of `self` and `b` represents the tangent of `q1`
    #[must_use]
    pub fn squad_full_path(self, a: Self, q1: Self, b: Self, i: T) -> Self where
        Radians<T>: Trig<Output = T> + InvTrig<T>
    {
        debug_assert!(self.is_normalized());
        debug_assert!(a.is_normalized());
        debug_assert!(q1.is_normalized());
        debug_assert!(b.is_normalized());

        let squad_i = T::from_i32(2) * i * (T::one() - i);
        self.slerp_full_path(q1, i).slerp_full_path(a.slerp_full_path(b, i), squad_i)
    }

    /// Spherical quadratic interpolation between 2 quaternions along the shortest path, with i being clamped between 0 and 1
    /// 
    /// `a` represent the tangent of `self` and `b` represents the tangent of `q1`
    #[must_use]
    pub fn squad(self, a: Self, q1: Self, b: Self, i: T) -> Self where
        Radians<T>: Trig<Output = T> + InvTrig<T>
    {
        debug_assert!(self.is_normalized());
        debug_assert!(a.is_normalized());
        debug_assert!(q1.is_normalized());
        debug_assert!(b.is_normalized());

        let squad_i = T::from_i32(2) * i * (T::one() - i);
        self.slerp(q1, i).slerp_full_path(a.slerp_full_path(b, i), squad_i)
    }

    /// Calculate the log of the quaternion, which results in (0, theta*v), where `|v| == 1`
    #[must_use]
    pub fn log(self) -> Self where
        Radians<T>: Trig<Output = T> + InvTrig<T>
    {
        let theta = Radians::acos(self.w);
        let sin = theta.sin();
        let zero = T::zero();

        if !sin.is_zero() {
            let scale = theta.0 / sin;
            Quat{ w: zero, x: self.x * scale, y: self.y * scale, z: self.z * scale }
        } else {
            Quat{ w: zero, x: zero, y: zero, z: zero }
        }
    }

    /// Calculat the exponential of the quaternion, which for (0, theta*v) is (cos(theta), sin(theta) * w)
    #[must_use]
    pub fn exp(self) -> Self where
        Radians<T>: Trig<Output = T>
    {
        let theta = Radians(self.imaginary().len());
        let normalized_img = self.imaginary() * theta.0.recip();
        let (sin, cos) = theta.sin_cos();
        Quat::from_real_and_imaginary(sin, normalized_img * cos)
    }

    /// Get the angle represented by the quaternion
    #[must_use]
    pub fn angle(self) -> Radians<T> where
        Radians<T>: InvTrig<T>
    {
        Radians::acos(self.w) * T::from_i32(2)
    }

    /// Calculate the angle between 2 quaternions
    #[must_use]
    pub fn angle_between(self, other: Self) -> Radians<T> where
        Radians<T>: InvTrig<T>
    {
        let y = Quat{ w: self.w - other.w, x: self.x - other.x, y: self.y - other.y, z: self.z - other.z }.norm();
        let x = Quat{ w: self.w + other.w, x: self.x + other.x, y: self.y + other.y, z: self.z + other.z }.norm();
        Radians::atan2(y, x) * T::from_i32(2)
    }

    /// Convert a quaternion to euler angles, according to the `order`
    #[must_use]
    pub fn to_euler(self, order: EulerOrder) -> EulerAngles<T> where
        Radians<T>: InvTrig<T>
    {
        let ww = self.w * self.w;
        let xx = self.x * self.x;
        let yy = self.y * self.y;
        let zz = self.z * self.z;
        let test = self.x * self.w + self.y * self.z;

        if test > T::from_f32(0.5) - T::EPSILON {
            return EulerAngles {
                pitch: Radians::new(T::HALF_PI),
                yaw:   Radians::atan2(self.y, self.x) * T::from_i32(2),
                roll:  Radians::zero()
            };
        } else if test < T::from_f32(-0.5) + T::EPSILON {
            return EulerAngles {
                pitch: -Radians(T::HALF_PI),
                yaw: Radians::atan2(self.y, self.x) * T::from_i32(2),
                roll: Radians::zero()
            }
        }

        let two = T::from_i32(2);
        match order {
            EulerOrder::XYZ => EulerAngles {
                pitch: Radians::atan2(-two * (self.y * self.z - self.w * self.x), ww - xx - yy + zz),
                yaw:   Radians::asin(two * (self.x * self.z + self.w * self.y)),
                roll:  Radians::atan2(-two * (self.x * self.y - self.w * self.z), ww + xx - yy - zz)
            },
            EulerOrder::XZY => EulerAngles { 
                pitch: Radians::atan2(two * (self.y * self.z + self.w * self.x), ww - xx + yy - zz),
                yaw:   Radians::atan2(two * (self.x * self.z + self.w * self.y), ww + xx - yy - zz),
                roll:  Radians::asin(-two * (self.x * self.y - self.w * self.z))
            },
            EulerOrder::YXZ => EulerAngles { 
                pitch: Radians::asin(-two * (self.y * self.z - self.w * self.x)),
                yaw:   Radians::atan2(two * (self.x * self.z + self.w * self.y), ww - xx - yy + zz),
                roll:  Radians::atan2(two * (self.x * self.y + self.w * self.z), ww - xx + yy - zz)
            },
            EulerOrder::YZX => EulerAngles { 
                pitch: Radians::atan2(-two * (self.y * self.z - self.w * self.x), ww - xx + yy - zz),
                yaw:   Radians::atan2(-two * (self.x * self.z - self.w * self.y), ww + xx - yy - zz),
                roll:  Radians::asin(two * (self.x * self.y + self.w * self.z))
            },
            EulerOrder::ZXY => EulerAngles { 
                pitch: Radians::asin(two * (self.y * self.z + self.w * self.x)), 
                yaw:   Radians::atan2(-two * (self.x * self.z - self.w * self.y), ww - xx - yy + zz),
                roll:  Radians::atan2(-two * (self.x * self.y - self.w * self.z), ww - xx + yy - zz)
            },
            EulerOrder::ZYX => EulerAngles { 
                pitch: Radians::atan2(two * (self.y * self.z + self.w * self.x), ww - xx - yy + zz),
                yaw:   Radians::asin(-two * (self.x * self.z - self.w * self.y)),
                roll:  Radians::atan2(two * (self.x * self.y + self.w * self.z), ww + xx - yy - zz)
            },
        }
    }

    /// Convert a quaternion to an axis to rotate around and an angle to rotate
    #[must_use]
    pub fn to_axis_angle(self) -> (Vec3<T>, Radians<T>) where
        Radians<T>: InvTrig<T>
    {
        (self.imaginary().normalize(), Radians::acos(self.w) * T::from_i32(2))
    }

    /// Convert a quaternion into a swing from an axis and a twist around the axis
    #[must_use]
    pub fn to_swing_twist(self, axis: Vec3<T>) -> (Self, Self) {
        let imaginary = self.imaginary();
        let projection = axis * axis.dot(imaginary);

        let twist = Quat::from_real_and_imaginary(self.w, projection);
        let twist = if twist.norm_sq().is_zero() {
                Quat::identity()
            } else {
                twist.normalize()    
            };
        
        let swing = self * twist.inverse();
        (swing, twist)
    }

    /// Calculate the inner quadrangle points to be used with `squad` (the `a` and `b` parameter)
    /// 
    /// `self` (N) and `other` (N+1) quaternions are the quaternions the squad is going in between,
    /// `prev` is the quaternion at location N-1, and next is the quaternion at location N+2
    #[must_use]
    pub fn calculate_inner_quadrangle(self, prev: Self, other: Self, next: Self) -> (Self, Self) where
        Radians<T>: Trig<Output = T> + InvTrig<T>
    {
        let q0 = if prev.add(self).norm_sq() < prev.sub(self).norm_sq() { -prev } else { prev };
        let q2 = if self.add(other).norm_sq() < self.sub(other).norm_sq() { -other } else { other };
        let q3 = if self.add(next).norm_sq() < self.sub(next).norm_sq() { -next } else { next };

        let q1_exp = self.inverse();
        let q1q2 = (q1_exp * q2).log();
        let q1q0 = (q1_exp * q0).log();
        let a = self * (q1q2.add(q1q0).scale(T::from_f32(-0.25))).exp();

        let q2_exp = q2.inverse();
        let q2q3 = (q2_exp * q3).log();
        let q2q1 = (q2_exp * self).log();
        let b = self * (q2q3.add(q2q1).scale(T::from_f32(-0.25))).exp();

        (a, b)
    }



    // We don't want add to be public, cause it makes no real sense, so we have our own
    fn add(self, other: Self) -> Self {
        Quat { w: self.w + other.w, x: self.x + other.x, y: self.y + other.y, z: self.z + other.z }
    }

    fn sub(self, other: Self) -> Self {
        Quat { w: self.w - other.w, x: self.x - other.x, y: self.y - other.y, z: self.z - other.z }
    }

    fn scale(self, scale: T) -> Self {
        Quat { w: self.w * scale, x: self.x * scale, y: self.y * scale, z: self.z * scale }
    }
}

impl<T: Real> Index<usize> for Quat<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.as_array()[index]
    }
}

impl<T: Real> IndexMut<usize> for Quat<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.as_mut_array()[index]
    }
}

impl<T: Real> Neg for Quat<T> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self { w: -self.w, x: -self.x, y: -self.y, z: -self.z }
    }
}

impl<T: Real> Mul for Quat<T> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self { w: self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z, 
               x: self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y, 
               y: self.w * rhs.y + self.y * rhs.w + self.z * rhs.x - self.x * rhs.z, 
               z: self.w * rhs.z + self.z * rhs.w + self.x * rhs.y - self.y * rhs.x}
    }
}

impl<T: Real> MulAssign for Quat<T> {
    fn mul_assign(&mut self, rhs: Self) {
        let w = self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z;
        let x = self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y;
        let y = self.w * rhs.y + self.y * rhs.w + self.z * rhs.x - self.x * rhs.z;
        let z = self.w * rhs.z + self.z * rhs.w + self.x * rhs.y - self.y * rhs.x;
        self.w = w;
        self.x = x;
        self.y = y;
        self.z = z;
    }
}

impl<T: Real> ApproxEq<T> for Quat<T> {
    const EPSILON: T = T::EPSILON;

    fn is_close_to(self, rhs: Self, epsilon: T) -> bool {
        self.w.is_close_to(rhs.w, epsilon) ||
        self.x.is_close_to(rhs.x, epsilon) ||
        self.y.is_close_to(rhs.y, epsilon) ||
        self.z.is_close_to(rhs.z, epsilon)
    }
}

impl<T: Real> ApproxZero<T> for Quat<T> {
    fn is_close_to_zero(self, epsilon: T) -> bool {
        self.w.is_close_to_zero(epsilon) ||
        self.x.is_close_to_zero(epsilon) ||
        self.y.is_close_to_zero(epsilon) ||
        self.z.is_close_to_zero(epsilon)
    }
}

impl <T: Real + Display> Display for Quat<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("( {}, ( {}, {}, {} ))", self.w, self.x, self.y, self.z))
    }
}

#[allow(non_camel_case_types)] type f32quat = Quat<f32>;
#[allow(non_camel_case_types)] type f64quat = Quat<f64>;