
use core::arch::x86_64::*;

use super::*;
use crate::{
    *,
    backend::*,
};

macro_rules! impl_signed {
    {
        $elem:ty, $lanes128:literal, $lanes256:literal, $lanes512:literal,
        $u_ty:ty, $i_ty:ty,
        $eq:ident, $gt:ident, $set1:ident, $max:ident, $min:ident
    } => {
        impl SimdCmpImpl<{BackendType::SSE}> for Simd<$elem, $lanes128>
        {
            type MaskT = Mask<<$elem as SimdElement>::Mask, $lanes128>;

            fn simd_eq_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : __m128i = (*self).into();
                    let b : __m128i = (*other).into();
                    let res = $eq(a, b);
                    Mask::<<$elem as SimdElement>::Mask, $lanes128>::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
                }
            }
        
            fn simd_ne_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : __m128i = (*self).into();
                    let b : __m128i = (*other).into();
                    let res = _mm_xor_si128($eq(a, b), $set1(<$u_ty>::MAX as $i_ty));
                    Mask::<<$elem as SimdElement>::Mask, $lanes128>::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
                }
            }
            
            fn simd_lt_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : __m128i = (*self).into();
                    let b : __m128i = (*other).into();
                    let res = $gt(b, a);
                    Mask::<<$elem as SimdElement>::Mask, $lanes128>::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
                }
            }

            fn simd_le_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : __m128i = (*self).into();
                    let b : __m128i = (*other).into();
                    let res = _mm_or_si128($eq(a, b), $gt(b, a));
                    Mask::<<$elem as SimdElement>::Mask, $lanes128>::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
                }
            }

            fn simd_gt_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : __m128i = (*self).into();
                    let b : __m128i = (*other).into();
                    let res = $gt(a, b);
                    Mask::<<$elem as SimdElement>::Mask, $lanes128>::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
                }
            }

            fn simd_ge_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : __m128i = (*self).into();
                    let b : __m128i = (*other).into();
                    let res = _mm_or_si128($eq(a, b), $gt(a, b));
                    Mask::<<$elem as SimdElement>::Mask, $lanes128>::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
                }
            }

            fn simd_max_impl(self, other: Self) -> Self {
                unsafe {
                    let a : __m128i = self.into();
                    let b : __m128i = other.into();
                    let res = $max(a, b);
                    res.into()
                }
            }

            fn simd_min_impl(self, other: Self) -> Self {
                unsafe {
                    let a : __m128i = self.into();
                    let b : __m128i = other.into();
                    let res = $min(a, b);
                    res.into()
                }
            }

            fn simd_clamp_impl(self, min: Self, max: Self) -> Self {
                unsafe {
                    let a  : __m128i = self.into();
                    let mi : __m128i = min.into();
                    let ma : __m128i = max.into();
                    let res = $max($min(a, mi), ma);
                    res.into()
                }
            }
        }

        impl SimdCmpImpl<{BackendType::SSE}> for Simd<$elem, $lanes256>
        {
            type MaskT = Mask<<$elem as SimdElement>::Mask, $lanes256>;

            fn simd_eq_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : [__m128i; 2] = (*self).into();
                    let b : [__m128i; 2] = (*other).into();
                    let res = [$eq(a[0], b[0]), 
                               $eq(a[1], b[1])];
                    Mask::<<$elem as SimdElement>::Mask, $lanes256>::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
                }
            }
        
            fn simd_ne_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : [__m128i; 2] = (*self).into();
                    let b : [__m128i; 2] = (*other).into();
                    let xor_mask = $set1(<$u_ty>::MAX as $i_ty);
                    let res = [_mm_xor_si128($eq(a[0], b[0]), xor_mask), 
                               _mm_xor_si128($eq(a[1], b[1]), xor_mask)];
                    Mask::<<$elem as SimdElement>::Mask, $lanes256>::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
                }
            }
            
            fn simd_lt_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : [__m128i; 2] = (*self).into();
                    let b : [__m128i; 2] = (*other).into();
                    let res = [$gt(b[0], a[0]), 
                               $gt(b[1], a[1])];
                    Mask::<<$elem as SimdElement>::Mask, $lanes256>::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
                }
            }

            fn simd_le_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : [__m128i; 2] = (*self).into();
                    let b : [__m128i; 2] = (*other).into();
                    let res = [_mm_xor_si128($eq(a[0], b[0]), $gt(b[0], a[0])), 
                               _mm_xor_si128($eq(a[1], b[1]), $gt(b[1], a[1]))];
                    Mask::<<$elem as SimdElement>::Mask, $lanes256>::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
                }
            }

            fn simd_gt_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : [__m128i; 2] = (*self).into();
                    let b : [__m128i; 2] = (*other).into();
                    let res = [$gt(a[0], b[0]), 
                               $gt(a[1], b[1])];
                    Mask::<<$elem as SimdElement>::Mask, $lanes256>::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
                }
            }

            fn simd_ge_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : [__m128i; 2] = (*self).into();
                    let b : [__m128i; 2] = (*other).into();
                    let res = [_mm_xor_si128($eq(a[0], b[0]), $gt(a[0], b[0])), 
                               _mm_xor_si128($eq(a[1], b[1]), $gt(a[1], b[1]))];
                    Mask::<<$elem as SimdElement>::Mask, $lanes256>::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
                }
            }

            fn simd_max_impl(self, other: Self) -> Self {
                unsafe {
                    let a : [__m128i; 2] = self.into();
                    let b : [__m128i; 2] = other.into();
                    let res = [$max(a[0], b[0]),
                               $max(a[1], b[1])];
                    res.into()
                }
            }

            fn simd_min_impl(self, other: Self) -> Self {
                unsafe {
                    let a : [__m128i; 2] = self.into();
                    let b : [__m128i; 2] = other.into();
                    let res = [$min(a[0], b[0]),
                               $min(a[1], b[1])];
                    res.into()
                }
            }

            fn simd_clamp_impl(self, min: Self, max: Self) -> Self {
                unsafe {
                    let a  : [__m128i; 2] = self.into();
                    let mi : [__m128i; 2] = min.into();
                    let ma : [__m128i; 2] = max.into();
                    let res = [$max($min(a[0], mi[0]), ma[0]),
                               $max($min(a[1], mi[1]), ma[1])];
                    res.into()
                }
            }
        }

        impl SimdCmpImpl<{BackendType::SSE}> for Simd<$elem, $lanes512>
        {
            type MaskT = Mask<<$elem as SimdElement>::Mask, $lanes512>;

            fn simd_eq_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : [__m128i; 4] = (*self).into();
                    let b : [__m128i; 4] = (*other).into();
                    let res = [$eq(a[0], b[0]), 
                               $eq(a[1], b[1]),
                               $eq(a[2], b[2]),
                               $eq(a[3], b[3])];
                    Mask::<<$elem as SimdElement>::Mask, $lanes512>::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
                }
            }
        
            fn simd_ne_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : [__m128i; 4] = (*self).into();
                    let b : [__m128i; 4] = (*other).into();
                    let xor_mask = $set1(<$u_ty>::MAX as $i_ty);
                    let res = [_mm_xor_si128($eq(a[0], b[0]), xor_mask), 
                               _mm_xor_si128($eq(a[1], b[1]), xor_mask),
                               _mm_xor_si128($eq(a[2], b[2]), xor_mask),
                               _mm_xor_si128($eq(a[3], b[3]), xor_mask)];
                    Mask::<<$elem as SimdElement>::Mask, $lanes512>::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
                }
            }
            
            fn simd_lt_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : [__m128i; 4] = (*self).into();
                    let b : [__m128i; 4] = (*other).into();
                    let res = [$gt(b[0], a[0]), 
                               $gt(b[1], a[1]),
                               $gt(b[2], a[2]),
                               $gt(b[3], a[3])];
                    Mask::<<$elem as SimdElement>::Mask, $lanes512>::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
                }
            }

            fn simd_le_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : [__m128i; 4] = (*self).into();
                    let b : [__m128i; 4] = (*other).into();
                    let res = [_mm_xor_si128($eq(a[0], b[0]), $gt(b[0], a[0])), 
                               _mm_xor_si128($eq(a[1], b[1]), $gt(b[1], a[1])),
                               _mm_xor_si128($eq(a[2], b[2]), $gt(b[2], a[2])),
                               _mm_xor_si128($eq(a[3], b[3]), $gt(b[3], a[3]))];
                    Mask::<<$elem as SimdElement>::Mask, $lanes512>::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
                }
            }

            fn simd_gt_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : [__m128i; 4] = (*self).into();
                    let b : [__m128i; 4] = (*other).into();
                    let res = [$gt(a[0], b[0]), 
                               $gt(a[1], b[1]),
                               $gt(a[2], b[2]),
                               $gt(a[3], b[3])];
                    Mask::<<$elem as SimdElement>::Mask, $lanes512>::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
                }
            }

            fn simd_ge_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : [__m128i; 4] = (*self).into();
                    let b : [__m128i; 4] = (*other).into();
                    let res = [_mm_xor_si128($eq(a[0], b[0]), $gt(a[0], b[0])), 
                               _mm_xor_si128($eq(a[1], b[1]), $gt(a[1], b[1])),
                               _mm_xor_si128($eq(a[2], b[2]), $gt(a[2], b[2])),
                               _mm_xor_si128($eq(a[3], b[3]), $gt(a[3], b[3]))];
                    Mask::<<$elem as SimdElement>::Mask, $lanes512>::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
                }
            }

            fn simd_max_impl(self, other: Self) -> Self {
                unsafe {
                    let a : [__m128i; 4] = self.into();
                    let b : [__m128i; 4] = other.into();
                    let res = [$max(a[0], b[0]),
                               $max(a[1], b[1]),
                               $max(a[2], b[2]),
                               $max(a[3], b[3])];
                    res.into()
                }
            }

            fn simd_min_impl(self, other: Self) -> Self {
                unsafe {
                    let a : [__m128i; 4] = self.into();
                    let b : [__m128i; 4] = other.into();
                    let res = [$min(a[0], b[0]),
                               $min(a[1], b[1]),
                               $min(a[2], b[2]),
                               $min(a[3], b[3])];
                    res.into()
                }
            }

            fn simd_clamp_impl(self, min: Self, max: Self) -> Self {
                unsafe {
                    let a  : [__m128i; 4] = self.into();
                    let mi : [__m128i; 4] = min.into();
                    let ma : [__m128i; 4] = max.into();
                    let res = [$max($min(a[0], mi[0]), ma[0]),
                               $max($min(a[1], mi[1]), ma[1]),
                               $max($min(a[2], mi[2]), ma[2]),
                               $max($min(a[3], mi[3]), ma[3])];
                    res.into()
                }
            }
        }
    };
}
impl_signed!{
    i8, 16, 32, 64,
    u8, i8,
    _mm_cmpeq_epi8, _mm_cmpgt_epi8, _mm_set1_epi8, _mm_max_epi8, _mm_min_epi8
}
impl_signed!{
    i16, 8, 16, 32,
    u16, i16,
    _mm_cmpeq_epi16, _mm_cmpgt_epi16, _mm_set1_epi16, _mm_max_epi16, _mm_min_epi16
}
impl_signed!{
    i32, 4, 8, 16,
    u32, i32,
    _mm_cmpeq_epi32, _mm_cmpgt_epi32, _mm_set1_epi32, _mm_max_epi32, _mm_min_epi32
}

//==============================================================================================

macro_rules! impl_unsigned {
    {
        $elem:ty, $lanes128:literal, $lanes256:literal, $lanes512:literal,
        $u_ty:ty, $i_ty:ty,
        $eq:ident, $gt:ident, $set1:ident, $max:ident, $min:ident
    } => {
        impl SimdCmpImpl<{BackendType::SSE}> for Simd<$elem, $lanes128>
        {
            type MaskT = Mask<<$elem as SimdElement>::Mask, $lanes128>;

            fn simd_eq_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : __m128i = (*self).into();
                    let b : __m128i = (*other).into();
                    let sign_mask = $set1(<$i_ty>::MIN);
                    let res = $eq(_mm_xor_si128(a, sign_mask), _mm_xor_si128(b, sign_mask));
                    Mask::<<$elem as SimdElement>::Mask, $lanes128>::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
                }
            }
        
            fn simd_ne_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : __m128i = (*self).into();
                    let b : __m128i = (*other).into();
                    let sign_mask = $set1(<$i_ty>::MIN);
                    let xor_mask = $set1(<$u_ty>::MAX as $i_ty);
                    let res = _mm_xor_si128($eq(_mm_xor_si128(a, sign_mask), _mm_xor_si128(b, sign_mask)), xor_mask);
                    Mask::<<$elem as SimdElement>::Mask, $lanes128>::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
                }
            }
            
            fn simd_lt_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : __m128i = (*self).into();
                    let b : __m128i = (*other).into();
                    let sign_mask = $set1(<$i_ty>::MIN);
                    let res = $gt(_mm_xor_si128(b, sign_mask), _mm_xor_si128(a, sign_mask));
                    Mask::<<$elem as SimdElement>::Mask, $lanes128>::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
                }
            }

            fn simd_le_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let sign_mask = $set1(<$i_ty>::MIN);
                    let a : __m128i = _mm_xor_si128((*self).into(), sign_mask);
                    let b : __m128i = _mm_xor_si128((*other).into(), sign_mask);
                    let res = _mm_or_si128($eq(a, b), $gt(b, a));
                    Mask::<<$elem as SimdElement>::Mask, $lanes128>::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
                }
            }

            fn simd_gt_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : __m128i = (*self).into();
                    let b : __m128i = (*other).into();
                    let sign_mask = $set1(<$i_ty>::MIN);
                    let res = $gt(_mm_xor_si128(a, sign_mask), _mm_xor_si128(b, sign_mask));
                    Mask::<<$elem as SimdElement>::Mask, $lanes128>::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
                }
            }

            fn simd_ge_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let sign_mask = $set1(<$i_ty>::MIN);
                    let a : __m128i = _mm_xor_si128((*self).into(), sign_mask);
                    let b : __m128i = _mm_xor_si128((*other).into(), sign_mask);
                    let res = _mm_or_si128($eq(a, b), $gt(a, b));
                    Mask::<<$elem as SimdElement>::Mask, $lanes128>::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
                }
            }

            fn simd_max_impl(self, other: Self) -> Self {
                unsafe {
                    let a : __m128i = self.into();
                    let b : __m128i = other.into();
                    let res = $max(a, b);
                    res.into()
                }
            }

            fn simd_min_impl(self, other: Self) -> Self {
                unsafe {
                    let a : __m128i = self.into();
                    let b : __m128i = other.into();
                    let res = $min(a, b);
                    res.into()
                }
            }

            fn simd_clamp_impl(self, min: Self, max: Self) -> Self {
                unsafe {
                    let a  : __m128i = self.into();
                    let mi : __m128i = min.into();
                    let ma : __m128i = max.into();
                    let res = $max($min(a, mi), ma);
                    res.into()
                }
            }
        }

        impl SimdCmpImpl<{BackendType::SSE}> for Simd<$elem, $lanes256>
        {
            type MaskT = Mask<<$elem as SimdElement>::Mask, $lanes256>;

            fn simd_eq_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : [__m128i; 2] = (*self).into();
                    let b : [__m128i; 2] = (*other).into();
                    let sign_mask = $set1(<$i_ty>::MIN);
                    let res = [$eq(_mm_xor_si128(a[0], sign_mask), _mm_xor_si128(b[0], sign_mask)), 
                               $eq(_mm_xor_si128(a[1], sign_mask), _mm_xor_si128(b[1], sign_mask))];
                    Mask::<<$elem as SimdElement>::Mask, $lanes256>::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
                }
            }
        
            fn simd_ne_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : [__m128i; 2] = (*self).into();
                    let b : [__m128i; 2] = (*other).into();
                    let sign_mask = $set1(<$i_ty>::MIN);
                    let xor_mask = $set1(<$u_ty>::MAX as $i_ty);
                    let res = [_mm_xor_si128($eq(_mm_xor_si128(a[0], sign_mask), _mm_xor_si128(b[0], sign_mask)), xor_mask), 
                               _mm_xor_si128($eq(_mm_xor_si128(a[1], sign_mask), _mm_xor_si128(b[1], sign_mask)), xor_mask)];
                    Mask::<<$elem as SimdElement>::Mask, $lanes256>::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
                }
            }
            
            fn simd_lt_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : [__m128i; 2] = (*self).into();
                    let b : [__m128i; 2] = (*other).into();
                    let sign_mask = $set1(<$i_ty>::MIN);
                    let res = [$gt(_mm_xor_si128(b[0], sign_mask), _mm_xor_si128(a[0], sign_mask)), 
                               $gt(_mm_xor_si128(b[1], sign_mask), _mm_xor_si128(a[1], sign_mask))];
                    Mask::<<$elem as SimdElement>::Mask, $lanes256>::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
                }
            }

            fn simd_le_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : [__m128i; 2] = (*self).into();
                    let b : [__m128i; 2] = (*other).into();
                    let sign_mask = $set1(<$i_ty>::MIN);

                    let a0 = _mm_xor_si128(a[0], sign_mask);
                    let a1 = _mm_xor_si128(a[1], sign_mask);
                    let b0 = _mm_xor_si128(b[0], sign_mask);
                    let b1 = _mm_xor_si128(b[1], sign_mask);

                    let res = [_mm_or_si128($eq(a0, b0), $gt(b0, a0)), 
                               _mm_or_si128($eq(a1, b1), $gt(b1, a1))];
                    Mask::<<$elem as SimdElement>::Mask, $lanes256>::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
                }
            }

            fn simd_gt_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : [__m128i; 2] = (*self).into();
                    let b : [__m128i; 2] = (*other).into();
                    let sign_mask = $set1(<$i_ty>::MIN);
                    let res = [$gt(_mm_xor_si128(a[0], sign_mask), _mm_xor_si128(b[0], sign_mask)), 
                               $gt(_mm_xor_si128(a[1], sign_mask), _mm_xor_si128(b[1], sign_mask))];
                    Mask::<<$elem as SimdElement>::Mask, $lanes256>::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
                }
            }

            fn simd_ge_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : [__m128i; 2] = (*self).into();
                    let b : [__m128i; 2] = (*other).into();
                    let sign_mask = $set1(<$i_ty>::MIN);

                    let a0 = _mm_xor_si128(a[0], sign_mask);
                    let a1 = _mm_xor_si128(a[1], sign_mask);
                    let b0 = _mm_xor_si128(b[0], sign_mask);
                    let b1 = _mm_xor_si128(b[1], sign_mask);

                    let res = [_mm_or_si128($eq(a0, b0), $gt(a0, b0)), 
                               _mm_or_si128($eq(a1, b1), $gt(a1, b1))];
                    Mask::<<$elem as SimdElement>::Mask, $lanes256>::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
                }
            }

            fn simd_max_impl(self, other: Self) -> Self {
                unsafe {
                    let a : [__m128i; 2] = self.into();
                    let b : [__m128i; 2] = other.into();
                    let res = [$max(a[0], b[0]),
                               $max(a[1], b[1])];
                    res.into()
                }
            }

            fn simd_min_impl(self, other: Self) -> Self {
                unsafe {
                    let a : [__m128i; 2] = self.into();
                    let b : [__m128i; 2] = other.into();
                    let res = [$min(a[0], b[0]),
                               $min(a[1], b[1])];
                    res.into()
                }
            }

            fn simd_clamp_impl(self, min: Self, max: Self) -> Self {
                unsafe {
                    let a  : [__m128i; 2] = self.into();
                    let mi : [__m128i; 2] = min.into();
                    let ma : [__m128i; 2] = max.into();
                    let res = [$max($min(a[0], mi[0]), ma[0]),
                               $max($min(a[1], mi[1]), ma[1])];
                    res.into()
                }
            }
        }

        impl SimdCmpImpl<{BackendType::SSE}> for Simd<$elem, $lanes512>
        {
            type MaskT = Mask<<$elem as SimdElement>::Mask, $lanes512>;

            fn simd_eq_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : [__m128i; 4] = (*self).into();
                    let b : [__m128i; 4] = (*other).into();
                    let sign_mask = $set1(<$i_ty>::MIN);
                    let res = [$eq(_mm_xor_si128(a[0],sign_mask), _mm_xor_si128(b[0], sign_mask)), 
                               $eq(_mm_xor_si128(a[1],sign_mask), _mm_xor_si128(b[1], sign_mask)),
                               $eq(_mm_xor_si128(a[2],sign_mask), _mm_xor_si128(b[2], sign_mask)),
                               $eq(_mm_xor_si128(a[3],sign_mask), _mm_xor_si128(b[3], sign_mask))];
                    Mask::<<$elem as SimdElement>::Mask, $lanes512>::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
                }
            }
        
            fn simd_ne_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : [__m128i; 4] = (*self).into();
                    let b : [__m128i; 4] = (*other).into();
                    let sign_mask = $set1(<$i_ty>::MIN);
                    let xor_mask = $set1(<$u_ty>::MAX as $i_ty);
                    let res = [_mm_xor_si128($eq(_mm_xor_si128(a[0], sign_mask), _mm_xor_si128(b[0], sign_mask)), xor_mask), 
                               _mm_xor_si128($eq(_mm_xor_si128(a[1], sign_mask), _mm_xor_si128(b[1], sign_mask)), xor_mask),
                               _mm_xor_si128($eq(_mm_xor_si128(a[2], sign_mask), _mm_xor_si128(b[2], sign_mask)), xor_mask),
                               _mm_xor_si128($eq(_mm_xor_si128(a[3], sign_mask), _mm_xor_si128(b[3], sign_mask)), xor_mask)];
                    Mask::<<$elem as SimdElement>::Mask, $lanes512>::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
                }
            }
            
            fn simd_lt_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : [__m128i; 4] = (*self).into();
                    let b : [__m128i; 4] = (*other).into();
                    let sign_mask = $set1(<$i_ty>::MIN);
                    let res = [$gt(_mm_xor_si128(b[0], sign_mask), _mm_xor_si128(a[0], sign_mask)), 
                               $gt(_mm_xor_si128(b[1], sign_mask), _mm_xor_si128(a[1], sign_mask)),
                               $gt(_mm_xor_si128(b[2], sign_mask), _mm_xor_si128(a[2], sign_mask)),
                               $gt(_mm_xor_si128(b[3], sign_mask), _mm_xor_si128(a[3], sign_mask))];
                    Mask::<<$elem as SimdElement>::Mask, $lanes512>::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
                }
            }

            fn simd_le_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : [__m128i; 4] = (*self).into();
                    let b : [__m128i; 4] = (*other).into();
                    let sign_mask = $set1(<$i_ty>::MIN);

                    let a0 = _mm_xor_si128(a[0], sign_mask);
                    let a1 = _mm_xor_si128(a[1], sign_mask);
                    let a2 = _mm_xor_si128(a[2], sign_mask);
                    let a3 = _mm_xor_si128(a[3], sign_mask);
                    let b0 = _mm_xor_si128(b[0], sign_mask);
                    let b1 = _mm_xor_si128(b[1], sign_mask);
                    let b2 = _mm_xor_si128(b[2], sign_mask);
                    let b3 = _mm_xor_si128(b[3], sign_mask);

                    let res = [_mm_or_si128($eq(a0, b0), $gt(b0, a0)), 
                               _mm_or_si128($eq(a1, b1), $gt(b1, a1)),
                               _mm_or_si128($eq(a2, b2), $gt(b2, a2)),
                               _mm_or_si128($eq(a3, b3), $gt(b3, a3))];
                    Mask::<<$elem as SimdElement>::Mask, $lanes512>::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
                }
            }

            fn simd_gt_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : [__m128i; 4] = (*self).into();
                    let b : [__m128i; 4] = (*other).into();
                    let sign_mask = $set1(<$i_ty>::MIN);
                    let res = [$gt(_mm_xor_si128(a[0], sign_mask), _mm_xor_si128(b[0], sign_mask)), 
                               $gt(_mm_xor_si128(a[1], sign_mask), _mm_xor_si128(b[1], sign_mask)),
                               $gt(_mm_xor_si128(a[2], sign_mask), _mm_xor_si128(b[2], sign_mask)),
                               $gt(_mm_xor_si128(a[3], sign_mask), _mm_xor_si128(b[3], sign_mask))];
                    Mask::<<$elem as SimdElement>::Mask, $lanes512>::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
                }
            }

            fn simd_ge_impl(&self, other: &Self) -> Self::MaskT {
                unsafe {
                    let a : [__m128i; 4] = (*self).into();
                    let b : [__m128i; 4] = (*other).into();
                    let sign_mask = $set1(<$i_ty>::MIN);

                    let a0 = _mm_xor_si128(a[0], sign_mask);
                    let a1 = _mm_xor_si128(a[1], sign_mask);
                    let a2 = _mm_xor_si128(a[2], sign_mask);
                    let a3 = _mm_xor_si128(a[3], sign_mask);
                    let b0 = _mm_xor_si128(b[0], sign_mask);
                    let b1 = _mm_xor_si128(b[1], sign_mask);
                    let b2 = _mm_xor_si128(b[2], sign_mask);
                    let b3 = _mm_xor_si128(b[3], sign_mask);

                    let res = [_mm_or_si128($eq(a0, b0), $gt(a0, b0)), 
                               _mm_or_si128($eq(a1, b1), $gt(a1, b1)),
                               _mm_or_si128($eq(a2, b2), $gt(a2, b2)),
                               _mm_or_si128($eq(a3, b3), $gt(a3, b3))];
                    Mask::<<$elem as SimdElement>::Mask, $lanes512>::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
                }
            }

            fn simd_max_impl(self, other: Self) -> Self {
                unsafe {
                    let a : [__m128i; 4] = self.into();
                    let b : [__m128i; 4] = other.into();
                    let res = [$max(a[0], b[0]),
                               $max(a[1], b[1]),
                               $max(a[2], b[2]),
                               $max(a[3], b[3])];
                    res.into()
                }
            }

            fn simd_min_impl(self, other: Self) -> Self {
                unsafe {
                    let a : [__m128i; 4] = self.into();
                    let b : [__m128i; 4] = other.into();
                    let res = [$min(a[0], b[0]),
                               $min(a[1], b[1]),
                               $min(a[2], b[2]),
                               $min(a[3], b[3])];
                    res.into()
                }
            }

            fn simd_clamp_impl(self, min: Self, max: Self) -> Self {
                unsafe {
                    let a  : [__m128i; 4] = self.into();
                    let mi : [__m128i; 4] = min.into();
                    let ma : [__m128i; 4] = max.into();
                    let res = [$max($min(a[0], mi[0]), ma[0]),
                               $max($min(a[1], mi[1]), ma[1]),
                               $max($min(a[2], mi[2]), ma[2]),
                               $max($min(a[3], mi[3]), ma[3])];
                    res.into()
                }
            }
        }
    };
}
impl_unsigned!{
    u8, 16, 32, 64,
    u8, i8,
    _mm_cmpeq_epi8, _mm_cmpgt_epi8, _mm_set1_epi8, _mm_max_epu8, _mm_min_epu8
}
impl_unsigned!{
    u16, 8, 16, 32,
    u16, i16,
    _mm_cmpeq_epi16, _mm_cmpgt_epi16, _mm_set1_epi16, _mm_max_epu16, _mm_min_epu16
}
impl_unsigned!{
    u32, 4, 8, 16,
    u32, i32,
    _mm_cmpeq_epi32, _mm_cmpgt_epi32, _mm_set1_epi32, _mm_max_epu32, _mm_min_epu32
}

//==============================================================================================

impl SimdCmpImpl<{BackendType::SSE}> for i64x2 {
    type MaskT = mask64x2;

    fn simd_eq_impl(&self, other: &Self) -> mask64x2 {
        unsafe {
            let a : __m128i = (*self).into();
            let b : __m128i = (*other).into();
            let res = _mm_cmpeq_epi64(a, b);
            mask64x2::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_ne_impl(&self, other: &Self) -> mask64x2 {
        unsafe {
            let a : __m128i = (*self).into();
            let b : __m128i = (*other).into();
            let res = _mm_xor_si128(_mm_cmpeq_epi64(a, b), _mm_set1_epi64x(u64::MAX as i64));
            mask64x2::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_lt_impl(&self, other: &Self) -> mask64x2 {
        unsafe {
            let a : __m128i = (*self).into();
            let b : __m128i = (*other).into();
            let res = _mm_cmpgt_epi64(b, a);
            mask64x2::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_le_impl(&self, other: &Self) -> mask64x2 {
        unsafe {
            let a : __m128i = (*self).into();
            let b : __m128i = (*other).into();
            let res = _mm_or_si128(_mm_cmpeq_epi64(a, b), _mm_cmpgt_epi64(b, a));
            mask64x2::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_gt_impl(&self, other: &Self) -> mask64x2 {
        unsafe {
            let a : __m128i = (*self).into();
            let b : __m128i = (*other).into();
            let res = _mm_cmpgt_epi64(a, b);
            mask64x2::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_ge_impl(&self, other: &Self) -> mask64x2 {
        unsafe {
            let a : __m128i = (*self).into();
            let b : __m128i = (*other).into();
            let res = _mm_or_si128(_mm_cmpeq_epi64(a, b), _mm_cmpgt_epi64(a, b));
            mask64x2::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_max_impl(self, other: Self) -> Self {
        unsafe {
            let a : __m128i = self.into();
            let b : __m128i = other.into();
            max_i64(a, b).into()
        }
    }

    fn simd_min_impl(self, other: Self) -> Self {
        unsafe {
            let a : __m128i = self.into();
            let b : __m128i = other.into();
            min_i64(a, b).into()
        }
    }

    fn simd_clamp_impl(self, min: Self, max: Self) -> Self {
        unsafe {
            let a  : __m128i = self.into();
            let mi : __m128i = min.into();
            let ma : __m128i = max.into();
            clamp_i64(a, mi, ma).into()
        }
    }
}

impl SimdCmpImpl<{BackendType::SSE}> for i64x4 {
    type MaskT = mask64x4;

    fn simd_eq_impl(&self, other: &i64x4) -> mask64x4 {
        unsafe {
            let a : [__m128i; 2] = (*self).into();
            let b : [__m128i; 2] = (*other).into();
            let res = [_mm_cmpeq_epi64(a[0], b[0]),
                       _mm_cmpeq_epi64(a[1], b[1])];
            mask64x4::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_ne_impl(&self, other: &i64x4) -> mask64x4 {
        unsafe {
            let a : [__m128i; 2] = (*self).into();
            let b : [__m128i; 2] = (*other).into();
            let res = [_mm_xor_si128(_mm_cmpeq_epi64(a[0], b[0]), _mm_set1_epi64x(u64::MAX as i64)),
                       _mm_xor_si128(_mm_cmpeq_epi64(a[1], b[1]), _mm_set1_epi64x(u64::MAX as i64))];
            mask64x4::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_lt_impl(&self, other: &i64x4) -> mask64x4 {
        unsafe {
            let a : [__m128i; 2] = (*self).into();
            let b : [__m128i; 2] = (*other).into();
            let res = [_mm_cmpgt_epi64(b[0], a[0]),
                       _mm_cmpgt_epi64(b[1], a[1])];
            mask64x4::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_le_impl(&self, other: &i64x4) -> mask64x4 {
        unsafe {
            let a : [__m128i; 2] = (*self).into();
            let b : [__m128i; 2] = (*other).into();
            let res = [_mm_or_si128(_mm_cmpeq_epi64(a[0], b[0]), _mm_cmpgt_epi64(b[0], a[0])),
                       _mm_or_si128(_mm_cmpeq_epi64(a[1], b[1]), _mm_cmpgt_epi64(b[1], a[1]))];
            mask64x4::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_gt_impl(&self, other: &i64x4) -> mask64x4 {
        unsafe {
            let a : [__m128i; 2] = (*self).into();
            let b : [__m128i; 2] = (*other).into();
            let res = [_mm_cmpgt_epi64(a[0], b[0]),
                       _mm_cmpgt_epi64(a[1], b[1])];
            mask64x4::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_ge_impl(&self, other: &i64x4) -> mask64x4 {
        unsafe {
            let a : [__m128i; 2] = (*self).into();
            let b : [__m128i; 2] = (*other).into();
            let res = [_mm_or_si128(_mm_cmpeq_epi64(a[0], b[0]), _mm_cmpgt_epi64(a[0], b[0])),
                       _mm_or_si128(_mm_cmpeq_epi64(a[1], b[1]), _mm_cmpgt_epi64(a[1], b[1]))];
            mask64x4::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_max_impl(self, other: i64x4) -> i64x4 {
        unsafe {
            let a : [__m128i; 2] = self.into();
            let b : [__m128i; 2] = other.into();
            let res = [max_i64(a[0], b[0]),
                       max_i64(a[1], b[1])];
            res.into()
        }
    }

    fn simd_min_impl(self, other: i64x4) -> i64x4 {
        unsafe {
            let a : [__m128i; 2] = self.into();
            let b : [__m128i; 2] = other.into();
            let res = [min_i64(a[0], b[0]),
                       min_i64(a[1], b[1])];
            res.into()
        }
    }

    fn simd_clamp_impl(self, min: i64x4, max: i64x4) -> i64x4 {
        unsafe {
            let a  : [__m128i; 2] = self.into();
            let mi : [__m128i; 2] = min.into();
            let ma : [__m128i; 2] = max.into();
            let res = [clamp_i64(a[0], mi[0], ma[0]),
                       clamp_i64(a[1], mi[1], ma[1]) ];
            res.into()
        }
    }
}

impl SimdCmpImpl<{BackendType::SSE}> for i64x8 {
    type MaskT = Mask<i64, 8>;

    fn simd_eq_impl(&self, other: &i64x8) -> Mask<<i64 as SimdElement>::Mask, 8> {
        unsafe {
            let a : [__m128i; 4] = (*self).into();
            let b : [__m128i; 4] = (*other).into();
            let res = [_mm_cmpeq_epi64(a[0], b[0]),
                       _mm_cmpeq_epi64(a[1], b[1]),
                       _mm_cmpeq_epi64(a[2], b[2]),
                       _mm_cmpeq_epi64(a[3], b[3])];
            mask64x8::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_ne_impl(&self, other: &i64x8) -> Mask<<i64 as SimdElement>::Mask, 8> {
        unsafe {
            let a : [__m128i; 4] = (*self).into();
            let b : [__m128i; 4] = (*other).into();
            let res = [_mm_xor_si128(_mm_cmpeq_epi64(a[0], b[0]), _mm_set1_epi64x(u64::MAX as i64)),
                       _mm_xor_si128(_mm_cmpeq_epi64(a[1], b[1]), _mm_set1_epi64x(u64::MAX as i64)),
                       _mm_xor_si128(_mm_cmpeq_epi64(a[2], b[2]), _mm_set1_epi64x(u64::MAX as i64)),
                       _mm_xor_si128(_mm_cmpeq_epi64(a[3], b[3]), _mm_set1_epi64x(u64::MAX as i64))];
            mask64x8::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_lt_impl(&self, other: &i64x8) -> Mask<<i64 as SimdElement>::Mask, 8> {
        unsafe {
            let a : [__m128i; 4] = (*self).into();
            let b : [__m128i; 4] = (*other).into();
            let res = [_mm_cmpgt_epi64(b[0], a[0]),
                       _mm_cmpgt_epi64(b[1], a[1]),
                       _mm_cmpgt_epi64(b[2], a[2]),
                       _mm_cmpgt_epi64(b[3], a[3])];
            mask64x8::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_le_impl(&self, other: &i64x8) -> Mask<<i64 as SimdElement>::Mask, 8> {
        unsafe {
            let a : [__m128i; 4] = (*self).into();
            let b : [__m128i; 4] = (*other).into();
            let res = [_mm_or_si128(_mm_cmpeq_epi64(a[0], b[0]), _mm_cmpgt_epi64(b[0], a[0])),
                       _mm_or_si128(_mm_cmpeq_epi64(a[1], b[1]), _mm_cmpgt_epi64(b[1], a[1])),
                       _mm_or_si128(_mm_cmpeq_epi64(a[2], b[2]), _mm_cmpgt_epi64(b[2], a[2])),
                       _mm_or_si128(_mm_cmpeq_epi64(a[3], b[3]), _mm_cmpgt_epi64(b[3], a[3]))];
            mask64x8::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_gt_impl(&self, other: &i64x8) -> Mask<<i64 as SimdElement>::Mask, 8> {
        unsafe {
            let a : [__m128i; 4] = (*self).into();
            let b : [__m128i; 4] = (*other).into();
            let res = [_mm_cmpgt_epi64(a[0], b[0]),
                       _mm_cmpgt_epi64(a[1], b[1]),
                       _mm_cmpgt_epi64(a[2], b[2]),
                       _mm_cmpgt_epi64(a[3], b[3])];
            mask64x8::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_ge_impl(&self, other: &i64x8) -> Mask<<i64 as SimdElement>::Mask, 8> {
        unsafe {
            let a : [__m128i; 4] = (*self).into();
            let b : [__m128i; 4] = (*other).into();
            let res = [_mm_or_si128(_mm_cmpeq_epi64(a[0], b[0]), _mm_cmpgt_epi64(a[0], b[0])),
                       _mm_or_si128(_mm_cmpeq_epi64(a[1], b[1]), _mm_cmpgt_epi64(a[1], b[1])),
                       _mm_or_si128(_mm_cmpeq_epi64(a[2], b[2]), _mm_cmpgt_epi64(a[2], b[2])),
                       _mm_or_si128(_mm_cmpeq_epi64(a[3], b[3]), _mm_cmpgt_epi64(a[3], b[3]))];
            mask64x8::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_max_impl(self: i64x8, other: i64x8) -> i64x8 {
        unsafe {
            let a : [__m128i; 4] = self.into();
            let b : [__m128i; 4] = other.into();
            let res = [max_i64(a[0], b[0]),
                       max_i64(a[1], b[1]),
                       max_i64(a[2], b[2]),
                       max_i64(a[3], b[3])];
            res.into()
        }
    }

    fn simd_min_impl(self: i64x8, other: i64x8) -> i64x8 {
        unsafe {
            let a : [__m128i; 4] = self.into();
            let b : [__m128i; 4] = other.into();
            let res = [min_i64(a[0], b[0]),
                       min_i64(a[1], b[1]),
                       min_i64(a[2], b[2]),
                       min_i64(a[3], b[3])];
            res.into()
        }
    }

    fn simd_clamp_impl(self: i64x8, min: i64x8, max: i64x8) -> i64x8 {
        unsafe {
            let a  : [__m128i; 4] = self.into();
            let mi : [__m128i; 4] = min.into();
            let ma : [__m128i; 4] = max.into();
            let res = [clamp_i64(a[0], mi[0], ma[0]),
                       clamp_i64(a[1], mi[1], ma[1]),
                       clamp_i64(a[2], mi[2], ma[2]),
                       clamp_i64(a[3], mi[3], ma[3])];
            res.into()
        }
    }
}

//==============================================================================================

impl SimdCmpImpl<{BackendType::SSE}> for Simd<u64, 2> {
    type MaskT = mask64x2;

    fn simd_eq_impl(&self, other: &Simd<u64, 2>) -> Mask<<i64 as SimdElement>::Mask, 2> {
        unsafe {
            let a : __m128i = (*self).into();
            let b : __m128i = (*other).into();
            let sign_mask = _mm_set1_epi64x(i64::MIN);
            let res = eq_u64(a, b, sign_mask);
            mask64x2::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_ne_impl(&self, other: &Simd<u64, 2>) -> Mask<<i64 as SimdElement>::Mask, 2> {
        unsafe {
            let a : __m128i = (*self).into();
            let b : __m128i = (*other).into();
            let sign_mask = _mm_set1_epi64x(i64::MIN);
            let xor_mask = _mm_set1_epi64x(u64::MAX as i64);
            let res = ne_u64(a, b, sign_mask, xor_mask);
            mask64x2::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_lt_impl(&self, other: &Simd<u64, 2>) -> Mask<<i64 as SimdElement>::Mask, 2> {
        unsafe {
            let a : __m128i = (*self).into();
            let b : __m128i = (*other).into();
            let sign_mask = _mm_set1_epi64x(i64::MIN);
            let res = lt_u64(a, b, sign_mask);
            mask64x2::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_le_impl(&self, other: &Simd<u64, 2>) -> Mask<<i64 as SimdElement>::Mask, 2> {
        unsafe {
            let sign_mask = _mm_set1_epi64x(i64::MIN);
            let a : __m128i = _mm_xor_si128((*self).into(), sign_mask);
            let b : __m128i = _mm_xor_si128((*other).into(), sign_mask);
            let res = le_u64(a, b, sign_mask);
            mask64x2::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_gt_impl(&self, other: &Simd<u64, 2>) -> Mask<<i64 as SimdElement>::Mask, 2> {
        unsafe {
            let a : __m128i = (*self).into();
            let b : __m128i = (*other).into();
            let sign_mask = _mm_set1_epi64x(i64::MIN);
            let res = gt_u64(a, b, sign_mask);
            mask64x2::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_ge_impl(&self, other: &Simd<u64, 2>) -> Mask<<i64 as SimdElement>::Mask, 2> {
        unsafe {
            let sign_mask = _mm_set1_epi64x(i64::MIN);
            let a : __m128i = _mm_xor_si128((*self).into(), sign_mask);
            let b : __m128i = _mm_xor_si128((*other).into(), sign_mask);
            let res = ge_u64(a, b, sign_mask);
            mask64x2::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_max_impl(self: Simd<u64, 2>, other: Simd<u64, 2>) -> Simd<u64, 2> {
        unsafe {
            let a : __m128i = self.into();
            let b : __m128i = other.into();
            let sign_mask = _mm_set1_epi64x(i64::MIN);
            max_u64(a, b, sign_mask).into()
        }
    }

    fn simd_min_impl(self: Simd<u64, 2>, other: Simd<u64, 2>) -> Simd<u64, 2> {
        unsafe {
            let a : __m128i = self.into();
            let b : __m128i = other.into();
            let sign_mask = _mm_set1_epi64x(i64::MIN);
            min_u64(a, b, sign_mask).into()
        }
    }

    fn simd_clamp_impl(self: Simd<u64, 2>, min: Simd<u64, 2>, max: Simd<u64, 2>) -> Simd<u64, 2> {
        unsafe {
            let a  : __m128i = self.into();
            let mi : __m128i = min.into();
            let ma : __m128i = max.into();
            let sign_mask = _mm_set1_epi64x(i64::MIN);
            clamp_u64(a, mi, ma, sign_mask).into()
        }
    }
}

impl SimdCmpImpl<{BackendType::SSE}> for Simd<u64, 4> {
    type MaskT = mask64x4;

    fn simd_eq_impl(&self, other: &Simd<u64, 4>) -> mask64x4 {
        unsafe {
            let a : [__m128i; 2] = (*self).into();
            let b : [__m128i; 2] = (*other).into();
            let sign_mask = _mm_set1_epi64x(i64::MIN);
            let res = [eq_u64(a[0], b[0], sign_mask),
                       eq_u64(a[1], b[1], sign_mask)];
            mask64x4::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_ne_impl(&self, other: &Simd<u64, 4>) -> mask64x4 {
        unsafe {
            let a : [__m128i; 2] = (*self).into();
            let b : [__m128i; 2] = (*other).into();
            let sign_mask = _mm_set1_epi64x(i64::MIN);
            let xor_mask = _mm_set1_epi64x(u64::MAX as i64);
            let res = [ne_u64(a[0], b[0], sign_mask, xor_mask),
                       ne_u64(a[1], b[1], sign_mask, xor_mask)];
            mask64x4::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_lt_impl(&self, other: &Simd<u64, 4>) -> mask64x4 {
        unsafe {
            let a : [__m128i; 2] = (*self).into();
            let b : [__m128i; 2] = (*other).into();
            let sign_mask = _mm_set1_epi64x(i64::MIN);
            let res = [lt_u64(a[0], b[0], sign_mask),
                       lt_u64(a[1], b[1], sign_mask)];
            mask64x4::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_le_impl(&self, other: &Simd<u64, 4>) -> mask64x4 {
        unsafe {
            let a : [__m128i; 2] = (*self).into();
            let b : [__m128i; 2] = (*other).into();
            let sign_mask = _mm_set1_epi64x(i64::MIN);

            let res = [le_u64(a[0], b[0], sign_mask),
                       le_u64(a[1], b[1], sign_mask)];
            mask64x4::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_gt_impl(&self, other: &Simd<u64, 4>) -> mask64x4 {
        unsafe {
            let a : [__m128i; 2] = (*self).into();
            let b : [__m128i; 2] = (*other).into();
            let sign_mask = _mm_set1_epi64x(i64::MIN);
            let res = [gt_u64(a[0], b[0], sign_mask),
                       gt_u64(a[1], b[1], sign_mask)];
            mask64x4::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_ge_impl(&self, other: &Simd<u64, 4>) -> mask64x4 {
        unsafe {
            let a : [__m128i; 2] = (*self).into();
            let b : [__m128i; 2] = (*other).into();
            let sign_mask = _mm_set1_epi64x(i64::MIN);

            let a0 = _mm_xor_si128(a[0], sign_mask);
            let a1 = _mm_xor_si128(a[1], sign_mask);
            let b0 = _mm_xor_si128(b[0], sign_mask);
            let b1 = _mm_xor_si128(b[1], sign_mask);
            
            let res = [ge_u64(a[0], b[0], sign_mask),
                       ge_u64(a[1], b[1], sign_mask)];
            mask64x4::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_max_impl(self: Simd<u64, 4>, other: Simd<u64, 4>) -> Simd<u64, 4> {
        unsafe {
            let a : [__m128i; 2] = self.into();
            let b : [__m128i; 2] = other.into();
            let sign_mask = _mm_set1_epi64x(i64::MIN);
            let res = [max_u64(a[0], b[0], sign_mask),
                       max_u64(a[1], b[1], sign_mask)];
            res.into()
        }
    }

    fn simd_min_impl(self: Simd<u64, 4>, other: Simd<u64, 4>) -> Simd<u64, 4> {
        unsafe {
            let a : [__m128i; 2] = self.into();
            let b : [__m128i; 2] = other.into();
            let sign_mask = _mm_set1_epi64x(i64::MIN);
            let res = [min_u64(a[0], b[0], sign_mask),
                       min_u64(a[1], b[1], sign_mask)];
            res.into()
        }
    }

    fn simd_clamp_impl(self: Simd<u64, 4>, min: Simd<u64, 4>, max: Simd<u64, 4>) -> Simd<u64, 4> {
        unsafe {
            let a  : [__m128i; 2] = self.into();
            let mi : [__m128i; 2] = min.into();
            let ma : [__m128i; 2] = max.into();
            let sign_mask = _mm_set1_epi64x(i64::MIN);
            let res = [clamp_u64(a[0], mi[0], ma[0], sign_mask),
                       clamp_u64(a[1], mi[1], ma[1], sign_mask)];
            res.into()
        }
    }
}

impl SimdCmpImpl<{BackendType::SSE}> for Simd<u64, 8> {
    type MaskT = Mask<i64, 8>;

    fn simd_eq_impl(&self, other: &Simd<u64, 8>) -> Mask<<i64 as SimdElement>::Mask, 8> {
        unsafe {
            let a : [__m128i; 4] = (*self).into();
            let b : [__m128i; 4] = (*other).into();
            let sign_mask = _mm_set1_epi64x(i64::MIN);
            let res = [eq_u64(a[0], b[0], sign_mask),
                       eq_u64(a[1], b[1], sign_mask),
                       eq_u64(a[2], b[2], sign_mask),
                       eq_u64(a[3], b[3], sign_mask)];
            mask64x8::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_ne_impl(&self, other: &Simd<u64, 8>) -> Mask<<i64 as SimdElement>::Mask, 8> {
        unsafe {
            let a : [__m128i; 4] = (*self).into();
            let b : [__m128i; 4] = (*other).into();
            let sign_mask = _mm_set1_epi64x(i64::MIN);
            let xor_mask = _mm_set1_epi64x(u64::MAX as i64);
            let res = [ne_u64(a[0], b[0], sign_mask, xor_mask),
                       ne_u64(a[1], b[1], sign_mask, xor_mask),
                       ne_u64(a[2], b[2], sign_mask, xor_mask),
                       ne_u64(a[3], b[3], sign_mask, xor_mask)];
            mask64x8::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_lt_impl(&self, other: &Simd<u64, 8>) -> Mask<<i64 as SimdElement>::Mask, 8> {
        unsafe {
            let a : [__m128i; 4] = (*self).into();
            let b : [__m128i; 4] = (*other).into();
            let sign_mask = _mm_set1_epi64x(i64::MIN);
            let res = [lt_u64(a[0], b[0], sign_mask),
                       lt_u64(a[1], b[1], sign_mask),
                       lt_u64(a[2], b[2], sign_mask),
                       lt_u64(a[3], b[3], sign_mask)];
            mask64x8::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_le_impl(&self, other: &Simd<u64, 8>) -> Mask<<i64 as SimdElement>::Mask, 8> {
        unsafe {
            let a : [__m128i; 4] = (*self).into();
            let b : [__m128i; 4] = (*other).into();
            let sign_mask = _mm_set1_epi64x(i64::MIN);

            let res = [le_u64(a[0], b[0], sign_mask),
                       le_u64(a[1], b[1], sign_mask),
                       le_u64(a[2], b[2], sign_mask),
                       le_u64(a[3], b[3], sign_mask)];
            mask64x8::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_gt_impl(&self, other: &Simd<u64, 8>) -> Mask<<i64 as SimdElement>::Mask, 8> {
        unsafe {
            let a : [__m128i; 4] = (*self).into();
            let b : [__m128i; 4] = (*other).into();
            let sign_mask = _mm_set1_epi64x(i64::MIN);
            let res = [gt_u64(a[0], b[0], sign_mask),
                       gt_u64(a[1], b[1], sign_mask),
                       gt_u64(a[2], b[2], sign_mask),
                       gt_u64(a[3], b[3], sign_mask)];
            mask64x8::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_ge_impl(&self, other: &Simd<u64, 8>) -> Mask<<i64 as SimdElement>::Mask, 8> {
        unsafe {
            let a : [__m128i; 4] = (*self).into();
            let b : [__m128i; 4] = (*other).into();
            let sign_mask = _mm_set1_epi64x(i64::MIN);

            let res = [ge_u64(a[0], b[0], sign_mask),
                       ge_u64(a[1], b[1], sign_mask),
                       ge_u64(a[2], b[2], sign_mask),
                       ge_u64(a[3], b[3], sign_mask)];
            mask64x8::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_max_impl(self: Simd<u64, 8>, other: Simd<u64, 8>) -> Simd<u64, 8> {
        unsafe {
            let a : [__m128i; 4] = self.into();
            let b : [__m128i; 4] = other.into();
            let sign_mask = _mm_set1_epi64x(i64::MIN);
            
            let res = [max_u64(a[0], b[0], sign_mask),
                       max_u64(a[1], b[1], sign_mask),
                       max_u64(a[2], b[2], sign_mask),
                       max_u64(a[3], b[3], sign_mask)];
            res.into()
        }
    }

    fn simd_min_impl(self: Simd<u64, 8>, other: Simd<u64, 8>) -> Simd<u64, 8> {
        unsafe {
            let a : [__m128i; 4] = self.into();
            let b : [__m128i; 4] = other.into();
            let sign_mask = _mm_set1_epi64x(i64::MIN);
            
            let res = [min_u64(a[0], b[0], sign_mask),
                       min_u64(a[1], b[1], sign_mask),
                       min_u64(a[2], b[2], sign_mask),
                       min_u64(a[3], b[3], sign_mask)];
            res.into()
        }
    }

    fn simd_clamp_impl(self: Simd<u64, 8>, min: Simd<u64, 8>, max: Simd<u64, 8>) -> Simd<u64, 8> {
        unsafe {
            let a  : [__m128i; 4] = self.into();
            let mi : [__m128i; 4] = min.into();
            let ma : [__m128i; 4] = max.into();
            let sign_mask = _mm_set1_epi64x(i64::MIN);
            
            let res = [clamp_u64(a[0], mi[0], ma[0], sign_mask),
                       clamp_u64(a[1], mi[1], ma[1], sign_mask),
                       clamp_u64(a[2], mi[2], ma[2], sign_mask),
                       clamp_u64(a[3], mi[3], ma[3], sign_mask)];
            res.into()
        }
    }
}

//==============================================================================================

impl SimdCmpImpl<{BackendType::SSE}> for f32x4 {
    type MaskT = mask32x4;

    fn simd_eq_impl(&self, other: &f32x4) -> mask32x4 {
        unsafe {
            let a : __m128 = (*self).into();
            let b : __m128 = (*other).into();
            let res = _mm_castps_si128(_mm_cmpeq_ps(a, b));
            mask32x4::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_ne_impl(&self, other: &f32x4) -> mask32x4 {
        unsafe {
            let a : __m128 = (*self).into();
            let b : __m128 = (*other).into();
            let res = _mm_castps_si128(_mm_cmpneq_ps(a, b));
            mask32x4::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }
    
    fn simd_lt_impl(&self, other: &f32x4) -> mask32x4 {
        unsafe {
            let a : __m128 = (*self).into();
            let b : __m128 = (*other).into();
            let res = _mm_castps_si128(_mm_cmplt_ps(a, b));
            mask32x4::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_le_impl(&self, other: &f32x4) -> mask32x4 {
        unsafe {
            let a : __m128 = (*self).into();
            let b : __m128 = (*other).into();
            let res = _mm_castps_si128(_mm_cmple_ps(a, b));
            mask32x4::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_gt_impl(&self, other: &f32x4) -> mask32x4 {
        unsafe {
            let a : __m128 = (*self).into();
            let b : __m128 = (*other).into();
            let res = _mm_castps_si128(_mm_cmpgt_ps(a, b));
            mask32x4::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_ge_impl(&self, other: &f32x4) -> mask32x4 {
        unsafe {
            let a : __m128 = (*self).into();
            let b : __m128 = (*other).into();
            let res = _mm_castps_si128(_mm_cmpge_ps(a, b));
            mask32x4::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
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

impl SimdCmpImpl<{BackendType::SSE}> for f32x8 {
    type MaskT = mask32x8;

    fn simd_eq_impl(&self, other: &f32x8) -> mask32x8 {
        unsafe {
            let a : [__m128; 2] = (*self).into();
            let b : [__m128; 2] = (*other).into();
            let res = [_mm_castps_si128(_mm_cmpeq_ps(a[0], b[0])), 
                       _mm_castps_si128(_mm_cmpeq_ps(a[1], b[1]))];
            mask32x8::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_ne_impl(&self, other: &f32x8) -> mask32x8 {
        unsafe {
            let a : [__m128; 2] = (*self).into();
            let b : [__m128; 2] = (*other).into();
            let res = [_mm_castps_si128(_mm_cmpneq_ps(a[0], b[0])), 
                       _mm_castps_si128(_mm_cmpneq_ps(a[1], b[1]))];
            mask32x8::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }
    
    fn simd_lt_impl(&self, other: &f32x8) -> mask32x8 {
        unsafe {
            let a : [__m128; 2] = (*self).into();
            let b : [__m128; 2] = (*other).into();
            let res = [_mm_castps_si128(_mm_cmplt_ps(a[0], b[0])), 
                       _mm_castps_si128(_mm_cmplt_ps(a[1], b[1]))];
            mask32x8::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_le_impl(&self, other: &f32x8) -> mask32x8 {
        unsafe {
            let a : [__m128; 2] = (*self).into();
            let b : [__m128; 2] = (*other).into();
            let res = [_mm_castps_si128(_mm_cmple_ps(a[0], b[0])), 
                       _mm_castps_si128(_mm_cmple_ps(a[1], b[1]))];
            mask32x8::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_gt_impl(&self, other: &f32x8) -> mask32x8 {
        unsafe {
            let a : [__m128; 2] = (*self).into();
            let b : [__m128; 2] = (*other).into();
            let res = [_mm_castps_si128(_mm_cmpgt_ps(a[0], b[0])), 
                       _mm_castps_si128(_mm_cmpgt_ps(a[1], b[1]))];
            mask32x8::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_ge_impl(&self, other: &f32x8) -> mask32x8 {
        unsafe {
            let a : [__m128; 2] = (*self).into();
            let b : [__m128; 2] = (*other).into();
            let res = [_mm_castps_si128(_mm_cmpge_ps(a[0], b[0])), 
                       _mm_castps_si128(_mm_cmpge_ps(a[1], b[1]))];
            mask32x8::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_max_impl(self: f32x8, other: f32x8) -> f32x8 {
        unsafe {
            let a : [__m128; 2] = self.into();
            let b : [__m128; 2] = other.into();
            let res = [_mm_max_ps(a[0], b[0]),
                       _mm_max_ps(a[1], b[1])];
            res.into()
        }
    }

    fn simd_min_impl(self: f32x8, other: f32x8) -> f32x8 {
        unsafe {
            let a : [__m128; 2] = self.into();
            let b : [__m128; 2] = other.into();
            let res = [_mm_min_ps(a[0], b[0]),
                       _mm_min_ps(a[1], b[1])];
            res.into()
        }
    }

    fn simd_clamp_impl(self: f32x8, min: f32x8, max: f32x8) -> f32x8 {
        unsafe {
            let a  : [__m128; 2] = self.into();
            let mi : [__m128; 2] = min.into();
            let ma : [__m128; 2] = max.into();
            let res = [_mm_max_ps(_mm_min_ps(a[0], ma[0]), mi[0]),
                       _mm_max_ps(_mm_min_ps(a[1], ma[1]), mi[1])];
            res.into()
        }
    }
}

impl SimdCmpImpl<{BackendType::SSE}> for f32x16 {
    type MaskT = mask32x16;

    fn simd_eq_impl(&self, other: &f32x16) -> mask32x16 {
        unsafe {
            let a : [__m128; 4] = (*self).into();
            let b : [__m128; 4] = (*other).into();
            let res = [_mm_castps_si128(_mm_cmpeq_ps(a[0], b[0])), 
                       _mm_castps_si128(_mm_cmpeq_ps(a[1], b[1])),
                       _mm_castps_si128(_mm_cmpeq_ps(a[2], b[2])),
                       _mm_castps_si128(_mm_cmpeq_ps(a[3], b[3]))];
            mask32x16::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_ne_impl(&self, other: &f32x16) -> mask32x16 {
        unsafe {
            let a : [__m128; 4] = (*self).into();
            let b : [__m128; 4] = (*other).into();
            let res = [_mm_castps_si128(_mm_cmpneq_ps(a[0], b[0])), 
                       _mm_castps_si128(_mm_cmpneq_ps(a[1], b[1])),
                       _mm_castps_si128(_mm_cmpneq_ps(a[2], b[2])),
                       _mm_castps_si128(_mm_cmpneq_ps(a[3], b[3]))];
            mask32x16::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }
    
    fn simd_lt_impl(&self, other: &f32x16) -> mask32x16 {
        unsafe {
            let a : [__m128; 4] = (*self).into();
            let b : [__m128; 4] = (*other).into();
            let res = [_mm_castps_si128(_mm_cmplt_ps(a[0], b[0])), 
                       _mm_castps_si128(_mm_cmplt_ps(a[1], b[1])),
                       _mm_castps_si128(_mm_cmplt_ps(a[2], b[2])),
                       _mm_castps_si128(_mm_cmplt_ps(a[3], b[3]))];
            mask32x16::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_le_impl(&self, other: &f32x16) -> mask32x16 {
        unsafe {
            let a : [__m128; 4] = (*self).into();
            let b : [__m128; 4] = (*other).into();
            let res = [_mm_castps_si128(_mm_cmple_ps(a[0], b[0])), 
                       _mm_castps_si128(_mm_cmple_ps(a[1], b[1])),
                       _mm_castps_si128(_mm_cmple_ps(a[2], b[2])),
                       _mm_castps_si128(_mm_cmple_ps(a[3], b[3]))];
            mask32x16::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_gt_impl(&self, other: &f32x16) -> mask32x16 {
        unsafe {
            let a : [__m128; 4] = (*self).into();
            let b : [__m128; 4] = (*other).into();
            let res = [_mm_castps_si128(_mm_cmpgt_ps(a[0], b[0])), 
                       _mm_castps_si128(_mm_cmpgt_ps(a[1], b[1])),
                       _mm_castps_si128(_mm_cmpgt_ps(a[2], b[2])),
                       _mm_castps_si128(_mm_cmpgt_ps(a[3], b[3]))];
            mask32x16::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_ge_impl(&self, other: &f32x16) -> mask32x16 {
        unsafe {
            let a : [__m128; 4] = (*self).into();
            let b : [__m128; 4] = (*other).into();
            let res = [_mm_castps_si128(_mm_cmpge_ps(a[0], b[0])), 
                       _mm_castps_si128(_mm_cmpge_ps(a[1], b[1])),
                       _mm_castps_si128(_mm_cmpge_ps(a[2], b[2])),
                       _mm_castps_si128(_mm_cmpge_ps(a[3], b[3]))];
            mask32x16::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }
    
    fn simd_max_impl(self: f32x16, other: f32x16) -> f32x16 {
        unsafe {
            let a : [__m128; 4] = self.into();
            let b : [__m128; 4] = other.into();
            let res = [_mm_max_ps(a[0], b[0]),
                       _mm_max_ps(a[1], b[1]),
                       _mm_max_ps(a[2], b[2]),
                       _mm_max_ps(a[3], b[3])];
            res.into()
        }
    }

    fn simd_min_impl(self: f32x16, other: f32x16) -> f32x16 {
        unsafe {
            let a : [__m128; 4] = self.into();
            let b : [__m128; 4] = other.into();
            let res = [_mm_min_ps(a[0], b[0]),
                       _mm_min_ps(a[1], b[1]),
                       _mm_min_ps(a[2], b[2]),
                       _mm_min_ps(a[3], b[3])];
            res.into()
        }
    }

    fn simd_clamp_impl(self: f32x16, min: f32x16, max: f32x16) -> f32x16 {
        unsafe {
            let a  : [__m128; 4] = self.into();
            let mi : [__m128; 4] = min.into();
            let ma : [__m128; 4] = max.into();
            let res = [_mm_max_ps(_mm_min_ps(a[0], ma[0]), mi[0]),
                       _mm_max_ps(_mm_min_ps(a[1], ma[1]), mi[1]),
                       _mm_max_ps(_mm_min_ps(a[2], ma[2]), mi[2]),
                       _mm_max_ps(_mm_min_ps(a[3], ma[3]), mi[3])];
            res.into()
        }
    }
}

//==============================================================================================

impl SimdCmpImpl<{BackendType::SSE}> for f64x2 {
    type MaskT = mask64x2;

    fn simd_eq_impl(&self, other: &f64x2) -> mask64x2 {
        unsafe {
            let a : __m128d = (*self).into();
            let b : __m128d = (*other).into();
            let res = _mm_castpd_si128(_mm_cmpeq_pd(a, b));
            mask64x2::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_ne_impl(&self, other: &f64x2) -> mask64x2 {
        unsafe {
            let a : __m128d = (*self).into();
            let b : __m128d = (*other).into();
            let res = _mm_castpd_si128(_mm_cmpneq_pd(a, b));
            mask64x2::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }
    
    fn simd_lt_impl(&self, other: &f64x2) -> mask64x2 {
        unsafe {
            let a : __m128d = (*self).into();
            let b : __m128d = (*other).into();
            let res = _mm_castpd_si128(_mm_cmplt_pd(a, b));
            mask64x2::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_le_impl(&self, other: &f64x2) -> mask64x2 {
        unsafe {
            let a : __m128d = (*self).into();
            let b : __m128d = (*other).into();
            let res = _mm_castpd_si128(_mm_cmple_pd(a, b));
            mask64x2::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_gt_impl(&self, other: &f64x2) -> mask64x2 {
        unsafe {
            let a : __m128d = (*self).into();
            let b : __m128d = (*other).into();
            let res = _mm_castpd_si128(_mm_cmpgt_pd(a, b));
            mask64x2::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_ge_impl(&self, other: &f64x2) -> mask64x2 {
        unsafe {
            let a : __m128d = (*self).into();
            let b : __m128d = (*other).into();
            let res = _mm_castpd_si128(_mm_cmpge_pd(a, b));
            mask64x2::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
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

impl SimdCmpImpl<{BackendType::SSE}> for f64x4 {
    type MaskT = mask64x4;

    fn simd_eq_impl(&self, other: &f64x4) -> mask64x4 {
        unsafe {
            let a : [__m128d; 2] = (*self).into();
            let b : [__m128d; 2] = (*other).into();
            let res = [_mm_castpd_si128(_mm_cmpeq_pd(a[0], b[0])), 
                       _mm_castpd_si128(_mm_cmpeq_pd(a[1], b[1]))];
            mask64x4::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_ne_impl(&self, other: &f64x4) -> mask64x4 {
        unsafe {
            let a : [__m128d; 2] = (*self).into();
            let b : [__m128d; 2] = (*other).into();
            let res = [_mm_castpd_si128(_mm_cmpneq_pd(a[0], b[0])), 
                       _mm_castpd_si128(_mm_cmpneq_pd(a[1], b[1]))];
            mask64x4::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }
    
    fn simd_lt_impl(&self, other: &f64x4) -> mask64x4 {
        unsafe {
            let a : [__m128d; 2] = (*self).into();
            let b : [__m128d; 2] = (*other).into();
            let res = [_mm_castpd_si128(_mm_cmplt_pd(a[0], b[0])), 
                       _mm_castpd_si128(_mm_cmplt_pd(a[1], b[1]))];
            mask64x4::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_le_impl(&self, other: &f64x4) -> mask64x4 {
        unsafe {
            let a : [__m128d; 2] = (*self).into();
            let b : [__m128d; 2] = (*other).into();
            let res = [_mm_castpd_si128(_mm_cmple_pd(a[0], b[0])), 
                       _mm_castpd_si128(_mm_cmple_pd(a[1], b[1]))];
            mask64x4::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_gt_impl(&self, other: &f64x4) -> mask64x4 {
        unsafe {
            let a : [__m128d; 2] = (*self).into();
            let b : [__m128d; 2] = (*other).into();
            let res = [_mm_castpd_si128(_mm_cmpgt_pd(a[0], b[0])), 
                       _mm_castpd_si128(_mm_cmpgt_pd(a[1], b[1]))];
            mask64x4::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_ge_impl(&self, other: &f64x4) -> mask64x4 {
        unsafe {
            let a : [__m128d; 2] = (*self).into();
            let b : [__m128d; 2] = (*other).into();
            let res = [_mm_castpd_si128(_mm_cmpge_pd(a[0], b[0])), 
                       _mm_castpd_si128(_mm_cmpge_pd(a[1], b[1]))];
            mask64x4::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_max_impl(self: f64x4, other: f64x4) -> f64x4 {
        unsafe {
            let a : [__m128d; 2] = self.into();
            let b : [__m128d; 2] = other.into();
            let res = [_mm_max_pd(a[0], b[0]),
                       _mm_max_pd(a[1], b[1])];
            res.into()
        }
    }

    fn simd_min_impl(self: f64x4, other: f64x4) -> f64x4 {
        unsafe {
            let a : [__m128d; 2] = self.into();
            let b : [__m128d; 2] = other.into();
            let res = [_mm_min_pd(a[0], b[0]),
                       _mm_min_pd(a[1], b[1])];
            res.into()
        }
    }

    fn simd_clamp_impl(self: f64x4, min: f64x4, max: f64x4) -> f64x4 {
        unsafe {
            let a  : [__m128d; 2] = self.into();
            let mi : [__m128d; 2] = min.into();
            let ma : [__m128d; 2] = max.into();
            let res = [_mm_max_pd(_mm_min_pd(a[0], ma[0]), mi[0]),
                       _mm_max_pd(_mm_min_pd(a[1], ma[1]), mi[1])];
            res.into()
        }
    }
}

impl SimdCmpImpl<{BackendType::SSE}> for f64x8 {
    type MaskT = mask64x8;

    fn simd_eq_impl(&self, other: &f64x8) -> mask64x8 {
        unsafe {
            let a : [__m128d; 4] = (*self).into();
            let b : [__m128d; 4] = (*other).into();
            let res = [_mm_castpd_si128(_mm_cmpeq_pd(a[0], b[0])), 
                       _mm_castpd_si128(_mm_cmpeq_pd(a[1], b[1])),
                       _mm_castpd_si128(_mm_cmpeq_pd(a[2], b[2])),
                       _mm_castpd_si128(_mm_cmpeq_pd(a[3], b[3]))];
            mask64x8::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_ne_impl(&self, other: &f64x8) -> mask64x8 {
        unsafe {
            let a : [__m128d; 4] = (*self).into();
            let b : [__m128d; 4] = (*other).into();
            let res = [_mm_castpd_si128(_mm_cmpneq_pd(a[0], b[0])), 
                       _mm_castpd_si128(_mm_cmpneq_pd(a[1], b[1])),
                       _mm_castpd_si128(_mm_cmpneq_pd(a[2], b[2])),
                       _mm_castpd_si128(_mm_cmpneq_pd(a[3], b[3]))];
            mask64x8::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }
    
    fn simd_lt_impl(&self, other: &f64x8) -> mask64x8 {
        unsafe {
            let a : [__m128d; 4] = (*self).into();
            let b : [__m128d; 4] = (*other).into();
            let res = [_mm_castpd_si128(_mm_cmplt_pd(a[0], b[0])), 
                       _mm_castpd_si128(_mm_cmplt_pd(a[1], b[1])),
                       _mm_castpd_si128(_mm_cmplt_pd(a[2], b[2])),
                       _mm_castpd_si128(_mm_cmplt_pd(a[3], b[3]))];
            mask64x8::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_le_impl(&self, other: &f64x8) -> mask64x8 {
        unsafe {
            let a : [__m128d; 4] = (*self).into();
            let b : [__m128d; 4] = (*other).into();
            let res = [_mm_castpd_si128(_mm_cmple_pd(a[0], b[0])), 
                       _mm_castpd_si128(_mm_cmple_pd(a[1], b[1])),
                       _mm_castpd_si128(_mm_cmple_pd(a[2], b[2])),
                       _mm_castpd_si128(_mm_cmple_pd(a[3], b[3]))];
            mask64x8::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_gt_impl(&self, other: &f64x8) -> mask64x8 {
        unsafe {
            let a : [__m128d; 4] = (*self).into();
            let b : [__m128d; 4] = (*other).into();
            let res = [_mm_castpd_si128(_mm_cmpgt_pd(a[0], b[0])), 
                       _mm_castpd_si128(_mm_cmpgt_pd(a[1], b[1])),
                       _mm_castpd_si128(_mm_cmpgt_pd(a[2], b[2])),
                       _mm_castpd_si128(_mm_cmpgt_pd(a[3], b[3]))];
            mask64x8::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_ge_impl(&self, other: &f64x8) -> mask64x8 {
        unsafe {
            let a : [__m128d; 4] = (*self).into();
            let b : [__m128d; 4] = (*other).into();
            let res = [_mm_castpd_si128(_mm_cmpge_pd(a[0], b[0])), 
                       _mm_castpd_si128(_mm_cmpge_pd(a[1], b[1])),
                       _mm_castpd_si128(_mm_cmpge_pd(a[2], b[2])),
                       _mm_castpd_si128(_mm_cmpge_pd(a[3], b[3]))];
            mask64x8::simd_from_int_unchecked::<{BackendType::SSE}>(res.into())
        }
    }

    fn simd_max_impl(self: f64x8, other: f64x8) -> f64x8 {
        unsafe {
            let a : [__m128d; 4] = self.into();
            let b : [__m128d; 4] = other.into();
            let res = [_mm_max_pd(a[0], b[0]),
                       _mm_max_pd(a[1], b[1]),
                       _mm_max_pd(a[2], b[2]),
                       _mm_max_pd(a[3], b[3])];
            res.into()
        }
    }

    fn simd_min_impl(self: f64x8, other: f64x8) -> f64x8 {
        unsafe {
            let a : [__m128d; 4] = self.into();
            let b : [__m128d; 4] = other.into();
            let res = [_mm_min_pd(a[0], b[0]),
                       _mm_min_pd(a[1], b[1]),
                       _mm_min_pd(a[2], b[2]),
                       _mm_min_pd(a[3], b[3])];
            res.into()
        }
    }

    fn simd_clamp_impl(self: f64x8, min: f64x8, max: f64x8) -> f64x8 {
        unsafe {
            let a  : [__m128d; 4] = self.into();
            let mi : [__m128d; 4] = min.into();
            let ma : [__m128d; 4] = max.into();
            let res = [_mm_max_pd(_mm_min_pd(a[0], ma[0]), mi[0]),
                       _mm_max_pd(_mm_min_pd(a[1], ma[1]), mi[1]),
                       _mm_max_pd(_mm_min_pd(a[2], ma[2]), mi[2]),
                       _mm_max_pd(_mm_min_pd(a[3], ma[3]), mi[3])];
            res.into()
        }
    }
}

//==============================================================================================================================
//  UTILITY
//==============================================================================================================================
#[inline(always)]
unsafe fn max_i64(a: __m128i, b: __m128i) -> __m128i {
    _mm_blendv_epi8(a, b, _mm_cmpgt_epi64(a, b))
}

#[inline(always)]
unsafe fn min_i64(a: __m128i, b: __m128i) -> __m128i {
    _mm_blendv_epi8(a, b, _mm_cmpgt_epi64(b, a))
}

#[inline(always)]
unsafe fn clamp_i64(a: __m128i, min: __m128i, max: __m128i) -> __m128i {
    min_i64(max_i64(a, min), max)
}

#[inline(always)]
unsafe fn eq_u64(a: __m128i, b: __m128i, sign_mask: __m128i) -> __m128i {
    _mm_cmpeq_epi64(_mm_xor_si128(a, sign_mask), _mm_xor_si128(b, sign_mask))
}

#[inline(always)]
unsafe fn ne_u64(a: __m128i, b: __m128i, sign_mask: __m128i, xor_mask: __m128i) -> __m128i {
    _mm_xor_si128(_mm_cmpeq_epi64(_mm_xor_si128(a, sign_mask), _mm_xor_si128(b, sign_mask)), xor_mask)
}

#[inline(always)]
unsafe fn lt_u64(a: __m128i, b: __m128i, sign_mask: __m128i) -> __m128i {
    _mm_cmpgt_epi64(_mm_xor_si128(b, sign_mask), _mm_xor_si128(a, sign_mask))
}

#[inline(always)]
unsafe fn le_u64(a: __m128i, b: __m128i, sign_mask: __m128i) -> __m128i {
    let imm_a = _mm_xor_si128(a, sign_mask);
    let imm_b = _mm_xor_si128(b, sign_mask);
    _mm_or_si128(_mm_cmpeq_epi64(imm_a, imm_b), _mm_cmpgt_epi64(imm_b, imm_a))
}

#[inline(always)]
unsafe fn gt_u64(a: __m128i, b: __m128i, sign_mask: __m128i) -> __m128i {
    _mm_cmpgt_epi64(_mm_xor_si128(a, sign_mask), _mm_xor_si128(b, sign_mask))
}

#[inline(always)]
unsafe fn ge_u64(a: __m128i, b: __m128i, sign_mask: __m128i) -> __m128i {
    let imm_a = _mm_xor_si128(a, sign_mask);
    let imm_b = _mm_xor_si128(b, sign_mask);
    _mm_or_si128(_mm_cmpeq_epi64(imm_a, imm_b), _mm_cmpgt_epi64(imm_a, imm_b))
}


#[inline(always)]
unsafe fn max_u64(a: __m128i, b: __m128i, sign_mask: __m128i) -> __m128i {
    let imm_a = _mm_xor_si128(a, sign_mask);
    let imm_b = _mm_xor_si128(b, sign_mask);
    _mm_blendv_epi8(a, b, _mm_cmpgt_epi64(imm_a, imm_b))
}

#[inline(always)]
unsafe fn min_u64(a: __m128i, b: __m128i, sign_mask: __m128i) -> __m128i {
    let imm_a = _mm_xor_si128(a, sign_mask);
    let imm_b = _mm_xor_si128(b, sign_mask);
    _mm_blendv_epi8(a, b, _mm_cmpgt_epi64(imm_b, imm_a))
}

#[inline(always)]
unsafe fn clamp_u64(a: __m128i, min: __m128i, max: __m128i, sign_mask: __m128i) -> __m128i {
    let imm_a = _mm_xor_si128(a, sign_mask);
    let imm_min = _mm_xor_si128(min, sign_mask);
    let imm_max = _mm_xor_si128(max, sign_mask);
    let imm = _mm_blendv_epi8(a, min, _mm_cmpgt_epi64(imm_a, imm_min));
    _mm_blendv_epi8(a, max, _mm_cmpgt_epi64(imm_max, imm_a))
}