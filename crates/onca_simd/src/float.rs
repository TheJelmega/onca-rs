#![allow(non_camel_case_types)]

use crate::Simd;

pub type f32x4  = Simd<f32, 4>;
pub type f32x8  = Simd<f32, 8>;
pub type f32x16 = Simd<f32, 16>;

pub type f64x2  = Simd<f64, 2>;
pub type f64x4  = Simd<f64, 4>;
pub type f64x8  = Simd<f64, 8>;