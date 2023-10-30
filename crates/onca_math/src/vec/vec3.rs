use std::{
    ops::*,
    fmt::Display
};
use crate::*;


impl<T: Copy> Vec3<T> {
    /// Extend a `Vec3` to a `Vec4`
    #[inline]
    #[must_use]
    pub fn extend(self, w: T) -> Vec4<T> {
        Vec4 { x: self.x, y: self.y, z: self.z, w }
    }

    /// Shrink a `Vec4` to a `Vec3`
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
    pub fn transform(self, mat: Mat3<T>) -> Self {
        mat.transform(self)
    }

    /// Get or flip the vector, so it's pointing in the opposite direction of the incidence vector, relative to the normal
    pub fn face_forward(self, incidence: Self, normal: Self) -> Self {
        if incidence.dot(normal) < T::zero() { self } else { -self }
    }

    /// Reflect a vector on a 'surface' with a normal
    pub fn reflect(self, normal: Self) -> Self where
        i32: NumericCast<T>
    {
        debug_assert!(normal.is_normalized());
        self - normal * self.dot(normal) * 2.cast()
    }

    /// Refract the vector through a `surface` with a given `normal` and `eta` (ratio of indices of refraction at the surface interface (outgoing / ingoing))
    pub fn refract(self, normal: Self, eta: T) -> Self {
        debug_assert!(normal.is_normalized());

        let cosi = self.dot(normal);
        debug_assert!(cosi < T::zero(), "vector should move into hte surface (`self` and `normal` should point in opposite directions)");

        let k = T::one() - eta * eta * (T::one() - cosi * cosi);
        if k >= T::zero() { self * eta - normal * (eta * cosi + k.sqrt()) } else { Vec3::zero() }
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
        f.write_fmt(format_args!("({}, {}, {})", self.x, self.y, self.z))
    }
}

// Swizzles
impl<T: Numeric> Vec3<T> {
    /// Swizzle the components of the vector into a `Vec2`
    pub fn swizzle2(self, x: u8, y: u8) -> Vec2<T> {
        debug_assert!(x < 3);
        debug_assert!(y < 3);
        Vec2 { x: self[x as usize], y: self[y as usize] }
    }

    /// Swizzle the components of the vector
    pub fn swizzle(self, x: u8, y: u8, z: u8) -> Self {
        debug_assert!(x < 3);
        debug_assert!(y < 3);
        debug_assert!(z < 3);
        Self { x: self[x as usize], y: self[y as usize], z: self[z as usize] }
    }

    create_swizzle!{@2d xx, x, x}
    create_swizzle!{@2d xy, x, y}
    create_swizzle!{@2d xz, x, z}
    create_swizzle!{@2d yx, y, x}
    create_swizzle!{@2d yy, y, y}
    create_swizzle!{@2d yz, y, z}
    create_swizzle!{@2d zx, z, x}
    create_swizzle!{@2d zy, z, y}
    create_swizzle!{@2d zz, z, z}

    create_swizzle!{@3d xxx, x, x, x}
    create_swizzle!{@3d xxy, x, x, y}
    create_swizzle!{@3d xxz, x, x, z}
    create_swizzle!{@3d xyx, x, y, x}
    create_swizzle!{@3d xyy, x, y, y}
    create_swizzle!{@3d xyz, x, y, z}
    create_swizzle!{@3d xzx, x, z, x}
    create_swizzle!{@3d xzy, x, z, y}
    create_swizzle!{@3d xzz, x, z, z}
    create_swizzle!{@3d yxx, y, x, x}
    create_swizzle!{@3d yxy, y, x, y}
    create_swizzle!{@3d yxz, y, x, z}
    create_swizzle!{@3d yyx, y, y, x}
    create_swizzle!{@3d yyy, y, y, y}
    create_swizzle!{@3d yyz, y, y, z}
    create_swizzle!{@3d yzx, y, z, x}
    create_swizzle!{@3d yzy, y, z, y}
    create_swizzle!{@3d yzz, y, z, z}
    create_swizzle!{@3d zxx, z, x, x}
    create_swizzle!{@3d zxy, z, x, y}
    create_swizzle!{@3d zxz, z, x, z}
    create_swizzle!{@3d zyx, z, y, x}
    create_swizzle!{@3d zyy, z, y, y}
    create_swizzle!{@3d zyz, z, y, z}
    create_swizzle!{@3d zzx, z, z, x}
    create_swizzle!{@3d zzy, z, z, y}
    create_swizzle!{@3d zzz, z, z, z}

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

#[allow(non_camel_case_types)] pub type i8v3  = Vec3<i8>;
#[allow(non_camel_case_types)] pub type i16v3 = Vec3<i16>;
#[allow(non_camel_case_types)] pub type i32v3 = Vec3<i32>;
#[allow(non_camel_case_types)] pub type i64v3 = Vec3<i64>;
#[allow(non_camel_case_types)] pub type u8v3  = Vec3<u8>;
#[allow(non_camel_case_types)] pub type u16v3 = Vec3<u16>;
#[allow(non_camel_case_types)] pub type u32v3 = Vec3<u32>;
#[allow(non_camel_case_types)] pub type u64v3 = Vec3<u64>;
#[allow(non_camel_case_types)] pub type f32v3 = Vec3<f32>;
#[allow(non_camel_case_types)] pub type f64v3 = Vec3<f64>;


#[cfg(test)]
mod tests {
    use crate::{Vec3, numeric::*};

    macro_rules! op_test {
        (@vec $arr0:expr, $arr1:expr, $op:tt) => {
            let a : Vec3<_> = $arr0.into();
            let b : Vec3<_> = $arr0.into();
            let res = a $op b;

            let expected_x = a.x $op b.x;
            assert_eq!(res.x, expected_x, "vec: got x-coord of {}, expected {}", res.x, expected_x);
            let expected_y = a.y $op b.y;
            assert_eq!(res.y, expected_y, "vec: got y-coord of {}, expected {}", res.y, expected_y);
            let expected_z = a.z $op b.z;
            assert_eq!(res.z, expected_z, "vec: got z-coord of {}, expected {}", res.z, expected_z);
        };
        (@vec_assign $arr0:expr, $arr1:expr, $op:tt) => {
            let a : Vec3<_> = $arr0.into();
            let b : Vec3<_> = $arr0.into();
            let mut res = a;
            res $op b;

            let mut expected_x = a.x;
            expected_x $op b.x;
            assert_eq!(res.x, expected_x, "vec assign: got x-coord of {}, expected {}", res.x, expected_x);
            let mut expected_y = a.y;
            expected_y $op b.y;
            assert_eq!(res.y, expected_y, "vec assign: got y-coord of {}, expected {}", res.y, expected_y);
            let mut expected_y = a.z;
            expected_y $op b.z;
            assert_eq!(res.z, expected_z, "vec assign: got z-coord of {}, expected {}", res.z, expected_z);
        };
        (@scalar $arr:expr, $scalar:expr, $op:tt) => {
            let a : Vec3<_> = $arr.into();
            let res = a $op $scalar;

            let expected_x = a.x $op $scalar;
            assert_eq!(res.x, expected_x, "scalar: got x-coord of {}, expected {}", res.x, expected_x);
            let expected_y = a.y $op $scalar;
            assert_eq!(res.y, expected_y, "scalar:got y-coord of {}, expected {}", res.y, expected_y);
            let expected_z = a.z $op $scalar;
            assert_eq!(res.z, expected_z, "scalar:got z-coord of {}, expected {}", res.z, expected_z);
        };
        (@scalar_assign $arr:expr, $scalar:expr, $op:tt) => {
            let a : Vec3<_> = $arr.into();
            let mut res = a;
            res $op $scalar;

            let mut expected_x = a.x;
            expected_x $op $scalar;
            assert_eq!(res.x, expected_x, "scalar assign:got x-coord of {}, expected {}", res.x, expected_x);
            let mut expected_y = a.y;
            expected_y $op $scalar;
            assert_eq!(res.y, expected_y, "scalar assign:got y-coord of {}, expected {}", res.y, expected_y);
            let mut expected_z = a.z;
            expected_y $op $scalar;
            assert_eq!(res.z, expected_z, "scalar assign:got z-coord of {}, expected {}", res.z, expected_z);
        };
        ($arr0:expr, $arr1:expr, $scalar:expr, $op:tt, $assign_op:tt) => {

        }
    }

    #[test]
    fn test_create_convert() {
        let vec = Vec3{ x: 1, y: 2, z: 3 };
        assert_eq!(vec.x, 1);
        assert_eq!(vec.y, 2);
        assert_eq!(vec.z, 3);

        let vec = Vec3::new(1, 2, 3);
        assert_eq!(vec.x, 1);
        assert_eq!(vec.y, 2);
        assert_eq!(vec.z, 3);

        let vec = Vec3::set(1);
        assert_eq!(vec.x, 1);
        assert_eq!(vec.y, 1);
        assert_eq!(vec.z, 1);

        let vec = Vec3::from_array([1, 2, 3]);
        assert_eq!(vec.x, 1);
        assert_eq!(vec.y, 2);
        assert_eq!(vec.z, 3);

        let mut arr = [1, 2, 3];
        let vec = Vec3::ref_from_array(&arr);
        assert_eq!(vec.x, 1);
        assert_eq!(vec.y, 2);
        assert_eq!(vec.z, 3);

        let vec : Vec3<_> = arr.into();
        assert_eq!(vec.x, 1);
        assert_eq!(vec.y, 2);
        assert_eq!(vec.z, 3);

        let vec = Vec3::mut_from_array(&mut arr);
        assert_eq!(vec.x, 1);
        assert_eq!(vec.y, 2);
        assert_eq!(vec.z, 3);

        let mut vec = Vec3::new(1, 2, 3);
        let arr = vec.to_array();
        assert_eq!(arr[0], 1);
        assert_eq!(arr[1], 2);
        assert_eq!(arr[2], 3);

        let arr : [_; 3] = vec.into();
        assert_eq!(arr[0], 1);
        assert_eq!(arr[1], 2);
        assert_eq!(arr[2], 3);

        let arr = (&vec).as_array();
        assert_eq!(arr[0], 1);
        assert_eq!(arr[1], 2);
        assert_eq!(arr[2], 3);

        let arr = (&mut vec).as_mut_array();
        assert_eq!(arr[0], 1);
        assert_eq!(arr[1], 2);
        assert_eq!(arr[2], 3);
    }

    #[test]
    fn test_ops() {
        op_test!([1, 2, 3], [3, 4, 5], 2, + ,  +=);
        op_test!([1, 2, 3], [3, 4, 5], 2, - ,  -=);
        op_test!([1, 2, 3], [3, 4, 5], 2, * ,  *=);
        op_test!([1, 2, 3], [3, 4, 5], 2, / ,  /=);
        op_test!([1, 2, 3], [3, 4, 5], 2, % ,  %=);
        op_test!([1, 2, 3], [3, 4, 5], 2, & ,  &=);
        op_test!([1, 2, 3], [3, 4, 5], 2, ^ ,  ^=);
        op_test!([1, 2, 3], [3, 4, 5], 2, | ,  |=);
        op_test!([1, 2, 3], [3, 4, 5], 2, <<, <<=);
        op_test!([1, 2, 3], [3, 4, 5], 2, >>, >>=);

        let a = Vec3::new(1, 2, 3);
        let res = -a;
        assert_eq!(res.x, -1);
        assert_eq!(res.y, -2);
        assert_eq!(res.z, -3);

        let res = !a;
        assert_eq!(res.x, !1);
        assert_eq!(res.y, !2);
        assert_eq!(res.z, !3);
    }

    #[test]
    fn test_cmp() {
        let a = Vec3::new(1, 2, 3);
        let b = Vec3::new(2, 3, 4);

        assert!(a == a);
        assert!(a != b);

        // ApproxEq
        assert!(a.is_close_to(a, 0));
        assert!(!a.is_close_to(b, 0));
        assert!(a.is_close_to(b, 1));

        assert!(a.is_approx_eq(a));
        assert!(!a.is_approx_eq(b));

        // ApproxZero
        assert!(!a.is_close_to_zero(0));
        assert!(a.is_close_to_zero(2));
        assert!(!a.is_zero());
    }

    #[test]
    fn test_common_funcs() {
        let a = Vec3::new(2, -3, 4);
        let b = Vec3::new(1, 4, -7);

        assert_eq!(a.min(b), Vec3::new(1, -3, -7));
        assert_eq!(a.max(b), Vec3::new(2, 4, 4));

        assert_eq!(b.clamp_scalar(1, 2), Vec3::new(1, 2, 1));
        assert_eq!(b.clamp_scalar(3, 4), Vec3::new(3, 4, 3));

        let min = Vec3::new(0, 2, -3);
        let max = Vec3::new(1, 5, 2);
        assert_eq!(a.clamp(min, max), Vec3::new(1, 2, 2));
        assert_eq!(b.clamp(min, max), Vec3::new(1, 4, -3));


        assert_eq!(a.snap(4), Vec3::new(4, -4, 4));
        assert_eq!(b.snap(3), Vec3::new(0, 3, -6));

        assert_eq!(Vec3::new(1.2f32, 1.6f32, 2.5f32).snap(1f32), Vec3::new(1f32, 2f32, 3f32));

        assert_eq!(Vec3::new(-0.2f32, 0.4f32, 1.5f32).saturate(), Vec3::new(0f32, 0.4f32, 1f32));

        let v0 = Vec3::new(2f32, 3f32, 6f32); // len == 7
        let v1 = Vec3::new(1f32, 4f32, 8f32); // len == 9
        let v2 = v0 / 7f32;
        let v3 = Vec3::new(2.2f32, 2.8f32, 5.7f32);

        assert_eq!(v0.lerp(v1, 0.25f32), Vec3::new(1.75f32, 3.25f32, 6.5f32));

        assert_eq!(v0.len_sq(), 49f32);
        assert_eq!(v0.len(), 7f32);

        assert_eq!(v0.dist_sq(v1), 6f32);
        assert_eq!(v0.dist(v1), 6f32.sqrt());

        assert!(v0.normalize().is_close_to(v2, 0.000001f32));
        assert_eq!(Vec3::set(0f32).normalize(), Vec3::set(0f32));

        assert!(v0.normalize_or(v1).is_close_to(v2, 0.000001f32));
        assert_eq!(Vec3::set(0f32).normalize_or(v1), v1);

        assert!(!v0.is_close_to_normalized(0f32));
        assert!(v0.is_close_to_normalized(49f32));
        assert!(v2.is_close_to_normalized(0f32));

        assert!(!v0.is_normalized());
        assert!(v2.is_normalized());

        let (dir, len) = v0.dir_and_len();
        assert!(dir.normalize().is_close_to(v2, 0.000001f32));
        assert_eq!(len, 7f32);

        assert!(v0.clamp_len(0f32, 5f32).is_close_to(v2 * 5f32, 0.000001f32));
        assert!(v0.clamp_len(16f32, 20f32).is_close_to(v2 * 16f32, 0.000001f32));

        assert_eq!(Vec3::new(-3f32, -4f32, 1f32).abs(), Vec3::new(3f32, 4f32, 1f32));
        assert_eq!(v3.ceil(), Vec3::new(3f32, 3f32, 6f32));
        assert_eq!(v3.floor(), Vec3::new(2f32, 2f32, 5f32));
        assert_eq!(v3.round(), v0);
        assert_eq!(Vec3::new(-4f32, 5f32, 0f32).sign(), Vec3::new(-1f32, 1f32, 0f32));
        assert!(v3.fract().is_close_to(Vec3::new(0.2f32, 0.8f32, 0.7f32), 0.0000001f32));

        //// Common per vec funcs
        let v0 = Vec3::new(2f32, -3f32, 4f32);

        assert!(!v0.is_uniform(0.1f32));
        assert!(v0.is_uniform(7f32));

        assert_eq!(v0.min_component(), -3f32);
        assert_eq!(v0.min_abs_component(), 2f32);
        assert_eq!(v0.max_component(), 4f32);
        assert_eq!(v0.max_abs_component(), 4f32);
    }

    #[test]
    fn test_spec_fun() {
        let v0 = Vec3::new(2f32, -3f32, 4f32);
        let v1 = Vec3::new(4f32, 5f32, -6f32);

        assert_eq!(v0.dot(v1), -31f32);

        // -3*-6 -  4* 5 = 18 -  20 = -2 <- x
        //  4* 4 -  2*-6 = 16 - -12 = 16 <- y
        //  2* 5 - -3* 4 = 10 - -12 = 22 <- z
        assert_eq!(v0.cross(v1), Vec3::new(-2f32, 28f32, 22f32));
    }
    
}