use core::{
    mem::MaybeUninit,
    ptr
};

use crate::{
    LaneCount, SupportedLaneCount,
    SimdElement, 
    Mask,
    backend::*,
    mask::sealed::Sealed, Simd
};

impl<T, const LANES: usize> SimdCmpImpl<{BackendType::Scalar}> for Simd<T, LANES>
    where T : SimdElement + PartialEq + PartialOrd,
          LaneCount<LANES> : SupportedLaneCount,
{
    type MaskT = Mask<T::Mask, LANES>;

    fn simd_eq_impl(&self, other: &Self) -> Self::MaskT {
        unsafe {
            let mut mask = MaybeUninit::<Mask<T::Mask, LANES>>::uninit();
            for i in 0..LANES {
                (*mask.as_mut_ptr()).set_unchecked(i, self[i] == other[i]);
            }
            mask.assume_init()
        }
    }

    fn simd_ne_impl(&self, other: &Self) -> Self::MaskT {
        unsafe {
            let mut mask = MaybeUninit::<Mask<T::Mask, LANES>>::uninit();
            for i in 0..LANES {
                (*mask.as_mut_ptr()).set_unchecked(i, self[i] != other[i]);
            }
            mask.assume_init()
        }
    }

    fn simd_lt_impl(&self, other: &Self) -> Self::MaskT {
        unsafe {
            let mut mask = MaybeUninit::<Mask<T::Mask, LANES>>::uninit();
            for i in 0..LANES {
                (*mask.as_mut_ptr()).set_unchecked(i, self[i] < other[i]);
            }
            mask.assume_init()
        }
    }

    fn simd_le_impl(&self, other: &Self) -> Self::MaskT {
        unsafe {
            let mut mask = MaybeUninit::<Mask<T::Mask, LANES>>::uninit();
            for i in 0..LANES {
                (*mask.as_mut_ptr()).set_unchecked(i, self[i] <= other[i]);
            }
            mask.assume_init()
        }
    }

    fn simd_gt_impl(&self, other: &Self) -> Self::MaskT {
        unsafe {
            let mut mask = MaybeUninit::<Mask<T::Mask, LANES>>::uninit();
            for i in 0..LANES {
                (*mask.as_mut_ptr()).set_unchecked(i, self[i] > other[i]);
            }
            mask.assume_init()
        }
    }

    fn simd_ge_impl(&self, other: &Self) -> Self::MaskT {
        unsafe {
            let mut mask = MaybeUninit::<Mask<T::Mask, LANES>>::uninit();
            for i in 0..LANES {
                (*mask.as_mut_ptr()).set_unchecked(i, self[i] >= other[i]);
            }
            mask.assume_init()
        }
    }

    fn simd_max_impl(self, other: Self) -> Self {
        unsafe {
            let mut mask = MaybeUninit::<Simd<T, LANES>>::uninit();
            for i in 0..LANES {
                (*mask.as_mut_ptr())[i] = if self[i] >= other[i] { self[i] } else { other[i] };
            }
            mask.assume_init()
        }
    }

    fn simd_min_impl(self, other: Self) -> Self {
        unsafe {
            let mut mask = MaybeUninit::<Simd<T, LANES>>::uninit();
            for i in 0..LANES {
                (*mask.as_mut_ptr())[i] = if self[i] <= other[i] { self[i] } else { other[i] };
            }
            mask.assume_init()
        }
    }

    fn simd_clamp_impl(self, min: Self, max: Self) -> Self {
        unsafe {
            let mut mask = MaybeUninit::<Simd<T, LANES>>::uninit();
            for i in 0..LANES {
                (*mask.as_mut_ptr())[i] = if min[i] >= self[i] { min[i] } else { if max[i] <= self[i] { max[i] } else { self[i] } };
            }
            mask.assume_init()
        }
    }
}