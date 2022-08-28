#![allow(non_camel_case_types)]

use crate::Simd;

pub type i8x16  = Simd<i8, 16>;
pub type i8x32  = Simd<i8, 32>;
pub type i8x64  = Simd<i8, 64>;

pub type i16x8  = Simd<i16, 8>;
pub type i16x16 = Simd<i16, 16>;
pub type i16x32 = Simd<i16, 32>;

pub type i32x4  = Simd<i32, 4>;
pub type i32x8  = Simd<i32, 8>;
pub type i32x16 = Simd<i32, 16>;

pub type i64x2  = Simd<i64, 2>;
pub type i64x4  = Simd<i64, 4>;
pub type i64x8  = Simd<i64, 8>;