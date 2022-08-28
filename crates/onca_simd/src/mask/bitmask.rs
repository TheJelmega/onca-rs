use core::{
    marker::PhantomData,
    ops::{BitAnd, BitOr, BitXor, Not},
    fmt::Debug
};

use super::MaskElement;
use crate::{LaneCount, SupportedLaneCount, Simd, ToBitMask, backend::BackendType};

#[repr(transparent)]
pub struct Mask<T, const LANES: usize>(
    <LaneCount<LANES> as SupportedLaneCount>::BitMask,
    PhantomData<T>
)
    where T : MaskElement,
          LaneCount<LANES> : SupportedLaneCount
;

impl<T, const LANES: usize> Copy for Mask<T, LANES>
    where T : MaskElement,
          LaneCount<LANES> : SupportedLaneCount
{
}


impl<T, const LANES: usize> Clone for Mask<T, LANES>
    where T : MaskElement,
          LaneCount<LANES> : SupportedLaneCount
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, const LANES: usize> PartialEq for Mask<T, LANES>
    where T : MaskElement,
          LaneCount<LANES> : SupportedLaneCount
{
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ref() == other.0.as_ref()
    }
}

impl<T, const LANES: usize> Eq for Mask<T, LANES>
    where T : MaskElement,
          LaneCount<LANES> : SupportedLaneCount
{
}

impl<T, const LANES: usize> PartialOrd for Mask<T, LANES>
    where T : MaskElement,
          LaneCount<LANES> : SupportedLaneCount
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.0.as_ref().partial_cmp(other.0.as_ref())
    }
}

impl<T, const LANES: usize> Ord for Mask<T, LANES>
    where T : MaskElement,
          LaneCount<LANES> : SupportedLaneCount
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.as_ref().cmp(other.0.as_ref())
    }
}

impl<T, const LANES: usize> Mask<T, LANES>
    where T : MaskElement,
          LaneCount<LANES> : SupportedLaneCount
{
    #[inline]
    #[must_use]
    pub fn splat(value: bool) -> Self {
        let mut mask = <LaneCount<LANES> as SupportedLaneCount>::BitMask::default();
        if value {
            mask.as_mut().fill(u8::MAX)
        } else {
            mask.as_mut().fill(u8::MIN)
        }
        if LANES % 8 > 0 {
            *mask.as_mut().last_mut().unwrap() &= u8::MAX >> (8 - LANES % 8);
        }
        Self(mask, PhantomData)
    }

    #[inline]
    #[must_use]
    pub unsafe fn test_unchecked(&self, lane: usize) -> bool {
        (self.0.as_ref()[lane / 8] >> (lane % 8)) & 0x01 > 0
    }

    #[inline]
    pub unsafe fn set_unchecked(&mut self, lane: usize, value: bool) {
        self.0.as_mut()[lane / 8] ^= ((value ^ self.test_unchecked(lane)) as u8) << (lane % 8)
    }

    #[inline]
    #[must_use]
    pub unsafe fn to_int(self) -> Simd<T, LANES> {
        todo!()
    }

    #[inline]
    #[must_use]
    pub unsafe fn from_int_unchecked(value: Simd<T, LANES>) -> Self {
        todo!()
    }

    #[inline]
    #[must_use]
    pub fn to_bitmask_array<const N: usize>(self) -> [u8; N] {
        assert!(core::mem::size_of::<Self>() == N);
        unsafe{ core::mem::transmute_copy(&self.0) }
    }

    #[inline]
    #[must_use]
    pub fn from_bitmask_array<const N: usize>(bitmask: [u8; N]) -> Self {
        assert!(core::mem::size_of::<Self>() == N);
        Self(unsafe{ core::mem::transmute_copy(&bitmask) }, PhantomData)
    }

    #[inline]
    #[must_use]
    pub fn to_bitmask_integer<U, const BACKEND_TYPE: BackendType>(self) -> U
        where super::Mask<T, LANES> : ToBitMask<BACKEND_TYPE, BitMask = U>
    {
        unsafe{ core::mem::transmute_copy(&self.0) }
    }

    #[inline]
    pub fn from_bitmask_integer<U, const BACKEND_TYPE: BackendType>(bitmask: U) -> Self
        where super::Mask<T, LANES> : ToBitMask<BACKEND_TYPE, BitMask = U>
    {
        Self(unsafe{ core::mem::transmute_copy(&bitmask) }, PhantomData)
    }

    #[inline]
    #[must_use]
    pub fn convert<U: MaskElement>(self) -> Mask<U, LANES>
    {
        unsafe{ core::mem::transmute_copy(&self) }
    }

    #[inline]
    #[must_use]
    pub fn any(self) -> bool {
        self != Self::splat(false)
    }

    #[inline]
    #[must_use]
    pub fn all(self) -> bool {
        self == Self::splat(true)
    }
}

impl<T, const LANES: usize> BitAnd for Mask<T, LANES>
    where T : MaskElement,
          LaneCount<LANES> : SupportedLaneCount
{
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl<T, const LANES: usize> BitOr for Mask<T, LANES> 
    where T : MaskElement,
          LaneCount<LANES> : SupportedLaneCount
{
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl<T, const LANES: usize> BitXor for Mask<T, LANES> 
    where T : MaskElement,
          LaneCount<LANES> : SupportedLaneCount
{
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl<T, const LANES: usize> Not for Mask<T, LANES> 
    where T : MaskElement,
          LaneCount<LANES> : SupportedLaneCount
{
    type Output = Self;

    fn not(self) -> Self::Output {
        todo!()
    }
}

impl<T, const LANES: usize> Debug for Mask<T, LANES> 
    where T : MaskElement,
          LaneCount<LANES> : SupportedLaneCount
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}