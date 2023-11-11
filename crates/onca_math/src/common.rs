
/// A trait to calculate the dot product of 2 elements
pub trait DotProduct<U: Copy>: Copy
{
    type Output;

    /// Calculate the dot product between 2 elements
    /// 
    /// The dot product `⋅` has the following properties:
    /// ```text
    ///       (u ⋅ v) = (v ⋅ u)
    ///      (su ⋅ v) = s(u ⋅ v)
    /// (u ⋅ (v + w)) = (u ⋅ v) + (u ⋅ w)
    /// ```
    /// 
    /// The dot product `⋅` also represent the relationship to the angle between the 2 vectors
    /// 
    /// `(v ⋅ w) = ||v|| ||w|| sin θ`
    /// 
    /// where `θ` is he angle between `v` and `w`, and `||v||` and `||w||` represent the length of `v` and `w` respectively
    fn dot(self, rhs: U) -> Self::Output;
}


macro_rules! tuple_common {
    (
        $docs:meta;
        $iden:ident,
        $elem_cnt:literal,
        $tup_ty:ty,
        $($comp:ident => $idx:tt),+;
        $($alias:ident => $alias_base:ty)*
    ) => {
        #[$docs]
        #[derive(Clone, Copy, PartialEq, Debug)]
        #[repr(C)]
        pub struct $iden<T: Numeric> {
            $(pub $comp: T),*
        }

        impl<T: Numeric> $iden<T> {
            #[doc = concat!("Create a new ", stringify!($iden), ".")]
            #[inline(always)]
            #[must_use]
            pub fn new($($comp: T),*) -> Self {
                Self { $($comp: $comp),+ }
            }

            #[doc = concat!("Create a ", stringify!($iden), " with all components set to `val`.")]
            #[inline(always)]
            #[must_use]
            pub fn set(val: T) -> Self {
                Self { $($comp: val),+ }
            }

            //--------------------------------------------------------------

            #[doc = concat!("Create a ", stringify!($iden), " from an array.")]
            #[inline(always)]
            #[must_use]
            pub fn from_array(arr: [T; $elem_cnt]) -> Self {
                Self { $($comp: arr[$idx]),+ }
            }

            #[doc = concat!("Create a reference to a ", stringify!($iden), " from a reference to an array")]
            #[inline(always)]
            #[must_use]
            pub fn ref_from_array(arr: &[T; $elem_cnt]) -> &Self {
                unsafe { std::mem::transmute(arr) }
            }

            #[doc = concat!("Create a mutable reference to a ", stringify!($iden), " from a mutalbe reference to an array")]
            #[inline(always)]
            #[must_use]
            pub fn mut_ref_from_array(arr: &mut [T; $elem_cnt]) -> &mut Self {
                unsafe { std::mem::transmute(arr) }
            }

            //------------------------------

            #[doc = concat!("Create a ", stringify!($iden), " from a tuple.")]
            #[inline(always)]
            #[must_use]
            pub fn from_tuple(tup: $tup_ty) -> Self {
                Self { $($comp: tup.$idx),+ }
            }

            #[doc = concat!("Create a reference to a ", stringify!($iden), " from a reference to an array")]
            #[inline(always)]
            #[must_use]
            pub fn ref_from_tuple(arr: &$tup_ty) -> &Self {
                unsafe { std::mem::transmute(arr) }
            }

            #[doc = concat!("Create a mutable reference to a ", stringify!($iden), " from a mutalbe reference to an array")]
            #[inline(always)]
            #[must_use]
            pub fn mut_ref_from_tuple(arr: &mut $tup_ty) -> &mut Self {
                unsafe { std::mem::transmute(arr) }
            }

            //------------------------------

            #[doc = concat!("Get the contents from the ", stringify!($iden), " as an array")]
            #[inline(always)]
            #[must_use]
            pub fn to_array(self) -> [T; $elem_cnt] {
                [$(self.$comp),+]
            }

            #[doc = concat!("Get a reference to the ", stringify!($iden), " as an array")]
            #[inline(always)]
            #[must_use]
            pub fn as_array(&self) -> &[T; $elem_cnt] {
                unsafe { std::mem::transmute(self) }
            }

            #[doc = concat!("Get a mutable reference to the ", stringify!($iden), " as an array")]
            #[inline(always)]
            #[must_use]
            pub fn as_mut_array(&mut self) -> &mut [T; $elem_cnt] {
                unsafe { std::mem::transmute(self) }
            }

            //------------------------------

            #[doc = concat!("Get the contents from the ", stringify!($iden), " as an tuple")]
            #[inline(always)]
            #[must_use]
            pub fn to_tuple(self) -> $tup_ty {
                ($(self.$comp),+)
            }

            #[doc = concat!("Get a reference to the ", stringify!($iden), " as an tuple")]
            #[inline(always)]
            #[must_use]
            pub fn as_tuple(&self) -> &$tup_ty {
                unsafe { std::mem::transmute(self) }
            }

            #[doc = concat!("Get a mutable reference to the ", stringify!($iden), " as an tuple")]
            #[inline(always)]
            #[must_use]
            pub fn as_mut_tuple(&mut self) -> &mut $tup_ty {
                unsafe { std::mem::transmute(self) }
            }

            $crate::common::tuple_common!{ @first $($comp),+ }
        }

        //--------------------------------------------------------------

        impl<T: Numeric> From<[T; $elem_cnt]> for $iden<T> {
            fn from(arr: [T; $elem_cnt]) -> Self {
                Self::from_array(arr)
            }
        }

        impl<T: Numeric> From<$iden<T>> for [T; $elem_cnt] {
            fn from(val: $iden<T>) -> Self {
                val.to_array()
            }
        }

        impl<T: Numeric> From<$tup_ty> for $iden<T> {
            fn from(tup: $tup_ty) -> Self {
                Self::from_tuple(tup)
            }
        }

        impl<T: Numeric> From<$iden<T>> for $tup_ty {
            fn from(val: $iden<T>) -> $tup_ty {
                val.to_tuple()
            }
        }

        //--------------------------------------------------------------

        impl<T: Numeric> Index<usize> for $iden<T> {
            type Output = T;

            fn index(&self, index: usize) -> &Self::Output {
                &self.as_array()[index]
            }
        }

        impl<T: Numeric> IndexMut<usize> for $iden<T> {
            fn index_mut(&mut self, index: usize) -> &mut Self::Output {
                &mut self.as_mut_array()[index]
            }
        }
        
        impl<T: Numeric> Zero for $iden<T> {
            fn zero() -> Self {
                Self{ $($comp: T::zero()),+ }
            }
        }

        //--------------------------------------------------------------
        
        impl<T: Signed> Neg for $iden<T> {
            type Output = Self;

            #[inline(always)]
            fn neg(self) -> Self {
                Self{ $($comp: -self.$comp),+ }
            }
        }

        //--------------------------------------------------------------

        impl<T: Numeric> MinMax for $iden<T> {
            fn min(self, rhs: Self) -> Self {
                Self { $($comp: self.$comp.min(rhs.$comp)),* }
            }

            fn max(self, rhs: Self) -> Self {
                Self { $($comp: self.$comp.max(rhs.$comp)),* }
            }
        }

        impl<T: Numeric> Clamp for $iden<T> {
            #[doc = concat!("Clamp `", stringify!($iden), "` component-wise.")]
            fn clamp(self, min: Self, max: Self) -> Self {
                Self { $($comp: self.$comp.clamp(min.$comp, max.$comp)),* }
            }
        }
        
        impl<T: Numeric> Clamp<T> for $iden<T> {
            #[doc = concat!("Clamp `", stringify!($iden), "` using scalars.")]
            fn clamp(self, min: T, max: T) -> Self {
                Self { $($comp: self.$comp.clamp(min, max)),* }
            }
        }

        impl<T: Numeric> Abs for $iden<T> {
            fn abs(self) -> Self {
                Self { $($comp: self.$comp.abs()),+ }
            }
        }

        impl<T: Numeric> Lerp<T> for $iden<T> {
            fn lerp(self, other: Self, interp: T) -> Self {
                Self { $($comp: self.$comp.lerp(other.$comp, interp)),+ }
            }
        }

        impl<T: Real> Round for $iden<T> {
            fn round(self) -> Self {
                Self { $($comp: self.$comp.round()),+ }
            }

            fn ceil(self) -> Self {
                Self { $($comp: self.$comp.ceil()),+ }
            }

            fn floor(self) -> Self {
                Self { $($comp: self.$comp.floor()),+ }
            }
        }

        impl<T: Real> Fract for $iden<T> {
            fn fract(self) -> Self {
                Self { $($comp: self.$comp.fract()),* }
            }
        }

        impl<T: Real> Trunc for $iden<T> {
            fn trunc(self) -> Self {
                Self { $($comp: self.$comp.trunc()),* }
            }
        }

        //--------------------------------------------------------------

        impl<T: Numeric> ApproxEq<T> for $iden<T> {
            const EPSILON: T = T::EPSILON;

            fn is_close_to(self, rhs: Self, epsilon: T) -> bool {
                $(self.$comp.is_close_to(rhs.$comp, epsilon))&&+
            }
        }

        impl<T: Numeric> ApproxZero<T> for $iden<T> {
            fn is_close_to_zero(self, epsilon: T) -> bool {
                $(self.$comp.is_close_to_zero(epsilon))&&+
            }
        }
        
        //--------------------------------------------------------------

        impl<T: Numeric + NumericCast<U>, U: Numeric> NumericCast<$iden<U>> for $iden<T> {
            fn cast(self) -> $iden<U> {
                $iden { $($comp: self.$comp.cast()),* }
            }
        }

        impl<T: Numeric> $iden<T> {
            /// Get the index of the component with the minimum value
            pub fn min_component_idx(self) -> u8 {
                tuple_common!(@idx <, self, $($comp => $idx),+)
            }

            /// Get the index of the component with the maximum value
            pub fn max_component_idx(self) -> u8 {
                tuple_common!(@idx >, self, $($comp => $idx),+)
            }

            /// Get the index of the component with the minimum absolute value
            pub fn min_abs_component_idx(self) -> u8 {
                tuple_common!(@idx_abs <, self, $($comp => $idx),+)
            }

            /// Get the index of the component with the maximum absolute value
            pub fn max_abs_component_idx(self) -> u8 {
                tuple_common!(@idx_abs >, self, $($comp => $idx),+)
            }
        }

        //--------------------------------------------------------------

        $(
            #[allow(non_camel_case_types)]
            pub type $alias = $iden<$alias_base>;
        )*
    };
    (@first $comp0:ident, $($comp:ident),*) => {
        /// Check if all elements are approximately equal, given an epsilon.
        #[must_use]
        pub fn is_close_to_uniform(self, epsilon: T) -> bool {
            $(self.$comp0.is_close_to(self.$comp, epsilon))||*
        }

        /// Check if all elements are approximately equal, given an epsilon.
        #[must_use]
        pub fn is_uniform(self) -> bool {
            $(self.$comp0.is_approx_eq(self.$comp))||*
        }

        /// Get the minimum component.
        #[must_use]
        pub fn min_component(self) -> T {
            self.$comp0$(.min(self.$comp))*
        }

        /// Get the minimum absolute component.
        #[must_use]
        pub fn min_abs_component(self) -> T {
            self.x.abs().min(self.y.abs())
        }

        /// Get the maximum component.
        #[must_use]
        pub fn max_component(self) -> T {
            self.x.max(self.y)
        }

        /// Get the maximum absolute component.
        #[must_use]
        pub fn max_abs_component(self) -> T {
            self.x.abs().max(self.y.abs())
        }
    };
    (@idx $op:tt, $self:expr, $comp0:ident => $idx0:tt, $($comp:ident => $idx:tt),+) => {
        if $($self.$comp0 $op $self.$comp)&&+ {
            $idx0
        } else {
            tuple_common!(@idx $op, $self, $($comp => $idx),+)
        }
    };
    (@idx $op:tt, $self:expr, $comp0:ident => $idx0:tt) => { 
        $idx0
    };
    (@idx_abs $op:tt, $self:expr, $comp0:ident => $idx0:tt, $($comp:ident => $idx:tt),+) => {
        if $($self.$comp0.abs() $op $self.$comp.abs())&&+ {
            $idx0
        } else {
            tuple_common!(@idx_abs $op, $self, $($comp => $idx),+)
        }
    };
    (@idx_abs $op:tt, $self:expr, $comp0:ident => $idx0:tt) => { 
        $idx0
    };
    (@add_sub_self $iden:ident, $other:ident, $($comp:ident),+) => {
        impl<T: Numeric> Add<$other<T>> for $iden<T> {
            type Output = Self;

            fn add(self, other: $other<T>) -> Self {
                Self { $($comp: self.$comp + other.$comp),+ }
            }
        }
        
        impl<T: Numeric> AddAssign<$other<T>> for $iden<T> {
            #[inline(always)]
            fn add_assign(&mut self, rhs: $other<T>) {
                $(self.$comp += rhs.$comp);+
            }
        }

        //--------------------------------------------------------------

        impl<T: Numeric> Sub<$other<T>> for $iden<T> {
            type Output = Self;

            #[inline(always)]
            fn sub(self, rhs: $other<T>) -> Self {
                Self{ $($comp: self.$comp - rhs.$comp),+ }
            }
        }

        impl<T: Numeric> SubAssign<$other<T>> for $iden<T> {
            #[inline(always)]
            fn sub_assign(&mut self, rhs: $other<T>) {
                $(self.$comp -= rhs.$comp);+
            }
        }

    };
    (@scale $iden:ident, $($comp:ident),+) => {
        impl<T: Numeric> Mul<T> for $iden<T> {
            type Output = Self;

            #[inline(always)]
            fn mul(self, rhs: T) -> Self {
                Self{ $($comp: self.$comp * rhs),+ }
            }
        }

        impl<T: Numeric> MulAssign<T> for $iden<T> {
            #[inline(always)]
            fn mul_assign(&mut self, rhs: T) {
                $(self.$comp *= rhs);+
            }
        }

        impl<T: Numeric> Div<T> for $iden<T> {
            type Output = Self;

            #[inline(always)]
            fn div(self, rhs: T) -> Self {
                Self{ $($comp: self.$comp / rhs),+ }
            }
        }

        impl<T: Numeric> DivAssign<T> for $iden<T> {
            #[inline(always)]
            fn div_assign(&mut self, rhs: T) {
                $(self.$comp /= rhs);+
            }
        }
    };
    (@len_and_normalize $iden:ident, $($comp:ident),+) => {
        impl<T: Numeric> $iden<T> {

            #[doc = concat!("Calculate the square of the lenght of the `", stringify!($iden), "`.")]
            #[must_use]
            pub fn len_sq(self) -> T where
                T: Add<Output = T> + Mul<Output = T>
            {
                crate::utils::strip_plus!($(+ self.$comp * self.$comp)+)
            }

            #[doc = concat!("Calculate the lenght of the `", stringify!($iden), "`.")]
            #[must_use]
            pub fn len(self) -> T  where
                T: Add<Output = T> + Mul<Output = T> + Sqrt
            {
                self.len_sq().sqrt()
            }

            /// Get a normalized version the vector.
            #[must_use]
            pub fn normalize(self) -> Self where
                T: Add<Output = T> + Mul<Output = T> + ApproxZero<T> + Rsqrt
            {
                if self.is_zero() {
                    self
                } else {
                    unsafe{ self.normalize_unsafe() }
                }
            }

            /// Normalize the vector (no check for a length of 0).
            #[must_use]
            pub unsafe fn normalize_unsafe(self) -> Self  where
                T: Add<Output = T> + Mul<Output = T> + Rsqrt
            {
                self * self.len_sq().rsqrt()
            }

            /// Normalize the vector if the length is not 0, return `or` otherwise.
            #[must_use]
            pub fn normalize_or(self, or: Self) -> Self  where
                T: Add<Output = T> + Mul<Output = T> + ApproxZero<T> + Rsqrt
            {
                if self.is_zero() {
                    or
                } else {
                    self.normalize()
                }
            }

            /// Check if the vector is close to being normalized, using a given epsilon, which defines the max difference `len` can be relative to 1.
            pub fn is_close_to_normalized(self, epsilon: T) -> bool where
                T: Add<Output = T> + Mul<Output = T> + One + ApproxEq
            {
                self.len_sq().is_close_to(T::one(), epsilon)
            }

            /// Ckeck if the vector is normalized, using the machine epsilon.
            pub fn is_normalized(self) -> bool where
                T: Add<Output = T> + Mul<Output = T> + One + ApproxEq
            {
                self.len_sq().is_approx_eq(T::one())
            }
        }
    };
    (@dot $iden:ident, $other:ident, $($comp:ident),+) => {
        impl<T: Numeric> DotProduct<$other<T>> for $iden<T> {
            type Output = T;

            fn dot(self, rhs: $other<T>) -> Self::Output {
                $crate::utils::strip_plus!($(+ self.$comp * rhs.$comp)+)
            }
        }
    };
    (@snap_scalar $iden:ident, $($comp:ident),+) => {
        impl<T: Numeric> Snap<T> for $iden<T> {
            fn snap(self, step: T) -> Self {
                Self { $($comp: self.$comp.snap(step)),* }
            }
        }

    };
    (@snap_comp $iden:ident, $other:ident, $($comp:ident),+) => {
        impl<T: Numeric> Snap<$other<T>> for $iden<T> {
            fn snap(self, step: $other<T>) -> Self {
                Self { $($comp: self.$comp.snap(step.$comp)),* }
            }
        }
    };
}
pub(crate) use tuple_common;

// Some tests to make sure no errors were made in the macro implementation
#[cfg(test)]
mod tests {
    use std::ops::*;
    use crate::*;

    super::tuple_common!{
        doc = "";
        Tup2_,
        2,
        (T, T),
        x => 0, y => 1;
    }

    super::tuple_common!{
        doc = "";
        Tup2,
        2,
        (T, T),
        x => 0, y => 1;
    }
    super::tuple_common!{ @add_sub_self Tup2, Tup2, x, y }
    super::tuple_common!{ @scale Tup2, x, y }
    super::tuple_common!{ @len_and_normalize Tup2, x, y }
    super::tuple_common!{ @dot Tup2, Tup2, x, y }

    #[test]
    fn test_create() {
        let mut arr = [1, 2];
        let mut tup = (1, 2);

        let t = Tup2 { x: 1, y: 2 };
        assert_eq!(t.x, 1);
        assert_eq!(t.y, 2);

        let t = Tup2::new(1, 2);
        assert_eq!(t.x, 1);
        assert_eq!(t.y, 2);

        let t = Tup2::set(1);
        assert_eq!(t.x, 1);
        assert_eq!(t.y, 1);

        let t = Tup2::from_array(arr);
        assert_eq!(t.x, 1);
        assert_eq!(t.y, 2);

        let t = Tup2::ref_from_array(&arr);
        assert_eq!(t.x, 1);
        assert_eq!(t.y, 2);

        let t = Tup2::mut_ref_from_array(&mut arr);
        assert_eq!(t.x, 1);
        assert_eq!(t.y, 2);

        let t = Tup2::from_tuple(tup);
        assert_eq!(t.x, 1);
        assert_eq!(t.y, 2);

        let t = Tup2::ref_from_tuple(&tup);
        assert_eq!(t.x, 1);
        assert_eq!(t.y, 2);

        let t = Tup2::mut_ref_from_tuple(&mut tup);
        assert_eq!(t.x, 1);
        assert_eq!(t.y, 2);
        

        let mut t = Tup2 { x: 1, y: 2 };
        let r = t.to_array();
        assert_eq!(r[0], 1);
        assert_eq!(r[1], 2);

        let r: [i32; 2] = t.into();
        assert_eq!(r[0], 1);
        assert_eq!(r[1], 2);

        let r = t.as_array();
        assert_eq!(r[0], 1);
        assert_eq!(r[1], 2);
        
        let r = t.as_mut_array();
        assert_eq!(r[0], 1);
        assert_eq!(r[1], 2);
        
        
        let r = t.to_tuple();
        assert_eq!(r.0, 1);
        assert_eq!(r.1, 2);

        let r: (i32, i32) = t.into();
        assert_eq!(r.0, 1);
        assert_eq!(r.1, 2);

        let r = t.as_tuple();
        assert_eq!(r.0, 1);
        assert_eq!(r.1, 2);
        
        let r = t.as_mut_tuple();
        assert_eq!(r.0, 1);
        assert_eq!(r.1, 2);
    }

    #[test]
    fn test_cmp() {
        let a = Tup2::new(1, 2);
        let b = Tup2::new(2, 3);
        
        assert!(!a.is_approx_eq(b));
        assert!(a.is_approx_eq(a));
        assert!(a.is_close_to(b, 1));

        assert!(!a.is_uniform());
        assert!(a.is_close_to_uniform(1));
        
        let a = Tup2::set(0);
        let b = Tup2::set(1);
        assert!(a.is_zero());
        assert!(!b.is_zero());
        assert!(b.is_close_to_zero(1));

        assert!(a.is_uniform());
    }

    #[test]
    fn test_common_ops() {
        let mut a = Tup2::new(1, 2);
        let b = Tup2::new(-1, 4);
        let c = Tup2::new(2, 3);
        let d = Tup2::new(1, -4);

        assert_eq!(a[0], 1);
        assert_eq!(a[1], 2);
        assert_eq!(*&mut a[0], 1);
        assert_eq!(*&mut a[1], 2);

        assert_eq!(a.min(b), Tup2::new(-1, 2));
        assert_eq!(a.max(b), Tup2::new(1, 4));

        assert_eq!(b.min_component(), -1);
        assert_eq!(b.max_component(), 4);

        assert_eq!(b.min_abs_component(), 1);
        assert_eq!(d.max_abs_component(), 4);

        assert_eq!(b.clamp(a, c), Tup2::new(1, 3));

        let a = Tup2::new(1.0, 2.0);
        let b = Tup2::new(0.0, 3.0);
        assert_eq!(a.lerp(b, 0.25), Tup2::new(0.75, 2.25));
    }

    #[test]
    fn test_add_sub() {
        let mut a = Tup2::new(1, 2);
        let b = Tup2::new(3, 5);

        assert_eq!(a + b, Tup2::new(4, 7));
        assert_eq!(a - b, Tup2::new(-2, -3));

        a += b;
        assert_eq!(a, Tup2::new(4, 7));

        a -= b;
        assert_eq!(a, Tup2::new(1, 2));
    }

    #[test]
    fn test_scale() {
        let mut a = Tup2::new(2, 4);

        assert_eq!(a * 2, Tup2::new(4, 8));
        assert_eq!(a / 2, Tup2::new(1, 2));

        a *= 2;
        assert_eq!(a, Tup2::new(4, 8));
        
        a /= 4;
        assert_eq!(a, Tup2::new(1, 2));
    }

    #[test]
    fn test_len_and_normalize() {
        let t0 = Tup2::new(3f32, 4f32); // len == 5
        let t1 = Tup2::new(5f32, 12f32); // len == 13
        let t2 = Tup2::new(0.6f32, 0.8f32); // len == 1

        assert_eq!(t0.normalize(), t2);
        assert_eq!(Tup2::set(0f32).normalize(),Tup2::set(0f32));

        assert_eq!(t0.normalize_or(t1), t2);
        assert_eq!(Tup2::set(0f32).normalize_or(t1), t1);

        assert!(!t0.is_close_to_normalized(0f32));
        assert!(t0.is_close_to_normalized(25f32));
        assert!(t2.is_close_to_normalized(0f32));

        assert!(!t0.is_normalized());
        assert!(t2.is_normalized());
    }

    #[test]
    fn test_dot() {
        let v0 = Tup2::new(2f32, -3f32);
        let v1 = Tup2::new(4f32, 5f32);

        assert_eq!(v0.dot(v1), -7f32);
    }
}