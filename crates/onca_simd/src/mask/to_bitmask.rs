use super::{Mask, MaskElement, full_mask, bitmask};
use crate::{LaneCount, SupportedLaneCount, BackendType};

mod sealed {
    pub trait Sealed {}
}
use sealed::Sealed;

impl<T, const LANES: usize> Sealed for Mask<T, LANES>
    where T : MaskElement,
          LaneCount<LANES> : SupportedLaneCount
{
}

/// Converts masks to and from integer bitmasks
/// 
/// Each bit of the bitmask corresponds to a mask lane, starting with the LSB
pub trait ToBitMask<const BACKEND_TYPE : BackendType> : Sealed {
    /// The integer bitmask type
    type BitMask;

    /// Converts a mask to a bitmask
    fn to_bitmask(self) -> Self::BitMask;

    /// Converts a bitmask to a mask
    fn from_bitmask(bitmask: Self::BitMask) -> Self;
}

/// Converts masks to and from byte array bitmasks.
/// 
/// Each bit of the bitmask corresponds to a mask lane, starting with the LSB of the first byte
pub trait ToBitMaskArray<const BACKEND_TYPE : BackendType> : Sealed {
    /// The length of the bitmask array
    const BYTES: usize;

    /// Converts a mask to a bitmask
    fn to_bitmask_array(self) -> [u8; Self::BYTES];

    /// Converts a bitmask to a mask
    fn from_bitmask_array(bitmask: [u8; Self::BYTES]) -> Self;
}

macro_rules! impl_integer_instrinsic {
    { $(impl ToBitMask<BitMask=$int:ty> for Mask<_, $lanes:literal>)* } => {
        $(
            impl<T: MaskElement, const BACKEND_TYPE : BackendType> ToBitMask<BACKEND_TYPE> for Mask<T, $lanes> {
                type BitMask = $int;

                fn to_bitmask(self) -> $int {
                    unsafe {
                        if BACKEND_TYPE == BackendType::AVX512 {
                            core::mem::transmute_copy(&self.bitmask)
                        } else {
                            todo!()
                        }
                    }
                }

                fn from_bitmask(bitmask: $int) -> Self {
                    unsafe {
                        if BACKEND_TYPE == BackendType::AVX512 {
                            Self{ bitmask: core::mem::transmute_copy(&bitmask) }
                        } else {
                            todo!()
                        }
                    }
                }
            }
        )*
    }
}

impl_integer_instrinsic! {
    impl ToBitMask<BitMask=u8> for Mask<_, 2>
    impl ToBitMask<BitMask=u8> for Mask<_, 4>
    impl ToBitMask<BitMask=u8> for Mask<_, 8>
    impl ToBitMask<BitMask=u16> for Mask<_, 16>
    impl ToBitMask<BitMask=u32> for Mask<_, 32>
    impl ToBitMask<BitMask=u64> for Mask<_, 64>
}

pub const fn bitmask_len(lanes: usize) -> usize {
    (lanes + 7) / 8
}

impl<T: MaskElement, const LANES: usize, const BACKEND_TYPE: BackendType> ToBitMaskArray<BACKEND_TYPE> for Mask<T, LANES>
    where LaneCount<LANES> : SupportedLaneCount
{
    const BYTES: usize = bitmask_len(LANES);

    fn to_bitmask_array(self) -> [u8; <Self as ToBitMaskArray<BACKEND_TYPE>>::BYTES] {
        unsafe {
            if BACKEND_TYPE == BackendType::AVX512 {
                core::mem::transmute_copy(&self.bitmask)
            } else {
                todo!()
            }
        }
    }

    fn from_bitmask_array(bitmask: [u8; <Self as ToBitMaskArray<BACKEND_TYPE>>::BYTES]) -> Self {
        unsafe {
            if BACKEND_TYPE == BackendType::AVX512 {
                Self{ bitmask: core::mem::transmute_copy(&bitmask) }
            } else {
                todo!()
            }
        }

    }
}