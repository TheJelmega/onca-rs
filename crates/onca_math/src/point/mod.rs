mod point2;
pub use point2::*;

mod point3;
pub use point3::*;

macro_rules! generic_point {
    (
        $docs:meta;
        $iden:ident,
        $vec:ident,
        $elem_cnt:literal,
        $tup_ty:ty,
        $($comp:ident => $idx:tt),+;
        $($alias:ident => $alias_base:ty)*
    ) => {
        crate::common::tuple_common! {
            $docs;
            $iden,
            $elem_cnt,
            $tup_ty,
            $($comp => $idx),+;
            $($alias => $alias_base)*
        }
        crate::common::tuple_common! { @add_sub_self $iden, $vec, $($comp),+ }
        crate::common::tuple_common! { @snap_scalar $iden, $($comp),+ }

        impl<T: Numeric> $iden<T> {
            #[doc = concat!("Create a `", stringify!($iden), "` from a `", stringify!($vec), "`.")]
            #[inline(always)]
            #[must_use]
            pub fn from_vec(vec: $vec<T>) -> Self {
                Self { $($comp: vec.$comp),* }
            }

            #[doc = concat!("Get a `", stringify!($vec), "` from the `", stringify!($iden), "`.")]
            #[inline(always)]
            #[must_use]
            pub fn to_vec(self) -> $vec<T> {
                $vec { $($comp: self.$comp),* }
            }

            /// Calculate the distance between 2 points
            #[must_use]
            pub fn dist_sq(self, other: Self) -> T where
                T: Add<Output = T> + Sub<Output = T> + Mul<Output = T>
            {
                (self - other).len_sq()
            }

            /// Calculate the distance between 2 points
            #[must_use]
            pub fn dist(self, other: Self) -> T where
                T: Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Sqrt
            {
                (self - other).len()
            }
        }

        //------------------------------------------------------------------------------------------------------------------------------

        impl<T: Numeric + Sub<Output = T>> Sub for $iden<T> {
            type Output = $vec<T>;

            fn sub(self, rhs: Self) -> Self::Output {
                $vec { $($comp: self.$comp - rhs.$comp),* }
            }
        }

        //--------------------------------------------------------------

        impl<T: Numeric> From<$vec<T>> for $iden<T> {
            fn from(vec: $vec<T>) -> Self {
                Self::from_vec(vec)
            }
        }

        impl<T: Numeric> Into<$vec<T>> for $iden<T> {
            fn into(self) -> $vec<T> {
                self.to_vec()
            }
        }
    };
}
pub(crate) use generic_point;