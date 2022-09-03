use core::arch::x86_64::*;

use super::*;
use crate::{
    *,
    backend::*,
    SimdElement, Simd, LaneCount, SupportedLaneCount
};

macro_rules! impl_cvt_int {
    {$([$ty:ty, $lanes128:literal, $lanes256:literal, $lanes512:literal <=> 
        $e_ty:ty, $e_lanes128:literal, $e_lanes256:literal, $e_lanes512:literal])*
    } => {
        $(
            impl SimdExtendCompressImpl<{BackendType::AVX}> for Simd<$ty, $lanes128> {
                type ExtendedType = Simd<$e_ty, $e_lanes128>;

                fn simd_extend_lower_impl(self) -> Self::ExtendedType {
                    <Self as SimdExtendCompressImpl<{BackendType::SSE}>>::simd_extend_lower_impl(self)
                }
            
                fn simd_extend_upper_impl(self) -> Self::ExtendedType {
                    <Self as SimdExtendCompressImpl<{BackendType::SSE}>>::simd_extend_upper_impl(self)
                }
            
                fn simd_compress_impl(a: Self::ExtendedType, b: Self::ExtendedType) -> Self {
                    <Self as SimdExtendCompressImpl<{BackendType::SSE}>>::simd_compress_impl(a, b)
                }
            }

            impl SimdExtendCompressImpl<{BackendType::AVX}> for Simd<$ty, $lanes256> {
                type ExtendedType = Simd<$e_ty, $e_lanes256>;
            
                fn simd_extend_lower_impl(self) -> Self::ExtendedType {
                    <Self as SimdExtendCompressImpl<{BackendType::SSE}>>::simd_extend_lower_impl(self)
                }
            
                fn simd_extend_upper_impl(self) -> Self::ExtendedType {
                    <Self as SimdExtendCompressImpl<{BackendType::SSE}>>::simd_extend_upper_impl(self)
                }
            
                fn simd_compress_impl(a: Self::ExtendedType, b: Self::ExtendedType) -> Self {
                    <Self as SimdExtendCompressImpl<{BackendType::SSE}>>::simd_compress_impl(a, b)
                }
            }
            
            impl SimdExtendCompressImpl<{BackendType::AVX}> for Simd<$ty, $lanes512> {
                type ExtendedType = Simd<$e_ty, $e_lanes512>;
            
                fn simd_extend_lower_impl(self) -> Self::ExtendedType {
                    <Self as SimdExtendCompressImpl<{BackendType::SSE}>>::simd_extend_lower_impl(self)
                }
            
                fn simd_extend_upper_impl(self) -> Self::ExtendedType {
                    <Self as SimdExtendCompressImpl<{BackendType::SSE}>>::simd_extend_upper_impl(self)
                }
            
                fn simd_compress_impl(a: Self::ExtendedType, b: Self::ExtendedType) -> Self {
                    <Self as SimdExtendCompressImpl<{BackendType::SSE}>>::simd_compress_impl(a, b)
                }
            }
        )*
    };
}
impl_cvt_int!{
    [i8 , 16, 32, 64 <=> i16, 8 , 16, 32]
    [i16, 8 , 16, 32 <=> i32, 4 , 8 , 16]
    [i32, 4 , 8 , 16 <=> i64, 2 , 4 , 8]

    [u8 , 16, 32, 64 <=> u16, 8 , 16, 32]
    [u16, 8 , 16, 32 <=> u32, 4 , 8 , 16]
    [u32, 4 , 8 , 16 <=> u64, 2 , 4 , 8]
}

impl SimdExtendCompressImpl<{BackendType::AVX}> for Simd<f32, 4> {
    type ExtendedType = Simd<f64, 2>;

    fn simd_extend_lower_impl(self) -> Self::ExtendedType {
        <Self as SimdExtendCompressImpl<{BackendType::SSE}>>::simd_extend_lower_impl(self)
    }

    fn simd_extend_upper_impl(self) -> Self::ExtendedType {
        <Self as SimdExtendCompressImpl<{BackendType::SSE}>>::simd_extend_upper_impl(self)
    }

    fn simd_compress_impl(a: Self::ExtendedType, b: Self::ExtendedType) -> Self {
        <Self as SimdExtendCompressImpl<{BackendType::SSE}>>::simd_compress_impl(a, b)
    }
}

impl SimdExtendCompressImpl<{BackendType::AVX}> for Simd<f32, 8> {
    type ExtendedType = Simd<f64, 4>;

    fn simd_extend_lower_impl(self) -> Self::ExtendedType {
        unsafe{ _mm256_cvtps_pd(_mm256_castps256_ps128(self.into())).into() }
    }

    fn simd_extend_upper_impl(self) -> Self::ExtendedType {
        unsafe {
            let upper = _mm256_extractf128_ps::<1>(self.into());
            _mm256_cvtps_pd(upper).into()
        }
    }

    fn simd_compress_impl(a: Self::ExtendedType, b: Self::ExtendedType) -> Self {
        unsafe {
            let lower = _mm256_cvtpd_ps(a.into());
            let upper = _mm256_cvtpd_ps(b.into());
            _mm256_insertf128_ps::<1>(_mm256_castps128_ps256(lower), upper).into()
        }
    }
}

impl SimdExtendCompressImpl<{BackendType::AVX}> for Simd<f32, 16> {
    type ExtendedType = Simd<f64, 8>;

    fn simd_extend_lower_impl(self) -> Self::ExtendedType {
        unsafe {
            let f : [__m256; 2] = self.into();
            let lower_pd = _mm256_castps256_ps128(f[0]);
            let lower = _mm256_cvtps_pd(lower_pd);
            let upper_pd = _mm256_extractf128_ps::<1>(f[0]);
            let upper = _mm256_cvtps_pd(upper_pd);
            [lower, upper].into()
        }
    }

    fn simd_extend_upper_impl(self) -> Self::ExtendedType {
        unsafe {
            let f : [__m256; 2] = self.into();
            let lower_pd = _mm256_castps256_ps128(f[1]);
            let lower = _mm256_cvtps_pd(lower_pd);
            let upper_pd = _mm256_extractf128_ps::<1>(f[1]);
            let upper = _mm256_cvtps_pd(upper_pd);
            [lower, upper].into()
        }
    }

    fn simd_compress_impl(a: Self::ExtendedType, b: Self::ExtendedType) -> Self {
        unsafe {
            let a : [__m256d; 2] = a.into();
            let a_lower = _mm256_cvtpd_ps(a[0]);
            let a_upper = _mm256_cvtpd_ps(a[1]);
            let a256 = _mm256_insertf128_ps::<1>(_mm256_castps128_ps256(a_lower), a_upper);

            let b : [__m256d; 2] = b.into();
            let b_lower = _mm256_cvtpd_ps(b[0]);
            let b_upper = _mm256_cvtpd_ps(b[1]);
            let b256 = _mm256_insertf128_ps::<1>(_mm256_castps128_ps256(b_lower), b_upper);

            [a256, b256].into()
        }
    }
}