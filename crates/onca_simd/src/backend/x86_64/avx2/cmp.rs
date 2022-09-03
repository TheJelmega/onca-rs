
use core::arch::x86_64::*;

use super::*;
use crate::{
    *,
    backend::*,
};

macro_rules! impl_via_avx {
    ($([$ty:ty, $lanes128:literal, $lanes256:literal, $lanes512:literal])*) => {
        $(
            impl_via_avx!{ $ty, $lanes128 }
            impl_via_avx!{ $ty, $lanes256 }
            impl_via_avx!{ $ty, $lanes512 }
        )*
    };
    ($ty:ty, $lanes:literal) => {
        impl SimdCmpImpl<{BackendType::AVX2}> for Simd<$ty, $lanes>
        {
            type MaskT = Mask<<$ty as SimdElement>::Mask, $lanes>;

            fn simd_eq_impl(&self, other: &Self) -> Self::MaskT {
                <Self as SimdCmpImpl<{BackendType::AVX}>>::simd_eq_impl(self, other)
            }
        
            fn simd_ne_impl(&self, other: &Self) -> Self::MaskT {
                <Self as SimdCmpImpl<{BackendType::AVX}>>::simd_ne_impl(self, other)
            }
            
            fn simd_lt_impl(&self, other: &Self) -> Self::MaskT {
                <Self as SimdCmpImpl<{BackendType::AVX}>>::simd_lt_impl(self, other)
            }

            fn simd_le_impl(&self, other: &Self) -> Self::MaskT {
                <Self as SimdCmpImpl<{BackendType::AVX}>>::simd_le_impl(self, other)
            }

            fn simd_gt_impl(&self, other: &Self) -> Self::MaskT {
                <Self as SimdCmpImpl<{BackendType::AVX}>>::simd_gt_impl(self, other)
            }

            fn simd_ge_impl(&self, other: &Self) -> Self::MaskT {
                <Self as SimdCmpImpl<{BackendType::AVX}>>::simd_ge_impl(self, other)
            }

            fn simd_max_impl(self, other: Self) -> Self {
                <Self as SimdCmpImpl<{BackendType::AVX}>>::simd_max_impl(self, other)
            }

            fn simd_min_impl(self, other: Self) -> Self {
                <Self as SimdCmpImpl<{BackendType::AVX}>>::simd_min_impl(self, other)
            }

            fn simd_clamp_impl(self, min: Self, max: Self) -> Self {
                <Self as SimdCmpImpl<{BackendType::AVX}>>::simd_clamp_impl(self, min, max)
            }
        }
    };
}
impl_via_avx!{
    [f32,  4,  8, 16]
    [f64,  2,  4,  8]
}

macro_rules! impl_signed {
    {
        $elem:ty, $lanes128:literal, $lanes256:literal, $lanes512:literal,
        $u_ty:ty, $i_ty:ty,
        $eq:ident, $gt:ident, $set1:ident, $max:ident, $min:ident
    } => {
        impl SimdCmpImpl<{BackendType::AVX2}> for Simd<$elem, $lanes128>
        {
            type MaskT = Mask<<$elem as SimdElement>::Mask, $lanes128>;

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

        impl SimdCmpImpl<{BackendType::AVX2}> for Simd<$elem, $lanes256>
        {
            type MaskT = Mask<<$elem as SimdElement>::Mask, $lanes256>;

            fn simd_eq_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : __m256i = (*self).into();
                    let b : __m256i = (*other).into();
                    let res = $eq(a, b);
                    Mask::<<$elem as SimdElement>::Mask, $lanes256>::simd_from_int_unchecked::<{BackendType::AVX2}>(res.into())
                }
            }
        
            fn simd_ne_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : __m256i = (*self).into();
                    let b : __m256i = (*other).into();
                    let res = _mm256_xor_si256($eq(a, b), $set1(<$u_ty>::MAX as $i_ty));
                    Mask::<<$elem as SimdElement>::Mask, $lanes256>::simd_from_int_unchecked::<{BackendType::AVX2}>(res.into())
                }
            }
            
            fn simd_lt_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : __m256i = (*self).into();
                    let b : __m256i = (*other).into();
                    let res = $gt(b, a);
                    Mask::<<$elem as SimdElement>::Mask, $lanes256>::simd_from_int_unchecked::<{BackendType::AVX2}>(res.into())
                }
            }

            fn simd_le_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : __m256i = (*self).into();
                    let b : __m256i = (*other).into();
                    let res = _mm256_or_si256($eq(a, b), $gt(b, a));
                    Mask::<<$elem as SimdElement>::Mask, $lanes256>::simd_from_int_unchecked::<{BackendType::AVX2}>(res.into())
                }
            }

            fn simd_gt_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : __m256i = (*self).into();
                    let b : __m256i = (*other).into();
                    let res = $gt(a, b);
                    Mask::<<$elem as SimdElement>::Mask, $lanes256>::simd_from_int_unchecked::<{BackendType::AVX2}>(res.into())
                }
            }

            fn simd_ge_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : __m256i = (*self).into();
                    let b : __m256i = (*other).into();
                    let res = _mm256_or_si256($eq(a, b), $gt(a, b));
                    Mask::<<$elem as SimdElement>::Mask, $lanes256>::simd_from_int_unchecked::<{BackendType::AVX2}>(res.into())
                }
            }

            fn simd_max_impl(self, other: Self) -> Self {
                unsafe {
                    let a : __m256i = self.into();
                    let b : __m256i = other.into();
                    let res = $max(a, b);
                    res.into()
                }
            }

            fn simd_min_impl(self, other: Self) -> Self {
                unsafe {
                    let a : __m256i = self.into();
                    let b : __m256i = other.into();
                    let res = $min(a, b);
                    res.into()
                }
            }

            fn simd_clamp_impl(self, min: Self, max: Self) -> Self {
                unsafe {
                    let a  : __m256i = self.into();
                    let mi : __m256i = min.into();
                    let ma : __m256i = max.into();
                    let res = $max($min(a, mi), ma);
                    res.into()
                }
            }
        }

        impl SimdCmpImpl<{BackendType::AVX2}> for Simd<$elem, $lanes512>
        {
            type MaskT = Mask<<$elem as SimdElement>::Mask, $lanes512>;

            fn simd_eq_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : [__m256i; 2] = (*self).into();
                    let b : [__m256i; 2] = (*other).into();
                    let res = [$eq(a[0], b[0]), 
                               $eq(a[1], b[1])];
                    Mask::<<$elem as SimdElement>::Mask, $lanes512>::simd_from_int_unchecked::<{BackendType::AVX2}>(res.into())
                }
            }
        
            fn simd_ne_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : [__m256i; 2] = (*self).into();
                    let b : [__m256i; 2] = (*other).into();
                    let xor_mask = $set1(<$u_ty>::MAX as $i_ty);
                    let res = [_mm256_xor_si256($eq(a[0], b[0]), xor_mask), 
                               _mm256_xor_si256($eq(a[1], b[1]), xor_mask)];
                    Mask::<<$elem as SimdElement>::Mask, $lanes512>::simd_from_int_unchecked::<{BackendType::AVX2}>(res.into())
                }
            }
            
            fn simd_lt_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : [__m256i; 2] = (*self).into();
                    let b : [__m256i; 2] = (*other).into();
                    let res = [$gt(b[0], a[0]), 
                               $gt(b[1], a[1])];
                    Mask::<<$elem as SimdElement>::Mask, $lanes512>::simd_from_int_unchecked::<{BackendType::AVX2}>(res.into())
                }
            }

            fn simd_le_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : [__m256i; 2] = (*self).into();
                    let b : [__m256i; 2] = (*other).into();
                    let res = [_mm256_xor_si256($eq(a[0], b[0]), $gt(b[0], a[0])), 
                               _mm256_xor_si256($eq(a[1], b[1]), $gt(b[1], a[1]))];
                    Mask::<<$elem as SimdElement>::Mask, $lanes512>::simd_from_int_unchecked::<{BackendType::AVX2}>(res.into())
                }
            }

            fn simd_gt_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : [__m256i; 2] = (*self).into();
                    let b : [__m256i; 2] = (*other).into();
                    let res = [$gt(a[0], b[0]), 
                               $gt(a[1], b[1])];
                    Mask::<<$elem as SimdElement>::Mask, $lanes512>::simd_from_int_unchecked::<{BackendType::AVX2}>(res.into())
                }
            }

            fn simd_ge_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : [__m256i; 2] = (*self).into();
                    let b : [__m256i; 2] = (*other).into();
                    let res = [_mm256_xor_si256($eq(a[0], b[0]), $gt(a[0], b[0])), 
                               _mm256_xor_si256($eq(a[1], b[1]), $gt(a[1], b[1]))];
                    Mask::<<$elem as SimdElement>::Mask, $lanes512>::simd_from_int_unchecked::<{BackendType::AVX2}>(res.into())
                }
            }

            fn simd_max_impl(self, other: Self) -> Self {
                unsafe {
                    let a : [__m256i; 2] = self.into();
                    let b : [__m256i; 2] = other.into();
                    let res = [$max(a[0], b[0]),
                               $max(a[1], b[1])];
                    res.into()
                }
            }

            fn simd_min_impl(self, other: Self) -> Self {
                unsafe {
                    let a : [__m256i; 2] = self.into();
                    let b : [__m256i; 2] = other.into();
                    let res = [$min(a[0], b[0]),
                               $min(a[1], b[1])];
                    res.into()
                }
            }

            fn simd_clamp_impl(self, min: Self, max: Self) -> Self {
                unsafe {
                    let a  : [__m256i; 2] = self.into();
                    let mi : [__m256i; 2] = min.into();
                    let ma : [__m256i; 2] = max.into();
                    let res = [$max($min(a[0], mi[0]), ma[0]),
                               $max($min(a[1], mi[1]), ma[1])];
                    res.into()
                }
            }
        }
    };
}
impl_signed!{
    i8, 16, 32, 64,
    u8, i8,
    _mm256_cmpeq_epi8, _mm256_cmpgt_epi8, _mm256_set1_epi8, _mm256_max_epi8, _mm256_min_epi8
}
impl_signed!{
    i16, 8, 16, 32,
    u16, i16,
    _mm256_cmpeq_epi16, _mm256_cmpgt_epi16, _mm256_set1_epi16, _mm256_max_epi16, _mm256_min_epi16
}
impl_signed!{
    i32, 4, 8, 16,
    u32, i32,
    _mm256_cmpeq_epi32, _mm256_cmpgt_epi32, _mm256_set1_epi32, _mm256_max_epi32, _mm256_min_epi32
}
impl_signed!{
    i64, 2, 4, 8,
    u64, i64,
    _mm256_cmpeq_epi64, _mm256_cmpgt_epi64, _mm256_set1_epi64x, max_i64, min_i64
}

//==============================================================================================

macro_rules! impl_unsigned {
    {
        $elem:ty, $lanes128:literal, $lanes256:literal, $lanes512:literal,
        $u_ty:ty, $i_ty:ty,
        $eq:ident, $gt:ident, $set1:ident, $max:ident, $min:ident
    } => {

        impl SimdCmpImpl<{BackendType::AVX2}> for Simd<$elem, $lanes128>
        {
            type MaskT = Mask<<$elem as SimdElement>::Mask, $lanes128>;

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

        impl SimdCmpImpl<{BackendType::AVX2}> for Simd<$elem, $lanes256>
        {
            type MaskT = Mask<<$elem as SimdElement>::Mask, $lanes256>;

            fn simd_eq_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : __m256i = (*self).into();
                    let b : __m256i = (*other).into();
                    let sign_mask = $set1(<$i_ty>::MIN);
                    let res = $eq(_mm256_xor_si256(a, sign_mask), _mm256_xor_si256(b, sign_mask));
                    Mask::<<$elem as SimdElement>::Mask, $lanes256>::simd_from_int_unchecked::<{BackendType::AVX2}>(res.into())
                }
            }
        
            fn simd_ne_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : __m256i = (*self).into();
                    let b : __m256i = (*other).into();
                    let sign_mask = $set1(<$i_ty>::MIN);
                    let xor_mask = $set1(<$u_ty>::MAX as $i_ty);
                    let res = _mm256_xor_si256($eq(_mm256_xor_si256(a, sign_mask), _mm256_xor_si256(b, sign_mask)), xor_mask);
                    Mask::<<$elem as SimdElement>::Mask, $lanes256>::simd_from_int_unchecked::<{BackendType::AVX2}>(res.into())
                }
            }
            
            fn simd_lt_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : __m256i = (*self).into();
                    let b : __m256i = (*other).into();
                    let sign_mask = $set1(<$i_ty>::MIN);
                    let res = $gt(_mm256_xor_si256(b, sign_mask), _mm256_xor_si256(a, sign_mask));
                    Mask::<<$elem as SimdElement>::Mask, $lanes256>::simd_from_int_unchecked::<{BackendType::AVX2}>(res.into())
                }
            }

            fn simd_le_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let sign_mask = $set1(<$i_ty>::MIN);
                    let a : __m256i = _mm256_xor_si256((*self).into(), sign_mask);
                    let b : __m256i = _mm256_xor_si256((*other).into(), sign_mask);
                    let res = _mm256_or_si256($eq(a, b), $gt(b, a));
                    Mask::<<$elem as SimdElement>::Mask, $lanes256>::simd_from_int_unchecked::<{BackendType::AVX2}>(res.into())
                }
            }

            fn simd_gt_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : __m256i = (*self).into();
                    let b : __m256i = (*other).into();
                    let sign_mask = $set1(<$i_ty>::MIN);
                    let res = $gt(_mm256_xor_si256(a, sign_mask), _mm256_xor_si256(b, sign_mask));
                    Mask::<<$elem as SimdElement>::Mask, $lanes256>::simd_from_int_unchecked::<{BackendType::AVX2}>(res.into())
                }
            }

            fn simd_ge_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let sign_mask = $set1(<$i_ty>::MIN);
                    let a : __m256i = _mm256_xor_si256((*self).into(), sign_mask);
                    let b : __m256i = _mm256_xor_si256((*other).into(), sign_mask);
                    let res = _mm256_or_si256($eq(a, b), $gt(a, b));
                    Mask::<<$elem as SimdElement>::Mask, $lanes256>::simd_from_int_unchecked::<{BackendType::AVX2}>(res.into())
                }
            }

            fn simd_max_impl(self, other: Self) -> Self {
                unsafe {
                    let a : __m256i = self.into();
                    let b : __m256i = other.into();
                    let res = $max(a, b);
                    res.into()
                }
            }

            fn simd_min_impl(self, other: Self) -> Self {
                unsafe {
                    let a : __m256i = self.into();
                    let b : __m256i = other.into();
                    let res = $min(a, b);
                    res.into()
                }
            }

            fn simd_clamp_impl(self, min: Self, max: Self) -> Self {
                unsafe {
                    let a  : __m256i = self.into();
                    let mi : __m256i = min.into();
                    let ma : __m256i = max.into();
                    let res = $max($min(a, mi), ma);
                    res.into()
                }
            }
        }

        impl SimdCmpImpl<{BackendType::AVX2}> for Simd<$elem, $lanes512>
        {
            type MaskT = Mask<<$elem as SimdElement>::Mask, $lanes512>;

            fn simd_eq_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : [__m256i; 2] = (*self).into();
                    let b : [__m256i; 2] = (*other).into();
                    let sign_mask = $set1(<$i_ty>::MIN);
                    let res = [$eq(_mm256_xor_si256(a[0], sign_mask), _mm256_xor_si256(b[0], sign_mask)), 
                               $eq(_mm256_xor_si256(a[1], sign_mask), _mm256_xor_si256(b[1], sign_mask))];
                    Mask::<<$elem as SimdElement>::Mask, $lanes512>::simd_from_int_unchecked::<{BackendType::AVX2}>(res.into())
                }
            }
        
            fn simd_ne_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : [__m256i; 2] = (*self).into();
                    let b : [__m256i; 2] = (*other).into();
                    let sign_mask = $set1(<$i_ty>::MIN);
                    let xor_mask = $set1(<$u_ty>::MAX as $i_ty);
                    let res = [_mm256_xor_si256($eq(_mm256_xor_si256(a[0], sign_mask), _mm256_xor_si256(b[0], sign_mask)), xor_mask), 
                               _mm256_xor_si256($eq(_mm256_xor_si256(a[1], sign_mask), _mm256_xor_si256(b[1], sign_mask)), xor_mask)];
                    Mask::<<$elem as SimdElement>::Mask, $lanes512>::simd_from_int_unchecked::<{BackendType::AVX2}>(res.into())
                }
            }
            
            fn simd_lt_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : [__m256i; 2] = (*self).into();
                    let b : [__m256i; 2] = (*other).into();
                    let sign_mask = $set1(<$i_ty>::MIN);
                    let res = [$gt(_mm256_xor_si256(b[0], sign_mask), _mm256_xor_si256(a[0], sign_mask)), 
                               $gt(_mm256_xor_si256(b[1], sign_mask), _mm256_xor_si256(a[1], sign_mask))];
                    Mask::<<$elem as SimdElement>::Mask, $lanes512>::simd_from_int_unchecked::<{BackendType::AVX2}>(res.into())
                }
            }

            fn simd_le_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : [__m256i; 2] = (*self).into();
                    let b : [__m256i; 2] = (*other).into();
                    let sign_mask = $set1(<$i_ty>::MIN);

                    let a0 = _mm256_xor_si256(a[0], sign_mask);
                    let a1 = _mm256_xor_si256(a[1], sign_mask);
                    let b0 = _mm256_xor_si256(b[0], sign_mask);
                    let b1 = _mm256_xor_si256(b[1], sign_mask);

                    let res = [_mm256_or_si256($eq(a0, b0), $gt(b0, a0)), 
                               _mm256_or_si256($eq(a1, b1), $gt(b1, a1))];
                    Mask::<<$elem as SimdElement>::Mask, $lanes512>::simd_from_int_unchecked::<{BackendType::AVX2}>(res.into())
                }
            }

            fn simd_gt_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : [__m256i; 2] = (*self).into();
                    let b : [__m256i; 2] = (*other).into();
                    let sign_mask = $set1(<$i_ty>::MIN);
                    let res = [$gt(_mm256_xor_si256(a[0], sign_mask), _mm256_xor_si256(b[0], sign_mask)), 
                               $gt(_mm256_xor_si256(a[1], sign_mask), _mm256_xor_si256(b[1], sign_mask))];
                    Mask::<<$elem as SimdElement>::Mask, $lanes512>::simd_from_int_unchecked::<{BackendType::AVX2}>(res.into())
                }
            }

            fn simd_ge_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : [__m256i; 2] = (*self).into();
                    let b : [__m256i; 2] = (*other).into();
                    let sign_mask = $set1(<$i_ty>::MIN);

                    let a0 = _mm256_xor_si256(a[0], sign_mask);
                    let a1 = _mm256_xor_si256(a[1], sign_mask);
                    let b0 = _mm256_xor_si256(b[0], sign_mask);
                    let b1 = _mm256_xor_si256(b[1], sign_mask);

                    let res = [_mm256_or_si256($eq(a0, b0), $gt(a0, b0)), 
                               _mm256_or_si256($eq(a1, b1), $gt(a1, b1))];
                    Mask::<<$elem as SimdElement>::Mask, $lanes512>::simd_from_int_unchecked::<{BackendType::AVX2}>(res.into())
                }
            }

            fn simd_max_impl(self, other: Self) -> Self {
                unsafe {
                    let a : [__m256i; 2] = self.into();
                    let b : [__m256i; 2] = other.into();
                    let res = [$max(a[0], b[0]),
                               $max(a[1], b[1])];
                    res.into()
                }
            }

            fn simd_min_impl(self, other: Self) -> Self {
                unsafe {
                    let a : [__m256i; 2] = self.into();
                    let b : [__m256i; 2] = other.into();
                    let res = [$min(a[0], b[0]),
                               $min(a[1], b[1])];
                    res.into()
                }
            }

            fn simd_clamp_impl(self, min: Self, max: Self) -> Self {
                unsafe {
                    let a  : [__m256i; 2] = self.into();
                    let mi : [__m256i; 2] = min.into();
                    let ma : [__m256i; 2] = max.into();
                    let res = [$max($min(a[0], mi[0]), ma[0]),
                               $max($min(a[1], mi[1]), ma[1])];
                    res.into()
                }
            }
        }
    };
}
impl_unsigned!{
    u8, 16, 32, 64,
    u8, i8,
    _mm256_cmpeq_epi8, _mm256_cmpgt_epi8, _mm256_set1_epi8, _mm256_max_epu8, _mm256_min_epu8
}
impl_unsigned!{
    u16, 8, 16, 32,
    u16, i16,
    _mm256_cmpeq_epi16, _mm256_cmpgt_epi16, _mm256_set1_epi16, _mm256_max_epu16, _mm256_min_epu16
}
impl_unsigned!{
    u32, 4, 8, 16,
    u32, i32,
    _mm256_cmpeq_epi32, _mm256_cmpgt_epi32, _mm256_set1_epi32, _mm256_max_epu32, _mm256_min_epu32
}
impl_unsigned!{
    u64, 2, 4, 8,
    u64, i64,
    _mm256_cmpeq_epi64, _mm256_cmpgt_epi64, _mm256_set1_epi64x, max_u64, min_u64
}


//==============================================================================================================================
//  UTILITY
//==============================================================================================================================
#[inline(always)]
unsafe fn max_i64(a: __m256i, b: __m256i) -> __m256i {
    _mm256_blendv_epi8(a, b, _mm256_cmpgt_epi64(a, b))
}

#[inline(always)]
unsafe fn min_i64(a: __m256i, b: __m256i) -> __m256i {
    _mm256_blendv_epi8(a, b, _mm256_cmpgt_epi64(b, a))
}

#[inline(always)]
unsafe fn max_u64(a: __m256i, b: __m256i) -> __m256i {
    let sign_mask = _mm256_set1_epi64x(i64::MIN);
    let imm_a = _mm256_xor_si256(a, sign_mask);
    let imm_b = _mm256_xor_si256(b, sign_mask);
    _mm256_blendv_epi8(a, b, _mm256_cmpgt_epi64(imm_a, imm_b))
}

#[inline(always)]
unsafe fn min_u64(a: __m256i, b: __m256i) -> __m256i {
    let sign_mask = _mm256_set1_epi64x(i64::MIN);
    let imm_a = _mm256_xor_si256(a, sign_mask);
    let imm_b = _mm256_xor_si256(b, sign_mask);
    _mm256_blendv_epi8(a, b, _mm256_cmpgt_epi64(imm_b, imm_a))
}