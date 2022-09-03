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
            impl SimdConvertImpl<$ty, $lanes128, {BackendType::AVX2}> for Simd<$ty, $lanes128>
                where LaneCount<$lanes128> : SupportedLaneCount
            {
                #[inline]
                fn simd_convert_impl(self) -> Self {
                    self
                }
            }

            impl SimdConvertImpl<$ty, $lanes256, {BackendType::AVX2}> for Simd<$ty, $lanes256>
                where LaneCount<$lanes256> : SupportedLaneCount
            {
                #[inline]
                fn simd_convert_impl(self) -> Self {
                    self
                }
            }

            impl SimdConvertImpl<$ty, $lanes512, {BackendType::AVX2}> for Simd<$ty, $lanes512>
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

macro_rules! impl_via_avx {
    { $([$from_ty:ty, $from_lanes128:literal, $from_lanes256:literal, $from_lanes512:literal => $to_ty:ty, $to_lanes128:literal, $to_lanes256:literal, $to_lanes512:literal])* } => {
        $(
            impl SimdConvertImpl<$to_ty, $to_lanes128, {BackendType::AVX2}> for Simd<$from_ty, $from_lanes128> {
                #[inline]
                fn simd_convert_impl(self) -> Simd<$to_ty, $to_lanes128> {
                    <Self as SimdConvertImpl<$to_ty, $to_lanes128, {BackendType::AVX}>>::simd_convert_impl(self)
                }

                #[inline]
                fn simd_convert_saturate_impl(self) -> Simd<$to_ty, $to_lanes128> {
                    <Self as SimdConvertImpl<$to_ty, $to_lanes128, {BackendType::AVX}>>::simd_convert_saturate_impl(self)
                }
            }

            impl SimdConvertImpl<$to_ty, $to_lanes256, {BackendType::AVX2}> for Simd<$from_ty, $from_lanes256> {
                #[inline]
                fn simd_convert_impl(self) -> Simd<$to_ty, $to_lanes256> {
                    <Self as SimdConvertImpl<$to_ty, $to_lanes256, {BackendType::AVX}>>::simd_convert_impl(self)
                }

                #[inline]
                fn simd_convert_saturate_impl(self) -> Simd<$to_ty, $to_lanes256> {
                    <Self as SimdConvertImpl<$to_ty, $to_lanes256, {BackendType::AVX}>>::simd_convert_saturate_impl(self)
                }
            }

            impl SimdConvertImpl<$to_ty, $to_lanes512, {BackendType::AVX2}> for Simd<$from_ty, $from_lanes512> {
                #[inline]
                fn simd_convert_impl(self) -> Simd<$to_ty, $to_lanes512> {
                    <Self as SimdConvertImpl<$to_ty, $to_lanes512, {BackendType::AVX}>>::simd_convert_impl(self)
                }

                #[inline]
                fn simd_convert_saturate_impl(self) -> Simd<$to_ty, $to_lanes512> {
                    <Self as SimdConvertImpl<$to_ty, $to_lanes512, {BackendType::AVX}>>::simd_convert_saturate_impl(self)
                }
            }
        )*
    };
}
impl_via_avx!{
    [i8 , 16, 32, 64 => u8 , 16, 32, 64]
    [i16, 8 , 16, 32 => u16, 8 , 16, 32]
    [i32, 4 , 8 , 16 => u32, 4 , 8 , 16]
    [i64, 2 , 4 , 8  => u64, 2 , 4 , 8 ]
}

// PERF(jel): Only use `min` for unsigned
macro_rules! impl_narrow {
    {@8
     $from_ty:ty, $from_lanes128:literal, $from_lanes256:literal, $from_lanes512:literal => 
     $to_ty:ty, $to_lanes128:literal, $to_lanes256:literal, $to_lanes512:literal,
     $signed_to_ty:ty
    } => {
        impl SimdConvertImpl<$to_ty, $to_lanes128, {BackendType::AVX2}> for Simd<$from_ty, $from_lanes128> {
            #[inline]
            fn simd_convert_impl(self) -> Simd<$to_ty, $to_lanes128> {
                <Self as SimdConvertImpl<$to_ty, $to_lanes128, {BackendType::AVX}>>::simd_convert_impl(self)
            }
        
            #[inline]
            fn simd_convert_saturate_impl(self) -> Simd<$to_ty, $to_lanes128> {
                <Self as SimdConvertImpl<$to_ty, $to_lanes128, {BackendType::AVX}>>::simd_convert_impl(self)
            }
        }
   
        impl SimdConvertImpl<$to_ty, $to_lanes256, {BackendType::AVX2}> for Simd<$from_ty, $from_lanes256> {
            #[inline]
            fn simd_convert_impl(self) -> Simd<$to_ty, $to_lanes256> {
                unsafe {
                    let shuffle_mask0 = _mm256_setr_epi8( 0,  8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
                                                         -1, -1,  0,  8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1);
                    let imm = _mm256_shuffle_epi8(self.into(), shuffle_mask0);
                    let upper = _mm256_permute4x64_epi64(imm, 0xFC);
                    _mm256_or_si256(imm, upper).into()
                }
            }
        
            #[inline]
            fn simd_convert_saturate_impl(self) -> Simd<$to_ty, $to_lanes256> {
                unsafe {
                    let min = Simd::<$from_ty, $from_lanes256>::splat(<$to_ty>::MIN as $from_ty);
                    let max = Simd::<$from_ty, $from_lanes256>::splat(<$to_ty>::MAX as $from_ty);
                    Self::convert(self.simd_clamp::<{BackendType::AVX2}>(min, max))
                }
            }
        }

        impl SimdConvertImpl<$to_ty, $to_lanes512, {BackendType::AVX2}> for Simd<$from_ty, $from_lanes512> {
            #[inline]
            fn simd_convert_impl(self) -> Simd<$to_ty, $to_lanes512> {
                unsafe {
                    let a : [__m256i; 2] = self.into();

                    let shuffle_mask0 = _mm256_setr_epi8( 0,  8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
                                                         -1, -1,  0,  8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1);
                    let shuffle_mask1 = _mm256_setr_epi8(-1, -1, -1, -1,  0,  8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
                                                         -1, -1, -1, -1, -1, -1,  0,  8, -1, -1, -1, -1, -1, -1, -1, -1);

                    let imm0 = _mm256_shuffle_epi8(a[0], shuffle_mask0);
                    let imm1 = _mm256_shuffle_epi8(a[1], shuffle_mask1);

                    let upper0 = _mm256_permute4x64_epi64(imm0, 0xFC);
                    let upper1 = _mm256_permute4x64_epi64(imm1, 0xFC);

                    let imm0 = _mm256_or_si256(imm0, upper0);
                    let imm1 = _mm256_or_si256(imm1, upper1);
                    [_mm256_or_si256(imm0, imm1), _mm256_setzero_si256()].into()
                }
            }
        
            #[inline]
            fn simd_convert_saturate_impl(self) -> Simd<$to_ty, $to_lanes512> {
                unsafe {
                    let min = Simd::<$from_ty, $from_lanes256>::splat(<$to_ty>::MIN as $from_ty);
                    let max = Simd::<$from_ty, $from_lanes256>::splat(<$to_ty>::MAX as $from_ty);

                    let unclamped = self.split_2();
                    let clamped = [
                        unclamped[0].simd_clamp::<{BackendType::AVX2}>(min, max),
                        unclamped[1].simd_clamp::<{BackendType::AVX2}>(min, max)
                    ];
                    Self::convert(clamped.into())
                }
            }
        }
    };
    {@64_16
     $from_ty:ty, $from_lanes128:literal, $from_lanes256:literal, $from_lanes512:literal => 
     $to_ty:ty, $to_lanes128:literal, $to_lanes256:literal, $to_lanes512:literal,
     $signed_to_ty:ty
    } => {
        impl SimdConvertImpl<$to_ty, $to_lanes128, {BackendType::AVX2}> for Simd<$from_ty, $from_lanes128> {
            #[inline]
            fn simd_convert_impl(self) -> Simd<$to_ty, $to_lanes128> {
                <Self as SimdConvertImpl<$to_ty, $to_lanes128, {BackendType::AVX}>>::simd_convert_impl(self)
            }
        
            #[inline]
            fn simd_convert_saturate_impl(self) -> Simd<$to_ty, $to_lanes128> {
                <Self as SimdConvertImpl<$to_ty, $to_lanes128, {BackendType::AVX}>>::simd_convert_impl(self)
            }
        }
   
        impl SimdConvertImpl<$to_ty, $to_lanes256, {BackendType::AVX2}> for Simd<$from_ty, $from_lanes256> {
            #[inline]
            fn simd_convert_impl(self) -> Simd<$to_ty, $to_lanes256> {
                unsafe {
                    let a : __m256i = self.into();
                    let shuffle_mask = _mm256_setr_epi8( 0,  1,  8,  9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
                                                        -1, -1, -1, -1,  0,  1,  8,  9, -1, -1, -1, -1, -1, -1, -1, -1);
                    let permute_mask = _mm256_setr_epi32(0, 4, 7, 7, 7, 7, 7, 7);
                    _mm256_permutevar8x32_epi32(_mm256_shuffle_epi8(self.into(), shuffle_mask), permute_mask).into()
                }
            }
        
            #[inline]
            fn simd_convert_saturate_impl(self) -> Simd<$to_ty, $to_lanes256> {
                unsafe {
                    let min = Simd::<$from_ty, $from_lanes256>::splat(<$to_ty>::MIN as $from_ty);
                    let max = Simd::<$from_ty, $from_lanes256>::splat(<$to_ty>::MAX as $from_ty);
                    Self::convert(self.simd_clamp::<{BackendType::AVX2}>(min, max))
                }
            }
        }

        impl SimdConvertImpl<$to_ty, $to_lanes512, {BackendType::AVX2}> for Simd<$from_ty, $from_lanes512> {
            #[inline]
            fn simd_convert_impl(self) -> Simd<$to_ty, $to_lanes512> {
                unsafe {
                    let a : [__m256i; 2] = self.into();
                    let shuffle_mask0 = _mm256_setr_epi8( 0,  1,  8,  9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
                                                         -1, -1, -1, -1,  0,  1,  8,  9, -1, -1, -1, -1, -1, -1, -1, -1);
                    let shuffle_mask1 = _mm256_setr_epi8(-1, -1, -1, -1, -1, -1, -1, -1,  0,  1,  8,  9, -1, -1, -1, -1,
                                                         -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,  0,  1,  8,  9);
                    let permute_mask0 = _mm256_setr_epi32(0, 4, 7, 7, 7, 7, 7, 7);
                    let permute_mask1 = _mm256_setr_epi32(0, 0, 2, 7, 0, 0, 0, 0);
                    let imm0 = _mm256_permutevar8x32_epi32(_mm256_shuffle_epi8(a[0], shuffle_mask0), permute_mask0);
                    let imm1 = _mm256_permutevar8x32_epi32(_mm256_shuffle_epi8(a[1], shuffle_mask1), permute_mask1);
                    [_mm256_or_si256(imm0, imm1), _mm256_setzero_si256()].into()
                }
            }
        
            #[inline]
            fn simd_convert_saturate_impl(self) -> Simd<$to_ty, $to_lanes512> {
                unsafe {
                    let min = Simd::<$from_ty, $from_lanes256>::splat(<$to_ty>::MIN as $from_ty);
                    let max = Simd::<$from_ty, $from_lanes256>::splat(<$to_ty>::MAX as $from_ty);

                    let unclamped = self.split_2();
                    let clamped = [
                        unclamped[0].simd_clamp::<{BackendType::AVX2}>(min, max),
                        unclamped[1].simd_clamp::<{BackendType::AVX2}>(min, max)
                    ];
                    Self::convert(clamped.into())
                }
            }
        }
    };
    {@64_32
     $from_ty:ty, $from_lanes128:literal, $from_lanes256:literal, $from_lanes512:literal => 
     $to_ty:ty, $to_lanes128:literal, $to_lanes256:literal, $to_lanes512:literal,
     $signed_to_ty:ty
    } => {
        impl SimdConvertImpl<$to_ty, $to_lanes128, {BackendType::AVX2}> for Simd<$from_ty, $from_lanes128> {
            #[inline]
            fn simd_convert_impl(self) -> Simd<$to_ty, $to_lanes128> {
                <Self as SimdConvertImpl<$to_ty, $to_lanes128, {BackendType::AVX}>>::simd_convert_impl(self)
            }
        
            #[inline]
            fn simd_convert_saturate_impl(self) -> Simd<$to_ty, $to_lanes128> {
                <Self as SimdConvertImpl<$to_ty, $to_lanes128, {BackendType::AVX}>>::simd_convert_impl(self)
            }
        }
   
        impl SimdConvertImpl<$to_ty, $to_lanes256, {BackendType::AVX2}> for Simd<$from_ty, $from_lanes256> {
            #[inline]
            fn simd_convert_impl(self) -> Simd<$to_ty, $to_lanes256> {
                unsafe {
                    let shuffle_mask = _mm256_setr_epi8( 0,  1,  2,  3,  8,  9, 10, 11, -1, -1, -1, -1, -1, -1, -1, -1,
                                                        -1, -1, -1, -1, -1, -1, -1, -1,  0,  1,  2,  3,  8,  9, 10, 11);
                    _mm256_permute4x64_epi64(_mm256_shuffle_epi8(self.into(), shuffle_mask), 0x5C).into()
                }
            }
        
            #[inline]
            fn simd_convert_saturate_impl(self) -> Simd<$to_ty, $to_lanes256> {
                unsafe {
                    let min = Simd::<$from_ty, $from_lanes256>::splat(<$to_ty>::MIN as $from_ty);
                    let max = Simd::<$from_ty, $from_lanes256>::splat(<$to_ty>::MAX as $from_ty);
                    Self::convert(self.simd_clamp::<{BackendType::AVX2}>(min, max))
                }
            }
        }

        impl SimdConvertImpl<$to_ty, $to_lanes512, {BackendType::AVX2}> for Simd<$from_ty, $from_lanes512> {
            #[inline]
            fn simd_convert_impl(self) -> Simd<$to_ty, $to_lanes512> {
                unsafe {
                    let a : [__m256i; 2] = self.into();
                    let shuffle_mask = _mm256_setr_epi8( 0,  1,  2,  3,  8,  9, 10, 11, -1, -1, -1, -1, -1, -1, -1, -1,
                                                        -1, -1, -1, -1, -1, -1, -1, -1,  0,  1,  2,  3,  8,  9, 10, 11);
                    let imm0 = _mm256_permute4x64_epi64(_mm256_shuffle_epi8(a[0], shuffle_mask), 0x5C);
                    let imm1 = _mm256_permute4x64_epi64(_mm256_shuffle_epi8(a[1], shuffle_mask), 0x35);
                    [_mm256_or_si256(imm0, imm1), _mm256_setzero_si256()].into()
                }
            }
        
            #[inline]
            fn simd_convert_saturate_impl(self) -> Simd<$to_ty, $to_lanes512> {
                unsafe {
                    let min = Simd::<$from_ty, $from_lanes256>::splat(<$to_ty>::MIN as $from_ty);
                    let max = Simd::<$from_ty, $from_lanes256>::splat(<$to_ty>::MAX as $from_ty);

                    let unclamped = self.split_2();
                    let clamped = [
                        unclamped[0].simd_clamp::<{BackendType::AVX2}>(min, max),
                        unclamped[1].simd_clamp::<{BackendType::AVX2}>(min, max),
                    ];
                    Self::convert(clamped.into())
                }
            }
        }
    };
    {@16_8 $from_ty:ty, $to_ty:ty, $pack16:ident } => {
        impl SimdConvertImpl<$to_ty, 16, {BackendType::AVX2}> for Simd<$from_ty, 8> {
            #[inline]
            fn simd_convert_impl(self) -> Simd<$to_ty, 16> {
                <Self as SimdConvertImpl<$to_ty, 16, {BackendType::AVX}>>::simd_convert_impl(self)
            }
        
            #[inline]
            fn simd_convert_saturate_impl(self) -> Simd<$to_ty, 16> {
                <Self as SimdConvertImpl<$to_ty, 16, {BackendType::AVX}>>::simd_convert_saturate_impl(self)
            }
        }

        impl SimdConvertImpl<$to_ty, 32, {BackendType::AVX2}> for Simd<$from_ty, 16> {
            #[inline]
            fn simd_convert_impl(self) -> Simd<$to_ty, 32> {
                unsafe {
                    let shuffle_mask = _mm256_setr_epi8( 0,  2,  4,  6,  8, 10, 12, 14, -1, -1, -1, -1, -1, -1, -1, -1,
                                                        -1, -1, -1, -1, -1, -1, -1, -1,  0,  2,  4,  6,  8, 10, 12, 14);
                    _mm256_permute4x64_epi64(_mm256_shuffle_epi8(self.into(), shuffle_mask), 0x5C).into()
                }
            }
        
            #[inline]
            fn simd_convert_saturate_impl(self) -> Simd<$to_ty, 32> {
                unsafe { $pack16(self.into(), _mm256_setzero_si256()).into() }
            }
        }

        impl SimdConvertImpl<$to_ty, 64, {BackendType::AVX2}> for Simd<$from_ty, 32> {
            #[inline]
            fn simd_convert_impl(self) -> Simd<$to_ty, 64> {
                unsafe {
                    let a : [__m256i; 2] = self.into();
                    let shuffle_mask = _mm256_setr_epi8( 0,  2,  4,  6,  8, 10, 12, 14, -1, -1, -1, -1, -1, -1, -1, -1,
                                                        -1, -1, -1, -1, -1, -1, -1, -1,  0,  2,  4,  6,  8, 10, 12, 14);
                    let imm1 = _mm256_permute4x64_epi64(_mm256_shuffle_epi8(a[0], shuffle_mask), 0x5C);
                    let imm0 = _mm256_permute4x64_epi64(_mm256_shuffle_epi8(a[1], shuffle_mask), 0x35);
                    [_mm256_or_si256(imm0, imm1), _mm256_setzero_si256()].into()
                }
            }
        
            #[inline]
            fn simd_convert_saturate_impl(self) -> Simd<$to_ty, 64> {
                unsafe {
                    let a : [__m256i; 2] = self.into();
                    let imm = $pack16(a[0], a[1]); // [a0..a7, b0..b7, a8..a15, b8..b15]
                    [_mm256_permute4x64_epi64(imm, 0x5C), _mm256_setzero_si256()].into()
                }
            }
        }
    };
    {@32_8 $from_ty:ty, $to_ty:ty, $pack16:ident, $pack32:ident } => {
        impl SimdConvertImpl<$to_ty, 16, {BackendType::AVX2}> for Simd<$from_ty, 4> {
            #[inline]
            fn simd_convert_impl(self) -> Simd<$to_ty, 16> {
                <Self as SimdConvertImpl<$to_ty, 16, {BackendType::AVX}>>::simd_convert_impl(self)
            }
        
            #[inline]
            fn simd_convert_saturate_impl(self) -> Simd<$to_ty, 16> {
                <Self as SimdConvertImpl<$to_ty, 16, {BackendType::AVX}>>::simd_convert_saturate_impl(self)
            }
        }

        impl SimdConvertImpl<$to_ty, 32, {BackendType::AVX2}> for Simd<$from_ty, 8> {
            #[inline]
            fn simd_convert_impl(self) -> Simd<$to_ty, 32> {
                unsafe {
                    let a : __m256i = self.into();
                    let shuffle_mask = _mm256_setr_epi8( 0,  4,  8, 12, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
                                                        -1, -1, -1, -1,  0,  4,  8, 12, -1, -1, -1, -1, -1, -1, -1, -1);
                    let permute_mask = _mm256_setr_epi32(0, 5, 7, 7, 7, 7, 7, 7);
                    _mm256_permutevar8x32_epi32(_mm256_shuffle_epi8(self.into(), shuffle_mask), permute_mask).into()
                }
            }
        
            #[inline]
            fn simd_convert_saturate_impl(self) -> Simd<$to_ty, 32> {
                unsafe {
                    let z = _mm256_setzero_si256();
                    $pack16($pack32(self.into(), z), z).into()
                }
            }
        }

        impl SimdConvertImpl<$to_ty, 64, {BackendType::AVX2}> for Simd<$from_ty, 16> {
            #[inline]
            fn simd_convert_impl(self) -> Simd<$to_ty, 64> {
                unsafe {
                    let a : [__m256i; 2] = self.into();
                    let shuffle_mask = _mm256_setr_epi8( 0,  4,  8, 12, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
                                                        -1, -1, -1, -1,  0,  4,  8, 12, -1, -1, -1, -1, -1, -1, -1, -1);
                    let permute_mask0 = _mm256_setr_epi32(0, 5, 7, 7, 7, 7, 7, 7);
                    let permute_mask1 = _mm256_setr_epi32(7, 7, 0, 5, 7, 7, 7, 7);
                    let imm0 = _mm256_permutevar8x32_epi32(_mm256_shuffle_epi8(a[0], shuffle_mask), permute_mask0);
                    let imm1 = _mm256_permutevar8x32_epi32(_mm256_shuffle_epi8(a[1], shuffle_mask), permute_mask1);
                    [_mm256_or_si256(imm0, imm1), _mm256_setzero_si256()].into()
                }
            }
        
            #[inline]
            fn simd_convert_saturate_impl(self) -> Simd<$to_ty, 64> {
                unsafe {
                    let a : [__m256i; 2] = self.into();
                    let mask = _mm256_setr_epi64x(u64::MAX as i64, u64::MAX as i64, 0, 0);
                    let imm = $pack32(a[0], a[1]); // [a0..a3, b0..b3, a4..a7, b4..b7]
                    let imm = $pack16(imm, imm); // [a0..a3, b0..b3, a4..a7, b4..b7, a0..a3, b0..b3, a4..a7, b4..b7]
                    [_mm256_and_si256(imm, mask), _mm256_setzero_si256()].into()
                }
            }
        }
    };
    {@32_16 $from_ty:ty, $to_ty:ty, $pack32:ident } => {
        impl SimdConvertImpl<$to_ty, 8, {BackendType::AVX2}> for Simd<$from_ty, 4> {
            #[inline]
            fn simd_convert_impl(self) -> Simd<$to_ty, 8> {
                <Self as SimdConvertImpl<$to_ty, 8, {BackendType::AVX}>>::simd_convert_impl(self)
            }
        
            #[inline]
            fn simd_convert_saturate_impl(self) -> Simd<$to_ty, 8> {
                <Self as SimdConvertImpl<$to_ty, 8, {BackendType::AVX}>>::simd_convert_saturate_impl(self)
            }
        }

        impl SimdConvertImpl<$to_ty, 16, {BackendType::AVX2}> for Simd<$from_ty, 8> {
            #[inline]
            fn simd_convert_impl(self) -> Simd<$to_ty, 16> {
                unsafe {
                    let shuffle_mask = _mm256_setr_epi8( 0,  1,  4,  5,  8,  9, 12, 13, -1, -1, -1, -1, -1, -1, -1, -1,
                                                        -1, -1, -1, -1, -1, -1, -1, -1,  0,  1,  4,  5,  8,  9, 12, 13);
                    _mm256_permute4x64_epi64(_mm256_shuffle_epi8(self.into(), shuffle_mask), 0x5C).into()
                }
            }
        
            #[inline]
            fn simd_convert_saturate_impl(self) -> Simd<$to_ty, 16> {
                unsafe { $pack32(self.into(), _mm256_setzero_si256()).into() }
            }
        }

        impl SimdConvertImpl<$to_ty, 32, {BackendType::AVX2}> for Simd<$from_ty, 16> {
            #[inline]
            fn simd_convert_impl(self) -> Simd<$to_ty, 32> {
                unsafe {
                    let a : [__m256i; 2] = self.into();
                    let shuffle_mask = _mm256_setr_epi8( 0,  1,  4,  5,  8,  9, 12, 13, -1, -1, -1, -1, -1, -1, -1, -1,
                                                        -1, -1, -1, -1, -1, -1, -1, -1,  0,  1,  4,  5,  8,  9, 12, 13);
                    let imm1 = _mm256_permute4x64_epi64(_mm256_shuffle_epi8(a[0], shuffle_mask), 0x5C);
                    let imm0 = _mm256_permute4x64_epi64(_mm256_shuffle_epi8(a[1], shuffle_mask), 0x35);
                    [_mm256_or_si256(imm0, imm1), _mm256_setzero_si256()].into()
                }
            }
        
            #[inline]
            fn simd_convert_saturate_impl(self) -> Simd<$to_ty, 32> {
                unsafe {
                    let a : [__m256i; 2] = self.into();
                    let imm = $pack32(a[0], a[1]); // [a0..a7, b0..b7, a8..a15, b8..b15]
                    [_mm256_permute4x64_epi64(imm, 0x5C), _mm256_setzero_si256()].into()
                }
            }
        }
    }
}

//==============================================================================================================================

impl_narrow!{ @16_8 i16, i8, _mm256_packs_epi16 }
impl_narrow!{ @32_8 i32, i8, _mm256_packs_epi16, _mm256_packs_epi32 }
impl_narrow!{ @8 i64, 2, 4, 8 => i8, 16, 32, 64, i8 }
impl_narrow!{ @32_16 i32, i16, _mm256_packs_epi32 }
impl_narrow!{ @64_16 i64, 2, 4, 8 => i16, 8, 16, 32, i16 }
impl_narrow!{ @64_32 i64, 2, 4, 8 => i32, 4, 8 , 16, i32 }

//==============================================================================================================================

impl_narrow!{ @16_8 u16, u8, _mm256_packus_epi16 }
impl_narrow!{ @32_8 u32, u8, _mm256_packus_epi16, _mm256_packs_epi32 }
impl_narrow!{ @8 u64, 2, 4, 8 => u8, 16, 32, 64, i8 }
impl_narrow!{ @32_16 u32, u16, _mm256_packus_epi32 }
impl_narrow!{ @64_16 u64, 2, 4, 8 => u16, 8, 16, 32, i16 }
impl_narrow!{ @64_32 u64, 2, 4, 8 => u32, 4, 8 , 16, i32 }

//==============================================================================================================================

impl_narrow!{ @16_8 u16, i8, _mm256_packs_epi16 }
impl_narrow!{ @32_8 u32, i8, _mm256_packs_epi16, _mm256_packs_epi32 }
impl_narrow!{ @8 u64, 2, 4, 8 => i8, 16, 32, 64, i8 }
impl_narrow!{ @32_16 u32, i16, _mm256_packs_epi32 }
impl_narrow!{ @64_16 u64, 2, 4, 8 => i16, 8, 16, 32, i16 }
impl_narrow!{ @64_32 u64, 2, 4, 8 => i32, 4, 8 , 16, i32 }

//==============================================================================================================================

impl_narrow!{ @16_8 i16, u8, _mm256_packus_epi16 }
impl_narrow!{ @32_8 i32, u8, _mm256_packus_epi16, _mm256_packs_epi32 }
impl_narrow!{ @8 i64, 2, 4, 8 => u8, 16, 32, 64, i8 }
impl_narrow!{ @32_16 i32, u16, _mm256_packus_epi32 }
impl_narrow!{ @64_16 i64, 2, 4, 8 => u16, 8, 16, 32, i16 }
impl_narrow!{ @64_32 i64, 2, 4, 8 => u32, 4, 8 , 16, i32 }

//==============================================================================================================================

macro_rules! impl_widen {
    { @2x
      $([$from_ty:ty => $to_ty:ty; 
         $from_lanes128:literal => $to_lanes128:literal, 
         $from_lanes256:literal => $to_lanes256:literal, 
         $from_lanes512:literal => $to_lanes512:literal, 
      $cvt:ident])* 
    } => {
        $(
            impl SimdConvertImpl<$to_ty, $to_lanes128, {BackendType::AVX2}> for Simd<$from_ty, $from_lanes128> {
                fn simd_convert_impl(self) -> Simd<$to_ty, $to_lanes128> {
                    <Self as SimdConvertImpl<$to_ty, $to_lanes128, {BackendType::AVX}>>::simd_convert_impl(self)
                }
            }
            
            impl SimdConvertImpl<$to_ty, $to_lanes256, {BackendType::AVX2}> for Simd<$from_ty, $from_lanes256> {
                fn simd_convert_impl(self) -> Simd<$to_ty, $to_lanes256> {
                    unsafe {
                        let a : [__m128i; 2] = self.into();
                        $cvt(a[0]).into()
                    }
                }
            }
            
            impl SimdConvertImpl<$to_ty, $to_lanes512, {BackendType::AVX2}> for Simd<$from_ty, $from_lanes512> {
                fn simd_convert_impl(self) -> Simd<$to_ty, $to_lanes512> {
                    unsafe {
                        let a : [__m128i; 4] = self.into();
                        let res = [$cvt(a[0]),
                                   $cvt(a[1])];
                        res.into()
                    }
                }
            }
        )*
    };
    { @4x
      $([$from_ty:ty => $to_ty:ty; 
         $from_lanes128:literal => $to_lanes128:literal, 
         $from_lanes256:literal => $to_lanes256:literal, 
         $from_lanes512:literal => $to_lanes512:literal, 
         $cvt:ident])* 
    } => {
       $(
           impl SimdConvertImpl<$to_ty, $to_lanes128, {BackendType::AVX2}> for Simd<$from_ty, $from_lanes128> {
            fn simd_convert_impl(self) -> Simd<$to_ty, $to_lanes128> {
                <Self as SimdConvertImpl<$to_ty, $to_lanes128, {BackendType::AVX}>>::simd_convert_impl(self)
               }
           }
           
           impl SimdConvertImpl<$to_ty, $to_lanes256, {BackendType::AVX2}> for Simd<$from_ty, $from_lanes256> {
            fn simd_convert_impl(self) -> Simd<$to_ty, $to_lanes256> {
                    unsafe {
                        let a : [__m128i; 2] = self.into();
                        $cvt(a[0]).into()
                    }
               }
           }
           
           impl SimdConvertImpl<$to_ty, $to_lanes512, {BackendType::AVX2}> for Simd<$from_ty, $from_lanes512> {
            fn simd_convert_impl(self) -> Simd<$to_ty, $to_lanes512> {
                   unsafe {
                       let a : [__m128i; 4] = self.into();
                       let res = [$cvt(a[0]),
                                  $cvt(_mm_srli_si128::<8>(a[0]))];
                       res.into()
                   }
               }
           }
       )*
   };
   { @8x
     $([$from_ty:ty => $to_ty:ty; 
        $from_lanes128:literal => $to_lanes128:literal, 
        $from_lanes256:literal => $to_lanes256:literal, 
        $from_lanes512:literal => $to_lanes512:literal, 
        $cvt:ident])* } => {
        $(
            impl SimdConvertImpl<$to_ty, $to_lanes128, {BackendType::AVX2}> for Simd<$from_ty, $from_lanes128> {
                fn simd_convert_impl(self) -> Simd<$to_ty, $to_lanes128> {
                    <Self as SimdConvertImpl<$to_ty, $to_lanes128, {BackendType::AVX}>>::simd_convert_impl(self)
                }
            }
            
            impl SimdConvertImpl<$to_ty, $to_lanes256, {BackendType::AVX2}> for Simd<$from_ty, $from_lanes256> {
                fn simd_convert_impl(self) -> Simd<$to_ty, $to_lanes256> {
                    unsafe {
                        let a : [__m128i; 2] = self.into();
                        $cvt(a[0]).into()
                    }
                }
            }
            
            impl SimdConvertImpl<$to_ty, $to_lanes512, {BackendType::AVX2}> for Simd<$from_ty, $from_lanes512> {
                fn simd_convert_impl(self) -> Simd<$to_ty, $to_lanes512> {
                    unsafe {
                        let a : [__m128i; 4] = self.into();
                        let res = [$cvt(a[0]),
                                   $cvt(_mm_srli_si128::<4>(a[0]))];
                        res.into()
                    }
                }
            }
        )*
    };
}
impl_widen!{ @2x
    [i8  => i16; 16 => 8, 32 => 16, 64 => 32, _mm256_cvtepi8_epi16]
    [i16 => i32; 8  => 4, 16 => 8 , 32 => 16, _mm256_cvtepi16_epi32]
    [i32 => i64; 4  => 2, 8  => 4 , 16 => 8 , _mm256_cvtepi32_epi64]

    [u8  => i16; 16 => 8, 32 => 16, 64 => 32, _mm256_cvtepu8_epi16]
    [u16 => i32; 8  => 4, 16 => 8 , 32 => 16, _mm256_cvtepu16_epi32]
    [u32 => i64; 4  => 2, 8  => 4 , 16 => 8 , _mm256_cvtepu32_epi64]

    [i8  => u16; 16 => 8, 32 => 16, 64 => 32, _mm256_cvtepu8_epi16]
    [i16 => u32; 8  => 4, 16 => 8 , 32 => 16, _mm256_cvtepu16_epi32]
    [i32 => u64; 4  => 2, 8  => 4 , 16 => 8 , _mm256_cvtepu32_epi64]

    [u8  => u16; 16 => 8, 32 => 16, 64 => 32, _mm256_cvtepu8_epi16]
    [u16 => u32; 8  => 4, 16 => 8 , 32 => 16, _mm256_cvtepu16_epi32]
    [u32 => u64; 4  => 2, 8  => 4 , 16 => 8 , _mm256_cvtepu32_epi64]
}
impl_widen!{ @4x
    [i8  => i32; 16 => 4, 32 => 8 , 64 => 16, _mm256_cvtepi8_epi32]
    [i16 => i64; 8  => 2, 16 => 4 , 32 => 8 , _mm256_cvtepi16_epi64]

    [u8  => i32; 16 => 4, 32 => 8 , 64 => 16, _mm256_cvtepu8_epi32]
    [u16 => i64; 8  => 2, 16 => 4 , 32 => 8 , _mm256_cvtepu16_epi64]

    [i8  => u32; 16 => 4, 32 => 8 , 64 => 16, _mm256_cvtepu8_epi32]
    [i16 => u64; 8  => 2, 16 => 4 , 32 => 8 , _mm256_cvtepu16_epi64]

    [u8  => u32; 16 => 4, 32 => 8 , 64 => 16, _mm256_cvtepu8_epi32]
    [u16 => u64; 8  => 2, 16 => 4 , 32 => 8 , _mm256_cvtepu16_epi64]
}
impl_widen!{ @8x
    [i8  => i64; 16 => 2, 32 => 4 , 64 => 8 , _mm256_cvtepi8_epi64]
    [u8  => i64; 16 => 2, 32 => 4 , 64 => 8 , _mm256_cvtepu8_epi64]
    [i8  => u64; 16 => 2, 32 => 4 , 64 => 8 , _mm256_cvtepu8_epi64]
    [u8  => u64; 16 => 2, 32 => 4 , 64 => 8 , _mm256_cvtepu8_epi64]
}


//==============================================================================================================================

macro_rules! impl_widen_elem {
    { $([$from_ty:ty => $to_ty:ty, 
         $lanes128:literal <=> $imm_lanes128:literal, 
         $lanes256:literal <=> $imm_lanes256:literal, 
         $lanes512:literal <=> $imm_lanes512:literal])*
    } => {
        $(
            impl SimdConvertImpl<$to_ty, $lanes128, {BackendType::AVX2}> for Simd<$from_ty, $lanes128> {
                fn simd_convert_impl(self) -> Simd<$to_ty, $lanes128> {
                    <Self as SimdConvertImpl<$to_ty, $lanes128, {BackendType::AVX}>>::simd_convert_impl(self)
                }
            }

            impl SimdConvertImpl<$to_ty, $lanes256, {BackendType::AVX2}> for Simd<$from_ty, $lanes256> {
                fn simd_convert_impl(self) -> Simd<$to_ty, $lanes256> {
                    #[repr(align(32))]
                    union LoadSrc {
                        simd: Simd<$from_ty, $lanes256>,
                        buf  : [$from_ty; $imm_lanes256]
                    }

                    unsafe { 
                        let load_src = LoadSrc{ simd: self };
                        let loaded : Simd<$from_ty, $imm_lanes256> = _mm256_load_si256( load_src.buf.as_ptr() as *const __m256i).into();
                        loaded.simd_convert::<$to_ty, $lanes256, {BackendType::AVX2}>()
                     }
                }
            }

            impl SimdConvertImpl<$to_ty, $lanes512, {BackendType::AVX2}> for Simd<$from_ty, $lanes512> {
                fn simd_convert_impl(self) -> Simd<$to_ty, $lanes512> {
                    #[repr(align(32))]
                    union LoadSrc {
                        simd: Simd<$from_ty, $lanes512>,
                        buf : [$from_ty; $imm_lanes512]
                    }

                    unsafe { 
                        let load_src = LoadSrc{ simd: self };
                        let loaded : Simd<$from_ty, $imm_lanes512> = [_mm256_load_si256( load_src.buf.as_ptr() as *const __m256i        ),
                                                                      _mm256_load_si256((load_src.buf.as_ptr() as *const __m256i).add(1))].into();
                        loaded.simd_convert::<$to_ty, $lanes512, {BackendType::AVX2}>()
                     }
                }
            }
        )*
    };
}
impl_widen_elem!{
    [ i8  => i16, 8 <=> 16, 16 <=> 32, 32 <=> 64]
    [ i8  => i32, 4 <=> 16, 8  <=> 32, 16 <=> 64]
    [ i8  => i64, 2 <=> 16, 4  <=> 32, 8  <=> 64]
    [ i16 => i32, 4 <=> 8 , 8  <=> 16, 16 <=> 32]
    [ i16 => i64, 2 <=> 8 , 4  <=> 16, 8  <=> 32]
    [ i32 => i64, 2 <=> 4 , 4  <=> 8 , 8  <=> 16]
}

//==============================================================================================================================

impl SimdConvertImpl<f64, 2, {BackendType::AVX2}> for Simd<f32, 4> {
    fn simd_convert_impl(self) -> Simd<f64, 2> {
        <Self as SimdConvertImpl<f64, 2, {BackendType::AVX}>>::simd_convert_impl(self)
    }
}

impl SimdConvertImpl<f64, 4, {BackendType::AVX2}> for Simd<f32, 8> {
    fn simd_convert_impl(self) -> Simd<f64, 4> {
        unsafe { 
            let a : [__m128; 2] = self.into();
            _mm256_cvtps_pd(a[0]).into()
        }
    }
}

impl SimdConvertImpl<f64, 8, {BackendType::AVX2}> for Simd<f32, 16> {
    fn simd_convert_impl(self) -> Simd<f64, 8> {
        unsafe {
            let a : [__m128; 4] = self.into();
            [_mm256_cvtps_pd(a[0]), _mm256_cvtps_pd(a[1])].into()
        }
    }
}

impl SimdConvertImpl<f32, 4, {BackendType::AVX2}> for Simd<f64, 2> {
    fn simd_convert_impl(self) -> Simd<f32, 4> {
        <Self as SimdConvertImpl<f32, 4, {BackendType::AVX}>>::simd_convert_impl(self)
    }

    fn simd_convert_saturate_impl(self) -> Simd<f32, 4> {
        <Self as SimdConvertImpl<f32, 4, {BackendType::AVX}>>::simd_convert_saturate_impl(self)
    }  
}

impl SimdConvertImpl<f32, 8, {BackendType::AVX2}> for Simd<f64, 4> {
    fn simd_convert_impl(self) -> Simd<f32, 8> {
        unsafe { _mm256_castps128_ps256(_mm256_cvtpd_ps(self.into())).into() }
    }

    fn simd_convert_saturate_impl(self) -> Simd<f32, 8> {
        let min = Simd::<f64, 4>::splat(f32::MIN as f64);
        let max = Simd::<f64, 4>::splat(f32::MAX as f64);
        Self::convert(self.simd_clamp::<{BackendType::AVX2}>(min, max))
    }
}

impl SimdConvertImpl<f32, 16, {BackendType::AVX2}> for Simd<f64, 8> {
    fn simd_convert_impl(self) -> Simd<f32, 16> {
        unsafe {
            let a : [__m256d; 2] = self.into();
            let res = [_mm256_cvtpd_ps(a[0]),
                       _mm256_cvtpd_ps(a[1]),
                       _mm_setzero_ps(),
                       _mm_setzero_ps()];
            res.into()
        }
    }

    fn simd_convert_saturate_impl(self) -> Simd<f32, 16> {
       let min = Simd::<f64, 4>::splat(f32::MIN as f64);
       let max = Simd::<f64, 4>::splat(f32::MAX as f64);

       let unclamped = self.split_2();
       let clamped = [
           unclamped[0].simd_clamp::<{BackendType::AVX2}>(min, max),
           unclamped[1].simd_clamp::<{BackendType::AVX2}>(min, max)
       ];
       Self::convert(clamped.into())
    }
}

//==============================================================================================================================

impl SimdConvertImpl<i32, 4, {BackendType::AVX2}> for Simd<f32, 4> {
    fn simd_convert_impl(self) -> Simd<i32, 4> {
        <Self as SimdConvertImpl<i32, 4, {BackendType::AVX}>>::simd_convert_impl(self)
    }

    fn simd_convert_saturate_impl(self) -> Simd<i32, 4> {
        <Self as SimdConvertImpl<i32, 4, {BackendType::AVX}>>::simd_convert_saturate_impl(self)
    }
}

impl SimdConvertImpl<i32, 8, {BackendType::AVX2}> for Simd<f32, 8> {
    fn simd_convert_impl(self) -> Simd<i32, 8> {
        unsafe { _mm256_cvtps_epi32(self.into()).into() }
    }

    fn simd_convert_saturate_impl(self) -> Simd<i32, 8> {
        let min = Simd::<f32, 8>::splat(i32::MIN as f32);
        let max = Simd::<f32, 8>::splat(i32::MAX as f32);
        Self::convert(self.simd_clamp::<{BackendType::AVX2}>(min, max))
    }
}

impl SimdConvertImpl<i32, 16, {BackendType::AVX2}> for Simd<f32, 16> {
    fn simd_convert_impl(self) -> Simd<i32, 16> {
        unsafe { 
            let a : [__m128; 4] = self.into();
            let res = [_mm_cvtps_epi32(a[0]),
                       _mm_cvtps_epi32(a[1]),
                       _mm_cvtps_epi32(a[2]),
                       _mm_cvtps_epi32(a[3])];
            res.into()
        }
    }

    fn simd_convert_saturate_impl(self) -> Simd<i32, 16> {
        let min = Simd::<f32, 8>::splat(i32::MIN as f32);
        let max = Simd::<f32, 8>::splat(i32::MAX as f32);

        let unclamped = self.split_2();
        let clamped = [
            unclamped[0].simd_clamp::<{BackendType::AVX2}>(min, max),
            unclamped[1].simd_clamp::<{BackendType::AVX2}>(min, max)
        ];
        Self::convert(clamped.into())
    }
}

//==============================================================================================================================

impl SimdConvertImpl<i32, 4, {BackendType::AVX2}> for Simd<f64, 2> {
    fn simd_convert_impl(self) -> Simd<i32, 4> {
        <Self as SimdConvertImpl<i32, 4, {BackendType::AVX}>>::simd_convert_impl(self)
    }

    fn simd_convert_saturate_impl(self) -> Simd<i32, 4> {
        <Self as SimdConvertImpl<i32, 4, {BackendType::AVX}>>::simd_convert_saturate_impl(self)
    }
}

impl SimdConvertImpl<i32, 8, {BackendType::AVX2}> for Simd<f64, 4> {
    fn simd_convert_impl(self) -> Simd<i32, 8> {
        unsafe { _mm256_castsi128_si256(_mm256_cvtpd_epi32(self.into())).into() }
    }

    fn simd_convert_saturate_impl(self) -> Simd<i32, 8> {
        let min = Simd::<f64, 4>::splat(i32::MIN as f64);
        let max = Simd::<f64, 4>::splat(i32::MAX as f64);
        Self::convert(self.simd_clamp::<{BackendType::AVX2}>(min, max))
    }
}

impl SimdConvertImpl<i32, 16, {BackendType::AVX2}> for Simd<f64, 8> {
    fn simd_convert_impl(self) -> Simd<i32, 16> {
        unsafe { 
            let a : [__m128d; 4] = self.into();

            let lower0 = _mm_cvtpd_epi32(a[0]);
            let upper0 = _mm_cvtpd_epi32(a[1]);
            let combined0 = _mm_or_si128(lower0, _mm_bslli_si128::<8>(upper0));

            let lower1 = _mm_cvtpd_epi32(a[0]);
            let upper1 = _mm_cvtpd_epi32(a[1]);
            let combined1 = _mm_or_si128(lower1, _mm_bslli_si128::<8>(upper1));

            let zero = _mm_setzero_si128();
            [combined0, combined1, zero, zero].into()
        }
    }

    fn simd_convert_saturate_impl(self) -> Simd<i32, 16> {
        let min = Simd::<f64, 4>::splat(i32::MIN as f64);
        let max = Simd::<f64, 4>::splat(i32::MAX as f64);

        let unclamped = self.split_2();
        let clamped = [
            unclamped[0].simd_clamp::<{BackendType::AVX2}>(min, max),
            unclamped[1].simd_clamp::<{BackendType::AVX2}>(min, max)
        ];
        Self::convert(clamped.into())
    }
}

//==============================================================================================================================

impl SimdConvertImpl<i64, 2, {BackendType::AVX2}> for Simd<f64, 2> {
    fn simd_convert_impl(self) -> Simd<i64, 2> {
        <Self as SimdConvertImpl<i64, 2, {BackendType::AVX}>>::simd_convert_impl(self)
    }

    fn simd_convert_saturate_impl(self) -> Simd<i64, 2> {
        <Self as SimdConvertImpl<i64, 2, {BackendType::AVX}>>::simd_convert_saturate_impl(self)
    }
}

impl SimdConvertImpl<i64, 4, {BackendType::AVX2}> for Simd<f64, 4> {
    // https://stackoverflow.com/questions/41144668/how-to-efficiently-perform-double-int64-conversions-with-sse-avx
    // FIXME(jel): Only for [-2^51, 2^51]
    fn simd_convert_impl(self) -> Simd<i64, 4> {
        unsafe {
            let cnst = _mm256_set1_pd(0x0018000000000000u64 as f64);        
            cvt_f64_i64(self.into(), cnst).into()
        }
    }

    fn simd_convert_saturate_impl(self) -> Simd<i64, 4> {
        let min = Simd::<f64, 4>::splat(i64::MIN as f64);
        let max = Simd::<f64, 4>::splat(i64::MAX as f64);
        Self::convert(self.simd_clamp::<{BackendType::AVX2}>(min, max))
    }
}

impl SimdConvertImpl<i64, 8, {BackendType::AVX2}> for Simd<f64, 8> {
    fn simd_convert_impl(self) -> Simd<i64, 8> {
        unsafe {
            let a : [__m256d; 2] = self.into();
            let cnst = _mm256_set1_pd(0x0018000000000000u64 as f64);

            let res = [cvt_f64_i64(a[0], cnst),
                       cvt_f64_i64(a[1], cnst)];            
            res.into()
        }
    }

    fn simd_convert_saturate_impl(self) -> Simd<i64, 8> {
        let min = Simd::<f64, 4>::splat(i64::MIN as f64);
        let max = Simd::<f64, 4>::splat(i64::MAX as f64);

        let unclamped = self.split_2();
        let clamped = [
            unclamped[0].simd_clamp::<{BackendType::AVX2}>(min, max),
            unclamped[1].simd_clamp::<{BackendType::AVX2}>(min, max)
        ];
        Self::convert(clamped.into())
    }
}

//==============================================================================================================================

impl SimdConvertImpl<u64, 2, {BackendType::AVX2}> for Simd<f64, 2> {
    fn simd_convert_impl(self) -> Simd<u64, 2> {
        <Self as SimdConvertImpl<u64, 2, {BackendType::AVX}>>::simd_convert_impl(self)
    }

    fn simd_convert_saturate_impl(self) -> Simd<u64, 2> {
        <Self as SimdConvertImpl<u64, 2, {BackendType::AVX}>>::simd_convert_saturate_impl(self)
    }
}

impl SimdConvertImpl<u64, 4, {BackendType::AVX2}> for Simd<f64, 4> {
    // https://stackoverflow.com/questions/41144668/how-to-efficiently-perform-double-int64-conversions-with-sse-avx
    fn simd_convert_impl(self) -> Simd<u64, 4> {
        unsafe {
            let cnst = _mm256_set1_pd(0x0010000000000000u64 as f64);        
            cvt_f64_u64(self.into(), cnst).into()
        }
    }

    fn simd_convert_saturate_impl(self) -> Simd<u64, 4> {
        unsafe {
            let min = Simd::<f64, 4>::splat(u64::MIN as f64);
            let max = Simd::<f64, 4>::splat(u64::MAX as f64);
            Self::convert(self.simd_clamp::<{BackendType::AVX2}>(min, max))
        }
    }
}

impl SimdConvertImpl<u64, 8, {BackendType::AVX2}> for Simd<f64, 8> {
    // https://stackoverflow.com/questions/41144668/how-to-efficiently-perform-double-int64-conversions-with-sse-avx
    fn simd_convert_impl(self) -> Simd<u64, 8> {
        unsafe {
            let a : [__m256d; 2] = self.into();
            let cnst = _mm256_set1_pd(0x0010000000000000u64 as f64);

            let res = [cvt_f64_u64(a[0], cnst),
                       cvt_f64_u64(a[1], cnst)];            
            res.into()
        }
    }

    fn simd_convert_saturate_impl(self) -> Simd<u64, 8> {
        let min = Simd::<f64, 4>::splat(u64::MIN as f64);
        let max = Simd::<f64, 4>::splat(u64::MAX as f64);

        let unclamped = self.split_2();
        let clamped = [
            unclamped[0].simd_clamp::<{BackendType::AVX2}>(min, max),
            unclamped[1].simd_clamp::<{BackendType::AVX2}>(min, max)
        ];
        Self::convert(clamped.into())
    }
}

//==============================================================================================================================

impl SimdConvertImpl<u32, 4, {BackendType::AVX2}> for Simd<f32, 4> {
    fn simd_convert_impl(self) -> Simd<u32, 4> {
        <Self as SimdConvertImpl<u32, 4, {BackendType::AVX}>>::simd_convert_impl(self)
    }

    fn simd_convert_saturate_impl(self) -> Simd<u32, 4> {
        <Self as SimdConvertImpl<u32, 4, {BackendType::AVX}>>::simd_convert_saturate_impl(self)
    }
}

impl SimdConvertImpl<u32, 8, {BackendType::AVX2}> for Simd<f32, 8> {
    // https://stackoverflow.com/questions/9157373/most-efficient-way-to-convert-vector-of-float-to-vector-of-uint32
    // Implements the algorith above, but does not include saturating the value
    fn simd_convert_impl(self) -> Simd<u32, 8> {
        unsafe {
            let two31 = _mm256_set1_ps(0x0f800000 as f32);
            let zero = _mm256_setzero_ps();
            cvt_f32_u32(self.into(), two31, zero).into()
        }
    }

    fn simd_convert_saturate_impl(self) -> Simd<u32, 8> {
        let min = Simd::<f32, 8>::splat(u32::MIN as f32);
        let max = Simd::<f32, 8>::splat(u32::MAX as f32);
        Self::convert(self.simd_clamp::<{BackendType::AVX2}>(min, max))
    }
}

impl SimdConvertImpl<u32, 16, {BackendType::AVX2}> for Simd<f32, 16> {
    // https://stackoverflow.com/questions/9157373/most-efficient-way-to-convert-vector-of-float-to-vector-of-uint32
    // Implements the algorith above, but does not include saturating the value
    fn simd_convert_impl(self) -> Simd<u32, 16> {
        unsafe {
            let a : [__m256; 2] = self.into();
            let two31 = _mm256_set1_ps(0x0f800000 as f32);
            let zero = _mm256_setzero_ps();

            let res = [cvt_f32_u32(a[0], two31, zero),
                       cvt_f32_u32(a[1], two31, zero)];
            res.into()
        }
    }

    fn simd_convert_saturate_impl(self) -> Simd<u32, 16> {
        let min = Simd::<f32, 8>::splat(u32::MIN as f32);
        let max = Simd::<f32, 8>::splat(u32::MAX as f32);

        let unclamped = self.split_2();
        let clamped = [
            unclamped[0].simd_clamp::<{BackendType::AVX2}>(min, max),
            unclamped[1].simd_clamp::<{BackendType::AVX2}>(min, max)
        ];
        Self::convert(clamped.into())
    }
}

//==============================================================================================================================

impl SimdConvertImpl<f32, 4, {BackendType::AVX2}> for Simd<i32, 4> {
    fn simd_convert_impl(self) -> Simd<f32, 4> {
        <Self as SimdConvertImpl<f32, 4, {BackendType::AVX}>>::simd_convert_impl(self)
    }
}

impl SimdConvertImpl<f32, 8, {BackendType::AVX2}> for Simd<i32, 8> {
    fn simd_convert_impl(self) -> Simd<f32, 8> {
        unsafe { _mm256_cvtepi32_ps(self.into()).into() }
    }
}

impl SimdConvertImpl<f32, 16, {BackendType::AVX2}> for Simd<i32, 16> {
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
impl SimdConvertImpl<f64, 2, {BackendType::AVX2}> for Simd<i32, 4> {
    fn simd_convert_impl(self) -> Simd<f64, 2> {
        <Self as SimdConvertImpl<f64, 2, {BackendType::AVX}>>::simd_convert_impl(self)
    }
}

impl SimdConvertImpl<f64, 4, {BackendType::AVX2}> for Simd<i32, 8> {
    fn simd_convert_impl(self) -> Simd<f64, 4> {
        unsafe { _mm256_cvtepi32_pd(_mm256_castsi256_si128(self.into())).into() }
    }
}

impl SimdConvertImpl<f64, 8, {BackendType::AVX2}> for Simd<i32, 16> {
    fn simd_convert_impl(self) -> Simd<f64, 8> {
        unsafe {
            let a : [__m128i; 4] = self.into();
            [_mm256_cvtepi32_pd(a[0]),
             _mm256_cvtepi32_pd(a[1])].into()
        }
    }
}

//==============================================================================================================================

impl SimdConvertImpl<f32, 4, {BackendType::AVX2}> for Simd<u32, 4> {
    fn simd_convert_impl(self) -> Simd<f32, 4> {
        <Self as SimdConvertImpl<f32, 4, {BackendType::AVX}>>::simd_convert_impl(self)
    }
}

impl SimdConvertImpl<f32, 8, {BackendType::AVX2}> for Simd<u32, 8> {
    // https://stackoverflow.com/questions/9151711/most-efficient-way-to-convert-vector-of-uint32-to-vector-of-float
    fn simd_convert_impl(self) -> Simd<f32, 8> {
        unsafe {
            let mask = _mm256_set1_epi32(0x0000FFFF);
            let onep39 = _mm256_set1_ps(0x53000000 as f32);

            cvt_u32_f32(self.into(), mask, onep39).into()
        }
    }
}

impl SimdConvertImpl<f32, 16, {BackendType::AVX2}> for Simd<u32, 16> {
    // https://stackoverflow.com/questions/9151711/most-efficient-way-to-convert-vector-of-uint32-to-vector-of-float
    fn simd_convert_impl(self) -> Simd<f32, 16> {
        unsafe {
            let a : [__m256i; 2] = self.into();
            let mask = _mm256_set1_epi32(0x0000FFFF);
            let onep39 = _mm256_set1_ps(0x53000000 as f32);

            let res = [cvt_u32_f32(a[0], mask, onep39),
                       cvt_u32_f32(a[1], mask, onep39)];
            res.into()
        }
    }
}

//==============================================================================================================================

impl SimdConvertImpl<f64, 2, {BackendType::AVX2}> for Simd<i64, 2> {
    fn simd_convert_impl(self) -> Simd<f64, 2> {
        <Self as SimdConvertImpl<f64, 2, {BackendType::AVX}>>::simd_convert_impl(self)
    }
}

impl SimdConvertImpl<f64, 4, {BackendType::AVX2}> for Simd<i64, 4> {
    // https://stackoverflow.com/questions/41144668/how-to-efficiently-perform-double-int64-conversions-with-sse-avx
    fn simd_convert_impl(self) -> Simd<f64, 4> {
        unsafe {
            let cnst0 = _mm256_castpd_si256(_mm256_set1_pd(442721857769029238784.0f64));
            let cnst1 = _mm256_castpd_si256(_mm256_set1_pd(0x0010000000000000u64 as f64));
            let cnst2 = _mm256_set1_pd(442726361368656609280.0f64);

            cvt_i64_f64(self.into(), cnst0, cnst1, cnst2).into()
        }
    }
}

impl SimdConvertImpl<f64, 8, {BackendType::AVX2}> for Simd<i64, 8> {
    // https://stackoverflow.com/questions/41144668/how-to-efficiently-perform-double-int64-conversions-with-sse-avx
    fn simd_convert_impl(self) -> Simd<f64, 8> {
        unsafe {
            let a : [__m256i; 2] = self.into();
            let cnst0 = _mm256_castpd_si256(_mm256_set1_pd(442721857769029238784.0f64));
            let cnst1 = _mm256_castpd_si256(_mm256_set1_pd(0x0010000000000000u64 as f64));
            let cnst2 = _mm256_set1_pd(442726361368656609280.0f64);

            let res = [cvt_i64_f64(a[0], cnst0, cnst1, cnst2),
                       cvt_i64_f64(a[1], cnst0, cnst1, cnst2)];
            res.into()
        }
    }
}

//==============================================================================================================================

impl SimdConvertImpl<f64, 2, {BackendType::AVX2}> for Simd<u64, 2> {
    fn simd_convert_impl(self) -> Simd<f64, 2> {
        <Self as SimdConvertImpl<f64, 2, {BackendType::AVX}>>::simd_convert_impl(self)
    }
}

impl SimdConvertImpl<f64, 4, {BackendType::AVX2}> for Simd<u64, 4> {
    // https://stackoverflow.com/questions/41144668/how-to-efficiently-perform-double-int64-conversions-with-sse-avx
    fn simd_convert_impl(self) -> Simd<f64, 4> {
        unsafe { 
            let cnst0 = _mm256_castpd_si256(_mm256_set1_pd(19342813113834066795298816.0f64));
            let cnst1 = _mm256_castpd_si256(_mm256_set1_pd(0x0010000000000000u64 as f64));
            let cnst2 = _mm256_set1_pd(19342813118337666422669312.0f64);

            cvt_u64_f64(self.into(), cnst0, cnst1, cnst2).into()
        }
    }
}

impl SimdConvertImpl<f64, 8, {BackendType::AVX2}> for Simd<u64, 8> {
    // https://stackoverflow.com/questions/41144668/how-to-efficiently-perform-double-int64-conversions-with-sse-avx
    fn simd_convert_impl(self) -> Simd<f64, 8> {
        unsafe { 
            let a : [__m256i; 2] = self.into();
            let cnst0 = _mm256_castpd_si256(_mm256_set1_pd(19342813113834066795298816.0f64));
            let cnst1 = _mm256_castpd_si256(_mm256_set1_pd(0x0010000000000000u64 as f64));
            let cnst2 = _mm256_set1_pd(19342813118337666422669312.0f64);

            let res = [cvt_u64_f64(a[0], cnst0, cnst1, cnst2),
                       cvt_u64_f64(a[1], cnst0, cnst1, cnst2)];
            res.into()
        }
    }
}

//==============================================================================================================================

macro_rules! impl_2_step_cvt {
    { $([$from_ty:ty, $f_lanes:literal => $imm_ty:ty, $imm_lanes:literal => $to_ty:ty, $t_lanes:literal])* } => {
        $(
            impl SimdConvertImpl<$to_ty, $t_lanes, {BackendType::AVX2}> for Simd<$from_ty, $f_lanes> 
                where Self                      : SimdConvertImpl<$imm_ty, $imm_lanes, {BackendType::AVX2}>,
                      Simd<$imm_ty, $imm_lanes> : SimdConvertImpl<$to_ty, $t_lanes, {BackendType::AVX2}>
            {
                fn simd_convert_impl(self) -> Simd<$to_ty, $t_lanes> {
                    self.simd_convert::<$imm_ty, $imm_lanes, {BackendType::AVX2}>().simd_convert::<$to_ty, $t_lanes, {BackendType::AVX2}>()
                }
            
                fn simd_convert_saturate_impl(self) -> Simd<$to_ty, $t_lanes> {
                    self.simd_convert_saturate::<$imm_ty, $imm_lanes, {BackendType::AVX2}>().simd_convert_saturate::<$to_ty, $t_lanes, {BackendType::AVX2}>()
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
#[inline]
unsafe fn cvt_f64_i64(val: __m256d, cnst: __m256d) -> __m256i {
    let imm = _mm256_add_pd(val, cnst);
    _mm256_sub_epi64(_mm256_castpd_si256(imm), _mm256_castpd_si256(cnst)) 
}

// https://stackoverflow.com/questions/41144668/how-to-efficiently-perform-double-int64-conversions-with-sse-avx
// FIXME(jel): Only for [-2^51, 2^51]
#[inline]
unsafe fn cvt_f64_u64(val: __m256d, cnst: __m256d) -> __m256i {
    let imm = _mm256_add_pd(val, cnst);
    _mm256_xor_epi64(_mm256_castpd_si256(imm), _mm256_castpd_si256(cnst)) 
}

// https://stackoverflow.com/questions/9157373/most-efficient-way-to-convert-vector-of-float-to-vector-of-uint32
// Implements the algorith above, but does not include saturating the value
#[inline]
unsafe fn cvt_f32_u32(val: __m256, two31: __m256, zero: __m256) -> __m256i {
    // check for overflow before conversion to int
    let overflow = _mm256_cmp_ps::<_CMP_GE_OQ>(val, two31);
    let sub_val = _mm256_and_ps(overflow, two31);
    let add_val = _mm256_slli_epi32::<32>(_mm256_castps_si256(overflow));

    // bias the value to signed space if it's >= 2^31
    let imm = _mm256_sub_ps(val, sub_val);

    // convert to int, and unbias
    // rounding mode should be rount to nearest
    _mm256_add_epi32(_mm256_cvtps_epi32(imm), add_val)
}

// https://stackoverflow.com/questions/9151711/most-efficient-way-to-convert-vector-of-uint32-to-vector-of-float
#[inline]
unsafe fn cvt_u32_f32(val: __m256i, mask: __m256i, onep39: __m256) -> __m256 {
    let hi = _mm256_srli_si256::<16>(val);
    let lo = _mm256_and_si256(val, mask);
    let f_hi = _mm256_sub_ps(_mm256_or_ps(_mm256_castsi256_ps(hi), onep39), onep39);
    let f_lo = _mm256_cvtepi32_ps(lo);
    _mm256_add_ps(f_hi, f_lo)
}

// https://stackoverflow.com/questions/41144668/how-to-efficiently-perform-double-int64-conversions-with-sse-avx
#[inline]
unsafe fn cvt_i64_f64(val: __m256i, cnst0: __m256i, cnst1: __m256i, cnst2: __m256d) -> __m256d {
    let hi = _mm256_add_epi64(_mm256_blend_epi16(_mm256_srai_epi32::<16>(val), _mm256_setzero_si256(), 0x33), cnst0);
    let lo = _mm256_blend_epi16(val, cnst1, 0x88);
    let f = _mm256_sub_pd(_mm256_castsi256_pd(hi), cnst2);
    _mm256_add_pd(f, _mm256_castsi256_pd(lo))
}

// https://stackoverflow.com/questions/41144668/how-to-efficiently-perform-double-int64-conversions-with-sse-avx
#[inline]
unsafe fn cvt_u64_f64(val: __m256i, cnst0: __m256i, cnst1: __m256i, cnst2: __m256d) -> __m256d {
    let hi = _mm256_or_si256(_mm256_srli_epi64::<32>(val), cnst0);
    let lo = _mm256_blend_epi16(val, cnst1, 0xCC);
    let f = _mm256_sub_pd(_mm256_castsi256_pd(hi), cnst2);
    _mm256_add_pd(f, _mm256_castsi256_pd(lo))
}