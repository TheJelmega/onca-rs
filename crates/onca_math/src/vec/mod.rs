use core::{
    mem,
    ops::*,
};
use crate::numeric::*;


mod vec2;
pub use vec2::*;

mod vec3;
pub use vec3::*;

mod vec4;
pub use vec4::*;

macro_rules! strip_plus {
    (+ $($rest:tt)*) => {
        $($rest)*
    };
}

macro_rules! generic_vec {
    {
        $docs:meta;
        $name:ident,
        $elem_cnt:literal,
        $($field:ident),+
    } => {
        #[$docs]
        #[derive(Clone, Copy, PartialEq, Debug)]
        pub struct $name<T: Numeric> {
            $(pub $field : T,)+
        }

        impl<T: Numeric> $name<T> {
            /// Create a new vector
            #[inline(always)]
            #[must_use]
            pub fn new($($field: T),+) -> Self {
                Self{ $($field: $field),+ }
            }

            /// Create a vector with all components set to `val`
            #[inline(always)]
            #[must_use]
            pub fn set(val: T) -> Self {
                Self{ $($field: val),+ }
            }

            /// Create a vector from an array
            #[inline(always)]
            #[must_use]
            pub fn from_array(arr: [T; $elem_cnt]) -> Self {
                unsafe { mem::transmute_copy(&arr) }
            }

            /// Interpret a reference to an array as a reference to a vector
            #[inline(always)]
            #[must_use]
            pub fn ref_from_array(arr: &[T; $elem_cnt]) -> &Self {
                unsafe { mem::transmute(arr) }
            }

            /// Interpret a mutable reference to an array as a mutable reference to a vector
            #[inline(always)]
            #[must_use]
            pub fn mut_from_array(arr: &mut [T; $elem_cnt]) -> &mut Self {
                unsafe { mem::transmute(arr) }
            }

            /// Get the content of the vector as an array
            #[inline(always)]
            #[must_use]
            pub fn to_array(self) -> [T; $elem_cnt] {
                unsafe{ mem::transmute_copy(&self) }
            }
        
            /// Interpret a reference to an vector as a reference to a array
            #[inline(always)]
            #[must_use]
            pub fn as_array(&self) -> &[T; $elem_cnt] {
                unsafe{ mem::transmute(self) }
            }
        
            /// Interpret a mutable reference to an vector as a mutable reference to a array
            #[inline(always)]
            #[must_use]
            pub fn as_mut_array(&mut self) -> &mut [T; $elem_cnt] {
                unsafe{ mem::transmute(self) }
            }

            /// Get a vector with the component-wise minimum component of 2 vectors
            pub fn min(self, other: Self) -> Self {
                Self{ $($field: self.$field.min(other.$field)),+ }
            }
        
            /// Get a vector with the component-wise maximum component of 2 vectors
            pub fn max(self, other: Self) -> Self {
                Self{ $($field: self.$field.max(other.$field)),+ }
            }

            /// Component-wise clamp of the vector, using scalar min and max
            pub fn clamp_scalar(self, min: T, max: T) -> Self {
                Self{ $($field: self.$field.clamp(min, max)),+ }
            }

            /// Component-wise clamp of the vector, using component-wise min and max
            pub fn clamp(self, min: Self, max: Self) -> Self {
                Self{ $($field: self.$field.clamp(min.$field, max.$field)),+ }
            }

            /// Clamp the length of the vector
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

            /// Clamp the components between 0 and 1
            pub fn saturated(self) -> Self {
                self.clamp_scalar(T::zero(), T::one())
            }

            /// Linearly interpolate between 2 vectors using `val`
            pub fn lerp(self, other: Self, val: T) -> Self {
                self + (other - self) * val
            }

            /// Snap each component to the nearest multiple of `step_size`
            #[inline]
            pub fn snap(self, step_size: T) -> Self {
                Self{ $($field: self.$field.snap(step_size)),+ }
            }

            /// Calculate the dot product of 2 vectors
            #[inline]
            pub fn dot(self, rhs: Self) -> T {
                strip_plus!($(+ self.$field * rhs.$field)+)
            }

            /// Calculate the square length of the vector
            pub fn len_sq(self) -> T {
                strip_plus!($(+ self.$field * self.$field)+)
            }

            /// Calculate the length of the vector
            pub fn len(self) -> T {
                self.len_sq().sqrt()
            }

            /// Calculate the square distance between 2 vectors
            pub fn dist_sq(self, other: Self) -> T {
                (other - self).len_sq()
            }

            /// Calculate the distance between 2 vectors
            pub fn dist(self, other: Self) -> T {
                self.dist_sq(other).sqrt()
            }

            /// Normalize the vector
            pub fn normalize(self) -> Self {
                if self.is_zero() {
                    self
                } else {
                    unsafe{ self.normalize_unsafe() }
                }
            }

            /// Normalize the vector (no check for a length of 0)
            pub unsafe fn normalize_unsafe(self) -> Self {
                self * self.len_sq().rsqrt()
            }

            /// Normalize the vector if the length is not 0, return `or` otherwise
            pub fn normalize_or(self, or: Self) -> Self {
                if self.is_zero() {
                    or
                } else {
                    self.normalize()
                }
            }

            /// Check if the vector is close to being normalized, using a given epsilon, which defines the max difference `len` can be relative to 1
            pub fn is_close_to_normalized(self, epsilon: T) -> bool {
                self.len_sq().is_close_to(T::one(), epsilon)
            }

            /// Ckeck if the vector is normalized, using the machine epsilon
            pub fn is_normalized(self) -> bool {
                self.len_sq().is_approx_eq(T::one())
            }

            /// Get the direction and length of the vector
            pub fn dir_and_len(self) -> (Self, T) {
                let len = self.len();
                (self * len.rcp(), len)
            }

            /// Get a vector the component-wise absolute values
            #[inline]
            pub fn abs(self) -> Self {
                Self{ $($field: self.$field.abs()),+ }
            }

            /// Get a vector the component-wise signs
            #[inline]
            pub fn sign(self) -> Self {
                Self{ $($field: self.$field.sign()),+ }
            }

        }

        impl<T: Real> $name<T> {
            /// Get a vector the component-wise ceiled values
            #[inline]
            pub fn ceil(self) -> Self {
                Self{ $($field: self.$field.ceil()),+ }
            }

            /// Get a vector the component-wise floored values
            #[inline]
            pub fn floor(self) -> Self {
                Self{ $($field: self.$field.floor()),+ }
            }

            /// Get a vector the component-wise round values
            #[inline]
            pub fn round(self) -> Self {
                Self{ $($field: self.$field.round()),+ }
            }

            /// Get a vector the component-wise fractional parts
            #[inline]
            pub fn fract(self) -> Self {
                Self{ $($field: self.$field.fract()),+ }
            }
        }

        impl<T: Numeric> Index<usize> for $name<T> {
            type Output = T;
        
            #[inline(always)]
            fn index(&self, index: usize) -> &Self::Output {
                debug_assert!(index < $elem_cnt);
                self.as_array().index(index)
            }
        }
        
        impl<T: Numeric> IndexMut<usize> for $name<T> {
            #[inline(always)]
            fn index_mut(&mut self, index: usize) -> &mut Self::Output {
                debug_assert!(index < $elem_cnt);
                self.as_mut_array().index_mut(index)
            }
        }

        impl<T: Numeric> Zero for $name<T> {
            fn zero() -> Self {
                Self{ $($field: T::zero()),+ }
            }
        }

        impl<T: Numeric> One for $name<T> {
            fn one() -> Self {
                Self{ $($field: T::one()),+ }
            }
        }

        //------------------------------------------------------------------------------------------------------------------------------

        impl<T: Numeric> Add for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn add(self, rhs: Self) -> Self {
                Self{ $($field: self.$field + rhs.$field),+ }
            }
        }

        impl<T: Numeric> AddAssign for $name<T> {
            #[inline(always)]
            fn add_assign(&mut self, rhs: Self) {
                $(self.$field += rhs.$field);+
            }
        }

        impl<T: Numeric> Add<T> for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn add(self, rhs: T) -> Self {
                Self{ $($field: self.$field + rhs),+ }
            }
        }

        impl<T: Numeric> AddAssign<T> for $name<T> {
            #[inline(always)]
            fn add_assign(&mut self, rhs: T) {
                $(self.$field += rhs);+
            }
        }

        //--------------------------------------------------------------

        impl<T: Numeric> Sub for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn sub(self, rhs: Self) -> Self {
                Self{ $($field: self.$field - rhs.$field),+ }
            }
        }

        impl<T: Numeric> SubAssign for $name<T> {
            #[inline(always)]
            fn sub_assign(&mut self, rhs: Self) {
                $(self.$field -= rhs.$field);+
            }
        }

        impl<T: Numeric> Sub<T> for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn sub(self, rhs: T) -> Self {
                Self{ $($field: self.$field - rhs),+ }
            }
        }

        impl<T: Numeric> SubAssign<T> for $name<T> {
            #[inline(always)]
            fn sub_assign(&mut self, rhs: T) {
                $(self.$field -= rhs);+
            }
        }

        //--------------------------------------------------------------

        impl<T: Numeric> Mul for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn mul(self, rhs: Self) -> Self {
                Self{ $($field: self.$field * rhs.$field),+ }
            }
        }

        impl<T: Numeric> MulAssign for $name<T> {
            #[inline(always)]
            fn mul_assign(&mut self, rhs: Self) {
                $(self.$field *= rhs.$field);+
            }
        }

        impl<T: Numeric> Mul<T> for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn mul(self, rhs: T) -> Self {
                Self{ $($field: self.$field * rhs),+ }
            }
        }

        impl<T: Numeric> MulAssign<T> for $name<T> {
            #[inline(always)]
            fn mul_assign(&mut self, rhs: T) {
                $(self.$field *= rhs);+
            }
        }

        impl Mul<$name<f32>> for f32 {
            type Output = $name<f32>;
        
            fn mul(self, rhs: $name<f32>) -> Self::Output {
                $name{ $($field: self * rhs.$field),+ }
            }
        }

        impl Mul<$name<f64>> for f64 {
            type Output = $name<f64>;
        
            fn mul(self, rhs: $name<f64>) -> Self::Output {
                $name{ $($field: self * rhs.$field),+ }
            }
        }

        //--------------------------------------------------------------

        impl<T: Numeric> Div for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn div(self, rhs: Self) -> Self {
                Self{ $($field: self.$field / rhs.$field),+ }
            }
        }

        impl<T: Numeric> DivAssign for $name<T> {
            #[inline(always)]
            fn div_assign(&mut self, rhs: Self) {
                $(self.$field /= rhs.$field);+
            }
        }

        impl<T: Numeric> Div<T> for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn div(self, rhs: T) -> Self {
                Self{ $($field: self.$field / rhs),+ }
            }
        }

        impl<T: Numeric> DivAssign<T> for $name<T> {
            #[inline(always)]
            fn div_assign(&mut self, rhs: T) {
                $(self.$field /= rhs);+
            }
        }

        //--------------------------------------------------------------

        impl<T: Numeric> Rem for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn rem(self, rhs: Self) -> Self {
                Self{ $($field: self.$field % rhs.$field),+ }
            }
        }

        impl<T: Numeric> RemAssign for $name<T> {
            #[inline(always)]
            fn rem_assign(&mut self, rhs: Self) {
                $(self.$field %= rhs.$field);+
            }
        }

        impl<T: Numeric> Rem<T> for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn rem(self, rhs: T) -> Self {
                Self{ $($field: self.$field % rhs),+ }
            }
        }

        impl<T: Numeric> RemAssign<T> for $name<T> {
            #[inline(always)]
            fn rem_assign(&mut self, rhs: T) {
                $(self.$field %= rhs);+
            }
        }

        //------------------------------------------------------------------------------------------------------------------------------

        impl<T: Numeric + Neg<Output = T>> Neg for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn neg(self) -> Self {
                Self{ $($field: -self.$field),+ }
            }
        }

        impl<T: Integral> Not for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn not(self) -> Self {
                Self{ $($field: !self.$field),+ }
            }
        }

        //--------------------------------------------------------------

        impl<T: Integral> BitAnd for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn bitand(self, rhs: Self) -> Self {
                Self{ $($field: self.$field & rhs.$field),+ }
            }
        }

        impl<T: Integral> BitAndAssign for $name<T> {
            #[inline(always)]
            fn bitand_assign(&mut self, rhs: Self) {
                $(self.$field &= rhs.$field);+
            }
        }

        impl<T: Integral> BitAnd<T> for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn bitand(self, rhs: T) -> Self {
                Self{ $($field: self.$field & rhs),+ }
            }
        }

        impl<T: Integral> BitAndAssign<T> for $name<T> {
            #[inline(always)]
            fn bitand_assign(&mut self, rhs: T) {
                $(self.$field &= rhs);+
            }
        }

        //--------------------------------------------------------------

        impl<T: Integral> BitXor for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn bitxor(self, rhs: Self) -> Self {
                Self{ $($field: self.$field ^ rhs.$field),+ }
            }
        }

        impl<T: Integral> BitXorAssign for $name<T> {
            #[inline(always)]
            fn bitxor_assign(&mut self, rhs: Self) {
                $(self.$field ^= rhs.$field);+
            }
        }

        impl<T: Integral> BitXor<T> for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn bitxor(self, rhs: T) -> Self {
                Self{ $($field: self.$field ^ rhs),+ }
            }
        }

        impl<T: Integral> BitXorAssign<T> for $name<T> {
            #[inline(always)]
            fn bitxor_assign(&mut self, rhs: T) {
                $(self.$field ^= rhs);+
            }
        }

        //--------------------------------------------------------------

        impl<T: Integral> BitOr for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn bitor(self, rhs: Self) -> Self {
                Self{ $($field: self.$field | rhs.$field),+ }
            }
        }

        impl<T: Integral> BitOrAssign for $name<T> {
            #[inline(always)]
            fn bitor_assign(&mut self, rhs: Self) {
                $(self.$field |= rhs.$field);+
            }
        }

        impl<T: Integral> BitOr<T> for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn bitor(self, rhs: T) -> Self {
                Self{ $($field: self.$field | rhs),+ }
            }
        }

        impl<T: Integral> BitOrAssign<T> for $name<T> {
            #[inline(always)]
            fn bitor_assign(&mut self, rhs: T) {
                $(self.$field |= rhs);+
            }
        }

        //--------------------------------------------------------------

        impl<T: Integral> Shl for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn shl(self, rhs: Self) -> Self {
                Self{ $($field: self.$field << rhs.$field),+ }
            }
        }

        impl<T: Integral> ShlAssign for $name<T> {
            #[inline(always)]
            fn shl_assign(&mut self, rhs: Self) {
                $(self.$field <<= rhs.$field);+
            }
        }

        impl<T: Integral> Shl<T> for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn shl(self, rhs: T) -> Self {
                Self{ $($field: self.$field << rhs),+ }
            }
        }

        impl<T: Integral> ShlAssign<T> for $name<T> {
            #[inline(always)]
            fn shl_assign(&mut self, rhs: T) {
                $(self.$field <<= rhs);+
            }
        }

        //--------------------------------------------------------------

        impl<T: Integral> Shr for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn shr(self, rhs: Self) -> Self {
                Self{ $($field: self.$field >> rhs.$field),+ }
            }
        }

        impl<T: Integral> ShrAssign for $name<T> {
            #[inline(always)]
            fn shr_assign(&mut self, rhs: Self) {
                $(self.$field >>= rhs.$field);+
            }
        }

        impl<T: Integral> Shr<T> for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn shr(self, rhs: T) -> Self {
                Self{ $($field: self.$field >> rhs),+ }
            }
        }

        impl<T: Integral> ShrAssign<T> for $name<T> {
            #[inline(always)]
            fn shr_assign(&mut self, rhs: T) {
                $(self.$field >>= rhs);+
            }
        }

        //------------------------------------------------------------------------------------------------------------------------------

        impl<T: Numeric> ApproxEq for $name<T> {
            type Epsilon = T;

            fn is_close_to(self, rhs: Self, epsilon: Self::Epsilon) -> bool {
                $(self.$field.is_close_to(rhs.$field, epsilon))||+
            }
        }
        
        //--------------------------------------------------------------

        impl<T: Numeric> ApproxZero for $name<T> {
            type Epsilon = T;

            fn is_close_to_zero(self, epsilon: Self::Epsilon) -> bool {
                $(self.$field.is_close_to_zero(epsilon))||+
            }
        }
        
        //------------------------------------------------------------------------------------------------------------------------------

        impl<T: Numeric> From<[T; $elem_cnt]> for $name<T> {
            fn from(arr: [T; $elem_cnt]) -> Self {
                Self::from_array(arr)
            }
        }

        impl<T: Numeric> From<$name<T>> for [T; $elem_cnt] {
            fn from(v: $name<T>) -> Self {
                v.to_array()
            }
        }
        
        //------------------------------------------------------------------------------------------------------------------------------

        
    };
}
generic_vec!{ doc = "2D Vector (row-major order)"; Vec2, 2, x, y }
generic_vec!{ doc = "3D Vector (row-major order)"; Vec3, 3, x, y, z }
generic_vec!{ doc = "4D Vector (row-major order)"; Vec4, 4, x, y, z, w }

macro_rules! create_swizzle {
    {@2d $fun:ident, $a:ident, $b:ident} => {
        #[doc = concat!("Swizzle the vector to `", stringify!($fun), "`")]
        pub fn $fun(self) -> Vec2<T> { Vec2 { x: self.$a, y: self.$b } }
    };
    {@3d $fun:ident, $a:ident, $b:ident, $c:ident} => {
        #[doc = concat!("Swizzle the vector to `", stringify!($fun), "`")]
        pub fn $fun(self) -> Vec3<T> { Vec3 { x: self.$a, y: self.$b, z: self.$c } }
    };
    {@4d $fun:ident, $a:ident, $b:ident, $c:ident, $d:ident} => {
        #[doc = concat!("Swizzle the vector to `", stringify!($fun), "`")]
        pub fn $fun(self) -> Vec4<T> { Vec4 { x: self.$a, y: self.$b, z: self.$c, w: self.$d } }
    };
}
pub(crate) use create_swizzle;