use core::{
    mem::MaybeUninit,
    ptr
};

use crate::{
    LaneCount, SupportedLaneCount,
    MaskElement, Mask,
    backend::{SimdMaskOpsImpl, BackendType},
    mask::sealed::Sealed, Simd
};

impl<T, const LANES: usize> SimdMaskOpsImpl<{BackendType::Scalar}> for Simd<T, LANES>
    where T : MaskElement + PartialEq,
          LaneCount<LANES> : SupportedLaneCount
{
    fn simd_all_impl(self) -> bool {
        for i in 0..LANES {
            if self[i] == T::FALSE {
                return false;
            }
        }
        true
    }

    fn simd_any_impl(self) -> bool {
        for i in 0..LANES {
            if self[i] == T::TRUE {
                return true;
            }
        }
        false
    }
}