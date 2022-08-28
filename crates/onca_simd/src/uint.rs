#![allow(non_camel_case_types)]

use crate::Simd;

pub type u8x16  = Simd<u8, 16>;
pub type u8x32  = Simd<u8, 32>;
pub type u8x64  = Simd<u8, 64>;

pub type u16x8  = Simd<u16, 8>;
pub type u16x16 = Simd<u16, 16>;
pub type u16x32 = Simd<u16, 32>;

pub type u32x4  = Simd<u32, 4>;
pub type u32x8  = Simd<u32, 8>;
pub type u32x16 = Simd<u32, 16>;

pub type u64x2  = Simd<u64, 2>;
pub type u64x4  = Simd<u64, 4>;
pub type u64x8  = Simd<u64, 8>;