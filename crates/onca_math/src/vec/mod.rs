use std::ops::*;
use crate::{
    numeric::*,
    smoothstep_interpolant,
};


mod vec2;
pub use vec2::*;

mod vec3;
pub use vec3::*;

mod vec4;
pub use vec4::*;

macro_rules! generic_vec {
    {
        $docs:meta;
        $name:ident,
        $elem_cnt:literal,
        $($comp:ident),+;
        $($alias_ty:ident => $base_ty:ty)*
    } => {
        #[$docs]
        #[derive(Clone, Copy, PartialEq, Debug)]
        pub struct $name<T: Copy> {
            $(pub $comp: T,)+
        }

        impl<T: Copy> $name<T> {
            /// Create a new vector
            #[inline(always)]
            #[must_use]
            pub fn new($($comp: T),+) -> Self {
                Self{ $($comp: $comp),+ }
            }

            /// Create a vector with all components set to `val`
            #[inline(always)]
            #[must_use]
            pub fn set(val: T) -> Self {
                Self{ $($comp: val),+ }
            }

            /// Create a vector from an array
            #[inline(always)]
            #[must_use]
            pub fn from_array(arr: [T; $elem_cnt]) -> Self {
                unsafe { std::mem::transmute_copy(&arr) }
            }

            /// Interpret a reference to an array as a reference to a vector
            #[inline(always)]
            #[must_use]
            pub fn ref_from_array(arr: &[T; $elem_cnt]) -> &Self {
                unsafe { std::mem::transmute(arr) }
            }

            /// Interpret a mutable reference to an array as a mutable reference to a vector
            #[inline(always)]
            #[must_use]
            pub fn mut_from_array(arr: &mut [T; $elem_cnt]) -> &mut Self {
                unsafe { std::mem::transmute(arr) }
            }

            /// Get the content of the vector as an array
            #[inline(always)]
            #[must_use]
            pub fn to_array(self) -> [T; $elem_cnt] {
                [$(self.$comp),*]
            }
        
            /// Interpret a reference to an vector as a reference to a array
            #[inline(always)]
            #[must_use]
            pub fn as_array(&self) -> &[T; $elem_cnt] {
                unsafe{ std::mem::transmute(self) }
            }
        
            /// Interpret a mutable reference to an vector as a mutable reference to a array
            #[inline(always)]
            #[must_use]
            pub fn as_mut_array(&mut self) -> &mut [T; $elem_cnt] {
                unsafe{ std::mem::transmute(self) }
            }

        //--------------------------------------------------------------

            /// Component-wise clamp of the vector, using scalar min and max
            pub fn clamp_scalar(self, min: T, max: T) -> Self where
                T: MinMax
            {
                Self{ $($comp: self.$comp.clamp(min, max)),+ }
            }
            /// Clamp the length of the vector
            pub fn clamp_len(self, min: T, max: T) -> Self where
                T: PartialOrd + Add<Output = T> + Mul<Output = T> + Div<Output = T> + Sqrt,
                Self: Mul<T, Output = Self>
            {
                let len = self.len();
                if len < min {
                    self * (min / len)
                } else if len > max {
                    self * (max / len)
                } else {
                    self
                }
            }

            /// Smoothstep between 2 vectors using `val`
            pub fn smoothstep(self, other: Self, val: T) -> Self where
                Self: Lerp<T>,
                T: PartialOrd + Sub<Output = T> + Mul<Output = T> + Zero + One,
                i32: NumericCast<T>,
            {
                self.lerp(other, smoothstep_interpolant(val))
            }

            /// Calculate the dot product of 2 vectors
            #[inline]
            pub fn dot(self, rhs: Self) -> T where
                T: Add<Output = T> + Mul<Output = T>
            {
                crate::utils::strip_plus!($(+ self.$comp * rhs.$comp)+)
            }

            /// Calculate the square length of the vector
            pub fn len_sq(self) -> T where
                T: Add<Output = T> + Mul<Output = T>
            {
                crate::utils::strip_plus!($(+ self.$comp * self.$comp)+)
            }

            /// Calculate the length of the vector
            pub fn len(self) -> T  where
                T: Add<Output = T> + Mul<Output = T> + Sqrt
            {
                self.len_sq().sqrt()
            }

            /// Calculate the square distance between 2 vectors
            pub fn dist_sq(self, other: Self) -> T where
                T: Add<Output = T> + Mul<Output = T>,
                Self: Sub<Output = Self>
            {
                (other - self).len_sq()
            }

            /// Calculate the distance between 2 vectors
            pub fn dist(self, other: Self) -> T where
                T: Add<Output = T> + Mul<Output = T> + Sqrt,
                Self: Sub<Output = Self>
            {
                self.dist_sq(other).sqrt()
            }

            /// Normalize the vector
            pub fn normalize(self) -> Self where
                T: Add<Output = T> + Mul<Output = T> + ApproxZero<T> + Rsqrt
            {
                if self.is_zero() {
                    self
                } else {
                    unsafe{ self.normalize_unsafe() }
                }
            }

            /// Normalize the vector (no check for a length of 0)
            pub unsafe fn normalize_unsafe(self) -> Self  where
                T: Add<Output = T> + Mul<Output = T> + Rsqrt
            {
                self * self.len_sq().rsqrt()
            }

            /// Normalize the vector if the length is not 0, return `or` otherwise
            pub fn normalize_or(self, or: Self) -> Self  where
                T: Add<Output = T> + Mul<Output = T> + ApproxZero<T> + Rsqrt
            {
                if self.is_zero() {
                    or
                } else {
                    self.normalize()
                }
            }

            /// Check if the vector is close to being normalized, using a given epsilon, which defines the max difference `len` can be relative to 1
            pub fn is_close_to_normalized(self, epsilon: T) -> bool where
                T: Add<Output = T> + Mul<Output = T> + One + ApproxEq
            {
                self.len_sq().is_close_to(T::one(), epsilon)
            }

            /// Ckeck if the vector is normalized, using the machine epsilon
            pub fn is_normalized(self) -> bool where
                T: Add<Output = T> + Mul<Output = T> + One + ApproxEq
            {
                self.len_sq().is_approx_eq(T::one())
            }

            /// Get the direction and length of the vector
            pub fn dir_and_len(self) -> (Self, T) where
                T: Add<Output = T> + Mul<Output = T> + Sqrt + Recip
            {
                let len = self.len();
                (self * len.recip(), len)
            }

            generic_vec!{ @first $($comp),* }
        }

        impl<T: Real> $name<T> {
            /// Get a vector the component-wise fractional parts
            #[inline]
            pub fn fract(self) -> Self {
                Self{ $($comp: self.$comp.fract()),+ }
            }
        }

        impl<T: Copy> Index<usize> for $name<T> {
            type Output = T;
        
            #[inline(always)]
            fn index(&self, index: usize) -> &Self::Output {
                debug_assert!(index < $elem_cnt);
                self.as_array().index(index)
            }
        }
        
        impl<T: Copy> IndexMut<usize> for $name<T> {
            #[inline(always)]
            fn index_mut(&mut self, index: usize) -> &mut Self::Output {
                debug_assert!(index < $elem_cnt);
                self.as_mut_array().index_mut(index)
            }
        }

        impl<T: Zero> Zero for $name<T> {
            fn zero() -> Self {
                Self{ $($comp: T::zero()),+ }
            }
        }

        impl<T: One> One for $name<T> {
            fn one() -> Self {
                Self{ $($comp: T::one()),+ }
            }
        }

        //------------------------------------------------------------------------------------------------------------------------------

        impl<T: Copy + Add<Output = T>> Add for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn add(self, rhs: Self) -> Self {
                Self{ $($comp: self.$comp + rhs.$comp),+ }
            }
        }

        impl<T: Copy + AddAssign> AddAssign for $name<T> {
            #[inline(always)]
            fn add_assign(&mut self, rhs: Self) {
                $(self.$comp += rhs.$comp);+
            }
        }

        impl<T: Copy + Add<Output = T>> Add<T> for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn add(self, rhs: T) -> Self {
                Self{ $($comp: self.$comp + rhs),+ }
            }
        }

        impl<T: Copy + AddAssign> AddAssign<T> for $name<T> {
            #[inline(always)]
            fn add_assign(&mut self, rhs: T) {
                $(self.$comp += rhs);+
            }
        }

        //--------------------------------------------------------------

        impl<T: Copy + Sub<Output = T>> Sub for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn sub(self, rhs: Self) -> Self {
                Self{ $($comp: self.$comp - rhs.$comp),+ }
            }
        }

        impl<T: Copy + SubAssign> SubAssign for $name<T> {
            #[inline(always)]
            fn sub_assign(&mut self, rhs: Self) {
                $(self.$comp -= rhs.$comp);+
            }
        }

        impl<T: Copy + Sub<Output = T>> Sub<T> for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn sub(self, rhs: T) -> Self {
                Self{ $($comp: self.$comp - rhs),+ }
            }
        }

        impl<T: Copy + SubAssign> SubAssign<T> for $name<T> {
            #[inline(always)]
            fn sub_assign(&mut self, rhs: T) {
                $(self.$comp -= rhs);+
            }
        }

        //--------------------------------------------------------------

        impl<T: Copy + Mul<Output = T>> Mul for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn mul(self, rhs: Self) -> Self {
                Self{ $($comp: self.$comp * rhs.$comp),+ }
            }
        }

        impl<T: Copy + MulAssign> MulAssign for $name<T> {
            #[inline(always)]
            fn mul_assign(&mut self, rhs: Self) {
                $(self.$comp *= rhs.$comp);+
            }
        }

        impl<T: Copy + Mul<Output = T>> Mul<T> for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn mul(self, rhs: T) -> Self {
                Self{ $($comp: self.$comp * rhs),+ }
            }
        }

        impl<T: Copy + MulAssign> MulAssign<T> for $name<T> {
            #[inline(always)]
            fn mul_assign(&mut self, rhs: T) {
                $(self.$comp *= rhs);+
            }
        }

        // We can't implement pre-multipy genericly here, so `impl_vec_premul` is used instead

        //--------------------------------------------------------------

        impl<T: Copy + Div<Output = T>> Div for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn div(self, rhs: Self) -> Self {
                Self{ $($comp: self.$comp / rhs.$comp),+ }
            }
        }

        impl<T: Copy + DivAssign> DivAssign for $name<T> {
            #[inline(always)]
            fn div_assign(&mut self, rhs: Self) {
                $(self.$comp /= rhs.$comp);+
            }
        }

        impl<T: Copy + Div<Output = T>> Div<T> for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn div(self, rhs: T) -> Self {
                Self{ $($comp: self.$comp / rhs),+ }
            }
        }

        impl<T: Copy + DivAssign> DivAssign<T> for $name<T> {
            #[inline(always)]
            fn div_assign(&mut self, rhs: T) {
                $(self.$comp /= rhs);+
            }
        }

        //--------------------------------------------------------------

        impl<T: Copy + Rem<Output = T>> Rem for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn rem(self, rhs: Self) -> Self {
                Self{ $($comp: self.$comp % rhs.$comp),+ }
            }
        }

        impl<T: Copy + RemAssign> RemAssign for $name<T> {
            #[inline(always)]
            fn rem_assign(&mut self, rhs: Self) {
                $(self.$comp %= rhs.$comp);+
            }
        }

        impl<T: Copy + Rem<Output = T>> Rem<T> for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn rem(self, rhs: T) -> Self {
                Self{ $($comp: self.$comp % rhs),+ }
            }
        }

        impl<T: Copy + RemAssign> RemAssign<T> for $name<T> {
            #[inline(always)]
            fn rem_assign(&mut self, rhs: T) {
                $(self.$comp %= rhs);+
            }
        }

        //------------------------------------------------------------------------------------------------------------------------------

        impl<T: Copy + Neg<Output = T>> Neg for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn neg(self) -> Self {
                Self{ $($comp: -self.$comp),+ }
            }
        }

        impl<T: Copy + Not<Output = T>> Not for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn not(self) -> Self {
                Self{ $($comp: !self.$comp),+ }
            }
        }

        //--------------------------------------------------------------

        impl<T: Copy + BitAnd<Output = T>> BitAnd for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn bitand(self, rhs: Self) -> Self {
                Self{ $($comp: self.$comp & rhs.$comp),+ }
            }
        }

        impl<T: Copy + BitAndAssign> BitAndAssign for $name<T> {
            #[inline(always)]
            fn bitand_assign(&mut self, rhs: Self) {
                $(self.$comp &= rhs.$comp);+
            }
        }

        impl<T: Copy + BitAnd<Output = T>> BitAnd<T> for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn bitand(self, rhs: T) -> Self {
                Self{ $($comp: self.$comp & rhs),+ }
            }
        }

        impl<T: Copy + BitAndAssign> BitAndAssign<T> for $name<T> {
            #[inline(always)]
            fn bitand_assign(&mut self, rhs: T) {
                $(self.$comp &= rhs);+
            }
        }

        //--------------------------------------------------------------

        impl<T: Copy + BitXor<Output = T>> BitXor for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn bitxor(self, rhs: Self) -> Self {
                Self{ $($comp: self.$comp ^ rhs.$comp),+ }
            }
        }

        impl<T: Copy + BitXorAssign> BitXorAssign for $name<T> {
            #[inline(always)]
            fn bitxor_assign(&mut self, rhs: Self) {
                $(self.$comp ^= rhs.$comp);+
            }
        }

        impl<T: Copy + BitXor<Output = T>> BitXor<T> for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn bitxor(self, rhs: T) -> Self {
                Self{ $($comp: self.$comp ^ rhs),+ }
            }
        }

        impl<T: Copy + BitXorAssign> BitXorAssign<T> for $name<T> {
            #[inline(always)]
            fn bitxor_assign(&mut self, rhs: T) {
                $(self.$comp ^= rhs);+
            }
        }

        //--------------------------------------------------------------

        impl<T: Copy + BitOr<Output = T>> BitOr for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn bitor(self, rhs: Self) -> Self {
                Self{ $($comp: self.$comp | rhs.$comp),+ }
            }
        }

        impl<T: Copy + BitOrAssign> BitOrAssign for $name<T> {
            #[inline(always)]
            fn bitor_assign(&mut self, rhs: Self) {
                $(self.$comp |= rhs.$comp);+
            }
        }

        impl<T: Copy + BitOr<Output = T>> BitOr<T> for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn bitor(self, rhs: T) -> Self {
                Self{ $($comp: self.$comp | rhs),+ }
            }
        }

        impl<T: Copy + BitOrAssign> BitOrAssign<T> for $name<T> {
            #[inline(always)]
            fn bitor_assign(&mut self, rhs: T) {
                $(self.$comp |= rhs);+
            }
        }

        //--------------------------------------------------------------

        impl<T: Copy + Shl<Output = T>> Shl for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn shl(self, rhs: Self) -> Self {
                Self{ $($comp: self.$comp << rhs.$comp),+ }
            }
        }

        impl<T: Copy + ShlAssign> ShlAssign for $name<T> {
            #[inline(always)]
            fn shl_assign(&mut self, rhs: Self) {
                $(self.$comp <<= rhs.$comp);+
            }
        }

        impl<T: Copy + Shl<Output = T>> Shl<T> for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn shl(self, rhs: T) -> Self {
                Self{ $($comp: self.$comp << rhs),+ }
            }
        }

        impl<T: Copy + ShlAssign> ShlAssign<T> for $name<T> {
            #[inline(always)]
            fn shl_assign(&mut self, rhs: T) {
                $(self.$comp <<= rhs);+
            }
        }

        //--------------------------------------------------------------

        impl<T: Copy + Shr<Output = T>> Shr for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn shr(self, rhs: Self) -> Self {
                Self{ $($comp: self.$comp >> rhs.$comp),+ }
            }
        }

        impl<T: Copy + ShrAssign> ShrAssign for $name<T> {
            #[inline(always)]
            fn shr_assign(&mut self, rhs: Self) {
                $(self.$comp >>= rhs.$comp);+
            }
        }

        impl<T: Copy + Shr<Output = T>> Shr<T> for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn shr(self, rhs: T) -> Self {
                Self{ $($comp: self.$comp >> rhs),+ }
            }
        }

        impl<T: Copy + ShrAssign> ShrAssign<T> for $name<T> {
            #[inline(always)]
            fn shr_assign(&mut self, rhs: T) {
                $(self.$comp >>= rhs);+
            }
        }

        //------------------------------------------------------------------------------------------------------------------------------

        impl<T: MinMax> MinMax for $name<T> {
            fn min(self, rhs: Self) -> Self {
                Self { $($comp: self.$comp.min(rhs.$comp)),* }
            }

            fn max(self, rhs: Self) -> Self {
                Self { $($comp: self.$comp.max(rhs.$comp)),* }
            }

            fn clamp(self, min: Self, max: Self) -> Self {
                Self { $($comp: self.$comp.clamp(min.$comp, max.$comp)),* }
            }
        }

        impl<T: Saturate> Saturate for $name<T> {
            fn saturate(self) -> Self {
                Self { $($comp: self.$comp.saturate()),* }
            }
        }

        impl<T: Abs> Abs for $name<T> {
            fn abs(self) -> Self {
                Self { $($comp: self.$comp.abs()),* }
            }
        }
        
        impl<T: AbsDiff> AbsDiff for $name<T> {
            type Output = $name<<T as AbsDiff>::Output>;

            fn abs_diff(self, rhs: Self) -> Self::Output {
                $name { $($comp: self.$comp.abs_diff(rhs.$comp)),* }
            }
        }

        impl<T: Sign> Sign for $name<T> {
            fn sign(self) -> Self {
                Self { $($comp: self.$comp.sign()),* }
            }
        }

        impl<T: Sqrt> Sqrt for $name<T> {
            fn sqrt(self) -> Self {
                Self { $($comp: self.$comp.sqrt()),* }
            }
        }

        impl<T: Rsqrt> Rsqrt for $name<T> {
            fn rsqrt(self) -> Self {
                Self { $($comp: self.$comp.rsqrt()),* }
            }
        }

        impl<T: Recip> Recip for $name<T> {
            fn recip(self) -> Self {
                Self { $($comp: self.$comp.recip()),* }
            }
        }

        impl<T: Snap> Snap<T> for $name<T> {
            fn snap(self, step: T) -> Self {
                Self { $($comp: self.$comp.snap(step)),* }
            }
        }

        impl<T: Lerp> Lerp<T> for $name<T> {
            fn lerp(self, other: Self, interp: T) -> Self {
                Self { $($comp: self.$comp.lerp(other.$comp, interp)),* }
            }
        }

        impl<T: Round> Round for $name<T> {
            fn round(self) -> Self {
                Self { $($comp: self.$comp.round()),* }
            }
            
            fn ceil(self) -> Self {
                Self { $($comp: self.$comp.ceil()),* }
            }
            
            fn floor(self) -> Self {
                Self { $($comp: self.$comp.floor()),* }
            }
        }

        impl<T: Fract> Fract for $name<T> {
            fn fract(self) -> Self {
                Self { $($comp: self.$comp.fract()),* }
            }
        }

        impl<T: Trig> Trig for $name<T> {
            type Output = $name<<T as Trig>::Output>;

            fn sin(self) -> Self::Output {
                $name { $($comp: self.$comp.sin()),* }
            }

            fn cos(self) -> Self::Output {
                $name { $($comp: self.$comp.cos()),* }
            }

            fn sin_cos(self) -> (Self::Output, Self::Output) {
                $(
                    let $comp = self.$comp.sin_cos();
                )*
                (
                    $name { $($comp: $comp.0),* },
                    $name { $($comp: $comp.1),* }
                )
            }

            fn tan(self) -> Self::Output  {
                $name { $($comp: self.$comp.tan()),* }
            }

            fn sinh(self) -> Self::Output {
                $name { $($comp: self.$comp.sinh()),* }
            }

            fn cosh(self) -> Self::Output {
                $name { $($comp: self.$comp.cosh()),* }
            }

            fn tanh(self) -> Self::Output {
                $name { $($comp: self.$comp.tanh()),* }
            }
        }

        
        impl<T: InvTrig<U>, U: Copy> InvTrig<$name<U>> for $name<T> {
            fn asin(val: $name<U>) -> Self{
                Self { $($comp: T::asin(val.$comp)),* }
            }

            fn acos(val: $name<U>) -> Self{
                Self { $($comp: T::acos(val.$comp)),* }
            }

            fn atan(val: $name<U>) -> Self{
                Self { $($comp: T::atan(val.$comp)),* }
            }

            fn atan2(y: $name<U>, x: $name<U>) -> Self{
                Self { $($comp: T::atan2(y.$comp, x.$comp)),* }
            }

            fn asinh(val: $name<U>) -> Self{
                Self { $($comp: T::asinh(val.$comp)),* }
            }

            fn acosh(val: $name<U>) -> Self{
                Self { $($comp: T::acosh(val.$comp)),* }
            }

            fn atanh(val: $name<U>) -> Self{
                Self { $($comp: T::atanh(val.$comp)),* }
            }

        }

        //--------------------------------------------------------------


        impl<T: ApproxEq> ApproxEq<T> for $name<T> {
            const EPSILON: T = T::EPSILON;
        
            fn is_close_to(self, rhs: Self, epsilon: T) -> bool {
                $(self.$comp.is_close_to(rhs.$comp, epsilon))||+
            }
        }
        
        impl<T: ApproxZero> ApproxZero<T> for $name<T> {
            const ZERO_EPSILON: T = T::ZERO_EPSILON;
        
            fn is_close_to_zero(self, epsilon: T) -> bool {
                $(self.$comp.is_close_to_zero(epsilon))||+
            }
        }
        
        //------------------------------------------------------------------------------------------------------------------------------

        impl<T: Copy> From<[T; $elem_cnt]> for $name<T> {
            fn from(arr: [T; $elem_cnt]) -> Self {
                Self::from_array(arr)
            }
        }

        impl<T: Copy> From<$name<T>> for [T; $elem_cnt] {
            fn from(v: $name<T>) -> Self {
                v.to_array()
            }
        }
        
        //------------------------------------------------------------------------------------------------------------------------------

        impl<T: NumericCast<U>, U: Copy> NumericCast<$name<U>> for $name<T>
        where
            T : NumericCast<U>
        {
            fn cast(self) -> $name<U> {
                $name{ $($comp: self.$comp.cast()),+ }
            }
        }

        //------------------------------------------------------------------------------------------------------------------------------

        $(
            #[allow(non_camel_case_types)]
            pub type $alias_ty = $name<$base_ty>;
        )*
    };
    (@first $comp0:ident, $($comp:ident),*) => {
        /// Check if all elements are approximately equal, given an epsilon
        pub fn is_uniform(self, epsilon: T) -> bool where
            T: ApproxEq
        {
            $(self.$comp0.is_close_to(self.$comp, epsilon))||*
        }

        /// Get the minimum component of the vector
        pub fn min_component(self) -> T where
            T: MinMax
        {
            self.$comp0$(.min(self.$comp))*
        }

        /// Get the minimum absolute component of the vector
        pub fn min_abs_component(self) -> T where
            T: MinMax + Abs
        {
            self.x.abs().min(self.y.abs())
        }

        /// Get the maximum component of the vector
        pub fn max_component(self) -> T where
            T: MinMax
        {
            self.x.max(self.y)
        }

        /// Get the maximum absolute component of the vector
        pub fn max_abs_component(self) -> T where
            T: MinMax + Abs
        {
            self.x.abs().max(self.y.abs())
        }
    }
}
pub(crate) use generic_vec;


generic_vec!{ doc = "3D Vector (row-major order)"; Vec3, 3, x, y, z; }
generic_vec!{ doc = "4D Vector (row-major order)"; Vec4, 4, x, y, z, w; }

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

pub const SWIZZLE_X : u8 = 0;
pub const SWIZZLE_Y : u8 = 1;
pub const SWIZZLE_Z : u8 = 2;
pub const SWIZZLE_W : u8 = 3;

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