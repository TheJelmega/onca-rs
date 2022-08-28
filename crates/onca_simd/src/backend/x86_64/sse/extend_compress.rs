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
            impl SimdExtendCompressImpl<{BackendType::SSE}> for Simd<$ty, $lanes128> {
                type ExtendedType = Simd<$e_ty, $e_lanes128>;

                fn simd_extend_lower_impl(self) -> Self::ExtendedType {
                    self.simd_convert::<$e_ty, $e_lanes128, {BackendType::SSE}>()
                }
            
                fn simd_extend_upper_impl(self) -> Self::ExtendedType {
                    unsafe {
                        let upper : Simd<$ty, $lanes128> = _mm_srli_si128::<8>(self.into()).into();
                        self.simd_convert::<$e_ty, $e_lanes128, {BackendType::SSE}>()
                    }
                }
            
                fn simd_compress_impl(a: Self::ExtendedType, b: Self::ExtendedType) -> Self {
                    unsafe {
                        let lower : __m128i = a.simd_convert::<$ty, $lanes128, {BackendType::SSE}>().into();
                        let upper : __m128i = b.simd_convert::<$ty, $lanes128, {BackendType::SSE}>().into();
                        let upper = _mm_slli_si128::<8>(upper);
                        _mm_or_si128(lower, upper).into()
                    }
                }
            }

            impl SimdExtendCompressImpl<{BackendType::SSE}> for Simd<$ty, $lanes256> {
                type ExtendedType = Simd<$e_ty, $e_lanes256>;
            
                fn simd_extend_lower_impl(self) -> Self::ExtendedType {
                    unsafe {
                        let f = self.split_2();
                        let lower = f[0].simd_convert::<$e_ty, $e_lanes128, {BackendType::SSE}>();
                        let upper : Simd<$ty, $lanes128> = _mm_srli_si128::<8>(f[0].into()).into();
                        let upper = upper.simd_convert::<$e_ty, $e_lanes128, {BackendType::SSE}>();
                        Simd::<$e_ty, $e_lanes256>::combine_2([lower, upper])
                    }
                }
            
                fn simd_extend_upper_impl(self) -> Self::ExtendedType {
                    unsafe {
                        let f = self.split_2();
                        let lower = f[1].simd_convert::<$e_ty, $e_lanes128, {BackendType::SSE}>();
                        let upper : Simd<$ty, $lanes128> = _mm_srli_si128::<8>(f[1].into()).into();
                        let upper = upper.simd_convert::<$e_ty, $e_lanes128, {BackendType::SSE}>();
                        Simd::<$e_ty, $e_lanes256>::combine_2([lower, upper])
                    }
                }
            
                fn simd_compress_impl(a: Self::ExtendedType, b: Self::ExtendedType) -> Self {
                    unsafe {
                        let zero = _mm_setzero_si128();

                        let a = a.split_2();
                        let a_lower : __m128i = a[0].simd_convert::<$ty, $lanes128, {BackendType::SSE}>().into();
                        let a_upper : __m128i = a[1].simd_convert::<$ty, $lanes128, {BackendType::SSE}>().into();
                        let a_upper = _mm_srli_si128::<8>(a_upper);
            
                        let b = b.split_2();
                        let b_lower : __m128i = b[0].simd_convert::<$ty, $lanes128, {BackendType::SSE}>().into();
                        let b_upper : __m128i = b[1].simd_convert::<$ty, $lanes128, {BackendType::SSE}>().into();
                        let b_upper = _mm_srli_si128::<8>(b_upper);
            
                        [_mm_or_si128(a_lower, a_upper), _mm_or_si128(b_lower, b_upper)].into()
                    }
                }
            }
            
            impl SimdExtendCompressImpl<{BackendType::SSE}> for Simd<$ty, $lanes512> {
                type ExtendedType = Simd<$e_ty, $e_lanes512>;
            
                fn simd_extend_lower_impl(self) -> Self::ExtendedType {
                    unsafe {
                        let f : [__m128i; 4] = self.into();
            
                        let lower0 = $cvt_ext(f[0]);
                        let upper0 = _mm_srli_si128::<8>(f[0]);
                        let upper0 = $cvt_ext(upper0);
            
                        let lower1 = $cvt_ext(f[1]);
                        let upper1 = _mm_srli_si128::<8>(f[1]);
                        let upper1 = $cvt_ext(upper1);
            
                        [lower0, upper0, lower1, upper1].into()
                    }
                }
            
                fn simd_extend_upper_impl(self) -> Self::ExtendedType {
                    unsafe {
                        let f : [__m128i; 4] = self.into();
            
                        let lower2 = $cvt_ext(f[2]);
                        let upper2 = _mm_srli_si128::<8>(f[2]);
                        let upper2 = $cvt_ext(upper2);
            
                        let lower3 = $cvt_ext(f[3]);
                        let upper3 = _mm_srli_si128::<8>(f[3]);
                        let upper3 = $cvt_ext(upper3);
            
                        [lower2, upper2, lower3, upper3].into()
                    }
                }
            
                fn simd_compress_impl(a: Self::ExtendedType, b: Self::ExtendedType) -> Self {
                    unsafe {
                        let zero = _mm_setzero_si128();

                        let a = a.split_4();
                        let a_lower0 : __m128i = a[0].simd_convert::<$ty, $lanes128, {BackendType::SSE}>().into();
                        let a_upper0 : __m128i = a[1].simd_convert::<$ty, $lanes128, {BackendType::SSE}>().into();
                        let a_upper0 = _mm_srli_si128::<8>(a_upper0);
            
                        let a_lower1 : __m128i = a[2].simd_convert::<$ty, $lanes128, {BackendType::SSE}>().into();
                        let a_upper1 : __m128i = a[3].simd_convert::<$ty, $lanes128, {BackendType::SSE}>().into();
                        let a_upper1 = _mm_srli_si128::<8>(a_upper1);
            
                        let b = b.split_4();
                        let b_lower0 : __m128i = b[0].simd_convert::<$ty, $lanes128, {BackendType::SSE}>().into();
                        let b_upper0 : __m128i = b[1].simd_convert::<$ty, $lanes128, {BackendType::SSE}>().into();
                        let b_upper0 = _mm_srli_si128::<8>(b_upper0);
            
                        let b_lower1 : __m128i = b[2].simd_convert::<$ty, $lanes128, {BackendType::SSE}>().into();
                        let b_upper1 : __m128i = b[3].simd_convert::<$ty, $lanes128, {BackendType::SSE}>().into();
                        let b_upper1 = _mm_srli_si128::<8>(b_upper1);
            
                        [_mm_or_si128(a_lower0, a_upper0), 
                         _mm_or_si128(a_lower1, a_upper1),
                         _mm_or_si128(b_lower0, b_upper0),
                         _mm_or_si128(b_lower1, b_upper1)].into()
                    }
                }
            }
        )*
    };
}
impl_cvt_int!{
    [i8 , 16, 32, 64 <=> i16, 8 , 16, 32; _mm_cvtepi8_epi16, _mm_packus_epi16]
    [i16, 8 , 16, 32 <=> i32, 4 , 8 , 16; _mm_cvtepi16_epi32, _mm_packus_epi32]
    //[i32, 4 , 8 , 16 <=> i64, 2 , 4 , 8 ; _mm_cvtepi32_epi64, _mm_packus_epi32]

    [u8 , 16, 32, 64 <=> u16, 8 , 16, 32; _mm_cvtepu8_epi16, _mm_packus_epi16]
    [u16, 8 , 16, 32 <=> u32, 4 , 8 , 16; _mm_cvtepu16_epi32, _mm_packus_epi32]
    //[u32, 4 , 8 , 16 <=> u64, 2 , 4 , 8 ; _mm_cvtepu32_epi64, _mm_packus_epi32]
}

impl SimdExtendCompressImpl<{BackendType::SSE}> for Simd<i32, 4> {
    type ExtendedType = Simd<i64, 2>;

    fn simd_extend_lower_impl(self) -> Self::ExtendedType {
        unsafe { _mm_cvtepi32_epi64(self.into()).into() }
    }

    fn simd_extend_upper_impl(self) -> Self::ExtendedType {
        unsafe {
            let upper = _mm_srli_si128::<8>(self.into());
            _mm_cvtepi32_epi64(upper).into()
        }
    }

    fn simd_compress_impl(a: Self::ExtendedType, b: Self::ExtendedType) -> Self {
        unsafe {
            let lower : __m128i = a.simd_convert::<i32, 4, {BackendType::SSE}>().into();
            let upper : __m128i = b.simd_convert::<i32, 4, {BackendType::SSE}>().into();
            let upper  = _mm_slli_si128::<8>(upper);
            _mm_or_si128(lower,  upper).into()
        }
    }
}

impl SimdExtendCompressImpl<{BackendType::SSE}> for Simd<i32, 8> {
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
            let a_lower : __m128i = a[0].simd_convert::<i32, 4, {BackendType::SSE}>().into();
            let a_upper : __m128i = a[1].simd_convert::<i32, 4, {BackendType::SSE}>().into();
            let a_upper = _mm_srli_si128::<8>(a_upper);

            let b = b.split_2();
            let b_lower : __m128i = b[0].simd_convert::<i32, 4, {BackendType::SSE}>().into();
            let b_upper : __m128i = b[1].simd_convert::<i32, 4, {BackendType::SSE}>().into();
            let b_upper = _mm_srli_si128::<8>(b_upper);

            [_mm_or_si128(a_lower, a_upper), _mm_or_si128(b_lower, b_upper)].into()
        }
    }
}

impl SimdExtendCompressImpl<{BackendType::SSE}> for Simd<i32, 16> {
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
            let a_lower0 : __m128i = a[0].simd_convert::<i32, 4, {BackendType::SSE}>().into();
            let a_upper0 : __m128i = a[1].simd_convert::<i32, 4, {BackendType::SSE}>().into();
            let a_upper0 = _mm_srli_si128::<8>(a_upper0);

            let a_lower1 : __m128i = a[2].simd_convert::<i32, 4, {BackendType::SSE}>().into();
            let a_upper1 : __m128i = a[3].simd_convert::<i32, 4, {BackendType::SSE}>().into();
            let a_upper1 = _mm_srli_si128::<8>(a_upper1);

            let b  = b.split_4();
            let b_lower0 : __m128i = b[0].simd_convert::<i32, 4, {BackendType::SSE}>().into();
            let b_upper0 : __m128i = b[1].simd_convert::<i32, 4, {BackendType::SSE}>().into();
            let b_upper0 = _mm_srli_si128::<8>(b_upper0);

            let b_lower1 : __m128i = b[2].simd_convert::<i32, 4, {BackendType::SSE}>().into();
            let b_upper1 : __m128i = b[3].simd_convert::<i32, 4, {BackendType::SSE}>().into();
            let b_upper1 = _mm_srli_si128::<8>(b_upper1);

            [_mm_or_si128(a_lower0, a_upper0), 
             _mm_or_si128(a_lower1, a_upper1),
             _mm_or_si128(b_lower0, b_upper0),
             _mm_or_si128(b_lower1, b_upper1)].into()
        }
    }
}

impl SimdExtendCompressImpl<{BackendType::SSE}> for Simd<u32, 4> {
    type ExtendedType = Simd<u64, 2>;

    fn simd_extend_lower_impl(self) -> Self::ExtendedType {
        unsafe { _mm_cvtepu32_epi64(self.into()).into() }
    }

    fn simd_extend_upper_impl(self) -> Self::ExtendedType {
        unsafe {
            let upper = _mm_srli_si128::<8>(self.into());
            _mm_cvtepu32_epi64(upper).into()
        }
    }

    fn simd_compress_impl(a: Self::ExtendedType, b: Self::ExtendedType) -> Self {
        unsafe {
            let lower : __m128i = a.simd_convert::<u32, 4, {BackendType::SSE}>().into();
            let upper : __m128i = b.simd_convert::<u32, 4, {BackendType::SSE}>().into();
            let upper  = _mm_slli_si128::<8>(upper);
            _mm_or_si128(lower,  upper).into()
        }
    }
}

impl SimdExtendCompressImpl<{BackendType::SSE}> for Simd<u32, 8> {
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
            let a_lower : __m128i = a[0].simd_convert::<u32, 4, {BackendType::SSE}>().into();
            let a_upper : __m128i = a[1].simd_convert::<u32, 4, {BackendType::SSE}>().into();
            let a_upper = _mm_srli_si128::<8>(a_upper);

            let b = b.split_2();
            let b_lower : __m128i = b[0].simd_convert::<u32, 4, {BackendType::SSE}>().into();
            let b_upper : __m128i = b[1].simd_convert::<u32, 4, {BackendType::SSE}>().into();
            let b_upper = _mm_srli_si128::<8>(b_upper);

            [_mm_or_si128(a_lower, a_upper), _mm_or_si128(b_lower, b_upper)].into()
        }
    }
}

impl SimdExtendCompressImpl<{BackendType::SSE}> for Simd<u32, 16> {
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
            let a_lower0 : __m128i = a[0].simd_convert::<u32, 4, {BackendType::SSE}>().into();
            let a_upper0 : __m128i = a[1].simd_convert::<u32, 4, {BackendType::SSE}>().into();
            let a_upper0 = _mm_srli_si128::<8>(a_upper0);

            let a_lower1 : __m128i = a[2].simd_convert::<u32, 4, {BackendType::SSE}>().into();
            let a_upper1 : __m128i = a[3].simd_convert::<u32, 4, {BackendType::SSE}>().into();
            let a_upper1 = _mm_srli_si128::<8>(a_upper1);

            let b  = b.split_4();
            let b_lower0 : __m128i = b[0].simd_convert::<u32, 4, {BackendType::SSE}>().into();
            let b_upper0 : __m128i = b[1].simd_convert::<u32, 4, {BackendType::SSE}>().into();
            let b_upper0 = _mm_srli_si128::<8>(b_upper0);

            let b_lower1 : __m128i = b[2].simd_convert::<u32, 4, {BackendType::SSE}>().into();
            let b_upper1 : __m128i = b[3].simd_convert::<u32, 4, {BackendType::SSE}>().into();
            let b_upper1 = _mm_srli_si128::<8>(b_upper1);

            [_mm_or_si128(a_lower0, a_upper0), 
             _mm_or_si128(a_lower1, a_upper1),
             _mm_or_si128(b_lower0, b_upper0),
             _mm_or_si128(b_lower1, b_upper1)].into()
        }
    }
}

impl SimdExtendCompressImpl<{BackendType::SSE}> for Simd<f32, 4> {
    type ExtendedType = Simd<f64, 2>;

    fn simd_extend_lower_impl(self) -> Self::ExtendedType {
        unsafe{ _mm_cvtps_pd(self.into()).into() }
    }

    fn simd_extend_upper_impl(self) -> Self::ExtendedType {
        unsafe {
            let upper = _mm_castsi128_ps(_mm_srli_si128::<8>(_mm_castps_si128(self.into())));
            _mm_cvtps_pd(upper).into()
        }
    }

    fn simd_compress_impl(a: Self::ExtendedType, b: Self::ExtendedType) -> Self {
        unsafe {
            let upper = _mm_castsi128_pd(_mm_slli_si128::<8>(_mm_castpd_si128(b.into())));
            _mm_or_ps(_mm_cvtpd_ps(a.into()), _mm_cvtpd_ps(upper)).into()
        }
    }
}

impl SimdExtendCompressImpl<{BackendType::SSE}> for Simd<f32, 8> {
    type ExtendedType = Simd<f64, 4>;

    fn simd_extend_lower_impl(self) -> Self::ExtendedType {
        unsafe {
            let f : [__m128; 2] = self.into();
            let lower = _mm_cvtps_pd(f[0]);
            let upper = _mm_castsi128_ps(_mm_srli_si128::<8>(_mm_castps_si128(f[0])));
            let upper = _mm_cvtps_pd(upper);
            [lower, upper].into()
        }
    }

    fn simd_extend_upper_impl(self) -> Self::ExtendedType {
        unsafe {
            let f : [__m128; 2] = self.into();
            let lower = _mm_cvtps_pd(f[1]);
            let upper = _mm_castsi128_ps(_mm_srli_si128::<8>(_mm_castps_si128(f[1])));
            let upper = _mm_cvtps_pd(upper);
            [lower, upper].into()
        }
    }

    fn simd_compress_impl(a: Self::ExtendedType, b: Self::ExtendedType) -> Self {
        unsafe {
            let a : [__m128d; 2] = a.into();
            let a_lower = _mm_cvtpd_ps(a[0]);
            let a_upper = _mm_cvtpd_ps(a[1]);
            let a_upper = _mm_castsi128_ps(_mm_srli_si128::<8>(_mm_castps_si128(a_upper)));

            let b : [__m128d; 2] = b.into();
            let b_lower = _mm_cvtpd_ps(b[0]);
            let b_upper = _mm_cvtpd_ps(b[1]);
            let b_upper = _mm_castsi128_ps(_mm_srli_si128::<8>(_mm_castps_si128(b_upper)));

            [_mm_or_ps(a_lower, a_upper), _mm_or_ps(b_lower, b_upper)].into()
        }
    }
}

impl SimdExtendCompressImpl<{BackendType::SSE}> for Simd<f32, 16> {
    type ExtendedType = Simd<f64, 8>;

    fn simd_extend_lower_impl(self) -> Self::ExtendedType {
        unsafe {
            let f : [__m128; 4] = self.into();

            let lower0 = _mm_cvtps_pd(f[0]);
            let upper0 = _mm_castsi128_ps(_mm_srli_si128::<8>(_mm_castps_si128(f[0])));
            let upper0 = _mm_cvtps_pd(upper0);

            let lower1 = _mm_cvtps_pd(f[1]);
            let upper1 = _mm_castsi128_ps(_mm_srli_si128::<8>(_mm_castps_si128(f[1])));
            let upper1 = _mm_cvtps_pd(upper1);

            [lower0, upper0, lower1, upper1].into()
        }
    }

    fn simd_extend_upper_impl(self) -> Self::ExtendedType {
        unsafe {
            let f : [__m128; 4] = self.into();

            let lower2 = _mm_cvtps_pd(f[2]);
            let upper2 = _mm_castsi128_ps(_mm_srli_si128::<8>(_mm_castps_si128(f[2])));
            let upper2 = _mm_cvtps_pd(upper2);

            let lower3 = _mm_cvtps_pd(f[3]);
            let upper3 = _mm_castsi128_ps(_mm_srli_si128::<8>(_mm_castps_si128(f[3])));
            let upper3 = _mm_cvtps_pd(upper3);

            [lower2, upper2, lower3, upper3].into()
        }
    }

    fn simd_compress_impl(a: Self::ExtendedType, b: Self::ExtendedType) -> Self {
        unsafe {
            let a : [__m128d; 4] = a.into();
            let a_lower0 = _mm_cvtpd_ps(a[0]);
            let a_upper0 = _mm_cvtpd_ps(a[1]);
            let a_upper0 = _mm_castsi128_ps(_mm_srli_si128::<8>(_mm_castps_si128(a_upper0)));

            let a_lower1 = _mm_cvtpd_ps(a[2]);
            let a_upper1 = _mm_cvtpd_ps(a[3]);
            let a_upper1 = _mm_castsi128_ps(_mm_srli_si128::<8>(_mm_castps_si128(a_upper1)));

            let b : [__m128d; 4] = b.into();
            let b_lower0 = _mm_cvtpd_ps(b[0]);
            let b_upper0 = _mm_cvtpd_ps(b[1]);
            let b_upper0 = _mm_castsi128_ps(_mm_srli_si128::<8>(_mm_castps_si128(b_upper0)));

            let b_lower1 = _mm_cvtpd_ps(b[2]);
            let b_upper1 = _mm_cvtpd_ps(b[3]);
            let b_upper1 = _mm_castsi128_ps(_mm_srli_si128::<8>(_mm_castps_si128(b_upper1)));

            [_mm_or_ps(a_lower0, a_upper0), 
             _mm_or_ps(a_lower1, a_upper1),
             _mm_or_ps(b_lower0, b_upper0),
             _mm_or_ps(b_lower1, b_upper1)].into()
        }
    }
}