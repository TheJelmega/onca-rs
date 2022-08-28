use core::{
    ops::{BitAnd, BitOr, BitXor, Not},
    mem::size_of,
    fmt::Debug
};

use super::{MaskElement, ToBitMask, ToBitMaskArray};
use crate::{
    Simd, SimdElement, LaneCount, SupportedLaneCount, Mask, 
    backend::*,
    DEF_BACKEND_TYPE};

/// Use for bitmask bit order
pub trait ReverseBits {
    /// Reverse the least significant `n` bits of `self`
    /// Remaining bits must be 0
    fn reverse_bits(self, n: usize) -> Self;
}

macro_rules! impl_reverse_bits {
    { $($int:ty),* } => {
        $(
            impl ReverseBits for $int {
                #[inline(always)]
                fn reverse_bits(self, n: usize) -> Self {
                    let rev = <$int>::reverse_bits(self);
                    let bitsize = core::mem::size_of::<$int>() * 8;
                    if n < bitsize {
                        rev >> (bitsize - n)
                    } else {
                        rev
                    }
                }
            }
        )*
    };
}
impl_reverse_bits!{ u8, u16, u32, u64 }


#[repr(transparent)]
pub struct FullMask<T, const LANES: usize>(pub(crate) Simd<T, LANES>)
    where T : SimdElement + MaskElement,
          LaneCount<LANES> : SupportedLaneCount;

impl<T, const LANES: usize> Copy for FullMask<T, LANES>
    where T : SimdElement  + MaskElement,
          LaneCount<LANES> : SupportedLaneCount
{
}

impl<T, const LANES: usize> Clone for FullMask<T, LANES> 
    where T : SimdElement  + MaskElement,
          LaneCount<LANES> : SupportedLaneCount
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, const LANES: usize> PartialEq for FullMask<T, LANES> 
    where T : SimdElement + MaskElement + PartialEq,
          LaneCount<LANES> : SupportedLaneCount,
          Simd<T, LANES> : SimdCmpImpl<DEF_BACKEND_TYPE, MaskT = Mask<T, LANES>> + 
                                 SimdMaskOpsImpl<DEF_BACKEND_TYPE>,
{
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0).all()
    }
}

impl<T, const LANES: usize> Eq for FullMask<T, LANES> 
    where T : SimdElement + MaskElement + Eq,
          LaneCount<LANES> : SupportedLaneCount,
          Simd<T, LANES> : SimdCmpImpl<DEF_BACKEND_TYPE, MaskT = Mask<T, LANES>> +
                           SimdMaskOpsImpl<DEF_BACKEND_TYPE>
{
}

impl<T, const LANES: usize> FullMask<T, LANES> 
    where T : MaskElement,
          LaneCount<LANES> : SupportedLaneCount
{
    #[inline]
    #[must_use]
    pub fn splat<const BACKEND_TYPE: BackendType>(value: bool) -> Self 
        where Simd<T, LANES> : SimdSetImpl<T, BACKEND_TYPE>
    {
        Self(Simd::simd_splat::<BACKEND_TYPE>(if value {T::TRUE} else {T::FALSE}))
    }

    #[inline]
    #[must_use]
    pub unsafe fn test_unchecked(&self, lane: usize) -> bool {
        self.0[lane] == T::TRUE
    }

    #[inline]
    #[must_use]
    pub unsafe fn set_unchecked(&mut self, lane: usize, value: bool) {
        self.0[lane] = if value {T::TRUE} else {T::FALSE}
    }

    #[inline]
    #[must_use]
    pub fn to_int(self) -> Simd<T, LANES> {
        self.0
    }

    #[inline]
    #[must_use]
    pub unsafe fn from_int_unchecked(value: Simd<T, LANES>) -> Self {
        Self(value)
    }

    #[inline]
    #[must_use]
    pub fn convert<U : MaskElement, const BACKEND_TYPE: BackendType>(self) -> FullMask<U, LANES>
        where Simd<T, LANES> : SimdConvertImpl<U, LANES, BACKEND_TYPE>
    {
        FullMask::<U, LANES>(self.0.simd_convert::<U, LANES, BACKEND_TYPE>())
    }

    #[inline]
    #[must_use]
    pub fn to_bitmask_array<const N: usize, const BACKEND_TYPE: BackendType>(self) -> [u8; N] {
        assert_eq!(<super::Mask<T, LANES> as ToBitMaskArray<BACKEND_TYPE>>::BYTES, N);
        todo!()
    }

    pub fn from_bitmask_array<const N: usize, const BACKEND_TYPE: BackendType>(mut bitmask: [u8; N]) -> Self {
        assert_eq!(<super::Mask<T, LANES> as ToBitMaskArray<BACKEND_TYPE>>::BYTES, N);
        todo!()
    }

    pub fn to_bitmask_integer<U: ReverseBits, const BACKEND_TYPE: BackendType>(self) -> U 
        where super::Mask<T, LANES>: ToBitMask<BACKEND_TYPE, BitMask = U>
    {
        todo!()
    }

    pub fn from_bitmask_integer<U: ReverseBits, const BACKEND_TYPE: BackendType>(bitmask: U) -> Self 
        where super::Mask<T, LANES>: ToBitMask<BACKEND_TYPE, BitMask = U>
    {
        todo!()
    }

    pub fn any<const BACKEND_TYPE: BackendType>(self) -> bool
        where Simd<T, LANES> : SimdMaskOpsImpl<BACKEND_TYPE>
    {
        <Simd<T, LANES> as SimdMaskOpsImpl<BACKEND_TYPE>>::simd_any_impl(self.0)
    }

    pub fn all<const BACKEND_TYPE: BackendType>(self) -> bool
        where Simd<T, LANES> : SimdMaskOpsImpl<BACKEND_TYPE>
    {
        <Simd<T, LANES> as SimdMaskOpsImpl<BACKEND_TYPE>>::simd_all_impl(self.0)
    }

    pub fn eq<const BACKEND_TYPE: BackendType>(&self, other: &Self) -> Self 
        where T: SimdElement<Mask = T> + PartialOrd
    {
        unsafe{ self.0.simd_eq(&other.0).full_mask }
    }

    pub fn ne<const BACKEND_TYPE: BackendType>(&self, other: &Self) -> Self 
        where T: SimdElement<Mask = T> + PartialOrd
    {
        unsafe{ self.0.simd_ne(&other.0).full_mask }
    }

    pub fn not<const BACKEND_TYPE: BackendType>(self) -> Self 
        where Simd<T, LANES> : SimdNotImpl<BACKEND_TYPE>
    {
        Self(self.0.simd_not::<BACKEND_TYPE>())
    }

    pub fn and<const BACKEND_TYPE: BackendType>(self, other: Self) -> Self 
        where Simd<T, LANES> : SimdAndImpl<BACKEND_TYPE>
    {
        Self(self.0.simd_and::<BACKEND_TYPE>(other.0))
    }

    pub fn xor<const BACKEND_TYPE: BackendType>(self, other: Self) -> Self 
        where Simd<T, LANES> : SimdXorImpl<BACKEND_TYPE>
    {
        Self(self.0.simd_xor::<BACKEND_TYPE>(other.0))
    }

    pub fn or<const BACKEND_TYPE: BackendType>(self, other: Self) -> Self 
        where Simd<T, LANES> : SimdOrImpl<BACKEND_TYPE>
    {
        Self(self.0.simd_or::<BACKEND_TYPE>(other.0))
    }
}

impl<T, const LANES: usize> From<FullMask<T, LANES>> for Simd<T, LANES> 
    where T : MaskElement,
          LaneCount<LANES> : SupportedLaneCount
{
    fn from(mask: FullMask<T, LANES>) -> Self {
        mask.0
    }
}
impl<T, const LANES: usize> Debug for FullMask<T, LANES> 
    where T : MaskElement,
          LaneCount<LANES> : SupportedLaneCount,
          Simd<T, LANES> : Debug
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}