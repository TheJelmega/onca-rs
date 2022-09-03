use core::arch::x86_64::*;

use super::*;
use crate::{
    *,
    backend::*,
    SimdElement, Simd, LaneCount, SupportedLaneCount
};

macro_rules! impl_cast_to_self {
    { $($ty:ty, $lanes128:literal, $lanes256:literal, $lanes512:literal),* } => {
        $(
            impl SimdConvertImpl<$ty, $lanes128, {BackendType::AVX}> for Simd<$ty, $lanes128>
                where LaneCount<$lanes128> : SupportedLaneCount
            {
                #[inline]
                fn simd_convert_impl(self) -> Self {
                    self
                }
            }

            impl SimdConvertImpl<$ty, $lanes256, {BackendType::AVX}> for Simd<$ty, $lanes256>
                where LaneCount<$lanes256> : SupportedLaneCount
            {
                #[inline]
                fn simd_convert_impl(self) -> Self {
                    self
                }
            }

            impl SimdConvertImpl<$ty, $lanes512, {BackendType::AVX}> for Simd<$ty, $lanes512>
                where LaneCount<$lanes512> : SupportedLaneCount
            {
                #[inline]
                fn simd_convert_impl(self) -> Self {
                    self
                }
            }
        )*
    };
}
impl_cast_to_self!{ 
    i8 , 16, 32, 64,
    i16, 8 , 16, 32,
    i32, 4 , 8 , 16,
    i64, 2 , 4 , 8 ,
    u8 , 16, 32, 64,
    u16, 8 , 16, 32,
    u32, 4 , 8 , 16,
    u64, 2 , 4 , 8 ,
    f32, 4 , 8 , 16,
    f64, 2 , 4 , 8 
}

macro_rules! impl_via_sse {
    { $([$from_ty:ty, $from_lanes128:literal, $from_lanes256:literal, $from_lanes512:literal => $to_ty:ty, $to_lanes128:literal, $to_lanes256:literal, $to_lanes512:literal])* } => {
        $(
            impl SimdConvertImpl<$to_ty, $to_lanes128, {BackendType::AVX}> for Simd<$from_ty, $from_lanes128> {
                #[inline]
                fn simd_convert_impl(self) -> Simd<$to_ty, $to_lanes128> {
                    <Self as SimdConvertImpl<$to_ty, $to_lanes128, {BackendType::SSE}>>::simd_convert_impl(self)
                }

                #[inline]
                fn simd_convert_saturate_impl(self) -> Simd<$to_ty, $to_lanes128> {
                    <Self as SimdConvertImpl<$to_ty, $to_lanes128, {BackendType::SSE}>>::simd_convert_saturate_impl(self)
                }
            }

            impl SimdConvertImpl<$to_ty, $to_lanes256, {BackendType::AVX}> for Simd<$from_ty, $from_lanes256> {
                #[inline]
                fn simd_convert_impl(self) -> Simd<$to_ty, $to_lanes256> {
                    <Self as SimdConvertImpl<$to_ty, $to_lanes256, {BackendType::SSE}>>::simd_convert_impl(self)
                }

                #[inline]
                fn simd_convert_saturate_impl(self) -> Simd<$to_ty, $to_lanes256> {
                    <Self as SimdConvertImpl<$to_ty, $to_lanes256, {BackendType::SSE}>>::simd_convert_saturate_impl(self)
                }
            }

            impl SimdConvertImpl<$to_ty, $to_lanes512, {BackendType::AVX}> for Simd<$from_ty, $from_lanes512> {
                #[inline]
                fn simd_convert_impl(self) -> Simd<$to_ty, $to_lanes512> {
                    <Self as SimdConvertImpl<$to_ty, $to_lanes512, {BackendType::SSE}>>::simd_convert_impl(self)
                }

                #[inline]
                fn simd_convert_saturate_impl(self) -> Simd<$to_ty, $to_lanes512> {
                    <Self as SimdConvertImpl<$to_ty, $to_lanes512, {BackendType::SSE}>>::simd_convert_saturate_impl(self)
                }
            }
        )*
    };
}
impl_via_sse!{
    [i8 , 16, 32, 64 => u8 , 16, 32, 64]
    [u8 , 16, 32, 64 => i8 , 16, 32, 64]
    [i16,  8, 16, 32 => u16,  8, 16, 32]
    [u16,  8, 16, 32 => i16,  8, 16, 32]
    [i32,  4,  8, 16 => u32,  4,  8, 16]
    [u32,  4,  8, 16 => i32,  4,  8, 16]
    [i64,  2,  4,  8 => u64,  2,  4,  8]
    [u64,  2,  4,  8 => i64,  2,  4,  8]

    [i64,  2,  4,  8 => i8 , 16, 32, 64]
    [i64,  2,  4,  8 => i16,  8, 16, 32]
    [i64,  2,  4,  8 => i32,  4,  8, 16]

    [i64,  2,  4,  8 => u8 , 16, 32, 64]
    [i64,  2,  4,  8 => u16,  8, 16, 32]
    [i64,  2,  4,  8 => u32,  4,  8, 16]

    [u64,  2,  4,  8 => u8 , 16, 32, 64]
    [u64,  2,  4,  8 => u16,  8, 16, 32]
    [u64,  2,  4,  8 => u32,  4,  8, 16]

    [u64,  2,  4,  8 => i8 , 16, 32, 64]
    [u64,  2,  4,  8 => i16,  8, 16, 32]
    [u64,  2,  4,  8 => i32,  4,  8, 16]

    [i16,  8, 16, 32 => i8 , 16, 32, 64]
    [i32,  4,  8, 16 => i8 , 16, 32, 64]
    [i32,  4,  8, 16 => i16,  8, 16, 32]

    [i16,  8, 16, 32 => u8 , 16, 32, 64]
    [i32,  4,  8, 16 => u8 , 16, 32, 64]
    [i32,  4,  8, 16 => u16,  8, 16, 32]

    [u16,  8, 16, 32 => u8 , 16, 32, 64]
    [u32,  4,  8, 16 => u8 , 16, 32, 64]
    [u32,  4,  8, 16 => u16,  8, 16, 32]

    [u16,  8, 16, 32 => i8 , 16, 32, 64]
    [u32,  4,  8, 16 => i8 , 16, 32, 64]
    [u32,  4,  8, 16 => i16,  8, 16, 32]

    [i8 , 16, 32, 64 => i16,  8, 16, 32]
    [i8 , 16, 32, 64 => i32,  4,  8, 16]
    [i8 , 16, 32, 64 => u64,  2,  4,  8]
    [i16,  8, 16, 32 => i32,  4,  8, 16]
    [i16,  8, 16, 32 => i64,  2,  4,  8]
    [i32,  4,  8, 16 => i64,  2,  4,  8]

    [i8 , 16, 32, 64 => u16,  8, 16, 32]
    [i8 , 16, 32, 64 => u32,  4,  8, 16]
    [i8 , 16, 32, 64 => i64,  2,  4,  8]
    [i16,  8, 16, 32 => u32,  4,  8, 16]
    [i16,  8, 16, 32 => u64,  2,  4,  8]
    [i32,  4,  8, 16 => u64,  2,  4,  8]

    [u8 , 16, 32, 64 => u16,  8, 16, 32]
    [u8 , 16, 32, 64 => u32,  4,  8, 16]
    [u8 , 16, 32, 64 => u64,  2,  4,  8]
    [u16,  8, 16, 32 => u32,  4,  8, 16]
    [u16,  8, 16, 32 => u64,  2,  4,  8]
    [u32,  4,  8, 16 => u64,  2,  4,  8]

    [u8 , 16, 32, 64 => i16,  8, 16, 32]
    [u8 , 16, 32, 64 => i32,  4,  8, 16]
    [u8 , 16, 32, 64 => i64,  2,  4,  8]
    [u16,  8, 16, 32 => i32,  4,  8, 16]
    [u16,  8, 16, 32 => i64,  2,  4,  8]
    [u32,  4,  8, 16 => i64,  2,  4,  8]

    [i8 ,  8, 16, 32 => i16,  8, 16, 32]
    [i8 ,  4,  8, 16 => i32,  4,  8, 16]
    [i8 ,  2,  4,  8 => i64,  2,  4,  8]
    [i16,  4,  8, 16 => i32,  4,  8, 16]
    [i16,  2,  4,  8 => i64,  2,  4,  8]
    [i32,  2,  4,  8 => i64,  2,  4,  8]
}

//==============================================================================================================================

impl SimdConvertImpl<f64, 2, {BackendType::AVX}> for Simd<f32, 4> {
    fn simd_convert_impl(self) -> Simd<f64, 2> {
        <Self as SimdConvertImpl<f64, 2, {BackendType::SSE}>>::simd_convert_impl(self)
    }
}

impl SimdConvertImpl<f64, 4, {BackendType::AVX}> for Simd<f32, 8> {
    fn simd_convert_impl(self) -> Simd<f64, 4> {
        unsafe { _mm256_cvtps_pd(_mm256_castps256_ps128(self.into())).into() }
    }
}

impl SimdConvertImpl<f64, 8, {BackendType::AVX}> for Simd<f32, 16> {
    fn simd_convert_impl(self) -> Simd<f64, 8> {
        unsafe {
            let a : [__m256; 2] = self.into();
            let lower = _mm256_cvtps_pd(_mm256_castps256_ps128(a[0]));
            let upper_ps = _mm256_extractf128_ps::<1>(a[0]);
            let upper = _mm256_cvtps_pd(upper_ps);
            [lower, upper].into()
        }
    }
}

impl SimdConvertImpl<f32, 4, {BackendType::AVX}> for Simd<f64, 2> {
    fn simd_convert_impl(self) -> Simd<f32, 4> {
        <Self as SimdConvertImpl<f32, 4, {BackendType::SSE}>>::simd_convert_impl(self)
    }

    fn simd_convert_saturate_impl(self) -> Simd<f32, 4> {
        <Self as SimdConvertImpl<f32, 4, {BackendType::SSE}>>::simd_convert_saturate_impl(self)
    }  
}

impl SimdConvertImpl<f32, 8, {BackendType::AVX}> for Simd<f64, 4> {
    fn simd_convert_impl(self) -> Simd<f32, 8> {
        unsafe { _mm256_castps128_ps256(_mm256_cvtpd_ps(self.into())).into() }
    }

    fn simd_convert_saturate_impl(self) -> Simd<f32, 8> {
        unsafe {
            let min = Simd::<f64, 4>::splat(f32::MIN as f64);
            let max = Simd::<f64, 4>::splat(f32::MAX as f64);
            Self::convert(self.simd_clamp::<{BackendType::AVX}>(min, max))
        }
    }
}

impl SimdConvertImpl<f32, 16, {BackendType::AVX}> for Simd<f64, 8> {
    fn simd_convert_impl(self) -> Simd<f32, 16> {
        unsafe {
            let a : [__m256d; 2] = self.into();
            let upper = _mm256_cvtpd_ps(a[0]);
            let lower = _mm256_cvtpd_ps(a[1]);
            [_mm256_insertf128_ps::<1>(_mm256_castps128_ps256(lower), upper), _mm256_setzero_ps()].into()
        }
    }

    fn simd_convert_saturate_impl(self) -> Simd<f32, 16> {
        unsafe {
            let min = Simd::<f64, 4>::splat(f32::MIN as f64);
            let max = Simd::<f64, 4>::splat(f32::MAX as f64);

            let unclamped = self.split_2();
            let clamped = [
                unclamped[0].simd_clamp::<{BackendType::AVX}>(min, max),
                unclamped[1].simd_clamp::<{BackendType::AVX}>(min, max),
            ];
            Self::convert(clamped.into())
        }
    }
}

//==============================================================================================================================

impl SimdConvertImpl<i32, 4, {BackendType::AVX}> for Simd<f32, 4> {
    fn simd_convert_impl(self) -> Simd<i32, 4> {
        <Self as SimdConvertImpl<i32, 4, {BackendType::SSE}>>::simd_convert_impl(self)
    }

    fn simd_convert_saturate_impl(self) -> Simd<i32, 4> {
        <Self as SimdConvertImpl<i32, 4, {BackendType::SSE}>>::simd_convert_saturate_impl(self)
    }
}

impl SimdConvertImpl<i32, 8, {BackendType::AVX}> for Simd<f32, 8> {
    fn simd_convert_impl(self) -> Simd<i32, 8> {
        unsafe { _mm256_cvtps_epi32(self.into()).into() }
    }

    fn simd_convert_saturate_impl(self) -> Simd<i32, 8> {
        unsafe {
            let min = Simd::<f32, 8>::splat(i32::MIN as f32);
            let max = Simd::<f32, 8>::splat(i32::MAX as f32);
            Self::convert(self.simd_clamp::<{BackendType::AVX}>(min, max))
        }
    }
}

impl SimdConvertImpl<i32, 16, {BackendType::AVX}> for Simd<f32, 16> {
    fn simd_convert_impl(self) -> Simd<i32, 16> {
        unsafe { 
            let a : [__m256; 2] = self.into();
            [_mm256_cvtps_epi32(a[0]), _mm256_cvtps_epi32(a[1])].into()
        }
    }

    fn simd_convert_saturate_impl(self) -> Simd<i32, 16> {
        unsafe {
            let min = Simd::<f32, 8>::splat(i32::MIN as f32);
            let max = Simd::<f32, 8>::splat(i32::MAX as f32);

            let unclamped = self.split_2();
            let clamped = [
                unclamped[0].simd_clamp::<{BackendType::AVX}>(min, max),
                unclamped[1].simd_clamp::<{BackendType::AVX}>(min, max),
            ];
            Self::convert(clamped.into())
        }
    }
}

//==============================================================================================================================

impl SimdConvertImpl<i32, 4, {BackendType::AVX}> for Simd<f64, 2> {
    fn simd_convert_impl(self) -> Simd<i32, 4> {
        <Self as SimdConvertImpl<i32, 4, {BackendType::SSE}>>::simd_convert_impl(self)
    }

    fn simd_convert_saturate_impl(self) -> Simd<i32, 4> {
        <Self as SimdConvertImpl<i32, 4, {BackendType::SSE}>>::simd_convert_saturate_impl(self)
    }
}

impl SimdConvertImpl<i32, 8, {BackendType::AVX}> for Simd<f64, 4> {
    fn simd_convert_impl(self) -> Simd<i32, 8> {
        unsafe { _mm256_castsi128_si256(_mm256_cvtpd_epi32(self.into())).into() }
    }

    fn simd_convert_saturate_impl(self) -> Simd<i32, 8> {
        unsafe {
            let min = Simd::<f64, 4>::splat(i32::MIN as f64);
            let max = Simd::<f64, 4>::splat(i32::MAX as f64);
            Self::convert(self.simd_clamp::<{BackendType::AVX}>(min, max))
        }
    }
}

impl SimdConvertImpl<i32, 16, {BackendType::AVX}> for Simd<f64, 8> {
    fn simd_convert_impl(self) -> Simd<i32, 16> {
        unsafe { 
            let a : [__m256d; 2] = self.into();
            let lower = _mm256_cvtpd_epi32(a[0]);
            let upper = _mm256_cvtpd_epi32(a[1]);
            let combined = _mm256_insertf128_si256::<1>(_mm256_castsi128_si256(lower), upper);
            [combined, _mm256_setzero_si256()].into()
        }
    }

    fn simd_convert_saturate_impl(self) -> Simd<i32, 16> {
        unsafe {
            let min = Simd::<f64, 4>::splat(i32::MIN as f64);
            let max = Simd::<f64, 4>::splat(i32::MAX as f64);

            let unclamped = self.split_2();
            let clamped = [
                unclamped[0].simd_clamp::<{BackendType::AVX}>(min, max),
                unclamped[1].simd_clamp::<{BackendType::AVX}>(min, max),
            ];
            Self::convert(clamped.into())
        }
    }
}

//==============================================================================================================================

impl SimdConvertImpl<i64, 2, {BackendType::AVX}> for Simd<f64, 2> {
    fn simd_convert_impl(self) -> Simd<i64, 2> {
        <Self as SimdConvertImpl<i64, 2, {BackendType::SSE}>>::simd_convert_impl(self)
    }

    fn simd_convert_saturate_impl(self) -> Simd<i64, 2> {
        <Self as SimdConvertImpl<i64, 2, {BackendType::SSE}>>::simd_convert_saturate_impl(self)
    }
}

impl SimdConvertImpl<i64, 4, {BackendType::AVX}> for Simd<f64, 4> {
    // https://stackoverflow.com/questions/41144668/how-to-efficiently-perform-double-int64-conversions-with-sse-avx
    // FIXME(jel): Only for [-2^51, 2^51]
    fn simd_convert_impl(self) -> Simd<i64, 4> {
        <Self as SimdConvertImpl<i64, 4, {BackendType::SSE}>>::simd_convert_impl(self)
    }

    fn simd_convert_saturate_impl(self) -> Simd<i64, 4> {
        unsafe {
            let min = Simd::<f64, 4>::splat(i64::MIN as f64);
            let max = Simd::<f64, 4>::splat(i64::MAX as f64);
            Self::convert(self.simd_clamp::<{BackendType::SSE}>(min, max))
        }
    }
}

impl SimdConvertImpl<i64, 8, {BackendType::AVX}> for Simd<f64, 8> {
    fn simd_convert_impl(self) -> Simd<i64, 8> {
        <Self as SimdConvertImpl<i64, 8, {BackendType::SSE}>>::simd_convert_impl(self)
    }

    fn simd_convert_saturate_impl(self) -> Simd<i64, 8> {
        unsafe {
            let min = Simd::<f64, 4>::splat(i64::MIN as f64);
            let max = Simd::<f64, 4>::splat(i64::MAX as f64);

            let unclamped = self.split_2();
            let clamped = [
                unclamped[0].simd_clamp::<{BackendType::AVX}>(min, max),
                unclamped[1].simd_clamp::<{BackendType::AVX}>(min, max)
            ];
            Self::convert(clamped.into())
        }
    }
}

//==============================================================================================================================

impl SimdConvertImpl<u64, 2, {BackendType::AVX}> for Simd<f64, 2> {
    fn simd_convert_impl(self) -> Simd<u64, 2> {
        <Self as SimdConvertImpl<u64, 2, {BackendType::SSE}>>::simd_convert_impl(self)
    }

    fn simd_convert_saturate_impl(self) -> Simd<u64, 2> {
        <Self as SimdConvertImpl<u64, 2, {BackendType::SSE}>>::simd_convert_saturate_impl(self)
    }
}

impl SimdConvertImpl<u64, 4, {BackendType::AVX}> for Simd<f64, 4> {
    // https://stackoverflow.com/questions/41144668/how-to-efficiently-perform-double-int64-conversions-with-sse-avx
    fn simd_convert_impl(self) -> Simd<u64, 4> {
        unsafe {
            let a : [__m128d; 2] = self.into();
            let cnst = _mm256_set1_pd(0x0010000000000000u64 as f64);
            cvt_f64_u64(self.into(), cnst).into()
        }
    }

    fn simd_convert_saturate_impl(self) -> Simd<u64, 4> {
        unsafe {
            let a : [__m128d; 2] = self.into();
            let min = Simd::<f64, 2>::splat(u64::MIN as f64);
            let max = Simd::<f64, 2>::splat(u64::MAX as f64);

            let clamped : [__m128d; 2] = [
                Simd::<f64, 2>::from(a[0]).simd_clamp::<{BackendType::AVX}>(min, max).into(),
                Simd::<f64, 2>::from(a[1]).simd_clamp::<{BackendType::AVX}>(min, max).into()
            ];
            Self::convert(Simd::<f64, 4>::from(clamped))
        }
    }
}

impl SimdConvertImpl<u64, 8, {BackendType::AVX}> for Simd<f64, 8> {
    fn simd_convert_impl(self) -> Simd<u64, 8> {
        unsafe { 
            let a : [__m256d; 2] = self.into();
            let cnst = _mm256_set1_pd(0x0010000000000000u64 as f64);
            [cvt_f64_u64(a[0], cnst), cvt_f64_u64(a[1], cnst)].into()
        }
    }

    fn simd_convert_saturate_impl(self) -> Simd<u64, 8> {
        unsafe {
            let min = Simd::<f64, 2>::splat(u64::MIN as f64);
            let max = Simd::<f64, 2>::splat(u64::MAX as f64);

            let unclamped = self.split_4();
            let clamped = [
                unclamped[0].simd_clamp::<{BackendType::AVX}>(min, max),
                unclamped[1].simd_clamp::<{BackendType::AVX}>(min, max),
                unclamped[2].simd_clamp::<{BackendType::AVX}>(min, max),
                unclamped[3].simd_clamp::<{BackendType::AVX}>(min, max),
            ];
            Self::convert(clamped.into())
        }
    }
}

//==============================================================================================================================

impl SimdConvertImpl<u32, 4, {BackendType::AVX}> for Simd<f32, 4> {
    fn simd_convert_impl(self) -> Simd<u32, 4> {
        <Self as SimdConvertImpl<u32, 4, {BackendType::SSE}>>::simd_convert_impl(self)
    }

    fn simd_convert_saturate_impl(self) -> Simd<u32, 4> {
        <Self as SimdConvertImpl<u32, 4, {BackendType::SSE}>>::simd_convert_saturate_impl(self)
    }
}

impl SimdConvertImpl<u32, 8, {BackendType::AVX}> for Simd<f32, 8> {
    // https://stackoverflow.com/questions/9157373/most-efficient-way-to-convert-vector-of-float-to-vector-of-uint32
    // Implements the algorith above, but does not include saturating the value
    fn simd_convert_impl(self) -> Simd<u32, 8> {
        <Self as SimdConvertImpl<u32, 8, {BackendType::SSE}>>::simd_convert_impl(self)
    }

    fn simd_convert_saturate_impl(self) -> Simd<u32, 8> {
        unsafe {
            let min = Simd::<f32, 8>::splat(u32::MIN as f32);
            let max = Simd::<f32, 8>::splat(u32::MAX as f32);
            Self::convert(self.simd_clamp::<{BackendType::AVX}>(min, max))
        }
    }
}

impl SimdConvertImpl<u32, 16, {BackendType::AVX}> for Simd<f32, 16> {
    fn simd_convert_impl(self) -> Simd<u32, 16> {
        <Self as SimdConvertImpl<u32, 16, {BackendType::SSE}>>::simd_convert_impl(self)
    }

    fn simd_convert_saturate_impl(self) -> Simd<u32, 16> {
        unsafe {
            let min = Simd::<f32, 8>::splat(u32::MIN as f32);
            let max = Simd::<f32, 8>::splat(u32::MAX as f32);

            let unclamped = self.split_2();
            let clamped = [
                unclamped[0].simd_clamp::<{BackendType::AVX}>(min, max),
                unclamped[1].simd_clamp::<{BackendType::AVX}>(min, max)
            ];
            Self::convert(clamped.into())
        }
    }
}

//==============================================================================================================================

impl SimdConvertImpl<f32, 4, {BackendType::AVX}> for Simd<i32, 4> {
    fn simd_convert_impl(self) -> Simd<f32, 4> {
        <Self as SimdConvertImpl<f32, 4, {BackendType::SSE}>>::simd_convert_impl(self)
    }
}

impl SimdConvertImpl<f32, 8, {BackendType::AVX}> for Simd<i32, 8> {
    fn simd_convert_impl(self) -> Simd<f32, 8> {
        unsafe { _mm256_cvtepi32_ps(self.into()).into() }
    }
}

impl SimdConvertImpl<f32, 16, {BackendType::AVX}> for Simd<i32, 16> {
    fn simd_convert_impl(self) -> Simd<f32, 16> {
        unsafe {
            let a : [__m256i; 2] = self.into();
            let res = [_mm256_cvtepi32_ps(a[0]),
                       _mm256_cvtepi32_ps(a[1])];
            res.into()
        }
    }
}

//==============================================================================================================================

impl SimdConvertImpl<f64, 2, {BackendType::AVX}> for Simd<i32, 4> {
    fn simd_convert_impl(self) -> Simd<f64, 2> {
        <Self as SimdConvertImpl<f64, 2, {BackendType::SSE}>>::simd_convert_impl(self)
    }
}

// TODO: WRONG FOR SSE
impl SimdConvertImpl<f64, 4, {BackendType::AVX}> for Simd<i32, 8> {
    fn simd_convert_impl(self) -> Simd<f64, 4> {
        unsafe { _mm256_cvtepi32_pd(_mm256_castsi256_si128(self.into())).into() }
    }
}

impl SimdConvertImpl<f64, 8, {BackendType::AVX}> for Simd<i32, 16> {
    fn simd_convert_impl(self) -> Simd<f64, 8> {
        unsafe {
            let a : [__m256i; 2] = self.into();
            
            let lower = _mm256_cvtepi32_pd(_mm256_castsi256_si128(a[0]));
            let upper_si = _mm256_extractf128_si256::<1>(a[0]);
            let upper = _mm256_cvtepi32_pd(upper_si);

            [lower, upper].into()
        }
    }
}

//==============================================================================================================================

impl SimdConvertImpl<f32, 4, {BackendType::AVX}> for Simd<u32, 4> {
    fn simd_convert_impl(self) -> Simd<f32, 4> {
        <Self as SimdConvertImpl<f32, 4, {BackendType::SSE}>>::simd_convert_impl(self)
    }
}

impl SimdConvertImpl<f32, 8, {BackendType::AVX}> for Simd<u32, 8> {
    fn simd_convert_impl(self) -> Simd<f32, 8> {
        <Self as SimdConvertImpl<f32, 8, {BackendType::SSE}>>::simd_convert_impl(self)
    }
}

impl SimdConvertImpl<f32, 16, {BackendType::AVX}> for Simd<u32, 16> {
    fn simd_convert_impl(self) -> Simd<f32, 16> {
        <Self as SimdConvertImpl<f32, 16, {BackendType::SSE}>>::simd_convert_impl(self)
    }
}

//==============================================================================================================================

impl SimdConvertImpl<f64, 2, {BackendType::AVX}> for Simd<i64, 2> {
    fn simd_convert_impl(self) -> Simd<f64, 2> {
        <Self as SimdConvertImpl<f64, 2, {BackendType::SSE}>>::simd_convert_impl(self)
    }
}

impl SimdConvertImpl<f64, 4, {BackendType::AVX}> for Simd<i64, 4> {
    // https://stackoverflow.com/questions/41144668/how-to-efficiently-perform-double-int64-conversions-with-sse-avx
    fn simd_convert_impl(self) -> Simd<f64, 4> {
        <Self as SimdConvertImpl<f64, 4, {BackendType::SSE}>>::simd_convert_impl(self)
    }
}

impl SimdConvertImpl<f64, 8, {BackendType::AVX}> for Simd<i64, 8> {
    // https://stackoverflow.com/questions/41144668/how-to-efficiently-perform-double-int64-conversions-with-sse-avx
    fn simd_convert_impl(self) -> Simd<f64, 8> {
        <Self as SimdConvertImpl<f64, 8, {BackendType::SSE}>>::simd_convert_impl(self)
    }
}

//==============================================================================================================================

impl SimdConvertImpl<f64, 2, {BackendType::AVX}> for Simd<u64, 2> {
    fn simd_convert_impl(self) -> Simd<f64, 2> {
        <Self as SimdConvertImpl<f64, 2, {BackendType::SSE}>>::simd_convert_impl(self)
    }
}

impl SimdConvertImpl<f64, 4, {BackendType::AVX}> for Simd<u64, 4> {
    // https://stackoverflow.com/questions/41144668/how-to-efficiently-perform-double-int64-conversions-with-sse-avx
    fn simd_convert_impl(self) -> Simd<f64, 4> {
        <Self as SimdConvertImpl<f64, 4, {BackendType::SSE}>>::simd_convert_impl(self)
    }
}

impl SimdConvertImpl<f64, 8, {BackendType::AVX}> for Simd<u64, 8> {
    // https://stackoverflow.com/questions/41144668/how-to-efficiently-perform-double-int64-conversions-with-sse-avx
    fn simd_convert_impl(self) -> Simd<f64, 8> {
        <Self as SimdConvertImpl<f64, 8, {BackendType::SSE}>>::simd_convert_impl(self)
    }
}

//==============================================================================================================================

macro_rules! impl_2_step_cvt {
    { $([$from_ty:ty, $f_lanes:literal => $imm_ty:ty, $imm_lanes:literal => $to_ty:ty, $t_lanes:literal])* } => {
        $(
            impl SimdConvertImpl<$to_ty, $t_lanes, {BackendType::AVX}> for Simd<$from_ty, $f_lanes> 
                where Self                      : SimdConvertImpl<$imm_ty, $imm_lanes, {BackendType::AVX}>,
                      Simd<$imm_ty, $imm_lanes> : SimdConvertImpl<$to_ty, $t_lanes, {BackendType::AVX}>
            {
                fn simd_convert_impl(self) -> Simd<$to_ty, $t_lanes> {
                    self.simd_convert::<$imm_ty, $imm_lanes, {BackendType::AVX}>().simd_convert::<$to_ty, $t_lanes, {BackendType::AVX}>()
                }
            
                fn simd_convert_saturate_impl(self) -> Simd<$to_ty, $t_lanes> {
                    self.simd_convert_saturate::<$imm_ty, $imm_lanes, {BackendType::AVX}>().simd_convert_saturate::<$to_ty, $t_lanes, {BackendType::AVX}>()
                }
            }
        )*
    };
}
impl_2_step_cvt!{
    [f32, 4  => i32, 4  => i8 , 16]
    [f32, 8  => i32, 8  => i8 , 32]
    [f32, 16 => i32, 16 => i8 , 64]

    [f32, 4  => i32, 4  => i16, 8 ]
    [f32, 8  => i32, 8  => i16, 16]
    [f32, 16 => i32, 16 => i16, 32]

    [f32, 4  => f64, 2  => i64, 2 ]
    [f32, 8  => f64, 4  => i64, 4 ]
    [f32, 16 => f64, 8  => i64, 8 ]

    [f32, 4  => i32, 4  => u8 , 16]
    [f32, 8  => i32, 8  => u8 , 32]
    [f32, 16 => i32, 16 => u8 , 64]

    [f32, 4  => i32, 4  => u16, 8 ]
    [f32, 8  => i32, 8  => u16, 16]
    [f32, 16 => i32, 16 => u16, 32]

    [f32, 4  => f64, 2  => u64, 2 ]
    [f32, 8  => f64, 4  => u64, 4 ]
    [f32, 16 => f64, 8  => u64, 8 ]

    [f64, 2  => i32, 4  => i8 , 16]
    [f64, 4  => i32, 8  => i8 , 32]
    [f64, 8  => i32, 16 => i8 , 64]

    [f64, 2  => i32, 4  => i16, 8 ]
    [f64, 4  => i32, 8  => i16, 16]
    [f64, 8  => i32, 16 => i16, 32]

    [f64, 2  => i32, 4  => u8 , 16]
    [f64, 4  => i32, 8  => u8 , 32]
    [f64, 8  => i32, 16 => u8 , 64]

    [f64, 2  => i32, 4  => u16, 8 ]
    [f64, 4  => i32, 8  => u16, 16]
    [f64, 8  => i32, 16 => u16, 32]

    [f64, 2  => u64, 2  => u32, 4 ]
    [f64, 4  => u64, 4  => u32, 8 ]
    [f64, 8  => u64, 8  => u32, 16]

    [i8 , 16 => i32, 4  => f32, 4 ]
    [i8 , 32 => i32, 8  => f32, 8 ]
    [i8 , 64 => i32, 16 => f32, 16]

    [i16, 8  => i32, 4  => f32, 4 ]
    [i16, 16 => i32, 8  => f32, 8 ]
    [i16, 32 => i32, 16 => f32, 16]

    [u8 , 16 => i32, 4  => f32, 4 ]
    [u8 , 32 => i32, 8  => f32, 8 ]
    [u8 , 64 => i32, 16 => f32, 16]

    [u16, 8  => i32, 4  => f32, 4 ]
    [u16, 16 => i32, 8  => f32, 8 ]
    [u16, 32 => i32, 16 => f32, 16]

    [i8 , 16 => i32, 4  => f64, 2 ]
    [i8 , 32 => i32, 8  => f64, 4 ]
    [i8 , 64 => i32, 16 => f64, 8 ]

    [i16, 8  => i32, 4  => f64, 2 ]
    [i16, 16 => i32, 8  => f64, 4 ]
    [i16, 32 => i32, 16 => f64, 8 ]

    [i64, 2  => f64, 2  => f32, 4 ]
    [i64, 4  => f64, 4  => f32, 8 ]
    [i64, 8  => f64, 8  => f32, 16]

    [u8 , 16 => i32, 4  => f64, 2 ]
    [u8 , 32 => i32, 8  => f64, 4 ]
    [u8 , 64 => i32, 16 => f64, 8 ]

    [u16, 8  => i32, 4  => f64, 2 ]
    [u16, 16 => i32, 8  => f64, 4 ]
    [u16, 32 => i32, 16 => f64, 8 ]

    [u64, 2  => f64, 2  => f32, 4 ]
    [u64, 4  => f64, 4  => f32, 8 ]
    [u64, 8  => f64, 8  => f32, 16]

    [u32, 4  => u64, 2  => f64, 2 ]
    [u32, 8  => u64, 4  => f64, 4 ]
    [u32, 16 => u64, 8  => f64, 8 ]
}


//==============================================================================================================================
//  UTILITY
//==============================================================================================================================
// https://stackoverflow.com/questions/41144668/how-to-efficiently-perform-double-int64-conversions-with-sse-avx
// FIXME(jel): Only for [-2^51, 2^51]
// PERF(jel): For AVX2, is the integer xor better? more execution ports?
#[inline]
unsafe fn cvt_f64_u64(val: __m256d, cnst: __m256d) -> __m256i {
    let imm = _mm256_add_pd(val, cnst);
    _mm256_castpd_si256(_mm256_xor_pd(imm, cnst))
}