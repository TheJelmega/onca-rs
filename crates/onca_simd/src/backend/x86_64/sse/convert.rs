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
            impl SimdConvertImpl<$ty, $lanes128, {BackendType::SSE}> for Simd<$ty, $lanes128>
                where LaneCount<$lanes128> : SupportedLaneCount
            {
                #[inline]
                fn simd_convert_impl(self) -> Self {
                    self
                }
            }

            impl SimdConvertImpl<$ty, $lanes256, {BackendType::SSE}> for Simd<$ty, $lanes256>
                where LaneCount<$lanes256> : SupportedLaneCount
            {
                #[inline]
                fn simd_convert_impl(self) -> Self {
                    self
                }
            }

            impl SimdConvertImpl<$ty, $lanes512, {BackendType::SSE}> for Simd<$ty, $lanes512>
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

macro_rules! impl_elem_bitcast {
    { $([$i_ty:ty => $u_ty:ty, $lanes128:literal, $lanes256:literal, $lanes512:literal])* } => {
        $(
            impl_elem_bitcast!{ @impl_i $i_ty => $u_ty, $lanes128, $lanes256, $lanes512 }
            impl_elem_bitcast!{ @impl_u $u_ty => $i_ty, $lanes128, $lanes256, $lanes512 }
        )*
    };
    { @impl_i $i_ty:ty => $u_ty:ty, $lanes128:literal, $lanes256:literal, $lanes512:literal } => {
        impl SimdConvertImpl<$u_ty, $lanes128, {BackendType::SSE}> for Simd<$i_ty, $lanes128> {
            #[inline]
            fn simd_convert_impl(self) -> Simd<$u_ty, $lanes128> {
                unsafe { core::mem::transmute_copy(&self) }
            }

            #[inline]
            fn simd_convert_saturate_impl(self) -> Simd<$u_ty, $lanes128>
                where Simd<$i_ty, $lanes128> : SimdSetImpl<$i_ty, {BackendType::SSE}>
            {
                unsafe {
                    let min = Simd::<$i_ty, $lanes128>::simd_splat::<{BackendType::SSE}>(<$u_ty>::MIN as $i_ty);
                    Self::convert(self.simd_max::<{BackendType::SSE}>(min))
                }
            }
        }

        impl SimdConvertImpl<$u_ty, $lanes256, {BackendType::SSE}> for Simd<$i_ty, $lanes256> {
            #[inline]
            fn simd_convert_impl(self) -> Simd<$u_ty, $lanes256> {
                unsafe { core::mem::transmute_copy(&self) }
            }

            #[inline]
            fn simd_convert_saturate_impl(self) -> Simd<$u_ty, $lanes256>
                where Simd<$i_ty, $lanes256> : SimdSetImpl<$i_ty, {BackendType::SSE}>
            {
                let min = Simd::<$i_ty, $lanes128>::simd_splat::<{BackendType::SSE}>(<$u_ty>::MIN as $i_ty);
                let unclamped = self.split_2();
                let clamped = [
                    unclamped[0].simd_max::<{BackendType::SSE}>(min),
                    unclamped[1].simd_max::<{BackendType::SSE}>(min)
                ];
                Self::convert(clamped.into())
            }
        }

        impl SimdConvertImpl<$u_ty, $lanes512, {BackendType::SSE}> for Simd<$i_ty, $lanes512> {
            #[inline]
            fn simd_convert_impl(self) -> Simd<$u_ty, $lanes512> {
                unsafe { core::mem::transmute_copy(&self) }
            }

            #[inline]
            fn simd_convert_saturate_impl(self) -> Simd<$u_ty, $lanes512>
                where Simd<$i_ty, $lanes512> : SimdSetImpl<$i_ty, {BackendType::SSE}>
            {
                let min = Simd::<$i_ty, $lanes128>::simd_splat::<{BackendType::SSE}>(<$u_ty>::MIN as $i_ty);
                let unclamped = self.split_4();
                let clamped = [
                    unclamped[0].simd_max::<{BackendType::SSE}>(min),
                    unclamped[1].simd_max::<{BackendType::SSE}>(min),
                    unclamped[2].simd_max::<{BackendType::SSE}>(min),
                    unclamped[3].simd_max::<{BackendType::SSE}>(min)
                ];
                Self::convert(clamped.into())
            }
        }
    };
    { @impl_u $u_ty:ty => $i_ty:ty, $lanes128:literal, $lanes256:literal, $lanes512:literal } => {
        impl SimdConvertImpl<$i_ty, $lanes128, {BackendType::SSE}> for Simd<$u_ty, $lanes128> {
            #[inline]
            fn simd_convert_impl(self) -> Simd<$i_ty, $lanes128> {
                unsafe { core::mem::transmute_copy(&self) }
            }

            #[inline]
            fn simd_convert_saturate_impl(self) -> Simd<$i_ty, $lanes128>
                where Simd<$u_ty, $lanes128> : SimdSetImpl<$u_ty, {BackendType::SSE}>
            {
                let max = Simd::<$u_ty, $lanes128>::simd_splat::<{BackendType::SSE}>(<$i_ty>::MAX as $u_ty);
                Self::convert(self.simd_min::<{BackendType::SSE}>(max))
            }
        }

        impl SimdConvertImpl<$i_ty, $lanes256, {BackendType::SSE}> for Simd<$u_ty, $lanes256> {
            #[inline]
            fn simd_convert_impl(self) -> Simd<$i_ty, $lanes256> {
                unsafe { core::mem::transmute_copy(&self) }
            }

            #[inline]
            fn simd_convert_saturate_impl(self) -> Simd<$i_ty, $lanes256>
                where Simd<$u_ty, $lanes256> : SimdSetImpl<$u_ty, {BackendType::SSE}>
            {
                let max = Simd::<$u_ty, $lanes128>::simd_splat::<{BackendType::SSE}>(<$i_ty>::MAX as $u_ty);
                let unclamped = self.split_2();
                let clamped = [
                    unclamped[0].simd_min::<{BackendType::SSE}>(max),
                    unclamped[1].simd_min::<{BackendType::SSE}>(max)
                ];
                Self::convert(clamped.into())
            }
        }

        impl SimdConvertImpl<$i_ty, $lanes512, {BackendType::SSE}> for Simd<$u_ty, $lanes512> {
            #[inline]
            fn simd_convert_impl(self) -> Simd<$i_ty, $lanes512> {
                unsafe { core::mem::transmute_copy(&self) }
            }

            #[inline]
            fn simd_convert_saturate_impl(self) -> Simd<$i_ty, $lanes512> 
                where Simd<$u_ty, $lanes512> : SimdSetImpl<$u_ty, {BackendType::SSE}>
            {
                let max = Simd::<$u_ty, $lanes128>::simd_splat::<{BackendType::SSE}>(<$i_ty>::MAX as $u_ty);
                let unclamped = self.split_4();
                let clamped = [
                    unclamped[0].simd_min::<{BackendType::SSE}>(max),
                    unclamped[1].simd_min::<{BackendType::SSE}>(max),
                    unclamped[2].simd_min::<{BackendType::SSE}>(max),
                    unclamped[3].simd_min::<{BackendType::SSE}>(max)
                ];
                Self::convert(clamped.into())
            }
        }
    };
}
impl_elem_bitcast!{
    [i8  => u8 , 16, 32, 64]
    [i16 => u16, 8 , 16, 32]
    [i32 => u32, 4 , 8 , 16]
    [i64 => u64, 2 , 4 , 8 ]
}

// PERF(jel): Only use `min` for unsigned
macro_rules! impl_narrow_64 {
    {$from_ty:ty, $to_ty:ty, $signed_to_ty:ty, 
     $from_lanes128:literal, $to_lanes128:literal,
     $from_lanes256:literal, $to_lanes256:literal,
     $from_lanes512:literal, $to_lanes512:literal,
     $set:ident,
     $mask0:expr;
     $mask1:expr;
     $mask2:expr;
     $mask3:expr;
    } => {
        impl_narrow_64!{
            @common
            $from_ty, $to_ty, $signed_to_ty,
            $from_lanes128, $to_lanes128,
            $from_lanes256, $to_lanes256,
            $set,
            $mask0;
            $mask1;
        }

        impl SimdConvertImpl<$to_ty, $to_lanes512, {BackendType::SSE}> for Simd<$from_ty, $from_lanes512> {
            #[inline]
            fn simd_convert_impl(self) -> Simd<$to_ty, $to_lanes512> {
                unsafe {
                    let a : [__m128i; 4] = self.into();
                    let shuffle_mask0 = $mask0;
                    let shuffle_mask1 = $mask1;
                    let shuffle_mask2 = $mask2;
                    let shuffle_mask3 = $mask3;
                    let res = [_mm_or_si128(_mm_or_si128(_mm_shuffle_epi8(a[0], shuffle_mask0), _mm_shuffle_epi8(a[1], shuffle_mask1)),
                                            _mm_or_si128(_mm_shuffle_epi8(a[2], shuffle_mask2), _mm_shuffle_epi8(a[3], shuffle_mask3))),
                               _mm_setzero_si128(),
                               _mm_setzero_si128(),
                               _mm_setzero_si128()];
                    res.into()
                }
            }
        
            #[inline]
            fn simd_convert_saturate_impl(self) -> Simd<$to_ty, $to_lanes512> {
                unsafe {
                    let min = Simd::<$from_ty, $from_lanes128>::splat(<$to_ty>::MIN as $from_ty);
                    let max = Simd::<$from_ty, $from_lanes128>::splat(<$to_ty>::MAX as $from_ty);

                    let unclamped = self.split_4();
                    let clamped = [
                        unclamped[0].simd_clamp::<{BackendType::SSE}>(min, max),
                        unclamped[1].simd_clamp::<{BackendType::SSE}>(min, max),
                        unclamped[2].simd_clamp::<{BackendType::SSE}>(min, max),
                        unclamped[3].simd_clamp::<{BackendType::SSE}>(min, max),
                    ];
                    Self::convert(clamped.into())
                }
            }
        }
    };
    {@common
     $from_ty:ty, $to_ty:ty, $signed_to_ty:ty, 
     $from_lanes128:literal, $to_lanes128:literal,
     $from_lanes256:literal, $to_lanes256:literal,
     $set:ident,
     $mask0:expr;
     $mask1:expr;
    } => {
        impl SimdConvertImpl<$to_ty, $to_lanes128, {BackendType::SSE}> for Simd<$from_ty, $from_lanes128> {
            #[inline]
            fn simd_convert_impl(self) -> Simd<$to_ty, $to_lanes128> {
                unsafe {
                    let a : __m128i = self.into();
                    let shuffle_mask = $mask0;
                    let res = _mm_shuffle_epi8(a, shuffle_mask);
                    res.into()
                }
            }
        
            #[inline]
            fn simd_convert_saturate_impl(self) -> Simd<$to_ty, $to_lanes128> {
                unsafe {
                    let min = Simd::<$from_ty, $from_lanes128>::splat(<$to_ty>::MIN as $from_ty);
                    let max = Simd::<$from_ty, $from_lanes128>::splat(<$to_ty>::MAX as $from_ty);
                    Self::convert(self.simd_clamp::<{BackendType::SSE}>(min, max))
                }
            }
        }
   
        impl SimdConvertImpl<$to_ty, $to_lanes256, {BackendType::SSE}> for Simd<$from_ty, $from_lanes256> {
            #[inline]
            fn simd_convert_impl(self) -> Simd<$to_ty, $to_lanes256> {
                unsafe {
                    let a : [__m128i; 2] = self.into();
                    let shuffle_mask0 = $mask0;
                    let shuffle_mask1 = $mask1;
                    let res = [_mm_or_si128(_mm_shuffle_epi8(a[0], shuffle_mask0), _mm_shuffle_epi8(a[1], shuffle_mask1)),
                               _mm_setzero_si128()];
                    res.into()
                }
            }
        
            #[inline]
            fn simd_convert_saturate_impl(self) -> Simd<$to_ty, $to_lanes256> {
                unsafe {
                    let a : [__m128i; 2] = self.into();
                    let min = Simd::<$from_ty, $from_lanes128>::splat(<$to_ty>::MIN as $from_ty);
                    let max = Simd::<$from_ty, $from_lanes128>::splat(<$to_ty>::MAX as $from_ty);

                    let clamped : [__m128i; 2] = [
                        Simd::<$from_ty, $from_lanes128>::from(a[0]).simd_clamp::<{BackendType::SSE}>(min, max).into(),
                        Simd::<$from_ty, $from_lanes128>::from(a[1]).simd_clamp::<{BackendType::SSE}>(min, max).into()
                    ];
                    Self::convert(Simd::<$from_ty, $from_lanes256>::from(clamped))
                }
            }
        }
    };
}

macro_rules! impl_narrow_64_32 {
    {$from_ty:ty, $to_ty:ty, $signed_to_ty:ty, 
     $from_lanes128:literal, $to_lanes128:literal,
     $from_lanes256:literal, $to_lanes256:literal,
     $from_lanes512:literal, $to_lanes512:literal,
     $set:ident,
     $mask0:expr;
     $mask1:expr;
    } => {
        impl_narrow_64!{
            @common
            $from_ty, $to_ty, $signed_to_ty,
            $from_lanes128, $to_lanes128,
            $from_lanes256, $to_lanes256,
            $set,
            $mask0;
            $mask1;
        }

        impl SimdConvertImpl<$to_ty, $to_lanes512, {BackendType::SSE}> for Simd<$from_ty, $from_lanes512> {
            #[inline]
            fn simd_convert_impl(self) -> Simd<$to_ty, $to_lanes512> {
                unsafe {
                    let a : [__m128i; 4] = self.into();
                    let shuffle_mask0 = $mask0;
                    let shuffle_mask1 = $mask1;
                    let res = [_mm_or_si128(_mm_shuffle_epi8(a[0], shuffle_mask0), _mm_shuffle_epi8(a[1], shuffle_mask1)),
                               _mm_or_si128(_mm_shuffle_epi8(a[2], shuffle_mask0), _mm_shuffle_epi8(a[3], shuffle_mask1)),
                               _mm_setzero_si128(),
                               _mm_setzero_si128()];
                    res.into()
                }
            }
        
            #[inline]
            fn simd_convert_saturate_impl(self) -> Simd<$to_ty, $to_lanes512> {
                unsafe {
                    let min = Simd::<$from_ty, $from_lanes128>::splat(<$to_ty>::MIN as $from_ty);
                    let max = Simd::<$from_ty, $from_lanes128>::splat(<$to_ty>::MAX as $from_ty);

                    let unclamped = self.split_4();
                    let clamped = [
                        unclamped[0].simd_clamp::<{BackendType::SSE}>(min, max),
                        unclamped[1].simd_clamp::<{BackendType::SSE}>(min, max),
                        unclamped[2].simd_clamp::<{BackendType::SSE}>(min, max),
                        unclamped[3].simd_clamp::<{BackendType::SSE}>(min, max),
                    ];
                    Self::convert(clamped.into())
                }
            }
        }
    };
}

macro_rules! impl_narrow {
    {@16_8 $from_ty:ty, $to_ty:ty, $pack16:ident } => {
        impl SimdConvertImpl<$to_ty, 16, {BackendType::SSE}> for Simd<$from_ty, 8> {
            #[inline]
            fn simd_convert_impl(self) -> Simd<$to_ty, 16> {
                unsafe {
                    let a : __m128i = self.into();
                    let shuffle_mask = _mm_setr_epi8(0, 2, 4, 6, 8, 10, 12, 14, -1, -1, -1, -1, -1, -1, -1, -1);
                    let res = _mm_shuffle_epi8(a, shuffle_mask);
                    res.into()
                }
            }
        
            #[inline]
            fn simd_convert_saturate_impl(self) -> Simd<$to_ty, 16> {
                unsafe {
                    let a : __m128i = self.into();
                    let res = $pack16(a, _mm_setzero_si128());
                    res.into()
                }
            }
        }

        impl SimdConvertImpl<$to_ty, 32, {BackendType::SSE}> for Simd<$from_ty, 16> {
            #[inline]
            fn simd_convert_impl(self) -> Simd<$to_ty, 32> {
                unsafe {
                    let a : [__m128i; 2] = self.into();
                    let shuffle_mask0 = _mm_setr_epi8( 0,  2,  4,  6,  8, 10, 12, 14, -1, -1, -1, -1, -1, -1, -1, -1);
                    let shuffle_mask1 = _mm_setr_epi8(-1, -1, -1, -1, -1, -1, -1, -1,  0,  2,  4,  6,  8, 10, 12, 14);
                    let res = [_mm_or_si128(_mm_shuffle_epi8(a[0], shuffle_mask0), _mm_shuffle_epi8(a[1], shuffle_mask1)),
                               _mm_setzero_si128()];
                    res.into()
                }
            }
        
            #[inline]
            fn simd_convert_saturate_impl(self) -> Simd<$to_ty, 32> {
                unsafe {
                    let a : [__m128i; 2] = self.into();
                    let res = [$pack16(a[0], a[1]), _mm_setzero_si128()];
                    res.into()
                }
            }
        }

        impl SimdConvertImpl<$to_ty, 64, {BackendType::SSE}> for Simd<$from_ty, 32> {
            #[inline]
            fn simd_convert_impl(self) -> Simd<$to_ty, 64> {
                unsafe {
                    let a : [__m128i; 4] = self.into();
                    let shuffle_mask0 = _mm_setr_epi8( 0,  2,  4,  6,  8, 10, 12, 14, -1, -1, -1, -1, -1, -1, -1, -1);
                    let shuffle_mask1 = _mm_setr_epi8(-1, -1, -1, -1, -1, -1, -1, -1,  0,  2,  4,  6,  8, 10, 12, 14);
                    let res = [_mm_or_si128(_mm_shuffle_epi8(a[0], shuffle_mask0), _mm_shuffle_epi8(a[1], shuffle_mask1)),
                               _mm_or_si128(_mm_shuffle_epi8(a[2], shuffle_mask0), _mm_shuffle_epi8(a[3], shuffle_mask1)),
                               _mm_setzero_si128(),
                               _mm_setzero_si128()];
                    res.into()
                }
            }
        
            #[inline]
            fn simd_convert_saturate_impl(self) -> Simd<$to_ty, 64> {
                unsafe {
                    let a : [__m128i; 4] = self.into();
                    let res = [$pack16(a[0], a[1]),
                               $pack16(a[2], a[3]),
                               _mm_setzero_si128(),
                               _mm_setzero_si128()];
                    res.into()
                }
            }
        }
    };
    {@32_8 $from_ty:ty, $to_ty:ty, $pack16:ident, $pack32:ident } => {
        impl SimdConvertImpl<$to_ty, 16, {BackendType::SSE}> for Simd<$from_ty, 4> {
            #[inline]
            fn simd_convert_impl(self) -> Simd<$to_ty, 16> {
                unsafe {
                    let a : __m128i = self.into();
                    let shuffle_mask = _mm_setr_epi8(0, 4, 8, 12, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1);
                    let res = _mm_shuffle_epi8(a, shuffle_mask);
                    res.into()
                }
            }
        
            #[inline]
            fn simd_convert_saturate_impl(self) -> Simd<$to_ty, 16> {
                unsafe {
                    let a : __m128i = self.into();
                    let z = _mm_setzero_si128();
                    let res = $pack16($pack32(a, z), z);
                    res.into()
                }
            }
        }

        impl SimdConvertImpl<$to_ty, 32, {BackendType::SSE}> for Simd<$from_ty, 8> {
            #[inline]
            fn simd_convert_impl(self) -> Simd<$to_ty, 32> {
                unsafe {
                    let a : [__m128i; 2] = self.into();
                    let shuffle_mask0 = _mm_setr_epi8( 0,  4,  8, 12, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1);
                    let shuffle_mask1 = _mm_setr_epi8(-1, -1, -1, -1,  0,  4,  8, 12, -1, -1, -1, -1, -1, -1, -1, -1);
                    let res = [_mm_or_si128(_mm_shuffle_epi8(a[0], shuffle_mask0), _mm_shuffle_epi8(a[1], shuffle_mask1)),
                               _mm_setzero_si128()];
                    res.into()
                }
            }
        
            #[inline]
            fn simd_convert_saturate_impl(self) -> Simd<$to_ty, 32> {
                unsafe {
                    let a : [__m128i; 2] = self.into();
                    let z = _mm_setzero_si128();
                    let res = [$pack16($pack32(a[0], a[1]), z), z];
                    res.into()
                }
            }
        }

        impl SimdConvertImpl<$to_ty, 64, {BackendType::SSE}> for Simd<$from_ty, 16> {
            #[inline]
            fn simd_convert_impl(self) -> Simd<$to_ty, 64> {
                unsafe {
                    let a : [__m128i; 4] = self.into();
                    let shuffle_mask0 = _mm_setr_epi8( 0,  4,  8, 12, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1);
                    let shuffle_mask1 = _mm_setr_epi8(-1, -1, -1, -1,  0,  4,  8, 12, -1, -1, -1, -1, -1, -1, -1, -1);
                    let shuffle_mask2 = _mm_setr_epi8(-1, -1, -1, -1, -1, -1, -1, -1,  0,  4,  8, 12, -1, -1, -1, -1);
                    let shuffle_mask3 = _mm_setr_epi8(-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,  0,  4,  8, 12);
                    let res = [_mm_or_si128(_mm_or_si128(_mm_shuffle_epi8(a[0], shuffle_mask0), _mm_shuffle_epi8(a[1], shuffle_mask1)), 
                                            _mm_or_si128(_mm_shuffle_epi8(a[2], shuffle_mask2), _mm_shuffle_epi8(a[3], shuffle_mask3))),
                               _mm_setzero_si128(),
                               _mm_setzero_si128(),
                               _mm_setzero_si128()];
                    res.into()
                }
            }
        
            #[inline]
            fn simd_convert_saturate_impl(self) -> Simd<$to_ty, 64> {
                unsafe {
                    let a : [__m128i; 4] = self.into();
                    let res = [$pack16($pack32(a[0], a[1]), $pack32(a[2], a[3])),
                               _mm_setzero_si128(),
                               _mm_setzero_si128(),
                               _mm_setzero_si128()];
                    res.into()
                }
            }
        }
    };
    {@32_16 $from_ty:ty, $to_ty:ty, $pack32:ident } => {
        impl SimdConvertImpl<$to_ty, 8, {BackendType::SSE}> for Simd<$from_ty, 4> {
            #[inline]
            fn simd_convert_impl(self) -> Simd<$to_ty, 8> {
                unsafe {
                    let a : __m128i = self.into();
                    let shuffle_mask = _mm_setr_epi8(0, 1, 4, 5, 8, 9, 12, 13, -1, -1, -1, -1, -1, -1, -1, -1);
                    let res = _mm_shuffle_epi8(a, shuffle_mask);
                    res.into()
                }
            }
        
            #[inline]
            fn simd_convert_saturate_impl(self) -> Simd<$to_ty, 8> {
                unsafe {
                    let a : __m128i = self.into();
                    let res = $pack32(a, _mm_setzero_si128());
                    res.into()
                }
            }
        }

        impl SimdConvertImpl<$to_ty, 16, {BackendType::SSE}> for Simd<$from_ty, 8> {
            #[inline]
            fn simd_convert_impl(self) -> Simd<$to_ty, 16> {
                unsafe {
                    let a : [__m128i; 2] = self.into();
                    let shuffle_mask0 = _mm_setr_epi8( 0,  1,  4,  5,  8,  9, 12, 13, -1, -1, -1, -1, -1, -1, -1, -1);
                    let shuffle_mask1 = _mm_setr_epi8(-1, -1, -1, -1, -1, -1, -1, -1,  0,  1,  4,  5,  8,  9, 12, 13);
                    let res = [_mm_or_si128(_mm_shuffle_epi8(a[0], shuffle_mask0), _mm_shuffle_epi8(a[1], shuffle_mask1)),
                               _mm_setzero_si128()];
                    res.into()
                }
            }
        
            #[inline]
            fn simd_convert_saturate_impl(self) -> Simd<$to_ty, 16> {
                unsafe {
                    let a : [__m128i; 2] = self.into();
                    let res = [$pack32(a[0], a[1]),
                               _mm_setzero_si128()];
                    res.into()
                }
            }
        }

        impl SimdConvertImpl<$to_ty, 32, {BackendType::SSE}> for Simd<$from_ty, 16> {
            #[inline]
            fn simd_convert_impl(self) -> Simd<$to_ty, 32> {
                unsafe {
                    let a : [__m128i; 4] = self.into();
                    let shuffle_mask0 = _mm_setr_epi8( 0,  1,  4,  5,  8,  9, 12, 13, -1, -1, -1, -1, -1, -1, -1, -1);
                    let shuffle_mask1 = _mm_setr_epi8(-1, -1, -1, -1, -1, -1, -1, -1,  0,  1,  4,  5,  8,  9, 12, 13);
                    let res = [_mm_or_si128(_mm_shuffle_epi8(a[0], shuffle_mask0), _mm_shuffle_epi8(a[1], shuffle_mask1)),
                               _mm_or_si128(_mm_shuffle_epi8(a[2], shuffle_mask0), _mm_shuffle_epi8(a[3], shuffle_mask1)),
                               _mm_setzero_si128(),
                               _mm_setzero_si128()];
                    res.into()
                }
            }
        
            #[inline]
            fn simd_convert_saturate_impl(self) -> Simd<$to_ty, 32> {
                unsafe {
                    let a : [__m128i; 4] = self.into();
                    let res = [$pack32(a[0], a[1]),
                               $pack32(a[2], a[3]),
                               _mm_setzero_si128(),
                               _mm_setzero_si128()];
                    res.into()
                }
            }
        }
    }
}

//==============================================================================================================================

impl_narrow!{ @16_8 i16, i8, _mm_packs_epi16 }
impl_narrow!{ @32_8 i32, i8, _mm_packs_epi16, _mm_packs_epi32 }
impl_narrow_64!{i64, i8, i8,
                2  , 16,
                4  , 32,
                8  , 64,
                _mm_set1_epi8,
                _mm_setr_epi8( 0,  8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1);
                _mm_setr_epi8(-1, -1,  0,  8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1);
                _mm_setr_epi8(-1, -1, -1, -1,  0,  8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1);
                _mm_setr_epi8(-1, -1, -1, -1, -1, -1,  0,  8, -1, -1, -1, -1, -1, -1, -1, -1);
}

impl_narrow!{ @32_16 i32, i16, _mm_packs_epi32 }
impl_narrow_64!{i64, i16, i16,
                2  , 8,
                4  , 16,
                8  , 32,
                _mm_set1_epi16,
                _mm_setr_epi8( 0,  1,  8,  9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1);
                _mm_setr_epi8(-1, -1, -1, -1,  0,  1,  8,  9, -1, -1, -1, -1, -1, -1, -1, -1);
                _mm_setr_epi8(-1, -1, -1, -1, -1, -1, -1, -1,  0,  1,  8,  9, -1, -1, -1, -1);
                _mm_setr_epi8(-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,  0,  1,  8,  9);
}

impl_narrow_64_32!{i64, i32, i32,
                   2  , 4,
                   4  , 8,
                   8  , 16,
                   _mm_set1_epi32,
                   _mm_setr_epi8( 0,  1,  2,  3,  8,  9, 10, 11, -1, -1, -1, -1, -1, -1, -1, -1);
                   _mm_setr_epi8(-1, -1, -1, -1, -1, -1, -1, -1,  0,  1,  2,  3,  8,  9, 10, 11);
}

//==============================================================================================================================

impl_narrow!{ @16_8 u16, u8, _mm_packus_epi16 }
impl_narrow!{ @32_8 u32, u8, _mm_packus_epi16, _mm_packs_epi32 }
impl_narrow_64!{u64, u8, i8,
                2  , 16,
                4  , 32,
                8  , 64,
                _mm_set1_epi8,
                _mm_setr_epi8( 0,  8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1);
                _mm_setr_epi8(-1, -1,  0,  8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1);
                _mm_setr_epi8(-1, -1, -1, -1,  0,  8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1);
                _mm_setr_epi8(-1, -1, -1, -1, -1, -1,  0,  8, -1, -1, -1, -1, -1, -1, -1, -1);
}

impl_narrow!{ @32_16 u32, u16, _mm_packus_epi32 }
impl_narrow_64!{u64, u16, i16,
                2  , 8,
                4  , 16,
                8  , 32,
                _mm_set1_epi16,
                _mm_setr_epi8( 0,  1,  8,  9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1);
                _mm_setr_epi8(-1, -1, -1, -1,  0,  1,  8,  9, -1, -1, -1, -1, -1, -1, -1, -1);
                _mm_setr_epi8(-1, -1, -1, -1, -1, -1, -1, -1,  0,  1,  8,  9, -1, -1, -1, -1);
                _mm_setr_epi8(-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,  0,  1,  8,  9);
}

impl_narrow_64_32!{u64, u32, i32,
                   2  , 4,
                   4  , 8,
                   8  , 16,
                   _mm_set1_epi32,
                   _mm_setr_epi8( 0,  1,  2,  3,  8,  9, 10, 11, -1, -1, -1, -1, -1, -1, -1, -1);
                   _mm_setr_epi8(-1, -1, -1, -1, -1, -1, -1, -1,  0,  1,  2,  3,  8,  9, 10, 11);
}

//==============================================================================================================================

impl_narrow!{ @16_8 u16, i8, _mm_packs_epi16 }
impl_narrow!{ @32_8 u32, i8, _mm_packs_epi16, _mm_packs_epi32 }
impl_narrow_64!{u64, i8, i8,
                2  , 16,
                4  , 32,
                8  , 64,
                _mm_set1_epi8,
                _mm_setr_epi8( 0,  8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1);
                _mm_setr_epi8(-1, -1,  0,  8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1);
                _mm_setr_epi8(-1, -1, -1, -1,  0,  8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1);
                _mm_setr_epi8(-1, -1, -1, -1, -1, -1,  0,  8, -1, -1, -1, -1, -1, -1, -1, -1);
}

impl_narrow!{ @32_16 u32, i16, _mm_packs_epi32 }
impl_narrow_64!{u64, i16, i16,
                2  , 8,
                4  , 16,
                8  , 32,
                _mm_set1_epi16,
                _mm_setr_epi8( 0,  1,  8,  9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1);
                _mm_setr_epi8(-1, -1, -1, -1,  0,  1,  8,  9, -1, -1, -1, -1, -1, -1, -1, -1);
                _mm_setr_epi8(-1, -1, -1, -1, -1, -1, -1, -1,  0,  1,  8,  9, -1, -1, -1, -1);
                _mm_setr_epi8(-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,  0,  1,  8,  9);
}

impl_narrow_64_32!{u64, i32, i32,
                   2  , 4,
                   4  , 8,
                   8  , 16,
                   _mm_set1_epi32,
                   _mm_setr_epi8( 0,  1,  2,  3,  8,  9, 10, 11, -1, -1, -1, -1, -1, -1, -1, -1);
                   _mm_setr_epi8(-1, -1, -1, -1, -1, -1, -1, -1,  0,  1,  2,  3,  8,  9, 10, 11);
}

//==============================================================================================================================

impl_narrow!{ @16_8 i16, u8, _mm_packus_epi16 }
impl_narrow!{ @32_8 i32, u8, _mm_packus_epi16, _mm_packs_epi32 }
impl_narrow_64!{i64, u8, i8,
                2  , 16,
                4  , 32,
                8  , 64,
                _mm_set1_epi8,
                _mm_setr_epi8( 0,  8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1);
                _mm_setr_epi8(-1, -1,  0,  8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1);
                _mm_setr_epi8(-1, -1, -1, -1,  0,  8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1);
                _mm_setr_epi8(-1, -1, -1, -1, -1, -1,  0,  8, -1, -1, -1, -1, -1, -1, -1, -1);
}

impl_narrow!{ @32_16 i32, u16, _mm_packus_epi32 }
impl_narrow_64!{i64, u16, i16,
                2  , 8,
                4  , 16,
                8  , 32,
                _mm_set1_epi16,
                _mm_setr_epi8( 0,  1,  8,  9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1);
                _mm_setr_epi8(-1, -1, -1, -1,  0,  1,  8,  9, -1, -1, -1, -1, -1, -1, -1, -1);
                _mm_setr_epi8(-1, -1, -1, -1, -1, -1, -1, -1,  0,  1,  8,  9, -1, -1, -1, -1);
                _mm_setr_epi8(-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,  0,  1,  8,  9);
}

impl_narrow_64_32!{i64, u32, i32,
                   2  , 4,
                   4  , 8,
                   8  , 16,
                   _mm_set1_epi32,
                   _mm_setr_epi8( 0,  1,  2,  3,  8,  9, 10, 11, -1, -1, -1, -1, -1, -1, -1, -1);
                   _mm_setr_epi8(-1, -1, -1, -1, -1, -1, -1, -1,  0,  1,  2,  3,  8,  9, 10, 11);
}

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
            impl SimdConvertImpl<$to_ty, $to_lanes128, {BackendType::SSE}> for Simd<$from_ty, $from_lanes128> {
                fn simd_convert_impl(self) -> Simd<$to_ty, $to_lanes128> {
                    unsafe { $cvt(self.into()).into() }
                }
            }
            
            impl SimdConvertImpl<$to_ty, $to_lanes256, {BackendType::SSE}> for Simd<$from_ty, $from_lanes256> {
                fn simd_convert_impl(self) -> Simd<$to_ty, $to_lanes256> {
                    unsafe {
                        let a : [__m128i; 2] = self.into();
                        let res = [$cvt(a[0]),
                                   $cvt(_mm_srli_si128::<8>(a[0]))];
                        res.into()
                    }
                }
            }
            
            impl SimdConvertImpl<$to_ty, $to_lanes512, {BackendType::SSE}> for Simd<$from_ty, $from_lanes512> {
                fn simd_convert_impl(self) -> Simd<$to_ty, $to_lanes512> {
                    unsafe {
                        let a : [__m128i; 4] = self.into();
                        let res = [$cvt(a[0]),
                                   $cvt(_mm_srli_si128::<8>(a[0])),
                                   $cvt(a[1]),
                                   $cvt(_mm_srli_si128::<8>(a[1]))];
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
           impl SimdConvertImpl<$to_ty, $to_lanes128, {BackendType::SSE}> for Simd<$from_ty, $from_lanes128> {
            fn simd_convert_impl(self) -> Simd<$to_ty, $to_lanes128> {
                   unsafe { $cvt(self.into()).into() }
               }
           }
           
           impl SimdConvertImpl<$to_ty, $to_lanes256, {BackendType::SSE}> for Simd<$from_ty, $from_lanes256> {
            fn simd_convert_impl(self) -> Simd<$to_ty, $to_lanes256> {
                   unsafe {
                       let a : [__m128i; 2] = self.into();
                       let res = [$cvt(a[0]),
                                  $cvt(_mm_srli_si128::<4>(a[0]))];
                       res.into()
                   }
               }
           }
           
           impl SimdConvertImpl<$to_ty, $to_lanes512, {BackendType::SSE}> for Simd<$from_ty, $from_lanes512> {
            fn simd_convert_impl(self) -> Simd<$to_ty, $to_lanes512> {
                   unsafe {
                       let a : [__m128i; 4] = self.into();
                       let res = [$cvt(a[0]),
                                  $cvt(_mm_srli_si128::<4>(a[0])),
                                  $cvt(_mm_srli_si128::<8>(a[0])),
                                  $cvt(_mm_srli_si128::<12>(a[0]))];
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
            impl SimdConvertImpl<$to_ty, $to_lanes128, {BackendType::SSE}> for Simd<$from_ty, $from_lanes128> {
                fn simd_convert_impl(self) -> Simd<$to_ty, $to_lanes128> {
                    unsafe { $cvt(self.into()).into() }
                }
            }
            
            impl SimdConvertImpl<$to_ty, $to_lanes256, {BackendType::SSE}> for Simd<$from_ty, $from_lanes256> {
                fn simd_convert_impl(self) -> Simd<$to_ty, $to_lanes256> {
                    unsafe {
                        let a : [__m128i; 2] = self.into();
                        let res = [$cvt(a[0]),
                                   $cvt(_mm_srli_si128::<2>(a[0]))];
                        res.into()
                    }
                }
            }
            
            impl SimdConvertImpl<$to_ty, $to_lanes512, {BackendType::SSE}> for Simd<$from_ty, $from_lanes512> {
                fn simd_convert_impl(self) -> Simd<$to_ty, $to_lanes512> {
                    unsafe {
                        let a : [__m128i; 4] = self.into();
                        let res = [$cvt(a[0]),
                                   $cvt(_mm_srli_si128::<2>(a[0])),
                                   $cvt(_mm_srli_si128::<4>(a[0])),
                                   $cvt(_mm_srli_si128::<6>(a[0]))];
                        res.into()
                    }
                }
            }
        )*
    };
}
impl_widen!{ @2x
    [i8  => i16; 16 => 8, 32 => 16, 64 => 32, _mm_cvtepi8_epi16]
    [i16 => i32; 8  => 4, 16 => 8 , 32 => 16, _mm_cvtepi16_epi32]
    [i32 => i64; 4  => 2, 8  => 4 , 16 => 8 , _mm_cvtepi32_epi64]

    [u8  => i16; 16 => 8, 32 => 16, 64 => 32, _mm_cvtepu8_epi16]
    [u16 => i32; 8  => 4, 16 => 8 , 32 => 16, _mm_cvtepu16_epi32]
    [u32 => i64; 4  => 2, 8  => 4 , 16 => 8 , _mm_cvtepu32_epi64]

    [i8  => u16; 16 => 8, 32 => 16, 64 => 32, _mm_cvtepu8_epi16]
    [i16 => u32; 8  => 4, 16 => 8 , 32 => 16, _mm_cvtepu16_epi32]
    [i32 => u64; 4  => 2, 8  => 4 , 16 => 8 , _mm_cvtepu32_epi64]

    [u8  => u16; 16 => 8, 32 => 16, 64 => 32, _mm_cvtepu8_epi16]
    [u16 => u32; 8  => 4, 16 => 8 , 32 => 16, _mm_cvtepu16_epi32]
    [u32 => u64; 4  => 2, 8  => 4 , 16 => 8 , _mm_cvtepu32_epi64]
}
impl_widen!{ @4x
    [i8  => i32; 16 => 4, 32 => 8 , 64 => 16, _mm_cvtepi8_epi32]
    [i16 => i64; 8  => 2, 16 => 4 , 32 => 8 , _mm_cvtepi16_epi64]

    [u8  => i32; 16 => 4, 32 => 8 , 64 => 16, _mm_cvtepu8_epi32]
    [u16 => i64; 8  => 2, 16 => 4 , 32 => 8 , _mm_cvtepu16_epi64]

    [i8  => u32; 16 => 4, 32 => 8 , 64 => 16, _mm_cvtepu8_epi32]
    [i16 => u64; 8  => 2, 16 => 4 , 32 => 8 , _mm_cvtepu16_epi64]

    [u8  => u32; 16 => 4, 32 => 8 , 64 => 16, _mm_cvtepu8_epi32]
    [u16 => u64; 8  => 2, 16 => 4 , 32 => 8 , _mm_cvtepu16_epi64]
}
impl_widen!{ @8x
    [i8  => i64; 16 => 2, 32 => 4 , 64 => 8 , _mm_cvtepi8_epi64]
    [u8  => i64; 16 => 2, 32 => 4 , 64 => 8 , _mm_cvtepu8_epi64]
    [i8  => u64; 16 => 2, 32 => 4 , 64 => 8 , _mm_cvtepu8_epi64]
    [u8  => u64; 16 => 2, 32 => 4 , 64 => 8 , _mm_cvtepu8_epi64]
}

//==============================================================================================================================

macro_rules! impl_widen_elem {
    { $([$from_ty:ty => $to_ty:ty, 
         $lanes128:literal <=> $imm_lanes128:literal, 
         $lanes256:literal <=> $imm_lanes256:literal, 
         $lanes512:literal <=> $imm_lanes512:literal])*
    } => {
        $(
            impl SimdConvertImpl<$to_ty, $lanes128, {BackendType::SSE}> for Simd<$from_ty, $lanes128> {
                fn simd_convert_impl(self) -> Simd<$to_ty, $lanes128> {
                    #[repr(align(16))]
                    union LoadSrc {
                        simd: Simd<$from_ty, $lanes128>,
                        buf  : [$from_ty; $imm_lanes128]
                    }

                    unsafe { 
                        let load_src = LoadSrc{ simd: self };
                        let loaded : Simd<$from_ty, $imm_lanes128> = _mm_load_si128(load_src.buf.as_ptr() as *const __m128i).into();
                        loaded.simd_convert::<$to_ty, $lanes128, {BackendType::SSE}>()
                     }
                }
            }

            impl SimdConvertImpl<$to_ty, $lanes256, {BackendType::SSE}> for Simd<$from_ty, $lanes256> {
                fn simd_convert_impl(self) -> Simd<$to_ty, $lanes256> {
                    #[repr(align(16))]
                    union LoadSrc {
                        simd: Simd<$from_ty, $lanes256>,
                        buf  : [$from_ty; $imm_lanes256]
                    }

                    unsafe { 
                        let load_src = LoadSrc{ simd: self };
                        let loaded : Simd<$from_ty, $imm_lanes256> = [_mm_load_si128( load_src.buf.as_ptr() as *const __m128i         ),
                                                                      _mm_load_si128((load_src.buf.as_ptr() as *const __m128i).add(1))].into();
                        loaded.simd_convert::<$to_ty, $lanes256, {BackendType::SSE}>()
                     }
                }
            }

            impl SimdConvertImpl<$to_ty, $lanes512, {BackendType::SSE}> for Simd<$from_ty, $lanes512> {
                fn simd_convert_impl(self) -> Simd<$to_ty, $lanes512> {
                    #[repr(align(16))]
                    union LoadSrc {
                        simd: Simd<$from_ty, $lanes512>,
                        buf : [$from_ty; $imm_lanes512]
                    }

                    unsafe { 
                        let load_src = LoadSrc{ simd: self };
                        let loaded : Simd<$from_ty, $imm_lanes512> = [_mm_load_si128( load_src.buf.as_ptr() as *const __m128i        ),
                                                                      _mm_load_si128((load_src.buf.as_ptr() as *const __m128i).add(1)),
                                                                      _mm_load_si128((load_src.buf.as_ptr() as *const __m128i).add(2)),
                                                                      _mm_load_si128((load_src.buf.as_ptr() as *const __m128i).add(3))].into();
                        loaded.simd_convert::<$to_ty, $lanes512, {BackendType::SSE}>()
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

impl SimdConvertImpl<f64, 2, {BackendType::SSE}> for Simd<f32, 4> {
    fn simd_convert_impl(self) -> Simd<f64, 2> {
        unsafe { _mm_cvtps_pd(self.into()).into() }
    }
}

impl SimdConvertImpl<f64, 4, {BackendType::SSE}> for Simd<f32, 8> {
    fn simd_convert_impl(self) -> Simd<f64, 4> {
        unsafe {
            let a : [__m128; 2] = self.into();
            let res = [_mm_cvtps_pd(a[0]), _mm_cvtps_pd(a[1])];
            res.into()
        }
    }
}

impl SimdConvertImpl<f64, 8, {BackendType::SSE}> for Simd<f32, 16> {
    fn simd_convert_impl(self) -> Simd<f64, 8> {
        unsafe {
            let a : [__m128; 4] = self.into();
            let res = [_mm_cvtps_pd(a[0]), _mm_cvtps_pd(a[1]), _mm_cvtps_pd(a[2]), _mm_cvtps_pd(a[3])];
            res.into()
        }
    }
}

impl SimdConvertImpl<f32, 4, {BackendType::SSE}> for Simd<f64, 2> {
    fn simd_convert_impl(self) -> Simd<f32, 4> {
        unsafe { _mm_cvtpd_ps(self.into()).into() }
    }

    fn simd_convert_saturate_impl(self) -> Simd<f32, 4> {
        unsafe {
            let min = Simd::<f64, 2>::splat(f32::MIN as f64);
            let max = Simd::<f64, 2>::splat(f32::MAX as f64);

            Self::convert(self.simd_clamp::<{BackendType::SSE}>(min, max))
        }
    }  
}

impl SimdConvertImpl<f32, 8, {BackendType::SSE}> for Simd<f64, 4> {
    fn simd_convert_impl(self) -> Simd<f32, 8> {
        unsafe {
            let a : [__m128d; 2] = self.into();
            let res = [_mm_or_ps(_mm_cvtpd_ps(a[0]), _mm_castsi128_ps(_mm_srli_si128::<8>(_mm_castps_si128(_mm_cvtpd_ps(a[1]))))),
                       _mm_setzero_ps()];
            res.into()
        }
    }

    fn simd_convert_saturate_impl(self) -> Simd<f32, 8> {
        unsafe {
            let a : [__m128d; 2] = self.into();
            let min = Simd::<f64, 2>::splat(f32::MIN as f64);
            let max = Simd::<f64, 2>::splat(f32::MAX as f64);

            let clamped : [__m128d; 2] = [
                Simd::<f64, 2>::from(a[0]).simd_clamp::<{BackendType::SSE}>(min, max).into(),
                Simd::<f64, 2>::from(a[1]).simd_clamp::<{BackendType::SSE}>(min, max).into()
            ];
            Self::convert(clamped.into())
        }
    }
}

impl SimdConvertImpl<f32, 16, {BackendType::SSE}> for Simd<f64, 8> {
    fn simd_convert_impl(self) -> Simd<f32, 16> {
        unsafe {
            let a : [__m128d; 4] = self.into();
            let res = [_mm_or_ps(_mm_cvtpd_ps(a[0]), _mm_castsi128_ps(_mm_srli_si128::<8>(_mm_castps_si128(_mm_cvtpd_ps(a[1]))))),
                       _mm_or_ps(_mm_cvtpd_ps(a[2]), _mm_castsi128_ps(_mm_srli_si128::<8>(_mm_castps_si128(_mm_cvtpd_ps(a[3]))))),
                       _mm_setzero_ps(),
                       _mm_setzero_ps()];
            res.into()
        }
    }

    fn simd_convert_saturate_impl(self) -> Simd<f32, 16> {
        unsafe {
            let min = Simd::<f64, 2>::splat(f32::MIN as f64);
            let max = Simd::<f64, 2>::splat(f32::MAX as f64);

            let unclamped = self.split_4();
            let clamped = [
                unclamped[0].simd_clamp::<{BackendType::SSE}>(min, max),
                unclamped[1].simd_clamp::<{BackendType::SSE}>(min, max),
                unclamped[2].simd_clamp::<{BackendType::SSE}>(min, max),
                unclamped[3].simd_clamp::<{BackendType::SSE}>(min, max),
            ];
            Self::convert(clamped.into())
        }
    }
}

//==============================================================================================================================

impl SimdConvertImpl<i32, 4, {BackendType::SSE}> for Simd<f32, 4> {
    fn simd_convert_impl(self) -> Simd<i32, 4> {
        unsafe { _mm_cvtps_epi32(self.into()).into() }
    }

    fn simd_convert_saturate_impl(self) -> Simd<i32, 4> {
        unsafe {
            let min = Simd::<f32, 4>::splat(i32::MIN as f32);
            let max = Simd::<f32, 4>::splat(i32::MAX as f32);

            Self::convert(self.simd_clamp::<{BackendType::SSE}>(min, max))
        }
    }
}

impl SimdConvertImpl<i32, 8, {BackendType::SSE}> for Simd<f32, 8> {
    fn simd_convert_impl(self) -> Simd<i32, 8> {
        unsafe { 
            let a : [__m128; 2] = self.into();
            let res = [_mm_cvtps_epi32(a[0]),
                       _mm_cvtps_epi32(a[1])];
            res.into()
        }
    }

    fn simd_convert_saturate_impl(self) -> Simd<i32, 8> {
        unsafe {
            let a : [__m128; 2] = self.into();
            let min = Simd::<f32, 4>::splat(i32::MIN as f32);
            let max = Simd::<f32, 4>::splat(i32::MAX as f32);

            let clamped : [__m128; 2] = [
                Simd::<f32, 4>::from(a[0]).simd_clamp::<{BackendType::SSE}>(min, max).into(),
                Simd::<f32, 4>::from(a[1]).simd_clamp::<{BackendType::SSE}>(min, max).into()
            ];
            Self::convert(clamped.into())
        }
    }
}

impl SimdConvertImpl<i32, 16, {BackendType::SSE}> for Simd<f32, 16> {
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
        unsafe {
            let min = Simd::<f32, 4>::splat(i32::MIN as f32);
            let max = Simd::<f32, 4>::splat(i32::MAX as f32);

            let unclamped = self.split_4();
            let clamped = [
                unclamped[0].simd_clamp::<{BackendType::SSE}>(min, max),
                unclamped[1].simd_clamp::<{BackendType::SSE}>(min, max),
                unclamped[2].simd_clamp::<{BackendType::SSE}>(min, max),
                unclamped[3].simd_clamp::<{BackendType::SSE}>(min, max),
            ];
            Self::convert(clamped.into())
        }
    }
}

//==============================================================================================================================

impl SimdConvertImpl<i32, 4, {BackendType::SSE}> for Simd<f64, 2> {
    fn simd_convert_impl(self) -> Simd<i32, 4> {
        unsafe { _mm_cvtpd_epi32(self.into()).into() }
    }

    fn simd_convert_saturate_impl(self) -> Simd<i32, 4> {
        unsafe {
            let min = Simd::<f64, 2>::splat(u64::MIN as f64);
            let max = Simd::<f64, 2>::splat(u64::MAX as f64);
            Self::convert(self.simd_clamp::<{BackendType::SSE}>(min, max))
        }
    }
}

impl SimdConvertImpl<i32, 8, {BackendType::SSE}> for Simd<f64, 4> {
    fn simd_convert_impl(self) -> Simd<i32, 8> {
        unsafe { 
            let a : [__m128d; 2] = self.into();
            let res = [_mm_cvtpd_epi32(a[0]),
                       _mm_cvtpd_epi32(a[1])];
            res.into()
        }
    }

    fn simd_convert_saturate_impl(self) -> Simd<i32, 8> {
        unsafe {
            let a : [__m128d; 2] = self.into();
            let min = Simd::<f64, 2>::splat(i32::MIN as f64);
            let max = Simd::<f64, 2>::splat(i32::MAX as f64);

            let clamped : [__m128d; 2] = [
                Simd::<f64, 2>::from(a[0]).simd_clamp::<{BackendType::SSE}>(min, max).into(),
                Simd::<f64, 2>::from(a[1]).simd_clamp::<{BackendType::SSE}>(min, max).into()
            ];
            Self::convert(clamped.into())
        }
    }
}

impl SimdConvertImpl<i32, 16, {BackendType::SSE}> for Simd<f64, 8> {
    fn simd_convert_impl(self) -> Simd<i32, 16> {
        unsafe { 
            let a : [__m128d; 4] = self.into();
            let res = [_mm_cvtpd_epi32(a[0]),
                       _mm_cvtpd_epi32(a[1]),
                       _mm_cvtpd_epi32(a[2]),
                       _mm_cvtpd_epi32(a[3])];
            res.into()
        }
    }

    fn simd_convert_saturate_impl(self) -> Simd<i32, 16> {
        unsafe {
            let min = Simd::<f64, 2>::splat(i32::MIN as f64);
            let max = Simd::<f64, 2>::splat(i32::MAX as f64);

            let unclamped = self.split_4();
            let clamped = [
                unclamped[0].simd_clamp::<{BackendType::SSE}>(min, max),
                unclamped[1].simd_clamp::<{BackendType::SSE}>(min, max),
                unclamped[2].simd_clamp::<{BackendType::SSE}>(min, max),
                unclamped[3].simd_clamp::<{BackendType::SSE}>(min, max),
            ];
            Self::convert(clamped.into())
        }
    }
}

//==============================================================================================================================

impl SimdConvertImpl<i64, 2, {BackendType::SSE}> for Simd<f64, 2> {
    // https://stackoverflow.com/questions/41144668/how-to-efficiently-perform-double-int64-conversions-with-sse-avx
    // FIXME(jel): Only for [-2^51, 2^51]
    fn simd_convert_impl(self) -> Simd<i64, 2> {
        unsafe {
            let a : __m128d = self.into();
            let cnst = _mm_set1_pd(0x0018000000000000u64 as f64);        
            cvt_f64_i64(a, cnst).into()
        }
    }

    fn simd_convert_saturate_impl(self) -> Simd<i64, 2> {
        unsafe {
            let min = Simd::<f64, 2>::splat(u64::MIN as f64);
            let max = Simd::<f64, 2>::splat(u64::MAX as f64);
            Self::convert(self.simd_clamp::<{BackendType::SSE}>(min, max))
        }
    }
}

impl SimdConvertImpl<i64, 4, {BackendType::SSE}> for Simd<f64, 4> {
    // https://stackoverflow.com/questions/41144668/how-to-efficiently-perform-double-int64-conversions-with-sse-avx
    // FIXME(jel): Only for [-2^51, 2^51]
    fn simd_convert_impl(self) -> Simd<i64, 4> {
        unsafe {
            let a : [__m128d; 2] = self.into();
            let cnst = _mm_set1_pd(0x0018000000000000u64 as f64);

            let res = [cvt_f64_i64(a[0], cnst),
                       cvt_f64_i64(a[1], cnst)];            
            res.into()
        }
    }

    fn simd_convert_saturate_impl(self) -> Simd<i64, 4> {
        unsafe {
            let a : [__m128d; 2] = self.into();
            let min = Simd::<f64, 2>::splat(i64::MIN as f64);
            let max = Simd::<f64, 2>::splat(i64::MAX as f64);

            let clamped : [__m128d; 2] = [
                Simd::<f64, 2>::from(a[0]).simd_clamp::<{BackendType::SSE}>(min, max).into(),
                Simd::<f64, 2>::from(a[1]).simd_clamp::<{BackendType::SSE}>(min, max).into()
            ];
            Self::convert(clamped.into())
        }
    }
}

impl SimdConvertImpl<i64, 8, {BackendType::SSE}> for Simd<f64, 8> {
    fn simd_convert_impl(self) -> Simd<i64, 8> {
        unsafe { 
            let a : [__m128d; 4] = self.into();
            let cnst = _mm_set1_pd(0x0018000000000000u64 as f64);

            let res = [cvt_f64_i64(a[0], cnst),
                       cvt_f64_i64(a[1], cnst),
                       cvt_f64_i64(a[2], cnst),
                       cvt_f64_i64(a[3], cnst)];            
            res.into()
        }
    }

    fn simd_convert_saturate_impl(self) -> Simd<i64, 8> {
        unsafe {
            let min = Simd::<f64, 2>::splat(i64::MIN as f64);
            let max = Simd::<f64, 2>::splat(i64::MAX as f64);

            let unclamped = self.split_4();
            let clamped = [
                unclamped[0].simd_clamp::<{BackendType::SSE}>(min, max),
                unclamped[1].simd_clamp::<{BackendType::SSE}>(min, max),
                unclamped[2].simd_clamp::<{BackendType::SSE}>(min, max),
                unclamped[3].simd_clamp::<{BackendType::SSE}>(min, max),
            ];
            Self::convert(clamped.into())
        }
    }
}

//==============================================================================================================================

impl SimdConvertImpl<u64, 2, {BackendType::SSE}> for Simd<f64, 2> {
    // https://stackoverflow.com/questions/41144668/how-to-efficiently-perform-double-int64-conversions-with-sse-avx
    fn simd_convert_impl(self) -> Simd<u64, 2> {
        unsafe {
            let a : __m128d = self.into();
            let cnst = _mm_set1_pd(0x0010000000000000u64 as f64);        
            cvt_f64_u64(a, cnst).into()
        }
    }

    fn simd_convert_saturate_impl(self) -> Simd<u64, 2> {
        unsafe {
            let min = Simd::<f64, 2>::splat(u64::MIN as f64);
            let max = Simd::<f64, 2>::splat(u64::MAX as f64);
            Self::convert(self.simd_clamp::<{BackendType::SSE}>(min, max))
        }
    }
}

impl SimdConvertImpl<u64, 4, {BackendType::SSE}> for Simd<f64, 4> {
    // https://stackoverflow.com/questions/41144668/how-to-efficiently-perform-double-int64-conversions-with-sse-avx
    fn simd_convert_impl(self) -> Simd<u64, 4> {
        unsafe {
            let a : [__m128d; 2] = self.into();
            let cnst = _mm_set1_pd(0x0010000000000000u64 as f64);

            let res = [cvt_f64_u64(a[0], cnst),
                       cvt_f64_u64(a[1], cnst)];            
            res.into()
        }
    }

    fn simd_convert_saturate_impl(self) -> Simd<u64, 4> {
        unsafe {
            let a : [__m128d; 2] = self.into();
            let min = Simd::<f64, 2>::splat(u64::MIN as f64);
            let max = Simd::<f64, 2>::splat(u64::MAX as f64);

            let clamped : [__m128d; 2] = [
                Simd::<f64, 2>::from(a[0]).simd_clamp::<{BackendType::SSE}>(min, max).into(),
                Simd::<f64, 2>::from(a[1]).simd_clamp::<{BackendType::SSE}>(min, max).into()
            ];
            Self::convert(Simd::<f64, 4>::from(clamped))
        }
    }
}

impl SimdConvertImpl<u64, 8, {BackendType::SSE}> for Simd<f64, 8> {
    fn simd_convert_impl(self) -> Simd<u64, 8> {
        unsafe { 
            let a : [__m128d; 4] = self.into();
            let cnst = _mm_set1_pd(0x0010000000000000u64 as f64);

            let res = [cvt_f64_u64(a[0], cnst),
                       cvt_f64_u64(a[1], cnst),
                       cvt_f64_u64(a[2], cnst),
                       cvt_f64_u64(a[3], cnst)];            
            res.into()
        }
    }

    fn simd_convert_saturate_impl(self) -> Simd<u64, 8> {
        unsafe {
            let min = Simd::<f64, 2>::splat(u64::MIN as f64);
            let max = Simd::<f64, 2>::splat(u64::MAX as f64);

            let unclamped = self.split_4();
            let clamped = [
                unclamped[0].simd_clamp::<{BackendType::SSE}>(min, max),
                unclamped[1].simd_clamp::<{BackendType::SSE}>(min, max),
                unclamped[2].simd_clamp::<{BackendType::SSE}>(min, max),
                unclamped[3].simd_clamp::<{BackendType::SSE}>(min, max),
            ];
            Self::convert(clamped.into())
        }
    }
}

//==============================================================================================================================

impl SimdConvertImpl<u32, 4, {BackendType::SSE}> for Simd<f32, 4> {
    // https://stackoverflow.com/questions/9157373/most-efficient-way-to-convert-vector-of-float-to-vector-of-uint32
    // Implements the algorith above, but does not include saturating the value
    fn simd_convert_impl(self) -> Simd<u32, 4> {
        unsafe {
            let a : __m128 = self.into();
            let two31 = _mm_set1_ps(0x0f800000 as f32);
            let zero = _mm_setzero_ps();

            cvt_f32_u32(a, two31, zero).into()
        }
    }

    fn simd_convert_saturate_impl(self) -> Simd<u32, 4> {
        unsafe {
            let min = Simd::<f32, 4>::splat(u32::MIN as f32);
            let max = Simd::<f32, 4>::splat(u32::MAX as f32);
            Self::convert(self.simd_clamp::<{BackendType::SSE}>(min, max))
        }
    }
}

impl SimdConvertImpl<u32, 8, {BackendType::SSE}> for Simd<f32, 8> {
    // https://stackoverflow.com/questions/9157373/most-efficient-way-to-convert-vector-of-float-to-vector-of-uint32
    // Implements the algorith above, but does not include saturating the value
    fn simd_convert_impl(self) -> Simd<u32, 8> {
        unsafe {
            let a : [__m128; 2] = self.into();
            let two31 = _mm_set1_ps(0x0f800000 as f32);
            let zero = _mm_setzero_ps();

            let res = [cvt_f32_u32(a[0], two31, zero),
                       cvt_f32_u32(a[1], two31, zero)];
            res.into()
        }
    }

    fn simd_convert_saturate_impl(self) -> Simd<u32, 8> {
        unsafe {
            let a : [__m128; 2] = self.into();
            let min = Simd::<f32, 4>::splat(u32::MIN as f32);
            let max = Simd::<f32, 4>::splat(u32::MAX as f32);

            let clamped : [__m128; 2] = [
                Simd::<f32, 4>::from(a[0]).simd_clamp::<{BackendType::SSE}>(min, max).into(),
                Simd::<f32, 4>::from(a[1]).simd_clamp::<{BackendType::SSE}>(min, max).into()
            ];
            Self::convert(Simd::<f32, 8>::from(clamped))
        }
    }
}

impl SimdConvertImpl<u32, 16, {BackendType::SSE}> for Simd<f32, 16> {
    // https://stackoverflow.com/questions/9157373/most-efficient-way-to-convert-vector-of-float-to-vector-of-uint32
    // Implements the algorith above, but does not include saturating the value
    fn simd_convert_impl(self) -> Simd<u32, 16> {
        unsafe {
            let a : [__m128; 4] = self.into();
            let two31 = _mm_set1_ps(0x0f800000 as f32);
            let zero = _mm_setzero_ps();

            let res = [cvt_f32_u32(a[0], two31, zero),
                       cvt_f32_u32(a[1], two31, zero),
                       cvt_f32_u32(a[2], two31, zero),
                       cvt_f32_u32(a[3], two31, zero)];

            res.into()
        }
    }

    fn simd_convert_saturate_impl(self) -> Simd<u32, 16> {
        unsafe {
            let min = Simd::<f32, 4>::splat(u32::MIN as f32);
            let max = Simd::<f32, 4>::splat(u32::MAX as f32);

            let unclamped = self.split_4();
            let clamped = [
                unclamped[0].simd_clamp::<{BackendType::SSE}>(min, max),
                unclamped[1].simd_clamp::<{BackendType::SSE}>(min, max),
                unclamped[2].simd_clamp::<{BackendType::SSE}>(min, max),
                unclamped[3].simd_clamp::<{BackendType::SSE}>(min, max),
            ];
            Self::convert(clamped.into())
        }
    }
}

//==============================================================================================================================

impl SimdConvertImpl<f32, 4, {BackendType::SSE}> for Simd<i32, 4> {
    fn simd_convert_impl(self) -> Simd<f32, 4> {
        unsafe { _mm_cvtepi32_ps(self.into()).into() }
    }
}

impl SimdConvertImpl<f32, 8, {BackendType::SSE}> for Simd<i32, 8> {
    fn simd_convert_impl(self) -> Simd<f32, 8> {
        unsafe {
            let a : [__m128i; 2] = self.into();
            let res = [_mm_cvtepi32_ps(a[0]),
                       _mm_cvtepi32_ps(a[1])];
            res.into()
        }
    }
}

impl SimdConvertImpl<f32, 16, {BackendType::SSE}> for Simd<i32, 16> {
    fn simd_convert_impl(self) -> Simd<f32, 16> {
        unsafe {
            let a : [__m128i; 4] = self.into();
            let res = [_mm_cvtepi32_ps(a[0]),
                       _mm_cvtepi32_ps(a[1]),
                       _mm_cvtepi32_ps(a[2]),
                       _mm_cvtepi32_ps(a[3])];
            res.into()
        }
    }
}

//==============================================================================================================================

impl SimdConvertImpl<f64, 2, {BackendType::SSE}> for Simd<i32, 4> {
    fn simd_convert_impl(self) -> Simd<f64, 2> {
        unsafe { _mm_cvtepi32_pd(self.into()).into() }
    }
}

impl SimdConvertImpl<f64, 4, {BackendType::SSE}> for Simd<i32, 8> {
    fn simd_convert_impl(self) -> Simd<f64, 4> {
        unsafe {
            let a : [__m128i; 2] = self.into();
            let res = [_mm_cvtepi32_pd(a[0]),
                       _mm_cvtepi32_pd(a[1])];
            res.into()
        }
    }
}

impl SimdConvertImpl<f64, 8, {BackendType::SSE}> for Simd<i32, 16> {
    fn simd_convert_impl(self) -> Simd<f64, 8> {
        unsafe {
            let a : [__m128i; 4] = self.into();
            let res = [_mm_cvtepi32_pd(a[0]),
                       _mm_cvtepi32_pd(a[1]),
                       _mm_cvtepi32_pd(a[2]),
                       _mm_cvtepi32_pd(a[3])];
            res.into()
        }
    }
}

//==============================================================================================================================

impl SimdConvertImpl<f32, 4, {BackendType::SSE}> for Simd<u32, 4> {
    // https://stackoverflow.com/questions/9151711/most-efficient-way-to-convert-vector-of-uint32-to-vector-of-float
    fn simd_convert_impl(self) -> Simd<f32, 4> {
        unsafe { 
            let a : __m128i = self.into();
            let mask = _mm_set1_epi32(0x0000FFFF);
            let onep39 = _mm_set1_ps(0x53000000 as f32);

            cvt_u32_f32(a, mask, onep39).into()
        }
    }
}

impl SimdConvertImpl<f32, 8, {BackendType::SSE}> for Simd<u32, 8> {
    // https://stackoverflow.com/questions/9151711/most-efficient-way-to-convert-vector-of-uint32-to-vector-of-float
    fn simd_convert_impl(self) -> Simd<f32, 8> {
        unsafe {
            let a : [__m128i; 2] = self.into();
            let mask = _mm_set1_epi32(0x0000FFFF);
            let onep39 = _mm_set1_ps(0x53000000 as f32);

            let res = [cvt_u32_f32(a[0], mask, onep39),
                       cvt_u32_f32(a[1], mask, onep39)];
            res.into()
        }
    }
}

impl SimdConvertImpl<f32, 16, {BackendType::SSE}> for Simd<u32, 16> {
    // https://stackoverflow.com/questions/9151711/most-efficient-way-to-convert-vector-of-uint32-to-vector-of-float
    fn simd_convert_impl(self) -> Simd<f32, 16> {
        unsafe {
            let a : [__m128i; 4] = self.into();
            let mask = _mm_set1_epi32(0x0000FFFF);
            let onep39 = _mm_set1_ps(0x53000000 as f32);

            let res = [cvt_u32_f32(a[0], mask, onep39),
                       cvt_u32_f32(a[1], mask, onep39),
                       cvt_u32_f32(a[2], mask, onep39),
                       cvt_u32_f32(a[3], mask, onep39)];
            res.into()
        }
    }
}

//==============================================================================================================================

impl SimdConvertImpl<f64, 2, {BackendType::SSE}> for Simd<i64, 2> {
    // https://stackoverflow.com/questions/41144668/how-to-efficiently-perform-double-int64-conversions-with-sse-avx
    fn simd_convert_impl(self) -> Simd<f64, 2> {
        unsafe {
            let a : __m128i = self.into();
            let cnst0 = _mm_castpd_si128(_mm_set1_pd(442721857769029238784.0f64));
            let cnst1 = _mm_castpd_si128(_mm_set1_pd(0x0010000000000000u64 as f64));
            let cnst2 = _mm_set1_pd(442726361368656609280.0f64);

            cvt_i64_f64(a, cnst0, cnst1, cnst2).into()
        }
    }
}

impl SimdConvertImpl<f64, 4, {BackendType::SSE}> for Simd<i64, 4> {
    // https://stackoverflow.com/questions/41144668/how-to-efficiently-perform-double-int64-conversions-with-sse-avx
    fn simd_convert_impl(self) -> Simd<f64, 4> {
        unsafe {
            let a : [__m128i; 2] = self.into();
            let cnst0 = _mm_castpd_si128(_mm_set1_pd(442721857769029238784.0f64));
            let cnst1 = _mm_castpd_si128(_mm_set1_pd(0x0010000000000000u64 as f64));
            let cnst2 = _mm_set1_pd(442726361368656609280.0f64);

            let res = [cvt_i64_f64(a[0], cnst0, cnst1, cnst2),
                       cvt_i64_f64(a[1], cnst0, cnst1, cnst2)];
            res.into()
        }
    }
}

impl SimdConvertImpl<f64, 8, {BackendType::SSE}> for Simd<i64, 8> {
    // https://stackoverflow.com/questions/41144668/how-to-efficiently-perform-double-int64-conversions-with-sse-avx
    fn simd_convert_impl(self) -> Simd<f64, 8> {
        unsafe {
            let a : [__m128i; 4] = self.into();
            let cnst0 = _mm_castpd_si128(_mm_set1_pd(442721857769029238784.0f64));
            let cnst1 = _mm_castpd_si128(_mm_set1_pd(0x0010000000000000u64 as f64));
            let cnst2 = _mm_set1_pd(442726361368656609280.0f64);

            let res = [cvt_i64_f64(a[0], cnst0, cnst1, cnst2),
                       cvt_i64_f64(a[1], cnst0, cnst1, cnst2),
                       cvt_i64_f64(a[2], cnst0, cnst1, cnst2),
                       cvt_i64_f64(a[3], cnst0, cnst1, cnst2)];
            res.into()
        }
    }
}

//==============================================================================================================================

impl SimdConvertImpl<f64, 2, {BackendType::SSE}> for Simd<u64, 2> {
    // https://stackoverflow.com/questions/41144668/how-to-efficiently-perform-double-int64-conversions-with-sse-avx
    fn simd_convert_impl(self) -> Simd<f64, 2> {
        unsafe { 
            let a : __m128i = self.into();
            let cnst0 = _mm_castpd_si128(_mm_set1_pd(19342813113834066795298816.0f64));
            let cnst1 = _mm_castpd_si128(_mm_set1_pd(0x0010000000000000u64 as f64));
            let cnst2 = _mm_set1_pd(19342813118337666422669312.0f64);

            cvt_u64_f64(a, cnst0, cnst1, cnst2).into()
        }
    }
}

impl SimdConvertImpl<f64, 4, {BackendType::SSE}> for Simd<u64, 4> {
    // https://stackoverflow.com/questions/41144668/how-to-efficiently-perform-double-int64-conversions-with-sse-avx
    fn simd_convert_impl(self) -> Simd<f64, 4> {
        unsafe { 
            let a : [__m128i; 2] = self.into();
            let cnst0 = _mm_castpd_si128(_mm_set1_pd(19342813113834066795298816.0f64));
            let cnst1 = _mm_castpd_si128(_mm_set1_pd(0x0010000000000000u64 as f64));
            let cnst2 = _mm_set1_pd(19342813118337666422669312.0f64);

            let res = [cvt_u64_f64(a[0], cnst0, cnst1, cnst2),
                       cvt_u64_f64(a[1], cnst0, cnst1, cnst2)];
            res.into()
        }
    }
}

impl SimdConvertImpl<f64, 8, {BackendType::SSE}> for Simd<u64, 8> {
    // https://stackoverflow.com/questions/41144668/how-to-efficiently-perform-double-int64-conversions-with-sse-avx
    fn simd_convert_impl(self) -> Simd<f64, 8> {
        unsafe { 
            let a : [__m128i; 4] = self.into();
            let cnst0 = _mm_castpd_si128(_mm_set1_pd(19342813113834066795298816.0f64));
            let cnst1 = _mm_castpd_si128(_mm_set1_pd(0x0010000000000000u64 as f64));
            let cnst2 = _mm_set1_pd(19342813118337666422669312.0f64);

            let res = [cvt_u64_f64(a[0], cnst0, cnst1, cnst2),
                       cvt_u64_f64(a[1], cnst0, cnst1, cnst2),
                       cvt_u64_f64(a[2], cnst0, cnst1, cnst2),
                       cvt_u64_f64(a[3], cnst0, cnst1, cnst2)];
            res.into()
        }
    }
}

//==============================================================================================================================

macro_rules! impl_2_step_cvt {
    { $([$from_ty:ty, $f_lanes:literal => $imm_ty:ty, $imm_lanes:literal => $to_ty:ty, $t_lanes:literal])* } => {
        $(
            impl SimdConvertImpl<$to_ty, $t_lanes, {BackendType::SSE}> for Simd<$from_ty, $f_lanes> 
                where Self                      : SimdConvertImpl<$imm_ty, $imm_lanes, {BackendType::SSE}>,
                      Simd<$imm_ty, $imm_lanes> : SimdConvertImpl<$to_ty, $t_lanes, {BackendType::SSE}>
            {
                fn simd_convert_impl(self) -> Simd<$to_ty, $t_lanes> {
                    self.simd_convert::<$imm_ty, $imm_lanes, {BackendType::SSE}>().simd_convert::<$to_ty, $t_lanes, {BackendType::SSE}>()
                }
            
                fn simd_convert_saturate_impl(self) -> Simd<$to_ty, $t_lanes> {
                    self.simd_convert_saturate::<$imm_ty, $imm_lanes, {BackendType::SSE}>().simd_convert_saturate::<$to_ty, $t_lanes, {BackendType::SSE}>()
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
unsafe fn cvt_f64_i64(val: __m128d, cnst: __m128d) -> __m128i {
    let imm = _mm_add_pd(val, cnst);
    _mm_sub_epi64(_mm_castpd_si128(imm), _mm_castpd_si128(cnst)) 
}

// https://stackoverflow.com/questions/41144668/how-to-efficiently-perform-double-int64-conversions-with-sse-avx
// FIXME(jel): Only for [-2^51, 2^51]
#[inline]
unsafe fn cvt_f64_u64(val: __m128d, cnst: __m128d) -> __m128i {
    let imm = _mm_add_pd(val, cnst);
    _mm_xor_epi64(_mm_castpd_si128(imm), _mm_castpd_si128(cnst)) 
}

// https://stackoverflow.com/questions/9157373/most-efficient-way-to-convert-vector-of-float-to-vector-of-uint32
// Implements the algorith above, but does not include saturating the value
#[inline]
unsafe fn cvt_f32_u32(val: __m128, two31: __m128, zero: __m128) -> __m128i {
    // check for overflow before conversion to int
    let overflow = _mm_cmpge_ps(val, two31);
    let sub_val = _mm_and_ps(overflow, two31);
    let add_val = _mm_slli_epi32::<32>(_mm_castps_si128(overflow));

    // bias the value to signed space if it's >= 2^31
    let imm = _mm_sub_ps(val, sub_val);

    // convert to int, and unbias
    // rounding mode should be rount to nearest
    _mm_add_epi32(_mm_cvtps_epi32(imm), add_val)
}

// https://stackoverflow.com/questions/9151711/most-efficient-way-to-convert-vector-of-uint32-to-vector-of-float
#[inline]
unsafe fn cvt_u32_f32(val: __m128i, mask: __m128i, onep39: __m128) -> __m128 {
    let hi = _mm_srli_si128::<16>(val);
    let lo = _mm_and_si128(val, mask);
    let f_hi = _mm_sub_ps(_mm_or_ps(_mm_castsi128_ps(hi), onep39), onep39);
    let f_lo = _mm_cvtepi32_ps(lo);
    _mm_add_ps(f_hi, f_lo)
}

// https://stackoverflow.com/questions/41144668/how-to-efficiently-perform-double-int64-conversions-with-sse-avx
#[inline]
unsafe fn cvt_i64_f64(val: __m128i, cnst0: __m128i, cnst1: __m128i, cnst2: __m128d) -> __m128d {
    let hi = _mm_add_epi64(_mm_blend_epi16(_mm_srai_epi32::<16>(val), _mm_setzero_si128(), 0x33), cnst0);
    let lo = _mm_blend_epi16(val, cnst1, 0x88);
    let f = _mm_sub_pd(_mm_castsi128_pd(hi), cnst2);
    _mm_add_pd(f, _mm_castsi128_pd(lo))
}

// https://stackoverflow.com/questions/41144668/how-to-efficiently-perform-double-int64-conversions-with-sse-avx
#[inline]
unsafe fn cvt_u64_f64(val: __m128i, cnst0: __m128i, cnst1: __m128i, cnst2: __m128d) -> __m128d {
    let hi = _mm_or_si128(_mm_srli_epi64::<32>(val), cnst0);
    let lo = _mm_blend_epi16(val, cnst1, 0xCC);
    let f = _mm_sub_pd(_mm_castsi128_pd(hi), cnst2);
    _mm_add_pd(f, _mm_castsi128_pd(lo))
}