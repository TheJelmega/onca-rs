
use core::arch::x86_64::*;

use super::*;
use crate::{
    *,
    backend::*,
};

macro_rules! impl_via_sse {
    ($([$ty:ty, $lanes128:literal, $lanes256:literal, $lanes512:literal])*) => {
        $(
            impl_via_sse!{ $ty, $lanes128 }
            impl_via_sse!{ $ty, $lanes256 }
            impl_via_sse!{ $ty, $lanes512 }
        )*
    };
    ($ty:ty, $lanes:literal) => {
        impl SimdCmpImpl<{BackendType::AVX}> for Simd<$ty, $lanes>
        {
            type MaskT = Mask<<$ty as SimdElement>::Mask, $lanes>;

            fn simd_eq_impl(&self, other: &Self) -> Self::MaskT {
                <Self as SimdCmpImpl<{BackendType::SSE}>>::simd_eq_impl(self, other)
            }
        
            fn simd_ne_impl(&self, other: &Self) -> Self::MaskT {
                <Self as SimdCmpImpl<{BackendType::SSE}>>::simd_ne_impl(self, other)
            }
            
            fn simd_lt_impl(&self, other: &Self) -> Self::MaskT {
                <Self as SimdCmpImpl<{BackendType::SSE}>>::simd_lt_impl(self, other)
            }

            fn simd_le_impl(&self, other: &Self) -> Self::MaskT {
                <Self as SimdCmpImpl<{BackendType::SSE}>>::simd_le_impl(self, other)
            }

            fn simd_gt_impl(&self, other: &Self) -> Self::MaskT {
                <Self as SimdCmpImpl<{BackendType::SSE}>>::simd_gt_impl(self, other)
            }

            fn simd_ge_impl(&self, other: &Self) -> Self::MaskT {
                <Self as SimdCmpImpl<{BackendType::SSE}>>::simd_ge_impl(self, other)
            }

            fn simd_max_impl(self, other: Self) -> Self {
                <Self as SimdCmpImpl<{BackendType::SSE}>>::simd_max_impl(self, other)
            }

            fn simd_min_impl(self, other: Self) -> Self {
                <Self as SimdCmpImpl<{BackendType::SSE}>>::simd_min_impl(self, other)
            }

            fn simd_clamp_impl(self, min: Self, max: Self) -> Self {
                <Self as SimdCmpImpl<{BackendType::SSE}>>::simd_clamp_impl(self, min, max)
            }
        }
    };
}
impl_via_sse!{
    [i8 , 16, 32, 64]
    [i16,  8, 16, 32]
    [i32,  4,  8, 16]
    [i64,  2,  4,  8]
    [u8 , 16, 32, 64]
    [u16,  8, 16, 32]
    [u32,  4,  8, 16]
    [u64,  2,  4,  8]
}

//==============================================================================================

impl SimdCmpImpl<{BackendType::AVX}> for f32x4 {
    type MaskT = mask32x4;

    fn simd_eq_impl(&self, other: &f32x4) -> mask32x4 {
        unsafe {
            let a : __m128 = (*self).into();
            let b : __m128 = (*other).into();
            let res = _mm_castps_si128(_mm_cmp_ps::<_CMP_EQ_OQ>(a, b));
            mask32x4::simd_from_int_unchecked::<{BackendType::AVX}>(res.into())
        }
    }

    fn simd_ne_impl(&self, other: &f32x4) -> mask32x4 {
        unsafe {
            let a : __m128 = (*self).into();
            let b : __m128 = (*other).into();
            let res = _mm_castps_si128(_mm_cmp_ps::<_CMP_NEQ_OQ>(a, b));
            mask32x4::simd_from_int_unchecked::<{BackendType::AVX}>(res.into())
        }
    }
    
    fn simd_lt_impl(&self, other: &f32x4) -> mask32x4 {
        unsafe {
            let a : __m128 = (*self).into();
            let b : __m128 = (*other).into();
            let res = _mm_castps_si128(_mm_cmp_ps::<_CMP_LT_OQ>(a, b));
            mask32x4::simd_from_int_unchecked::<{BackendType::AVX}>(res.into())
        }
    }

    fn simd_le_impl(&self, other: &f32x4) -> mask32x4 {
        unsafe {
            let a : __m128 = (*self).into();
            let b : __m128 = (*other).into();
            let res = _mm_castps_si128(_mm_cmp_ps::<_CMP_LE_OQ>(a, b));
            mask32x4::simd_from_int_unchecked::<{BackendType::AVX}>(res.into())
        }
    }

    fn simd_gt_impl(&self, other: &f32x4) -> mask32x4 {
        unsafe {
            let a : __m128 = (*self).into();
            let b : __m128 = (*other).into();
            let res = _mm_castps_si128(_mm_cmp_ps::<_CMP_GT_OQ>(a, b));
            mask32x4::simd_from_int_unchecked::<{BackendType::AVX}>(res.into())
        }
    }

    fn simd_ge_impl(&self, other: &f32x4) -> mask32x4 {
        unsafe {
            let a : __m128 = (*self).into();
            let b : __m128 = (*other).into();
            let res = _mm_castps_si128(_mm_cmp_ps::<_CMP_GE_OQ>(a, b));
            mask32x4::simd_from_int_unchecked::<{BackendType::AVX}>(res.into())
        }
    }

    fn simd_max_impl(self: f32x4, other: f32x4) -> f32x4 {
        unsafe {
            let a : __m128 = self.into();
            let b : __m128 = other.into();
            let res = _mm_max_ps(a, b);
            res.into()
        }
    }

    fn simd_min_impl(self: f32x4, other: f32x4) -> f32x4 {
        unsafe {
            let a : __m128 = self.into();
            let b : __m128 = other.into();
            let res = _mm_min_ps(a, b);
            res.into()
        }
    }

    fn simd_clamp_impl(self: f32x4, min: f32x4, max: f32x4) -> f32x4 {
        unsafe {
            let a  : __m128 = self.into();
            let mi : __m128 = min.into();
            let ma : __m128 = max.into();
            let res = _mm_max_ps(_mm_min_ps(a, ma), mi);
            res.into()
        }
    }
}

impl SimdCmpImpl<{BackendType::AVX}> for f32x8 {
    type MaskT = mask32x8;

    fn simd_eq_impl(&self, other: &f32x8) -> mask32x8 {
        unsafe {
            let a : __m256 = (*self).into();
            let b : __m256 = (*other).into();
            let res = _mm256_castps_si256(_mm256_cmp_ps::<_CMP_EQ_OQ>(a, b));
            mask32x8::simd_from_int_unchecked::<{BackendType::AVX}>(res.into())
        }
    }

    fn simd_ne_impl(&self, other: &f32x8) -> mask32x8 {
        unsafe {
            let a : __m256 = (*self).into();
            let b : __m256 = (*other).into();
            let res = _mm256_castps_si256(_mm256_cmp_ps::<_CMP_NEQ_OQ>(a, b));
            mask32x8::simd_from_int_unchecked::<{BackendType::AVX}>(res.into())
        }
    }
    
    fn simd_lt_impl(&self, other: &f32x8) -> mask32x8 {
        unsafe {
            let a : __m256 = (*self).into();
            let b : __m256 = (*other).into();
            let res = _mm256_castps_si256(_mm256_cmp_ps::<_CMP_LT_OQ>(a, b));
            mask32x8::simd_from_int_unchecked::<{BackendType::AVX}>(res.into())
        }
    }

    fn simd_le_impl(&self, other: &f32x8) -> mask32x8 {
        unsafe {
            let a : __m256 = (*self).into();
            let b : __m256 = (*other).into();
            let res = _mm256_castps_si256(_mm256_cmp_ps::<_CMP_LE_OQ>(a, b));
            mask32x8::simd_from_int_unchecked::<{BackendType::AVX}>(res.into())
        }
    }

    fn simd_gt_impl(&self, other: &f32x8) -> mask32x8 {
        unsafe {
            let a : __m256 = (*self).into();
            let b : __m256 = (*other).into();
            let res = _mm256_castps_si256(_mm256_cmp_ps::<_CMP_GT_OQ>(a, b));
            mask32x8::simd_from_int_unchecked::<{BackendType::AVX}>(res.into())
        }
    }

    fn simd_ge_impl(&self, other: &f32x8) -> mask32x8 {
        unsafe {
            let a : __m256 = (*self).into();
            let b : __m256 = (*other).into();
            let res = _mm256_castps_si256(_mm256_cmp_ps::<_CMP_GE_OQ>(a, b));
            mask32x8::simd_from_int_unchecked::<{BackendType::AVX}>(res.into())
        }
    }

    fn simd_max_impl(self: f32x8, other: f32x8) -> f32x8 {
        unsafe {
            let a : __m256 = self.into();
            let b : __m256 = other.into();
            _mm256_max_ps(a, b).into()
        }
    }

    fn simd_min_impl(self: f32x8, other: f32x8) -> f32x8 {
        unsafe {
            let a : __m256 = self.into();
            let b : __m256 = other.into();
            _mm256_min_ps(a, b).into()
        }
    }

    fn simd_clamp_impl(self: f32x8, min: f32x8, max: f32x8) -> f32x8 {
        unsafe {
            let a  : __m256 = self.into();
            let mi : __m256 = min.into();
            let ma : __m256 = max.into();
            _mm256_max_ps(_mm256_min_ps(a, ma), mi).into()
        }
    }
}

impl SimdCmpImpl<{BackendType::AVX}> for f32x16 {
    type MaskT = mask32x16;

    fn simd_eq_impl(&self, other: &f32x16) -> mask32x16 {
        unsafe {
            let a : [__m256; 2] = (*self).into();
            let b : [__m256; 2] = (*other).into();
            let res = [_mm256_castps_si256(_mm256_cmp_ps::<_CMP_EQ_OQ>(a[0], b[0])), 
                       _mm256_castps_si256(_mm256_cmp_ps::<_CMP_EQ_OQ>(a[1], b[1]))];
            mask32x16::simd_from_int_unchecked::<{BackendType::AVX}>(res.into())
        }
    }

    fn simd_ne_impl(&self, other: &f32x16) -> mask32x16 {
        unsafe {
            let a : [__m256; 2] = (*self).into();
            let b : [__m256; 2] = (*other).into();
            let res = [_mm256_castps_si256(_mm256_cmp_ps::<_CMP_NEQ_OQ>(a[0], b[0])), 
                       _mm256_castps_si256(_mm256_cmp_ps::<_CMP_NEQ_OQ>(a[1], b[1]))];
            mask32x16::simd_from_int_unchecked::<{BackendType::AVX}>(res.into())
        }
    }
    
    fn simd_lt_impl(&self, other: &f32x16) -> mask32x16 {
        unsafe {
            let a : [__m256; 2] = (*self).into();
            let b : [__m256; 2] = (*other).into();
            let res = [_mm256_castps_si256(_mm256_cmp_ps::<_CMP_LT_OQ>(a[0], b[0])), 
                       _mm256_castps_si256(_mm256_cmp_ps::<_CMP_LT_OQ>(a[1], b[1]))];
            mask32x16::simd_from_int_unchecked::<{BackendType::AVX}>(res.into())
        }
    }

    fn simd_le_impl(&self, other: &f32x16) -> mask32x16 {
        unsafe {
            let a : [__m256; 2] = (*self).into();
            let b : [__m256; 2] = (*other).into();
            let res = [_mm256_castps_si256(_mm256_cmp_ps::<_CMP_LE_OQ>(a[0], b[0])), 
                       _mm256_castps_si256(_mm256_cmp_ps::<_CMP_LE_OQ>(a[1], b[1]))];
            mask32x16::simd_from_int_unchecked::<{BackendType::AVX}>(res.into())
        }
    }

    fn simd_gt_impl(&self, other: &f32x16) -> mask32x16 {
        unsafe {
            let a : [__m256; 2] = (*self).into();
            let b : [__m256; 2] = (*other).into();
            let res = [_mm256_castps_si256(_mm256_cmp_ps::<_CMP_GT_OQ>(a[0], b[0])), 
                       _mm256_castps_si256(_mm256_cmp_ps::<_CMP_GT_OQ>(a[1], b[1]))];
            mask32x16::simd_from_int_unchecked::<{BackendType::AVX}>(res.into())
        }
    }

    fn simd_ge_impl(&self, other: &f32x16) -> mask32x16 {
        unsafe {
            let a : [__m256; 2] = (*self).into();
            let b : [__m256; 2] = (*other).into();
            let res = [_mm256_castps_si256(_mm256_cmp_ps::<_CMP_GE_OQ>(a[0], b[0])), 
                       _mm256_castps_si256(_mm256_cmp_ps::<_CMP_GE_OQ>(a[1], b[1]))];
            mask32x16::simd_from_int_unchecked::<{BackendType::AVX}>(res.into())
        }
    }
    
    fn simd_max_impl(self: f32x16, other: f32x16) -> f32x16 {
        unsafe {
            let a : [__m256; 2] = self.into();
            let b : [__m256; 2] = other.into();
            let res = [_mm256_max_ps(a[0], b[0]),
                       _mm256_max_ps(a[1], b[1])];
            res.into()
        }
    }

    fn simd_min_impl(self: f32x16, other: f32x16) -> f32x16 {
        unsafe {
            let a : [__m256; 2] = self.into();
            let b : [__m256; 2] = other.into();
            let res = [_mm256_min_ps(a[0], b[0]),
                       _mm256_min_ps(a[1], b[1])];
            res.into()
        }
    }

    fn simd_clamp_impl(self: f32x16, min: f32x16, max: f32x16) -> f32x16 {
        unsafe {
            let a  : [__m256; 2] = self.into();
            let mi : [__m256; 2] = min.into();
            let ma : [__m256; 2] = max.into();
            let res = [_mm256_max_ps(_mm256_min_ps(a[0], ma[0]), mi[0]),
                       _mm256_max_ps(_mm256_min_ps(a[1], ma[1]), mi[1])];
            res.into()
        }
    }
}

//==============================================================================================

impl SimdCmpImpl<{BackendType::AVX}> for f64x2 {
    type MaskT = mask64x2;

    fn simd_eq_impl(&self, other: &f64x2) -> mask64x2 {
        unsafe {
            let a : __m128d = (*self).into();
            let b : __m128d = (*other).into();
            let res = _mm_castpd_si128(_mm_cmp_pd::<_CMP_EQ_OQ>(a, b));
            mask64x2::simd_from_int_unchecked::<{BackendType::AVX}>(res.into())
        }
    }

    fn simd_ne_impl(&self, other: &f64x2) -> mask64x2 {
        unsafe {
            let a : __m128d = (*self).into();
            let b : __m128d = (*other).into();
            let res = _mm_castpd_si128(_mm_cmp_pd::<_CMP_NEQ_OQ>(a, b));
            mask64x2::simd_from_int_unchecked::<{BackendType::AVX}>(res.into())
        }
    }
    
    fn simd_lt_impl(&self, other: &f64x2) -> mask64x2 {
        unsafe {
            let a : __m128d = (*self).into();
            let b : __m128d = (*other).into();
            let res = _mm_castpd_si128(_mm_cmp_pd::<_CMP_LT_OQ>(a, b));
            mask64x2::simd_from_int_unchecked::<{BackendType::AVX}>(res.into())
        }
    }

    fn simd_le_impl(&self, other: &f64x2) -> mask64x2 {
        unsafe {
            let a : __m128d = (*self).into();
            let b : __m128d = (*other).into();
            let res = _mm_castpd_si128(_mm_cmp_pd::<_CMP_LE_OQ>(a, b));
            mask64x2::simd_from_int_unchecked::<{BackendType::AVX}>(res.into())
        }
    }

    fn simd_gt_impl(&self, other: &f64x2) -> mask64x2 {
        unsafe {
            let a : __m128d = (*self).into();
            let b : __m128d = (*other).into();
            let res = _mm_castpd_si128(_mm_cmp_pd::<_CMP_GT_OQ>(a, b));
            mask64x2::simd_from_int_unchecked::<{BackendType::AVX}>(res.into())
        }
    }

    fn simd_ge_impl(&self, other: &f64x2) -> mask64x2 {
        unsafe {
            let a : __m128d = (*self).into();
            let b : __m128d = (*other).into();
            let res = _mm_castpd_si128(_mm_cmp_pd::<_CMP_GE_OQ>(a, b));
            mask64x2::simd_from_int_unchecked::<{BackendType::AVX}>(res.into())
        }
    }

    fn simd_max_impl(self: f64x2, other: f64x2) -> f64x2 {
        unsafe {
            let a : __m128d = self.into();
            let b : __m128d = other.into();
            let res = _mm_max_pd(a, b);
            res.into()
        }
    }

    fn simd_min_impl(self: f64x2, other: f64x2) -> f64x2 {
        unsafe {
            let a : __m128d = self.into();
            let b : __m128d = other.into();
            let res = _mm_min_pd(a, b);
            res.into()
        }
    }

    fn simd_clamp_impl(self: f64x2, min: f64x2, max: f64x2) -> f64x2 {
        unsafe {
            let a  : __m128d = self.into();
            let mi : __m128d = min.into();
            let ma : __m128d = max.into();
            let res = _mm_max_pd(_mm_min_pd(a, ma), mi);
            res.into()
        }
    }
}

impl SimdCmpImpl<{BackendType::AVX}> for f64x4 {
    type MaskT = mask64x4;

    fn simd_eq_impl(&self, other: &f64x4) -> mask64x4 {
        unsafe {
            let a : __m256d = (*self).into();
            let b : __m256d = (*other).into();
            let res = _mm256_castpd_si256(_mm256_cmp_pd::<_CMP_EQ_OQ>(a, b));
            mask64x4::simd_from_int_unchecked::<{BackendType::AVX}>(res.into())
        }
    }

    fn simd_ne_impl(&self, other: &f64x4) -> mask64x4 {
        unsafe {
            let a : __m256d = (*self).into();
            let b : __m256d = (*other).into();
            let res = _mm256_castpd_si256(_mm256_cmp_pd::<_CMP_NEQ_OQ>(a, b));
            mask64x4::simd_from_int_unchecked::<{BackendType::AVX}>(res.into())
        }
    }
    
    fn simd_lt_impl(&self, other: &f64x4) -> mask64x4 {
        unsafe {
            let a : __m256d = (*self).into();
            let b : __m256d = (*other).into();
            let res = _mm256_castpd_si256(_mm256_cmp_pd::<_CMP_LT_OQ>(a, b));
            mask64x4::simd_from_int_unchecked::<{BackendType::AVX}>(res.into())
        }
    }

    fn simd_le_impl(&self, other: &f64x4) -> mask64x4 {
        unsafe {
            let a : __m256d = (*self).into();
            let b : __m256d = (*other).into();
            let res = _mm256_castpd_si256(_mm256_cmp_pd::<_CMP_LE_OQ>(a, b));
            mask64x4::simd_from_int_unchecked::<{BackendType::AVX}>(res.into())
        }
    }

    fn simd_gt_impl(&self, other: &f64x4) -> mask64x4 {
        unsafe {
            let a : __m256d = (*self).into();
            let b : __m256d = (*other).into();
            let res = _mm256_castpd_si256(_mm256_cmp_pd::<_CMP_GT_OQ>(a, b));
            mask64x4::simd_from_int_unchecked::<{BackendType::AVX}>(res.into())
        }
    }

    fn simd_ge_impl(&self, other: &f64x4) -> mask64x4 {
        unsafe {
            let a : __m256d = (*self).into();
            let b : __m256d = (*other).into();
            let res = _mm256_castpd_si256(_mm256_cmp_pd::<_CMP_GE_OQ>(a, b));
            mask64x4::simd_from_int_unchecked::<{BackendType::AVX}>(res.into())
        }
    }

    fn simd_max_impl(self: f64x4, other: f64x4) -> f64x4 {
        unsafe {
            let a : __m256d = self.into();
            let b : __m256d = other.into();
            _mm256_max_pd(a, b).into()
        }
    }

    fn simd_min_impl(self: f64x4, other: f64x4) -> f64x4 {
        unsafe {
            let a : __m256d = self.into();
            let b : __m256d = other.into();
            _mm256_min_pd(a, b).into()
        }
    }

    fn simd_clamp_impl(self: f64x4, min: f64x4, max: f64x4) -> f64x4 {
        unsafe {
            let a  : __m256d = self.into();
            let mi : __m256d = min.into();
            let ma : __m256d = max.into();
            _mm256_max_pd(_mm256_min_pd(a, ma), mi).into()
        }
    }
}

impl SimdCmpImpl<{BackendType::AVX}> for f64x8 {
    type MaskT = mask64x8;

    fn simd_eq_impl(&self, other: &f64x8) -> mask64x8 {
        unsafe {
            let a : [__m256d; 2] = (*self).into();
            let b : [__m256d; 2] = (*other).into();
            let res = [_mm256_castpd_si256(_mm256_cmp_pd::<_CMP_EQ_OQ>(a[0], b[0])), 
                       _mm256_castpd_si256(_mm256_cmp_pd::<_CMP_EQ_OQ>(a[1], b[1]))];
            mask64x8::simd_from_int_unchecked::<{BackendType::AVX}>(res.into())
        }
    }

    fn simd_ne_impl(&self, other: &f64x8) -> mask64x8 {
        unsafe {
            let a : [__m256d; 2] = (*self).into();
            let b : [__m256d; 2] = (*other).into();
            let res = [_mm256_castpd_si256(_mm256_cmp_pd::<_CMP_NEQ_OQ>(a[0], b[0])), 
                       _mm256_castpd_si256(_mm256_cmp_pd::<_CMP_NEQ_OQ>(a[1], b[1]))];
            mask64x8::simd_from_int_unchecked::<{BackendType::AVX}>(res.into())
        }
    }
    
    fn simd_lt_impl(&self, other: &f64x8) -> mask64x8 {
        unsafe {
            let a : [__m256d; 2] = (*self).into();
            let b : [__m256d; 2] = (*other).into();
            let res = [_mm256_castpd_si256(_mm256_cmp_pd::<_CMP_LT_OQ>(a[0], b[0])), 
                       _mm256_castpd_si256(_mm256_cmp_pd::<_CMP_LT_OQ>(a[1], b[1]))];
            mask64x8::simd_from_int_unchecked::<{BackendType::AVX}>(res.into())
        }
    }

    fn simd_le_impl(&self, other: &f64x8) -> mask64x8 {
        unsafe {
            let a : [__m256d; 2] = (*self).into();
            let b : [__m256d; 2] = (*other).into();
            let res = [_mm256_castpd_si256(_mm256_cmp_pd::<_CMP_LE_OQ>(a[0], b[0])), 
                       _mm256_castpd_si256(_mm256_cmp_pd::<_CMP_LE_OQ>(a[1], b[1]))];
            mask64x8::simd_from_int_unchecked::<{BackendType::AVX}>(res.into())
        }
    }

    fn simd_gt_impl(&self, other: &f64x8) -> mask64x8 {
        unsafe {
            let a : [__m256d; 2] = (*self).into();
            let b : [__m256d; 2] = (*other).into();
            let res = [_mm256_castpd_si256(_mm256_cmp_pd::<_CMP_GT_OQ>(a[0], b[0])), 
                       _mm256_castpd_si256(_mm256_cmp_pd::<_CMP_GT_OQ>(a[1], b[1]))];
            mask64x8::simd_from_int_unchecked::<{BackendType::AVX}>(res.into())
        }
    }

    fn simd_ge_impl(&self, other: &f64x8) -> mask64x8 {
        unsafe {
            let a : [__m256d; 2] = (*self).into();
            let b : [__m256d; 2] = (*other).into();
            let res = [_mm256_castpd_si256(_mm256_cmp_pd::<_CMP_GE_OQ>(a[0], b[0])), 
                       _mm256_castpd_si256(_mm256_cmp_pd::<_CMP_GE_OQ>(a[1], b[1]))];
            mask64x8::simd_from_int_unchecked::<{BackendType::AVX}>(res.into())
        }
    }

    fn simd_max_impl(self: f64x8, other: f64x8) -> f64x8 {
        unsafe {
            let a : [__m256d; 2] = self.into();
            let b : [__m256d; 2] = other.into();
            let res = [_mm256_max_pd(a[0], b[0]),
                       _mm256_max_pd(a[1], b[1])];
            res.into()
        }
    }

    fn simd_min_impl(self: f64x8, other: f64x8) -> f64x8 {
        unsafe {
            let a : [__m256d; 2] = self.into();
            let b : [__m256d; 2] = other.into();
            let res = [_mm256_min_pd(a[0], b[0]),
                       _mm256_min_pd(a[1], b[1])];
            res.into()
        }
    }

    fn simd_clamp_impl(self: f64x8, min: f64x8, max: f64x8) -> f64x8 {
        unsafe {
            let a  : [__m256d; 2] = self.into();
            let mi : [__m256d; 2] = min.into();
            let ma : [__m256d; 2] = max.into();
            let res = [_mm256_max_pd(_mm256_min_pd(a[0], ma[0]), mi[0]),
                       _mm256_max_pd(_mm256_min_pd(a[1], ma[1]), mi[1])];
            res.into()
        }
    }
}
