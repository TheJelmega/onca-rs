use std::{
    ops::*,
    fmt::Display
};
use crate::*;

generic_vec!{ doc = "2D Vector (row-major order)"; Vec2, 2, (T, T), x => 0, y => 1;
    i8v2  => i8
    i16v2 => i16
    i32v2 => i32
    i64v2 => i64
    u8v2  => u8
    u16v2 => u16
    u32v2 => u32
    u64v2 => u64
    f32v2 => f32
    f64v2 => f64
}

impl<T: Numeric> Vec2<T> {
    /// Extend a `Vec2` to a `Vec3`
    #[inline]
    #[must_use]
    pub fn extend(self, z: T) -> Vec3<T> {
        Vec3 { x: self.x, y: self.y, z }
    }

    /// Calculate the 1D cross product of 2 vectors
    #[inline]
    pub fn cross(self, rhs: Self) -> T where
        T: Sub<Output = T> + Mul<Output = T>
    {
        self.x * rhs.y - self.y * rhs.x
    }
}

impl<T: Signed> Vec2<T> {
    /// Get a vector that's perpendicular to the vector, rotated clockwise
    #[inline]
    pub fn perpendicular_cw(self) -> Self {
        Vec2 { x: self.y, y: -self.x }
    }

    /// Get a vector that's perpendicular to the vector, rotated counter-clockwise
    #[inline]
    pub fn perpendicular_ccw(self) -> Self {
        Vec2 { x: -self.y, y: self.x }
    }
}

impl<T: Real> Vec2<T> {
    /// Transform the vector by a matrix
    #[inline(always)]
    pub fn transform(self, mat: Mat2<T>) -> Self {
        mat.transform(self)
    }

    //// Rotate the vector by a given angle
    pub fn rotate(self, angle: Radians<T>) -> Self where
        Radians<T>: Trig<Output = T>
    {
        let (sin, cos) = angle.sin_cos();
        Vec2 { x: self.x * cos - self.y * sin, y: self.x * sin + self.y * cos }
    }

    /// Get the angle the vector makes (with the x-axis)
    pub fn angle(self) -> Radians<T> where
        Radians<T>: Zero + InvTrig<T>
    {
        if self.x.is_zero() && self.y.is_zero() {
            Radians::zero()
        } else {
            Radians::atan2(self.y, self.x)
        }
    }

    /// Find the shortest angle with another vector 
    pub fn angle_with(self, other: Self) -> Radians<T> where
        Radians<T>: InvTrig<T>
    {
        Self::angle_with_normalized(self.normalize(), other.normalize())
    }

    /// Find the shortest angle with another vector, where the given vectors are normalized (avoid division by the product of the lengths)
    pub fn angle_with_normalized(self, other: Self) -> Radians<T> where
        Radians<T>: InvTrig<T>
    {
        debug_assert!(self.is_normalized());
        debug_assert!(other.is_normalized());

        // More accurate version from `Physically Based Rendering 4th edition`
        if self.dot(other) > T::zero() {
            Radians::new(T::PI) - Radians::safe_asin(T::from_i32(2) * ((self + other).len() / T::from_i32(2)))
        } else {
            Radians::safe_asin(T::from_i32(2) * ((self + other).len() / T::from_i32(2)))
        }
    }

    /// Find the angle with another vector, respecting the order of the vectors
    pub fn angle_with_full(self, other: Self) -> Radians<T> where
        Radians<T>: InvTrig<T>
    {
        Self::angle_with_full_normalized(self.normalize(), other.normalize())
    }

    /// Find the angle with another vector, respecting the order of the vectors (avoid division by the product of the lengths)
    pub fn angle_with_full_normalized(self, other: Self) -> Radians<T> where
        Radians<T>: InvTrig<T>
    {
        debug_assert!(self.is_normalized());
        debug_assert!(other.is_normalized());
        let angle = if self.dot(other) > T::zero() {
            Radians::new(T::PI) - Radians::asin(T::from_i32(2) * ((self + other).len() / T::from_i32(2)))
        } else {
            Radians::asin(T::from_i32(2) * ((self + other).len() / T::from_i32(2)))
        };
        let cross = self.cross(other);
        if cross < T::zero() { -angle } else { angle }
    }

    /// Convert the vector into one that is orthogonal to the `base` vector.
    #[must_use]
    pub fn gram_schmidt(self, base: Self) -> Self {
        self - base * self.dot(base)
    }

}

impl<T: Real> Mul<Mat2<T>> for Vec2<T> {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Mat2<T>) -> Self::Output {
        self.transform(rhs)
    }
}

impl<T: Real> MulAssign<Mat2<T>> for Vec2<T> {
    #[inline]
    fn mul_assign(&mut self, rhs: Mat2<T>) {
        *self = self.transform(rhs)
    }
}

impl<T: Numeric + Display> Display for Vec2<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

// Swizzles
impl<T: Numeric> Vec2<T> {
    /// Swizzle the components of the vector
    pub fn swizzle(self, x: Swizzle, y: Swizzle) -> Self {
        debug_assert!(x <= Swizzle::Y);
        debug_assert!(y <= Swizzle::Y);
        Self { x: self[x as usize], y: self[y as usize] }
    }

    /// Swizzle the components of the vector into a `Vec3`
    pub fn sizzle3(self, x: Swizzle, y: Swizzle, z: Swizzle) -> Vec3<T> {
        debug_assert!(x <= Swizzle::Y);
        debug_assert!(y <= Swizzle::Y);
        debug_assert!(z <= Swizzle::Y);
        Vec3 { x: self[x as usize], y: self[y as usize], z: self[z as usize] }
    }

    /// Swizzle the components of the vector into a `Vec4`
    pub fn sizzle4(self, x: Swizzle, y: Swizzle, z: Swizzle, w: Swizzle) -> Vec4<T> {
        debug_assert!(x <= Swizzle::Y);
        debug_assert!(y <= Swizzle::Y);
        debug_assert!(z <= Swizzle::Y);
        Vec4 { x: self[x as usize], y: self[y as usize], z: self[z as usize], w: self[w as usize] }
    }

    create_swizzle!{@2d 
        xx => x, x   xy => x, y
        yx => y, x   yy => y, y
    }

    create_swizzle!{@3d
        xxx => x, x, x  xxy => x, x, y
        xyx => x, y, x  xyy => x, y, y

        yxx => y, x, x  yxy => y, x, y
        yyx => y, y, x  yyy => y, y, y
    }

    create_swizzle!{@4d
        xxxx => x, x, x, x  xxxy => x, x, x, y
        xxyx => x, x, y, x  xxyy => x, x, y, y
        xyxx => x, y, x, x  xyxy => x, y, x, y
        xyyx => x, y, y, x  xyyy => x, y, y, y

        yxxx => y, x, x, x  yxxy => y, x, x, y
        yxyx => y, x, y, x  yxyy => y, x, y, y
        yyxx => y, y, x, x  yyxy => y, y, x, y
        yyyx => y, y, y, x  yyyy => y, y, y, y
    }
}

// Constants
impl<T: Signed> Vec2<T> {
    pub fn left()  -> Self { Self{ x:  T::one() , y:  T::zero() } }
    pub fn right() -> Self { Self{ x: -T::one() , y:  T::zero() } }
    pub fn up()    -> Self { Self{ x:  T::zero(), y:  T::one()  } }
    pub fn down()  -> Self { Self{ x:  T::zero(), y: -T::one()  } }
}

#[cfg(test)]
mod tests {
    use crate::Vec2;

    #[test]
    fn test_spec_fun() {
        let v0 = Vec2::new(2f32, -3f32);
        let v1 = Vec2::new(4f32, 5f32);

        assert_eq!(v0.cross(v1), 22f32);

        assert_eq!(v0.perpendicular_cw(), Vec2::new(-3f32, -2f32));
        assert_eq!(v0.perpendicular_ccw(), Vec2::new(3f32, 2f32));
    }
}