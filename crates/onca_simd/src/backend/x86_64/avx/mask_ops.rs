use core::arch::x86_64::*;

use crate::{
    LaneCount, SupportedLaneCount,
    MaskElement, Mask,
    backend::{SimdMaskOpsImpl, BackendType},
    mask::sealed::Sealed, Simd
};

macro_rules! impl_all_any {
    { $($ty: ty, $lanes128:literal, $lanes256:literal, $lanes512:literal),* } => {
        $(
            impl SimdMaskOpsImpl<{BackendType::AVX}> for Simd<$ty, $lanes128> {
                fn simd_all_impl(self) -> bool {
                    <Self as SimdMaskOpsImpl<{BackendType::SSE}>>::simd_all_impl(self)
                }
            
                fn simd_any_impl(self) -> bool {
                    <Self as SimdMaskOpsImpl<{BackendType::SSE}>>::simd_any_impl(self)
                }
            }

            impl SimdMaskOpsImpl<{BackendType::AVX}> for Simd<$ty, $lanes256> {
                fn simd_all_impl(self) -> bool {
                    <Self as SimdMaskOpsImpl<{BackendType::SSE}>>::simd_all_impl(self)
                }
            
                fn simd_any_impl(self) -> bool {
                    <Self as SimdMaskOpsImpl<{BackendType::SSE}>>::simd_any_impl(self)
                }
            }

            impl SimdMaskOpsImpl<{BackendType::AVX}> for Simd<$ty, $lanes512> {
                fn simd_all_impl(self) -> bool {
                    <Self as SimdMaskOpsImpl<{BackendType::SSE}>>::simd_all_impl(self)
                }
            
                fn simd_any_impl(self) -> bool {
                    <Self as SimdMaskOpsImpl<{BackendType::SSE}>>::simd_any_impl(self)
                }
            }
        )*
    };
}
impl_all_any!{
    i8 , 16, 32, 64,
    i16, 8 , 16, 32,
    i32, 4 , 8 , 16,
    i64, 2 , 4 , 8
}