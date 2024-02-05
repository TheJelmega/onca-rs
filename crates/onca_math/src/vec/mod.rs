use std::ops::*;

mod vec2;
pub use vec2::*;

mod vec3;
pub use vec3::*;

mod vec4;
pub use vec4::*;

macro_rules! generic_vec {
    {
        $docs:meta;
        $iden:ident,
        $elem_cnt:literal,
        $tup_ty:ty,
        $($comp:ident => $idx:tt),+;
        $($alias_ty:ident => $base_ty:ty)*
    } => {
        $crate::common::tuple_common!{
            $docs;
            $iden,
            $elem_cnt,
            $tup_ty,
            $($comp => $idx),+;
            $($alias_ty => $base_ty)*
        }

        $crate::common::tuple_common!{ @add_sub_self $iden, $iden, $($comp),+ }
        $crate::common::tuple_common!{ @scale $iden, $($comp),+ }
        $crate::common::tuple_common!{ @len_and_normalize $iden, $($comp),+ }
        $crate::common::tuple_common!{ @dot $iden, $iden, $($comp),+ }
        $crate::common::tuple_common!{ @snap_scalar $iden, $($comp),+ }
        $crate::common::tuple_common!{ @snap_comp $iden, $iden, $($comp),+ }

        impl<T: Numeric> $iden<T> {
            /// Clamp the length of the vector.
            #[must_use]
            pub fn clamp_len(self, min: T, max: T) -> Self {
                let len = self.len();
                if len < min {
                    self * (min / len)
                } else if len > max {
                    self * (max / len)
                } else {
                    self
                }
            }

            /// Calculate the square distance between 2 vectors.
            #[must_use]
            pub fn dist_sq(self, other: Self) -> T {
                (other - self).len_sq()
            }

            /// Calculate the distance between 2 vectors.
            #[must_use]
            pub fn dist(self, other: Self) -> T  {
                self.dist_sq(other).sqrt()
            }

            /// Calculate the horizontal product of the vector.
            #[must_use]
            pub fn h_prod(self) -> T {
                $crate::utils::strip_mul!($(* self.$comp)+)
            }
        }

        impl<T: Real> $iden<T> {
            /// Get the direction and length of the vector.
            pub fn dir_and_len(self) -> (Self, T) {
                let len = self.len();
                (self * len.recip(), len)
            }

            /// Get a vector the component-wise fractional parts.
            #[inline]
            pub fn fract(self) -> Self {
                Self{ $($comp: self.$comp.fract()),+ }
            }
        }

        impl<T: Numeric> One for $iden<T> {
            fn one() -> Self {
                Self{ $($comp: T::one()),+ }
            }
        }

        
        impl<T: Numeric> Default for $iden<T> {
            fn default() -> Self {
                Self::zero()
            }
        }

        //------------------------------------------------------------------------------------------------------------------------------

        impl<T: Numeric> Mul for $iden<T> {
            type Output = Self;

            #[inline(always)]
            fn mul(self, rhs: Self) -> Self {
                Self{ $($comp: self.$comp * rhs.$comp),+ }
            }
        }

        impl<T: Numeric> MulAssign for $iden<T> {
            #[inline(always)]
            fn mul_assign(&mut self, rhs: Self) {
                $(self.$comp *= rhs.$comp);+
            }
        }

        // We can't implement pre-multipy genericly here, so `impl_vec_premul` is used instead

        //--------------------------------------------------------------

        impl<T: Numeric> Div for $iden<T> {
            type Output = Self;

            #[inline(always)]
            fn div(self, rhs: Self) -> Self {
                Self{ $($comp: self.$comp / rhs.$comp),+ }
            }
        }

        impl<T: Numeric> DivAssign for $iden<T> {
            #[inline(always)]
            fn div_assign(&mut self, rhs: Self) {
                $(self.$comp /= rhs.$comp);+
            }
        }

        //--------------------------------------------------------------

        impl<T: Numeric> Rem for $iden<T> {
            type Output = Self;

            #[inline(always)]
            fn rem(self, rhs: Self) -> Self {
                Self{ $($comp: self.$comp % rhs.$comp),+ }
            }
        }

        impl<T: Numeric> RemAssign for $iden<T> {
            #[inline(always)]
            fn rem_assign(&mut self, rhs: Self) {
                $(self.$comp %= rhs.$comp);+
            }
        }

        impl<T: Numeric> Rem<T> for $iden<T> {
            type Output = Self;

            #[inline(always)]
            fn rem(self, rhs: T) -> Self {
                Self{ $($comp: self.$comp % rhs),+ }
            }
        }

        impl<T: Numeric> RemAssign<T> for $iden<T> {
            #[inline(always)]
            fn rem_assign(&mut self, rhs: T) {
                $(self.$comp %= rhs);+
            }
        }

        //------------------------------------------------------------------------------------------------------------------------------

        impl<T: Integer> Not for $iden<T> {
            type Output = Self;

            #[inline(always)]
            fn not(self) -> Self {
                Self{ $($comp: !self.$comp),+ }
            }
        }

        //--------------------------------------------------------------

        impl<T: Integer> BitAnd for $iden<T> {
            type Output = Self;

            #[inline(always)]
            fn bitand(self, rhs: Self) -> Self {
                Self{ $($comp: self.$comp & rhs.$comp),+ }
            }
        }

        impl<T: Integer> BitAndAssign for $iden<T> {
            #[inline(always)]
            fn bitand_assign(&mut self, rhs: Self) {
                $(self.$comp &= rhs.$comp);+
            }
        }

        impl<T: Integer> BitAnd<T> for $iden<T> {
            type Output = Self;

            #[inline(always)]
            fn bitand(self, rhs: T) -> Self {
                Self{ $($comp: self.$comp & rhs),+ }
            }
        }

        impl<T: Integer> BitAndAssign<T> for $iden<T> {
            #[inline(always)]
            fn bitand_assign(&mut self, rhs: T) {
                $(self.$comp &= rhs);+
            }
        }

        //--------------------------------------------------------------

        impl<T: Integer> BitXor for $iden<T> {
            type Output = Self;

            #[inline(always)]
            fn bitxor(self, rhs: Self) -> Self {
                Self{ $($comp: self.$comp ^ rhs.$comp),+ }
            }
        }

        impl<T: Integer> BitXorAssign for $iden<T> {
            #[inline(always)]
            fn bitxor_assign(&mut self, rhs: Self) {
                $(self.$comp ^= rhs.$comp);+
            }
        }

        impl<T: Integer> BitXor<T> for $iden<T> {
            type Output = Self;

            #[inline(always)]
            fn bitxor(self, rhs: T) -> Self {
                Self{ $($comp: self.$comp ^ rhs),+ }
            }
        }

        impl<T: Integer> BitXorAssign<T> for $iden<T> {
            #[inline(always)]
            fn bitxor_assign(&mut self, rhs: T) {
                $(self.$comp ^= rhs);+
            }
        }

        //--------------------------------------------------------------

        impl<T: Integer> BitOr for $iden<T> {
            type Output = Self;

            #[inline(always)]
            fn bitor(self, rhs: Self) -> Self {
                Self{ $($comp: self.$comp | rhs.$comp),+ }
            }
        }

        impl<T: Integer> BitOrAssign for $iden<T> {
            #[inline(always)]
            fn bitor_assign(&mut self, rhs: Self) {
                $(self.$comp |= rhs.$comp);+
            }
        }

        impl<T: Integer> BitOr<T> for $iden<T> {
            type Output = Self;

            #[inline(always)]
            fn bitor(self, rhs: T) -> Self {
                Self{ $($comp: self.$comp | rhs),+ }
            }
        }

        impl<T: Integer> BitOrAssign<T> for $iden<T> {
            #[inline(always)]
            fn bitor_assign(&mut self, rhs: T) {
                $(self.$comp |= rhs);+
            }
        }

        //--------------------------------------------------------------

        impl<T: Integer> Shl for $iden<T> {
            type Output = Self;

            #[inline(always)]
            fn shl(self, rhs: Self) -> Self {
                Self{ $($comp: self.$comp << rhs.$comp),+ }
            }
        }

        impl<T: Integer> ShlAssign for $iden<T> {
            #[inline(always)]
            fn shl_assign(&mut self, rhs: Self) {
                $(self.$comp <<= rhs.$comp);+
            }
        }

        impl<T: Integer> Shl<T> for $iden<T> {
            type Output = Self;

            #[inline(always)]
            fn shl(self, rhs: T) -> Self {
                Self{ $($comp: self.$comp << rhs),+ }
            }
        }

        impl<T: Integer> ShlAssign<T> for $iden<T> {
            #[inline(always)]
            fn shl_assign(&mut self, rhs: T) {
                $(self.$comp <<= rhs);+
            }
        }

        //--------------------------------------------------------------

        impl<T: Integer> Shr for $iden<T> {
            type Output = Self;

            #[inline(always)]
            fn shr(self, rhs: Self) -> Self {
                Self{ $($comp: self.$comp >> rhs.$comp),+ }
            }
        }

        impl<T: Integer> ShrAssign for $iden<T> {
            #[inline(always)]
            fn shr_assign(&mut self, rhs: Self) {
                $(self.$comp >>= rhs.$comp);+
            }
        }

        impl<T: Integer> Shr<T> for $iden<T> {
            type Output = Self;

            #[inline(always)]
            fn shr(self, rhs: T) -> Self {
                Self{ $($comp: self.$comp >> rhs),+ }
            }
        }

        impl<T: Integer> ShrAssign<T> for $iden<T> {
            #[inline(always)]
            fn shr_assign(&mut self, rhs: T) {
                $(self.$comp >>= rhs);+
            }
        }

        //------------------------------------------------------------------------------------------------------------------------------

        impl<T: Numeric> Saturate for $iden<T> {
            fn saturate(self) -> Self {
                Self { $($comp: self.$comp.saturate()),* }
            }
        }

        impl<T: Numeric> AbsDiff for $iden<T> where
            <T as AbsDiff>::Output: Numeric
        {
            type Output = $iden<<T as AbsDiff>::Output>;

            fn abs_diff(self, rhs: Self) -> Self::Output {
                $iden { $($comp: self.$comp.abs_diff(rhs.$comp)),* }
            }
        }

        impl<T: Numeric> Sign for $iden<T> {
            fn sign(self) -> Self {
                Self { $($comp: self.$comp.sign()),* }
            }

            fn copy_sign(self, sign: Self) -> Self {
                Self { $($comp: self.$comp.copy_sign(sign.$comp)),* }
            }
        }

        impl<T: Real> Recip for $iden<T> {
            fn recip(self) -> Self {
                Self { $($comp: self.$comp.recip()),* }
            }
        }

        impl<T: Numeric> FMulAdd for $iden<T> {
            fn fma(self, b: Self, c: Self) -> Self {
                Self { $($comp: self.$comp.fma(b.$comp, c.$comp)),+ }
            }
        }
    };
}
pub(crate) use generic_vec;

#[macro_export]
macro_rules! impl_vec_premul {
    ($iden:ident, $($ty:ty)*) => {
        $(
            impl Mul<$iden<$ty>> for $ty {
                type Output = $iden<$ty>;

                fn mul(self, rhs: $iden<$ty>) -> $iden<$ty> {
                    rhs * self
                }
            }
        )*
    };
}
impl_vec_premul!{ Vec2, i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 }
impl_vec_premul!{ Vec3, i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 }
impl_vec_premul!{ Vec4, i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 }

/// Swizzle component
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Swizzle {
    X,
    Y,
    Z,
    W,
}

macro_rules! create_swizzle {
    {@2d $($fun:ident => $a:ident, $b:ident)*} => {
        $(
            #[doc = concat!("Swizzle the vector to `", stringify!($fun), "`")]
            pub fn $fun(self) -> Vec2<T> { Vec2 { x: self.$a, y: self.$b } }
        )*
    };
    {@3d $($fun:ident => $a:ident, $b:ident, $c:ident)*} => {
        $(
            #[doc = concat!("Swizzle the vector to `", stringify!($fun), "`")]
            pub fn $fun(self) -> Vec3<T> { Vec3 { x: self.$a, y: self.$b, z: self.$c } }
        )*
    };
    {@4d $($fun:ident => $a:ident, $b:ident, $c:ident, $d:ident)*} => {
        $(
            #[doc = concat!("Swizzle the vector to `", stringify!($ fun), "`")]
            pub fn $fun(self) -> Vec4<T> { Vec4 { x: self.$a, y: self.$b, z: self.$c, w: self.$d } }
        )*
    };
}
pub(crate) use create_swizzle;


// Some tests to make sure no errors were made in the macro implementation.
// If the implementation works for a Vec2, if should also work for the other vectors
#[cfg(test)]
mod tests {
    use crate::{Vec2, numeric::*, DotProduct};

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

        assert_eq!(b.clamp(1, 2), Vec2::new(1, 2));
        assert_eq!(b.clamp(3, 4), Vec2::new(3, 4));

        let min = Vec2::new(0, 2);
        let max = Vec2::new(1, 5);
        assert_eq!(a.clamp(min, max), Vec2::new(1, 3));
        assert_eq!(b.clamp(min, max), Vec2::new(1, 4));


        assert_eq!(a.snap(4), Vec2::new(4, 4)); // x snaps to 4, cause the value is in the middle and rounds up
        assert_eq!(b.snap(3), Vec2::new(0, 3));

        assert_eq!(Vec2::new(1.2f32, 1.6f32).snap(1f32), Vec2::new(1f32, 2f32));

        assert_eq!(Vec2::new(-0.2f32, 1.5f32).saturate(), Vec2::new(0f32, 1f32));

        let v0 = Vec2::new(3f32, 4f32); // len == 5
        let v1 = Vec2::new(5f32, 12f32); // len == 13
        let v2 = Vec2::new(0.6f32, 0.8f32);
        let v3 = Vec2::new(3.2f32, 3.8f32);

        assert_eq!(v0.lerp(v1, 0.25f32), Vec2::new(3.5f32, 6f32));

        assert_eq!(v0.dist_sq(v1), 68f32);
        assert_eq!(v0.dist(v1), 68f32.sqrt());

        assert_eq!(v0.dir_and_len(), (v2, 5f32));

        assert!(v0.clamp_len(0f32, 4f32).is_close_to(v2 * 4f32, 0.000001f32));
        assert!(v0.clamp_len(16f32, 20f32).is_close_to(v2 * 16f32, 0.000001f32));

        assert_eq!(Vec2::new(-3f32, -4f32).abs(), v0);
        assert_eq!(v3.ceil(), Vec2::new(4f32, 4f32));
        assert_eq!(v3.floor(), Vec2::new(3f32, 3f32));
        assert_eq!(v3.round(), v0);
        assert_eq!(Vec2::new(-4f32, 5f32).sign(), Vec2::new(-1f32, 1f32));
        assert!(v3.fract().is_close_to(Vec2::new(0.2f32, 0.8f32), 0.0000001f32));

        let v0 = Vec2::new(2f32, -3f32);
        let v1 = Vec2::new(4f32, 5f32);

        assert_eq!(v0.dot(v1), -7f32);
    }

}