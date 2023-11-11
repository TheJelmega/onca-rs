use std::{
    ops::*,
    fmt::Display
};
use crate::*;

generic_vec!{ doc = "3D Vector (row-major order)"; Vec3, 3, (T, T, T), x => 0, y => 1, z => 2;
    i8v3  => i8
    i16v3 => i16
    i32v3 => i32
    i64v3 => i64
    u8v3  => u8
    u16v3 => u16
    u32v3 => u32
    u64v3 => u64
    f32v3 => f32
    f64v3 => f64
}

impl<T: Numeric> Vec3<T> {
    /// Extend a `Vec3` to a `Vec4`
    #[inline]
    #[must_use]
    pub fn extend(self, w: T) -> Vec4<T> {
        Vec4 { x: self.x, y: self.y, z: self.z, w }
    }

    /// Shrink a `Vec3` to a `Vec2`
    #[inline]
    #[must_use]
    pub fn shrink(self) -> Vec2<T> {
        Vec2 { x: self.x, y: self.y }
    }

    /// Check if the 3d vector represents a 2d vector (z-coord == 0)
    pub fn represents_2d_vector(self) -> bool where
        T: ApproxZero
    {
        self.z.is_zero()
    }

    /// Check if the 3d vector represents a 2d point (z-coord == 1)
    pub fn represents_2d_point(self) -> bool where
        T: One + ApproxEq
    {
        self.z.is_approx_eq(T::one())
    }

    /// Calculate the cross product of 2 vectors
    #[inline]
    pub fn cross(self, rhs: Self) -> Self where
        T: Sub<Output = T> + Mul<Output = T>
    {
        Vec3 { x: self.y * rhs.z - self.z * rhs.y, 
               y: self.z * rhs.x - self.x * rhs.z, 
               z: self.x * rhs.y - self.y * rhs.x }
    }
}

impl<T: Real> Vec3<T> {
    /// Transform the vector by a matrix
    #[inline(always)]
    #[must_use]
    pub fn transform(self, mat: Mat3<T>) -> Self {
        mat.transform(self)
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

    /// Convert the vector into one that is orthogonal to the `base` vector.
    #[must_use]
    pub fn gram_schmidt(self, base: Self) -> Self {
        self - base * self.dot(base)
    }
}

impl<T: Real> Mul<Mat3<T>> for Vec3<T> {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Mat3<T>) -> Self::Output {
        self.transform(rhs)
    }
}

impl<T: Real> MulAssign<Mat3<T>> for Vec3<T> {
    #[inline]
    fn mul_assign(&mut self, rhs: Mat3<T>) {
        *self = self.transform(rhs)
    }
}

impl<T: Numeric + Display> Display for Vec3<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

// Swizzles
impl<T: Numeric> Vec3<T> {
    /// Swizzle the components of the vector into a `Vec2`
    pub fn swizzle2(self, x: Swizzle, y: Swizzle) -> Vec2<T> {
        debug_assert!(x <= Swizzle::Z);
        debug_assert!(y <= Swizzle::Z);
        Vec2 { x: self[x as usize], y: self[y as usize] }
    }

    /// Swizzle the components of the vector
    pub fn swizzle(self, x: Swizzle, y: Swizzle, z: Swizzle) -> Self {
        debug_assert!(x <= Swizzle::Z);
        debug_assert!(y <= Swizzle::Z);
        debug_assert!(z <= Swizzle::Z);
        Self { x: self[x as usize], y: self[y as usize], z: self[z as usize] }
    }

    /// Swizzle the components of the vector into a `Vec4`
    pub fn swizzle4(self, x: Swizzle, y: Swizzle, z: Swizzle, w: Swizzle) -> Vec4<T> {
        debug_assert!(x <= Swizzle::Z);
        debug_assert!(y <= Swizzle::Z);
        debug_assert!(z <= Swizzle::Z);
        Vec4 { x: self[x as usize], y: self[y as usize], z: self[z as usize], w: self[w as usize] }
    }

    create_swizzle!{@2d
        xx => x, x  xy => x, y  xz => x, z
        yx => y, x  yy => y, y  yz => y, z
        zx => z, x  zy => z, y  zz => z, z}

    create_swizzle!{@3d 
        xxx => x, x, x  xxy => x, x, y  xxz => x, x, z
        xyx => x, y, x  xyy => x, y, y  xyz => x, y, z
        xzx => x, z, x  xzy => x, z, y  xzz => x, z, z
        yxx => y, x, x  yxy => y, x, y  yxz => y, x, z
        yyx => y, y, x  yyy => y, y, y  yyz => y, y, z
        yzx => y, z, x  yzy => y, z, y  yzz => y, z, z
        zxx => z, x, x  zxy => z, x, y  zxz => z, x, z
        zyx => z, y, x  zyy => z, y, y  zyz => z, y, z
        zzx => z, z, x  zzy => z, z, y  zzz => z, z, z
    }

}

// Constants
impl<T: Signed> Vec3<T> {
    pub fn left()     -> Self { Self{ x:  T::one() , y:  T::zero(), z: T::zero() } }
    pub fn right()    -> Self { Self{ x: -T::one() , y:  T::zero(), z: T::zero() } }
    pub fn up()       -> Self { Self{ x:  T::zero(), y:  T::one() , z: T::zero() } }
    pub fn down()     -> Self { Self{ x:  T::zero(), y: -T::one() , z: T::zero() } }
    pub fn forward()  -> Self { Self{ x:  T::zero(), y:  T::zero(), z:  T::one()  } }
    pub fn backward() -> Self { Self{ x:  T::zero(), y:  T::zero(), z: -T::one()  } }
}


#[cfg(test)]
mod tests {
    use crate::Vec3;

    #[test]
    fn test_spec_fun() {
        let v0 = Vec3::new(2f32, -3f32, 4f32);
        let v1 = Vec3::new(4f32, 5f32, -6f32);

        // -3*-6 -  4* 5 = 18 -  20 = -2 <- x
        //  4* 4 -  2*-6 = 16 - -12 = 16 <- y
        //  2* 5 - -3* 4 = 10 - -12 = 22 <- z
        assert_eq!(v0.cross(v1), Vec3::new(-2f32, 28f32, 22f32));
    }
    
}