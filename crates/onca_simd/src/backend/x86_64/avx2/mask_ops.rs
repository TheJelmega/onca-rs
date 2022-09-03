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
            impl SimdMaskOpsImpl<{BackendType::AVX2}> for Simd<$ty, $lanes128> {
                fn simd_all_impl(self) -> bool {
                    unsafe {
                        let a : __m128i = self.into();
                        _mm_movemask_epi8(a) == 0xFFFF
                    }
                }
            
                fn simd_any_impl(self) -> bool {
                    unsafe {
                        let a : __m128i = self.into();
                        _mm_movemask_epi8(a) != 0
                    }
                }
            }

            impl SimdMaskOpsImpl<{BackendType::AVX2}> for Simd<$ty, $lanes256> {
                fn simd_all_impl(self) -> bool {
                    unsafe { _mm256_movemask_epi8(self.into()) == 0xFFFF_FFFFu32 as i32 }
                }
            
                fn simd_any_impl(self) -> bool {
                    unsafe { _mm256_movemask_epi8(self.into()) != 0 }
                }
            }

            impl SimdMaskOpsImpl<{BackendType::AVX2}> for Simd<$ty, $lanes512> {
                fn simd_all_impl(self) -> bool {
                    unsafe {
                        let a : [__m256i; 2] = self.into();
                        _mm256_movemask_epi8(a[0]) == 0xFFFF_FFFFu32 as i32 &&
                        _mm256_movemask_epi8(a[1]) == 0xFFFF_FFFFu32 as i32
                    }
                }
            
                fn simd_any_impl(self) -> bool {
                    unsafe {
                        let a : [__m256i; 2] = self.into();
                        _mm256_movemask_epi8(a[0]) != 0 &&
                        _mm256_movemask_epi8(a[1]) != 0
                    }
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