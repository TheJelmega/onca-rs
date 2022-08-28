//! SIMD abstraction based on rust's portable_simd
//! 
//! Currently no runtime dynamic dispatch is supported, so the instructions sets will be decided at compile time
//! 
//! This is not a generic SIMD library, as this is designed to work with the onca engine, meaning that there are certain expectation that a machine has:
//! - Only 64-bit is supported
//! - 64-bit wide registers (i.e. ARM NEON D-registers) are not supported
//! - Currently only x86_64 is supported, with aarch64 coming later. Other ISAs are currently unknown
//! - for x86_64, a x86-64-v2 CPU is expected at minimum for SIMD support (i.e. supports SSE4.2 and POPCNT). v3 is prefered
//! 
#![no_std]
#![allow(unused)]
#![allow(incomplete_features)]

#![feature(repr_simd)]
#![feature(generic_const_exprs)]
#![feature(stdsimd)]
#![feature(unchecked_math)]
#![feature(adt_const_params)]

// for stuff like `instrinsics::floorf32()`, cause `f32::floor()` is only available with `std`
#![feature(core_intrinsics)]

#[cfg(not(target_pointer_width = "64"))]
compile_error!("Only 64-bit architectures are supported");

mod lanes;
mod simd;
mod mask;

mod float;
mod int;
mod uint;

mod backend;

pub use backend::*;
pub use lanes::*;
pub use simd::*;
pub use mask::*;

pub use float::*;
pub use int::*;
pub use uint::*;


pub const DEF_BACKEND_TYPE : BackendType = BackendType::SSE;

//#[cfg(not(all(target_feature = "sse")))]
//pub const DEF_BACKEND_TYPE : BackendType = BackendType::Scalar;
//#[cfg(all(target_feature = "sse", not(target_feature = "avx")))]
//pub const DEF_BACKEND_TYPE : BackendType = BackendType::SSE;
//#[cfg(all(target_feature = "avx", not(target_feature = "avx2")))]
//pub const DEF_BACKEND_TYPE : BackendType = BackendType::AVX;
//#[cfg(all(target_feature = "avx2", not(target_feature = "avx512")))]
//pub const DEF_BACKEND_TYPE : BackendType = BackendType::AVX2;
//#[cfg(all(target_feature = "avx512"))]
//pub const DEF_BACKEND_TYPE : BackendType = BackendType::AVX512;

/// Check if an intrinsic can be used on the current machine
/// 
/// #Note
/// 
/// Currently, no dynamic detection has been implemented, so values returned depend on the machine the binary was compiled on
pub fn has_intrin(intrin: BackendType) -> bool {

    #[cfg(target_arch = "x86_64")]
    {
        match intrin {
            BackendType::Scalar => true,
            BackendType::SSE => cfg!(target_feature = "sse4.2"),
            BackendType::AVX => cfg!(target_feature = "avx"),
            BackendType::AVX2 => cfg!(target_feature = "avx2"),
            BackendType::AVX512 => cfg!(target_feature = "avx512"),
            BackendType::NEON => false,
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    (intrin == BackendType::Scalar)
}