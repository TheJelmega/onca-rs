use core::arch::x86_64::*;

use crate::{
    LaneCount, SupportedLaneCount,
    SimdElement, 
    Mask,
    backend::*,
    mask::sealed::Sealed, Simd
};

macro_rules! impl_via_avx {
    ($([$ty:ty, $lanes128:literal, $lanes256:literal, $lanes512:literal])*) => {
        $(
            impl_via_avx!{$ty, $lanes128}
            impl_via_avx!{$ty, $lanes256}
            impl_via_avx!{$ty, $lanes512}
        )*
    };
    ($ty:ty, $lanes:literal) => {
        impl SimdSetImpl<$ty, {BackendType::AVX2}> for Simd<$ty, $lanes> {
            fn simd_zero_impl() -> Self {
                <Self as SimdSetImpl<$ty, {BackendType::AVX}>>::simd_zero_impl()
            }

            fn simd_splat_impl(val: $ty) -> Self {
                <Self as SimdSetImpl<$ty, {BackendType::AVX}>>::simd_splat_impl(val)
            }
        }

        impl SimdLoadStoreImpl<$ty, {BackendType::AVX2}> for Simd<$ty, $lanes> {
            fn simd_load_impl(mem: *const $ty) -> Self {
                <Self as SimdLoadStoreImpl<$ty, {BackendType::AVX}>>::simd_load_impl(mem)
            }
        
            fn simd_store_impl(self, mem: *mut $ty) {
                <Self as SimdLoadStoreImpl<$ty, {BackendType::AVX}>>::simd_store_impl(self, mem)
            }
        }
    };
}