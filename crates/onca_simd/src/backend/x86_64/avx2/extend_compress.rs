use core::arch::x86_64::*;

use super::*;
use crate::{
    *,
    backend::*,
    SimdElement, Simd, LaneCount, SupportedLaneCount
};

macro_rules! impl_cvt_int {
    {$([$ty:ty, $lanes128:literal, $lanes256:literal, $lanes512:literal <=> 
        $e_ty:ty, $e_lanes128:literal, $e_lanes256:literal, $e_lanes512:literal;
        $cvt_ext:ident, $pack:ident])*
    } => {
        $(
            impl SimdExtendCompressImpl<{BackendType::AVX2}> for Simd<$ty, $lanes128> {
                type ExtendedType = Simd<$e_ty, $e_lanes128>;

                fn simd_extend_lower_impl(self) -> Self::ExtendedType {
                    <Self as SimdExtendCompressImpl<{BackendType::AVX}>>::simd_extend_lower_impl(self)
                }
            
                fn simd_extend_upper_impl(self) -> Self::ExtendedType {
                    <Self as SimdExtendCompressImpl<{BackendType::AVX}>>::simd_extend_upper_impl(self)
                }
            
                fn simd_compress_impl(a: Self::ExtendedType, b: Self::ExtendedType) -> Self {
                    <Self as SimdExtendCompressImpl<{BackendType::AVX}>>::simd_compress_impl(a, b)
                }
            }

            impl SimdExtendCompressImpl<{BackendType::AVX2}> for Simd<$ty, $lanes256> {
                type ExtendedType = Simd<$e_ty, $e_lanes256>;
            
                fn simd_extend_lower_impl(self) -> Self::ExtendedType {
                    self.simd_convert::<$e_ty, $e_lanes256, {BackendType::AVX2}>()
                }
            
                fn simd_extend_upper_impl(self) -> Self::ExtendedType {
                    unsafe {
                        let upper : Self = _mm256_castsi128_si256(_mm256_extracti128_si256::<1>(self.into())).into();
                        upper.simd_convert::<$e_ty, $e_lanes256, {BackendType::AVX2}>()
                    }
                }
            
                fn simd_compress_impl(a: Self::ExtendedType, b: Self::ExtendedType) -> Self {
                    unsafe {
                        let lower = a.simd_convert::<$ty, $lanes256, {BackendType::AVX2}>();
                        let upper = b.simd_convert::<$ty, $lanes256, {BackendType::AVX2}>();
                        _mm256_inserti128_si256::<1>(lower.into(), _mm256_castsi256_si128(upper.into())).into()
                    }
                }
            }
            
            impl SimdExtendCompressImpl<{BackendType::AVX2}> for Simd<$ty, $lanes512> {
                type ExtendedType = Simd<$e_ty, $e_lanes512>;
            
                fn simd_extend_lower_impl(self) -> Self::ExtendedType {
                    self.simd_convert::<$e_ty, $e_lanes512, {BackendType::AVX2}>()
                }
            
                fn simd_extend_upper_impl(self) -> Self::ExtendedType {
                    unsafe {
                        let halfs : [__m256i; 2] = self.into();
                        let upper : Simd<$ty, $lanes512> = [halfs[1], _mm256_setzero_si256()].into();
                        upper.simd_convert::<$e_ty, $e_lanes512, {BackendType::AVX2}>()
                    }
                }
            
                fn simd_compress_impl(a: Self::ExtendedType, b: Self::ExtendedType) -> Self {
                    unsafe {
                        let a_halfs = a.split_2();
                        let a_lower = a_halfs[0].simd_convert::<$ty, $lanes256, {BackendType::AVX2}>();
                        let a_upper = a_halfs[1].simd_convert::<$ty, $lanes256, {BackendType::AVX2}>();
                        let lower = _mm256_inserti128_si256::<1>(a_lower.into(), _mm256_castsi256_si128(a_upper.into()));

                        let b_halfs = b.split_2();
                        let b_lower = b_halfs[0].simd_convert::<$ty, $lanes256, {BackendType::AVX2}>();
                        let b_upper = b_halfs[1].simd_convert::<$ty, $lanes256, {BackendType::AVX2}>();
                        let upper = _mm256_inserti128_si256::<1>(b_lower.into(), _mm256_castsi256_si128(b_upper.into()));

                        [lower, upper].into()
                    }
                }
            }
        )*
    };
}
impl_cvt_int!{
    [i8 , 16, 32, 64 <=> i16, 8 , 16, 32; _mm256_cvtepi8_epi16, _mm256_packus_epi16]
    [i16, 8 , 16, 32 <=> i32, 4 , 8 , 16; _mm256_cvtepi16_epi32, _mm256_packus_epi32]

    [u8 , 16, 32, 64 <=> u16, 8 , 16, 32; _mm256_cvtepu8_epi16, _mm256_packus_epi16]
    [u16, 8 , 16, 32 <=> u32, 4 , 8 , 16; _mm256_cvtepu16_epi32, _mm256_packus_epi32]
}

impl SimdExtendCompressImpl<{BackendType::AVX2}> for Simd<i32, 4> {
    type ExtendedType = Simd<i64, 2>;

    fn simd_extend_lower_impl(self) -> Self::ExtendedType {
        <Self as SimdExtendCompressImpl<{BackendType::AVX}>>::simd_extend_lower_impl(self)
    }

    fn simd_extend_upper_impl(self) -> Self::ExtendedType {
        <Self as SimdExtendCompressImpl<{BackendType::AVX}>>::simd_extend_upper_impl(self)
    }

    fn simd_compress_impl(a: Self::ExtendedType, b: Self::ExtendedType) -> Self {
        <Self as SimdExtendCompressImpl<{BackendType::AVX}>>::simd_compress_impl(a, b)
    }
}

impl SimdExtendCompressImpl<{BackendType::AVX2}> for Simd<i32, 8> {
    type ExtendedType = Simd<i64, 4>;

    fn simd_extend_lower_impl(self) -> Self::ExtendedType {
        unsafe {
            let f : [__m128i; 2] = self.into();
            let lower = _mm_cvtepi32_epi64(f[0]);
            let upper = _mm_srli_si128::<8>(f[0]);
            let upper = _mm_cvtepi32_epi64(upper);
            [lower, upper].into()
        }
    }

    fn simd_extend_upper_impl(self) -> Self::ExtendedType {
        unsafe {
            let f : [__m128i; 2] = self.into();
            let lower = _mm_cvtepi32_epi64(f[1]);
            let upper = _mm_srli_si128::<8>(f[1]);
            let upper = _mm_cvtepi32_epi64(upper);
            [lower, upper].into()
        }
    }

    fn simd_compress_impl(a: Self::ExtendedType, b: Self::ExtendedType) -> Self {
        unsafe {
            let zero = _mm_setzero_si128();

            let a = a.split_2();
            let a_lower : __m128i = a[0].simd_convert::<i32, 4, {BackendType::AVX2}>().into();
            let a_upper : __m128i = a[1].simd_convert::<i32, 4, {BackendType::AVX2}>().into();
            let a_upper = _mm_srli_si128::<8>(a_upper);

            let b = b.split_2();
            let b_lower : __m128i = b[0].simd_convert::<i32, 4, {BackendType::AVX2}>().into();
            let b_upper : __m128i = b[1].simd_convert::<i32, 4, {BackendType::AVX2}>().into();
            let b_upper = _mm_srli_si128::<8>(b_upper);

            [_mm_or_si128(a_lower, a_upper), _mm_or_si128(b_lower, b_upper)].into()
        }
    }
}

impl SimdExtendCompressImpl<{BackendType::AVX2}> for Simd<i32, 16> {
    type ExtendedType = Simd<i64, 8>;

    fn simd_extend_lower_impl(self) -> Self::ExtendedType {
        unsafe {
            let f : [__m128i; 4] = self.into();

            let lower0 = _mm_cvtepi32_epi64(f[0]);
            let upper0 = _mm_srli_si128::<8>(f[0]);
            let upper0 = _mm_cvtepi32_epi64(upper0);

            let lower1 = _mm_cvtepi32_epi64(f[1]);
            let upper1 = _mm_srli_si128::<8>(f[1]);
            let upper1 = _mm_cvtepi32_epi64(upper1);

            [lower0, upper0, lower1, upper1].into()
        }
    }

    fn simd_extend_upper_impl(self) -> Self::ExtendedType {
        unsafe {
            let f : [__m128i; 4] = self.into();

            let lower2 = _mm_cvtepi32_epi64(f[2]);
            let upper2 = _mm_srli_si128::<8>(f[2]);
            let upper2 = _mm_cvtepi32_epi64(upper2);

            let lower3 = _mm_cvtepi32_epi64(f[3]);
            let upper3 = _mm_srli_si128::<8>(f[3]);
            let upper3 = _mm_cvtepi32_epi64(upper3);

            [lower2, upper2, lower3, upper3].into()
        }
    }

    fn simd_compress_impl(a: Self::ExtendedType, b: Self::ExtendedType) -> Self {
        unsafe {
            let zero = _mm_setzero_si128();

            let a  = a.split_4();
            let a_lower0 : __m128i = a[0].simd_convert::<i32, 4, {BackendType::AVX2}>().into();
            let a_upper0 : __m128i = a[1].simd_convert::<i32, 4, {BackendType::AVX2}>().into();
            let a_upper0 = _mm_srli_si128::<8>(a_upper0);

            let a_lower1 : __m128i = a[2].simd_convert::<i32, 4, {BackendType::AVX2}>().into();
            let a_upper1 : __m128i = a[3].simd_convert::<i32, 4, {BackendType::AVX2}>().into();
            let a_upper1 = _mm_srli_si128::<8>(a_upper1);

            let b  = b.split_4();
            let b_lower0 : __m128i = b[0].simd_convert::<i32, 4, {BackendType::AVX2}>().into();
            let b_upper0 : __m128i = b[1].simd_convert::<i32, 4, {BackendType::AVX2}>().into();
            let b_upper0 = _mm_srli_si128::<8>(b_upper0);

            let b_lower1 : __m128i = b[2].simd_convert::<i32, 4, {BackendType::AVX2}>().into();
            let b_upper1 : __m128i = b[3].simd_convert::<i32, 4, {BackendType::AVX2}>().into();
            let b_upper1 = _mm_srli_si128::<8>(b_upper1);

            [_mm_or_si128(a_lower0, a_upper0), 
             _mm_or_si128(a_lower1, a_upper1),
             _mm_or_si128(b_lower0, b_upper0),
             _mm_or_si128(b_lower1, b_upper1)].into()
        }
    }
}

impl SimdExtendCompressImpl<{BackendType::AVX2}> for Simd<u32, 4> {
    type ExtendedType = Simd<u64, 2>;

    fn simd_extend_lower_impl(self) -> Self::ExtendedType {
        <Self as SimdExtendCompressImpl<{BackendType::AVX}>>::simd_extend_lower_impl(self)
    }

    fn simd_extend_upper_impl(self) -> Self::ExtendedType {
        <Self as SimdExtendCompressImpl<{BackendType::AVX}>>::simd_extend_upper_impl(self)
    }

    fn simd_compress_impl(a: Self::ExtendedType, b: Self::ExtendedType) -> Self {
        <Self as SimdExtendCompressImpl<{BackendType::AVX}>>::simd_compress_impl(a, b)
    }
}

impl SimdExtendCompressImpl<{BackendType::AVX2}> for Simd<u32, 8> {
    type ExtendedType = Simd<u64, 4>;

    fn simd_extend_lower_impl(self) -> Self::ExtendedType {
        unsafe {
            let f : [__m128i; 2] = self.into();
            let lower = _mm_cvtepu32_epi64(f[0]);
            let upper = _mm_srli_si128::<8>(f[0]);
            let upper = _mm_cvtepu32_epi64(upper);
            [lower, upper].into()
        }
    }

    fn simd_extend_upper_impl(self) -> Self::ExtendedType {
        unsafe {
            let f : [__m128i; 2] = self.into();
            let lower = _mm_cvtepu32_epi64(f[1]);
            let upper = _mm_srli_si128::<8>(f[1]);
            let upper = _mm_cvtepu32_epi64(upper);
            [lower, upper].into()
        }
    }

    fn simd_compress_impl(a: Self::ExtendedType, b: Self::ExtendedType) -> Self {
        unsafe {
            let zero = _mm_setzero_si128();

            let a = a.split_2();
            let a_lower : __m128i = a[0].simd_convert::<u32, 4, {BackendType::AVX2}>().into();
            let a_upper : __m128i = a[1].simd_convert::<u32, 4, {BackendType::AVX2}>().into();
            let a_upper = _mm_srli_si128::<8>(a_upper);

            let b = b.split_2();
            let b_lower : __m128i = b[0].simd_convert::<u32, 4, {BackendType::AVX2}>().into();
            let b_upper : __m128i = b[1].simd_convert::<u32, 4, {BackendType::AVX2}>().into();
            let b_upper = _mm_srli_si128::<8>(b_upper);

            [_mm_or_si128(a_lower, a_upper), _mm_or_si128(b_lower, b_upper)].into()
        }
    }
}

impl SimdExtendCompressImpl<{BackendType::AVX2}> for Simd<u32, 16> {
    type ExtendedType = Simd<u64, 8>;

    fn simd_extend_lower_impl(self) -> Self::ExtendedType {
        unsafe {
            let f : [__m128i; 4] = self.into();

            let lower0 = _mm_cvtepu32_epi64(f[0]);
            let upper0 = _mm_srli_si128::<8>(f[0]);
            let upper0 = _mm_cvtepu32_epi64(upper0);

            let lower1 = _mm_cvtepu32_epi64(f[1]);
            let upper1 = _mm_srli_si128::<8>(f[1]);
            let upper1 = _mm_cvtepu32_epi64(upper1);

            [lower0, upper0, lower1, upper1].into()
        }
    }

    fn simd_extend_upper_impl(self) -> Self::ExtendedType {
        unsafe {
            let f : [__m128i; 4] = self.into();

            let lower2 = _mm_cvtepu32_epi64(f[2]);
            let upper2 = _mm_srli_si128::<8>(f[2]);
            let upper2 = _mm_cvtepu32_epi64(upper2);

            let lower3 = _mm_cvtepu32_epi64(f[3]);
            let upper3 = _mm_srli_si128::<8>(f[3]);
            let upper3 = _mm_cvtepu32_epi64(upper3);

            [lower2, upper2, lower3, upper3].into()
        }
    }

    fn simd_compress_impl(a: Self::ExtendedType, b: Self::ExtendedType) -> Self {
        unsafe {
            let zero = _mm_setzero_si128();

            let a  = a.split_4();
            let a_lower0 : __m128i = a[0].simd_convert::<u32, 4, {BackendType::AVX2}>().into();
            let a_upper0 : __m128i = a[1].simd_convert::<u32, 4, {BackendType::AVX2}>().into();
            let a_upper0 = _mm_srli_si128::<8>(a_upper0);

            let a_lower1 : __m128i = a[2].simd_convert::<u32, 4, {BackendType::AVX2}>().into();
            let a_upper1 : __m128i = a[3].simd_convert::<u32, 4, {BackendType::AVX2}>().into();
            let a_upper1 = _mm_srli_si128::<8>(a_upper1);

            let b  = b.split_4();
            let b_lower0 : __m128i = b[0].simd_convert::<u32, 4, {BackendType::AVX2}>().into();
            let b_upper0 : __m128i = b[1].simd_convert::<u32, 4, {BackendType::AVX2}>().into();
            let b_upper0 = _mm_srli_si128::<8>(b_upper0);

            let b_lower1 : __m128i = b[2].simd_convert::<u32, 4, {BackendType::AVX2}>().into();
            let b_upper1 : __m128i = b[3].simd_convert::<u32, 4, {BackendType::AVX2}>().into();
            let b_upper1 = _mm_srli_si128::<8>(b_upper1);

            [_mm_or_si128(a_lower0, a_upper0), 
             _mm_or_si128(a_lower1, a_upper1),
             _mm_or_si128(b_lower0, b_upper0),
             _mm_or_si128(b_lower1, b_upper1)].into()
        }
    }
}

impl SimdExtendCompressImpl<{BackendType::AVX2}> for Simd<f32, 4> {
    type ExtendedType = Simd<f64, 2>;

    fn simd_extend_lower_impl(self) -> Self::ExtendedType {
        <Self as SimdExtendCompressImpl<{BackendType::AVX}>>::simd_extend_lower_impl(self)
    }

    fn simd_extend_upper_impl(self) -> Self::ExtendedType {
        <Self as SimdExtendCompressImpl<{BackendType::AVX}>>::simd_extend_upper_impl(self)
    }

    fn simd_compress_impl(a: Self::ExtendedType, b: Self::ExtendedType) -> Self {
        <Self as SimdExtendCompressImpl<{BackendType::AVX}>>::simd_compress_impl(a, b)
    }
}

impl SimdExtendCompressImpl<{BackendType::AVX2}> for Simd<f32, 8> {
    type ExtendedType = Simd<f64, 4>;

    fn simd_extend_lower_impl(self) -> Self::ExtendedType {
        <Self as SimdExtendCompressImpl<{BackendType::AVX}>>::simd_extend_lower_impl(self)
    }

    fn simd_extend_upper_impl(self) -> Self::ExtendedType {
        <Self as SimdExtendCompressImpl<{BackendType::AVX}>>::simd_extend_upper_impl(self)
    }

    fn simd_compress_impl(a: Self::ExtendedType, b: Self::ExtendedType) -> Self {
        <Self as SimdExtendCompressImpl<{BackendType::AVX}>>::simd_compress_impl(a, b)
    }
}

impl SimdExtendCompressImpl<{BackendType::AVX2}> for Simd<f32, 16> {
    type ExtendedType = Simd<f64, 8>;

    fn simd_extend_lower_impl(self) -> Self::ExtendedType {
        <Self as SimdExtendCompressImpl<{BackendType::AVX}>>::simd_extend_lower_impl(self)
    }

    fn simd_extend_upper_impl(self) -> Self::ExtendedType {
        <Self as SimdExtendCompressImpl<{BackendType::AVX}>>::simd_extend_upper_impl(self)
    }

    fn simd_compress_impl(a: Self::ExtendedType, b: Self::ExtendedType) -> Self {
        <Self as SimdExtendCompressImpl<{BackendType::AVX}>>::simd_compress_impl(a, b)
    }
}