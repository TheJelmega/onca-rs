
mod normal2;
pub use normal2::*;

mod normal3;
pub use normal3::*;

macro_rules! generic_normal {
    (
        $docs:meta;
        $iden:ident,
        $vec:ident,
        $elem_cnt:literal,
        $tup_ty:ty,
        $($comp:ident => $idx:tt),+;
        $($alias_ty:ident => $base_ty:ty)*
    ) => {
        $crate::common::tuple_common!{
            $docs;
            $iden,
            $elem_cnt,
            $tup_ty,
            $($comp => $idx),+;
        }
        $crate::common::tuple_common!{ @scale $iden, $($comp),+ }
        $crate::common::tuple_common!{ @len_and_normalize $iden, $($comp),+ }
        $crate::common::tuple_common!{ @dot $iden, $iden, $($comp),+ }
        $crate::common::tuple_common!{ @dot $iden, $vec, $($comp),+ }
        $crate::common::tuple_common!{ @dot $vec, $iden, $($comp),+ }
        
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

            /// Get a normal in the same hemisphere of an outgoing vector
            pub fn face_forward(self, incidence: $vec<T>) -> Self where
                Self: DotProduct<$vec<T>, Output = T> + Neg<Output = Self>,
                T: Zero + PartialOrd,
            {
                if self.dot(incidence) <= T::zero() { self } else { -self }
            }

            /// Reflect an incoming vector around the normal
            fn reflect(self, incoming: $vec<T>) -> $vec<T> {
                debug_assert!(self.is_normalized());
                incoming - self.to_vec() * self.dot(incoming) * T::from_i32(2)
            }
        
            /// Refacts an incoming vector using a normal
            /// 
            /// `eta` 
            fn refract(self, incoming: $vec<T>, eta: T) -> $vec<T> {
                debug_assert!(self.is_normalized());
            
                let cosi = self.dot(incoming);
                debug_assert!(cosi < T::zero());
            
                let sini = T::one() - cosi * cosi;
                let sint = eta * eta * sini;

                let k = eta * eta * (T::one() - cosi * cosi);
                if sint > T::one() {
                    let cost = (T::one() - k).sqrt();
                    incoming * eta - self.to_vec() * (eta * cosi - cost)
                } else {
                    $vec::zero()
                }
            }
        }

        //------------------------------------------------------------------------------------------------------------------------------

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
pub(crate) use generic_normal;
