use std::{ops::{Mul, MulAssign}, fmt::Display};
use crate::*;

impl<T: Numeric> Vec2<T> {
    /// Extend a `Vec2` to a `Vec3`
    #[inline]
    #[must_use]
    pub fn extend(self, z: T) -> Vec3<T> {
        Vec3 { x: self.x, y: self.y, z }
    }

    /// Check if all elements are approximately equal, given an epsilon
    pub fn is_uniform(self, epsilon: T) -> bool {
        self.x.abs_diff(self.y) <= epsilon
    }

    /// Get the minimum component of the vector
    pub fn min_component(self) -> T {
        self.x.min(self.y)
    }

    /// Get the minimum absolute component of the vector
    pub fn min_abs_component(self) -> T {
        self.x.abs().min(self.y.abs())
    }

    /// Get the maximum component of the vector
    pub fn max_component(self) -> T {
        self.x.max(self.y)
    }

    /// Get the maximum absolute component of the vector
    pub fn max_abs_component(self) -> T {
        self.x.abs().max(self.y.abs())
    }

    /// Calculate the 1D cross product of 2 vectors
    #[inline]
    pub fn cross(self, rhs: Self) -> T {
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
    pub fn rotate(self, angle: Radians<T>) -> Self {
        let (sin, cos) = angle.sin_cos();
        Vec2 { x: self.x * cos - self.y * sin, y: self.x * sin + self.y * cos }
    }

    /// Get the angle the vector makes (with the x-axis)
    pub fn angle(self) -> T {
        if self.x.is_zero() && self.y.is_zero() {
            T::zero()
        } else {
            T::atan2(self.y, self.x)
        }
    }

    /// Find the shortest angle with another vector 
    pub fn angle_with(self, other: Self) -> T {
        let acos = self.dot(other) / (self.len() * other.len());
        T::acos(acos)
    }

    /// Find the shortest angle with another vector, where the given vectors are normalized (avoid division by the product of the lengths)
    pub fn angle_with_normalized(self, other: Self) -> Radians<T> {
        debug_assert!(self.is_normalized());
        debug_assert!(other.is_normalized());
        let acos = self.dot(other);
        Radians::acos(acos)
    }

    /// Find the angle with another vector, respecting the order of the vectors
    pub fn angle_with_full(self, other: Self) -> Radians<T> {
        let acos = self.dot(other) / (self.len() * other.len());
        let angle = Radians::acos(acos);
        let cross = self.cross(other);
        if cross < T::zero() { -angle } else { angle }
    }

    /// Find the angle with another vector, respecting the order of the vectors (avoid division by the product of the lengths)
    pub fn angle_with_full_normalized(self, other: Self) -> Radians<T> {
        debug_assert!(self.is_normalized());
        debug_assert!(other.is_normalized());
        let acos = self.dot(other);
        let angle = Radians::acos(acos);
        let cross = self.cross(other);
        if cross < T::zero() { -angle } else { angle }
    }

    /// Get or flip the vector, so it's pointing in the opposite direction of the incidence vector, relative to the normal
    pub fn face_forward(self, incidence: Self, normal: Self) -> Self {
        if incidence.dot(normal) < T::zero() { self } else { -self }
    }

    /// Reflect a vector on a 'surface' with a normal
    pub fn reflect(self, normal: Self) -> Self {
        debug_assert!(normal.is_normalized());
        self - normal * self.dot(normal) * T::from_f32(2f32)
    }

    /// Refract the vector through a `surface` with a given `normal` and `eta` (ratio of indices of refraction at the surface interface (outgoing / ingoing))
    pub fn refract(self, normal: Self, eta: T) -> Self {
        debug_assert!(normal.is_normalized());

        let cosi = self.dot(normal);
        debug_assert!(cosi < T::zero(), "vector should move into hte surface (`self` and `normal` should point in opposite directions)");

        let k = T::one() - eta * eta * (T::one() - cosi * cosi);
        if k >= T::zero() { self * eta - normal * (eta * cosi + k.sqrt()) } else { Vec2::zero() }
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
        f.write_fmt(format_args!("({}, {})", self.x, self.y))
    }
}

// Swizzles
impl<T: Numeric> Vec2<T> {
    /// Swizzle the components of the vector
    pub fn swizzle(self, x: u8, y: u8) -> Self {
        debug_assert!(x < 2);
        debug_assert!(y < 2);
        Self { x: self[x as usize], y: self[y as usize] }
    }

    create_swizzle!{@2d xx, x, x}
    create_swizzle!{@2d xy, x, y}
    create_swizzle!{@2d yx, y, x}
    create_swizzle!{@2d yy, y, y}
}

// Constants
impl<T: Signed> Vec2<T> {
    fn left()  -> Self { Self{ x:  T::one() , y:  T::zero() } }
    fn right() -> Self { Self{ x: -T::one() , y:  T::zero() } }
    fn up()    -> Self { Self{ x:  T::zero(), y:  T::one()  } }
    fn down()  -> Self { Self{ x:  T::zero(), y: -T::one()  } }
}

#[allow(non_camel_case_types)] pub type i8v2  = Vec2<i8>;
#[allow(non_camel_case_types)] pub type i16v2 = Vec2<i16>;
#[allow(non_camel_case_types)] pub type i32v2 = Vec2<i32>;
#[allow(non_camel_case_types)] pub type i64v2 = Vec2<i64>;
#[allow(non_camel_case_types)] pub type u8v2  = Vec2<u8>;
#[allow(non_camel_case_types)] pub type u16v2 = Vec2<u16>;
#[allow(non_camel_case_types)] pub type u32v2 = Vec2<u32>;
#[allow(non_camel_case_types)] pub type u64v2 = Vec2<u64>;
#[allow(non_camel_case_types)] pub type f32v2 = Vec2<f32>;
#[allow(non_camel_case_types)] pub type f64v2 = Vec2<f64>;

#[cfg(test)]
mod tests {
    use crate::{Vec2, numeric::*};

    macro_rules! op_test {
        (@vec $arr0:expr, $arr1:expr, $op:tt) => {
            let a : Vec2<_> = $arr0.into();
            let b : Vec2<_> = $arr0.into();
            let res = a $op b;

            let expected_x = a.x $op b.x;
            assert_eq!(res.x, expected_x, "vec: got x-coord of {}, expected {}", res.x, expected_x);
            let expected_y = a.y $op b.y;
            assert_eq!(res.y, expected_y, "vec: got y-coord of {}, expected {}", res.y, expected_y);
        };
        (@vec_assign $arr0:expr, $arr1:expr, $op:tt) => {
            let a : Vec2<_> = $arr0.into();
            let b : Vec2<_> = $arr0.into();
            let mut res = a;
            res $op b;

            let mut expected_x = a.x;
            expected_x $op b.x;
            assert_eq!(res.x, expected_x, "vec assign: got x-coord of {}, expected {}", res.x, expected_x);
            let mut expected_y = a.y;
            expected_y $op b.y;
            assert_eq!(res.y, expected_y, "vec assign: got y-coord of {}, expected {}", res.y, expected_y);
        };
        (@scalar $arr:expr, $scalar:expr, $op:tt) => {
            let a : Vec2<_> = $arr.into();
            let res = a $op $scalar;

            let expected_x = a.x $op $scalar;
            assert_eq!(res.x, expected_x, "scalar: got x-coord of {}, expected {}", res.x, expected_x);
            let expected_y = a.y $op $scalar;
            assert_eq!(res.y, expected_y, "scalar:got y-coord of {}, expected {}", res.y, expected_y);
        };
        (@scalar_assign $arr:expr, $scalar:expr, $op:tt) => {
            let a : Vec2<_> = $arr.into();
            let mut res = a;
            res $op $scalar;

            let mut expected_x = a.x;
            expected_x $op $scalar;
            assert_eq!(res.x, expected_x, "scalar assign:got x-coord of {}, expected {}", res.x, expected_x);
            let mut expected_y = a.y;
            expected_y $op $scalar;
            assert_eq!(res.y, expected_y, "scalar assign:got y-coord of {}, expected {}", res.y, expected_y);
        };
        ($arr0:expr, $arr1:expr, $scalar:expr, $op:tt, $assign_op:tt) => {

        }
    }

    #[test]
    fn test_create_convert() {
        let vec = Vec2{ x: 1, y: 2 };
        assert_eq!(vec.x, 1);
        assert_eq!(vec.y, 2);

        let vec = Vec2::new(1, 2);
        assert_eq!(vec.x, 1);
        assert_eq!(vec.y, 2);

        let vec = Vec2::set(1);
        assert_eq!(vec.x, 1);
        assert_eq!(vec.y, 1);

        let vec = Vec2::from_array([1, 2]);
        assert_eq!(vec.x, 1);
        assert_eq!(vec.y, 2);

        let mut arr = [1, 2];
        let vec = Vec2::ref_from_array(&arr);
        assert_eq!(vec.x, 1);
        assert_eq!(vec.y, 2);

        let vec : Vec2<_> = arr.into();
        assert_eq!(vec.x, 1);
        assert_eq!(vec.y, 2);

        let vec = Vec2::mut_from_array(&mut arr);
        assert_eq!(vec.x, 1);
        assert_eq!(vec.y, 2);

        let mut vec = Vec2::new(1, 2);
        let arr = vec.to_array();
        assert_eq!(arr[0], 1);
        assert_eq!(arr[1], 2);

        let arr : [_; 2] = vec.into();
        assert_eq!(arr[0], 1);
        assert_eq!(arr[1], 2);

        let arr = (&vec).as_array();
        assert_eq!(arr[0], 1);
        assert_eq!(arr[1], 2);

        let arr = (&mut vec).as_mut_array();
        assert_eq!(arr[0], 1);
        assert_eq!(arr[1], 2);
    }

    #[test]
    fn test_ops() {
        op_test!([1, 2], [3, 4], 2, + ,  +=);
        op_test!([1, 2], [3, 4], 2, - ,  -=);
        op_test!([1, 2], [3, 4], 2, * ,  *=);
        op_test!([1, 2], [3, 4], 2, / ,  /=);
        op_test!([1, 2], [3, 4], 2, % ,  %=);
        op_test!([1, 2], [3, 4], 2, & ,  &=);
        op_test!([1, 2], [3, 4], 2, ^ ,  ^=);
        op_test!([1, 2], [3, 4], 2, | ,  |=);
        op_test!([1, 2], [3, 4], 2, <<, <<=);
        op_test!([1, 2], [3, 4], 2, >>, >>=);

        let a = Vec2::new(1, 2);
        let res = -a;
        assert_eq!(res.x, -1);
        assert_eq!(res.y, -2);

        let res = !a;
        assert_eq!(res.x, !1);
        assert_eq!(res.y, !2);
    }

    #[test]
    fn test_cmp() {
        let a = Vec2::new(1, 2);
        let b = Vec2::new(2, 3);

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
        let a = Vec2::new(2, 3);
        let b = Vec2::new(1, 4);

        assert_eq!(a.min(b), Vec2::new(1, 3));
        assert_eq!(a.max(b), Vec2::new(2, 4));

        assert_eq!(b.clamp_scalar(1, 2), Vec2::new(1, 2));
        assert_eq!(b.clamp_scalar(3, 4), Vec2::new(3, 4));

        let min = Vec2::new(0, 2);
        let max = Vec2::new(1, 5);
        assert_eq!(a.clamp(min, max), Vec2::new(1, 3));
        assert_eq!(b.clamp(min, max), Vec2::new(1, 4));


        assert_eq!(a.snap(4), Vec2::new(4, 4)); // x snaps to 4, cause the value is in the middle and rounds up
        assert_eq!(b.snap(3), Vec2::new(0, 3));

        assert_eq!(Vec2::new(1.2f32, 1.6f32).snap(1f32), Vec2::new(1f32, 2f32));

        assert_eq!(Vec2::new(-0.2f32, 1.5f32).saturated(), Vec2::new(0f32, 1f32));

        let v0 = Vec2::new(3f32, 4f32); // len == 5
        let v1 = Vec2::new(5f32, 12f32); // len == 13
        let v2 = Vec2::new(0.6f32, 0.8f32); // len == 5
        let v3 = Vec2::new(3.2f32, 3.8f32);

        assert_eq!(v0.lerp(v1, 0.25f32), Vec2::new(3.5f32, 6f32));

        assert_eq!(v0.len_sq(), 25f32);
        assert_eq!(v0.len(), 5f32);

        assert_eq!(v0.dist_sq(v1), 68f32);
        assert_eq!(v0.dist(v1), 68f32.sqrt());

        assert_eq!(v0.normalize(), v2);
        assert_eq!(Vec2::set(0f32).normalize(), Vec2::set(0f32));

        assert_eq!(v0.normalize_or(v1), v2);
        assert_eq!(Vec2::set(0f32).normalize_or(v1), v1);

        assert!(!v0.is_close_to_normalized(0f32));
        assert!(v0.is_close_to_normalized(25f32));
        assert!(v2.is_close_to_normalized(0f32));

        assert!(!v0.is_normalized());
        assert!(v2.is_normalized());

        assert_eq!(v0.dir_and_len(), (v2, 5f32));

        assert!(v0.clamp_len(0f32, 4f32).is_close_to(v2 * 4f32, 0.000001f32));
        assert!(v0.clamp_len(16f32, 20f32).is_close_to(v2 * 16f32, 0.000001f32));

        assert_eq!(Vec2::new(-3f32, -4f32).abs(), v0);
        assert_eq!(v3.ceil(), Vec2::new(4f32, 4f32));
        assert_eq!(v3.floor(), Vec2::new(3f32, 3f32));
        assert_eq!(v3.round(), v0);
        assert_eq!(Vec2::new(-4f32, 5f32).sign(), Vec2::new(-1f32, 1f32));
        assert!(v3.fract().is_close_to(Vec2::new(0.2f32, 0.8f32), 0.0000001f32));

        // Common per vec funcs
        let v0 = Vec2::new(2f32, -3f32);

        assert!(!v0.is_uniform(0.1f32));
        assert!(v0.is_uniform(5f32));

        assert_eq!(v0.min_component(), -3f32);
        assert_eq!(v0.min_abs_component(), 2f32);
        assert_eq!(v0.max_component(), 2f32);
        assert_eq!(v0.max_abs_component(), 3f32);
    }

    #[test]
    fn test_spec_fun() {
        let v0 = Vec2::new(2f32, -3f32);
        let v1 = Vec2::new(4f32, 5f32);

        assert_eq!(v0.dot(v1), -7f32);
        assert_eq!(v0.cross(v1), 22f32);

        assert_eq!(v0.perpendicular_cw(), Vec2::new(-3f32, -2f32));
        assert_eq!(v0.perpendicular_ccw(), Vec2::new(3f32, 2f32));
    }
}