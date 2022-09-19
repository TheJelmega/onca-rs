use crate::*;

use core::{
    mem,
    ops::*,
};

mod mat4;
pub use mat4::*;

mod mat3;
pub use mat3::*;

mod mat2;
pub use mat2::*;

mod mat4x3;
pub use mat4x3::*;

macro_rules! matrix_pre_multiplication {
    {$name:ident, $m:literal, $n:literal, $($ty:ty),*} => {
        $(
            impl Mul<$name<$ty>> for $ty {
                type Output = $name<$ty>;
            
                fn mul(self, rhs: $name<$ty>) -> Self::Output {
                    let mut res = $name { vals: [<$ty as Zero>::zero(); $m * $n] };
                    for i in 0..($m * $n) {
                        res[i] = self * rhs[i];
                    }
                    res
                }
            }
        )*
    };
}

macro_rules! generic_matrix {
    {$docs:meta; $name:ident, $m:literal, $n:literal} => {
        #[$docs]
        #[derive(Clone, Copy, PartialEq, Debug)]
        pub struct $name<T: Real> {
            vals : [T; $m * $n]
        }

        impl<T: Real> $name<T> {
            /// Create a matrix from an array
            #[inline(always)]
            #[must_use]
            pub fn from_array(vals: [T; $m * $n]) -> Self {
                Self { vals }
            }

            /// Interpret a reference to an array as a reference to a vector
            #[inline(always)]
            #[must_use]
            pub fn ref_from_array(vals: &[T; $m * $n]) -> &Self {
                unsafe { mem::transmute(vals) }
            }

            /// Interpret a mutable reference to an array as a mutable reference to a vector
            #[inline(always)]
            #[must_use]
            pub fn mut_from_array(vals: &mut [T; $m * $n]) -> &mut Self {
                unsafe { mem::transmute(vals) }
            }

            /// Get the content of the vector as an array
            #[inline(always)]
            #[must_use]
            pub fn to_array(self) -> [T; $m * $n] {
                self.vals
            }

            /// Interpret a reference to an vector as a reference to a array
            #[inline(always)]
            #[must_use]
            pub fn as_array(&self) -> &[T; $m * $n] {
                unsafe{ mem::transmute(self) }
            }

            /// Interpret a mutable reference to an vector as a mutable reference to a array
            #[inline(always)]
            #[must_use]
            pub fn as_mut_array(&mut self) -> &mut [T; $m * $n] {
                unsafe{ mem::transmute(self) }
            }
        }

        //------------------------------------------------------------------------------------------------------------------------------

        impl<T: Real> Index<usize> for $name<T> {
            type Output = T;
        
            fn index(&self, index: usize) -> &Self::Output {
                debug_assert!(index < $m * $n);
                &self.vals[index]
            }
        }

        impl<T: Real> IndexMut<usize> for $name<T> {
            fn index_mut(&mut self, index: usize) -> &mut Self::Output {
                debug_assert!(index < $m * $n);
                &mut self.vals[index]
            }
        }

        impl<T: Real> Index<(usize, usize)> for $name<T> {
            type Output = T;
        
            fn index(&self, index: (usize, usize)) -> &Self::Output {
                debug_assert!(index.0 < $m);
                debug_assert!(index.1 < $n);
                &self.vals[index.0 * $n + index.1]
            }
        }
        
        impl<T: Real> IndexMut<(usize, usize)> for $name<T> {
            fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
                debug_assert!(index.0 < $m);
                debug_assert!(index.1 < $n);
                &mut self.vals[index.0 * $n + index.1]
            }
        }
        
        //------------------------------------------------------------------------------------------------------------------------------

        impl<T: Real> Neg for $name<T> {
            type Output = Self;

            fn neg(self) -> Self::Output {
                let mut res = Self { vals: [T::zero(); $m * $n] };
                for i in 0..($m * $n) {
                    res[i] = -self[i];
                }
                res
            }
        }

        //--------------------------------------------------------------

        impl<T: Real> Add for $name<T> {
            type Output = Self;

            fn add(self, rhs: Self) -> Self::Output {
                let mut res = Self { vals: [T::zero(); $m * $n] };
                for i in 0..($m * $n) {
                    res[i] = self[i] + rhs[i];
                }
                res
            }
        }

        impl<T: Real> AddAssign for $name<T> {
            fn add_assign(&mut self, rhs: Self) {
                for i in 0..($m * $n) {
                    self[i] += rhs[i];
                }
            }
        }

        //--------------------------------------------------------------

        impl<T: Real> Sub for $name<T> {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self::Output {
                let mut res = Self { vals: [T::zero(); $m * $n] };
                for i in 0..($m * $n) {
                    res[i] = self[i] - rhs[i];
                }
                res
            }
        }

        impl<T: Real> SubAssign for $name<T> {
            fn sub_assign(&mut self, rhs: Self) {
                for i in 0..($m * $n) {
                    self[i] -= rhs[i];
                }
            }
        }
        
        //--------------------------------------------------------------

        impl<T: Real> Mul<T> for $name<T> {
            type Output = Self;

            fn mul(self, rhs: T) -> Self::Output {
                let mut res = Self { vals: [T::zero(); $m * $n] };
                for i in 0..($m * $n) {
                    res[i] = self[i] * rhs;
                }
                res
            }
        }

        impl<T: Real> MulAssign<T> for $name<T> {
            fn mul_assign(&mut self, rhs: T) {
                for i in 0..($m * $n) {
                    self[i] *= rhs;
                }
            }
        }
        
        //--------------------------------------------------------------

        impl<T: Real> Div<T> for $name<T> {
            type Output = Self;

            fn div(self, rhs: T) -> Self::Output {
                let mut res = Self { vals: [T::zero(); $m * $n] };
                for i in 0..($m * $n) {
                    res[i] = self[i] / rhs;
                }
                res
            }
        }

        impl<T: Real> DivAssign<T> for $name<T> {
            fn div_assign(&mut self, rhs: T) {
                for i in 0..($m * $n) {
                    self[i] /= rhs;
                }
            }
        }

        //--------------------------------------------------------------

        impl<T: Real> Zero for $name<T> {
            fn zero() -> Self {
                Self { vals: [T::zero(); $m * $n] }
            }
        }

        //--------------------------------------------------------------

        matrix_pre_multiplication!{$name, $m, $n, f32, f64}
    };
}

generic_matrix!{doc = "4x4 matrix (row-major order)"; Mat4, 4, 4}
generic_matrix!{doc = "3x3 matrix (row-major order)"; Mat3, 3, 3}
generic_matrix!{doc = "2x2 matrix (row-major order)"; Mat2, 2, 2}
generic_matrix!{doc = "4x3 matrix (row-major order), with an implicit (0, 0, 0, 1) column at the end"; Mat4x3, 4, 3}