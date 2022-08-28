#![allow(non_camel_case_types)]

use core::{
    mem,
    ops::{BitAnd, BitOr, BitXor, Not, BitAndAssign, BitOrAssign, BitXorAssign}, fmt::Debug
};
use crate::{
    lanes::*,
    simd::*, 
    backend::*, 
    DEF_BACKEND_TYPE
};

mod full_mask;
mod bitmask;

mod to_bitmask;
pub use to_bitmask::{ToBitMask, ToBitMaskArray, bitmask_len};

pub(crate) mod sealed {
    use crate::{
        lanes::*,
        simd::{Simd, SimdElement}, 
        backend::BackendType 
    };

    pub trait Sealed {
        const TRUE: Self;
        const FALSE: Self;
    }
}
use sealed::Sealed;

pub unsafe trait MaskElement : SimdElement<Mask = Self> + Sealed + PartialEq {}

macro_rules! impl_element {
    { $ty:ty } => {
        impl Sealed for $ty {
            const TRUE: Self = -1;
            const FALSE: Self = 0;
        }

        unsafe impl MaskElement for $ty {}
    };
}

impl_element!{i8}
impl_element!{i16}
impl_element!{i32}
impl_element!{i64}

/// A SIMD vector mask for `LANES` elements of width specified by `Element`
/// 
/// The layout of this type is unspecified
pub union Mask<T, const LANES: usize>
    where T : MaskElement,
          LaneCount<LANES> : SupportedLaneCount
{
    pub(crate) full_mask : full_mask::FullMask<T, LANES>,
    pub(crate) bitmask : bitmask::Mask<T, LANES>
}

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

impl<T, const LANES: usize> Mask<T, LANES> 
    where T : MaskElement,
          LaneCount<LANES> : SupportedLaneCount
{
    /// Construct a mask by setting all lanes to the given value
    #[inline]
    pub fn splat(value: bool) -> Self {
        Self::simd_splat::<DEF_BACKEND_TYPE>(value)
    }

    /// Convert an array of bools to a SIMD mask
    #[inline]
    pub fn from_array(array: [bool; LANES]) -> Self 
        where Simd<T , LANES> : SimdCmpImpl<DEF_BACKEND_TYPE, MaskT = Self> +
                                SimdSetImpl<T, DEF_BACKEND_TYPE>,
              Simd<i8, LANES> : SimdConvertImpl<T, LANES, DEF_BACKEND_TYPE>
    {
        Self::simd_from_array::<DEF_BACKEND_TYPE>(array)
    }

    /// Convert a SIMD mask to an array of bools
    #[inline]
    pub fn to_array(self) -> [bool; LANES] {
        self.simd_to_array::<DEF_BACKEND_TYPE>()
    }

    /// Converts a vector of integers to a mask, where 0 represents `false` and -1 represents `true`
    /// 
    /// # Safety
    /// All lanes must be either 0 or -1
    #[inline]
    pub unsafe fn from_int_unchecked(value: Simd<T, LANES>) -> Self {
        Self::simd_from_int_unchecked::<DEF_BACKEND_TYPE>(value)
    }

    /// Convert a vector of integers to a mask, where 0 represents `false` and -1 represents `true`
    /// 
    /// # Panics
    /// Panics if any lane is not 0 or -1
    #[inline]
    pub fn from_int(value: Simd<T, LANES>) -> Self 
        where Simd<T, LANES> : SimdOrImpl<DEF_BACKEND_TYPE> + 
                               SimdCmpImpl<DEF_BACKEND_TYPE, MaskT = Self> + 
                               SimdMaskOpsImpl<DEF_BACKEND_TYPE> + 
                               SimdSetImpl<T, DEF_BACKEND_TYPE>,
    {
        Self::simd_from_int::<DEF_BACKEND_TYPE>(value)
    }

    /// Convert the mask to a vector of integers, where 0 represents `false` and -1 represents `true`
    #[inline]
    #[must_use]
    pub fn to_int(self) -> Simd<T, LANES> {
        self.simd_to_int::<DEF_BACKEND_TYPE>()
    }

    
    /// Test the value of the specific lane
    /// 
    /// # Safety
    /// `lane` must be less than `LANES`
    #[inline]
    #[must_use]
    pub fn test_unchecked(&self, lane: usize) -> bool {
        self.simd_test_unchecked::<DEF_BACKEND_TYPE>(lane)
    }

    
    /// Test the value of the specific lane
    /// 
    /// # Panics
    /// Panics if `lane` is greater of equal to the number of lanes in the vector
    #[inline]
    #[must_use]
    pub fn test(&self, lane: usize) -> bool {
        self.simd_test::<DEF_BACKEND_TYPE>(lane)
    }

    /// Sets the value of the specific lane
    /// 
    /// # Safety
    /// `lane` must be less than `LANES`
    #[inline]
    pub fn set_unchecked(&mut self, lane: usize, value: bool) {
        self.simd_set_unchecked::<DEF_BACKEND_TYPE>(lane, value)
    }

    /// Sets the value of the specific lane
    /// 
    /// # Panics
    /// Panics if `lane` is greater of equal to the number of lanes in the vector
    #[inline]
    pub fn set(&mut self, lane: usize, value: bool) {
        self.simd_set::<DEF_BACKEND_TYPE>(lane, value)
    }

    /// Returns true if any lane is set, or false otherwise
    #[inline]
    #[must_use]
    pub fn any(self) -> bool
        where Simd<T, LANES> : SimdMaskOpsImpl<DEF_BACKEND_TYPE>
    {
        self.simd_any::<DEF_BACKEND_TYPE>()
    }
    
    /// Returns true if all lanes are set, or false otherwise
    #[inline]
    #[must_use]
    pub fn all(self) -> bool 
        where Simd<T, LANES> : SimdMaskOpsImpl<DEF_BACKEND_TYPE>
    {
        self.simd_all::<DEF_BACKEND_TYPE>()
    }


    pub fn convert<U>(self) -> Mask<U, LANES>
        where U : MaskElement,
              Simd<T, LANES> : SimdConvertImpl<U, LANES, DEF_BACKEND_TYPE>
    {
        self.simd_convert::<U, DEF_BACKEND_TYPE>()
    }
}

impl<T, const LANES: usize> Mask<T, LANES> 
    where T : MaskElement,
          LaneCount<LANES> : SupportedLaneCount
{
    /// Construct a mask by setting all lanes to the given value
    #[inline]
    pub fn simd_splat<const BACKEND_TYPE: BackendType>(value: bool) -> Self {
        if BACKEND_TYPE == BackendType::AVX512 {
            Self{ bitmask: bitmask::Mask::splat(value) }
        } else {
            Self{ full_mask: full_mask::FullMask::splat(value) }
        }
    }

    /// Convert an array of bools to a SIMD mask 
    #[inline]
    pub fn simd_from_array<const BACKEND_TYPE: BackendType>(array: [bool; LANES]) -> Self 
        where Simd<T , LANES> : SimdCmpImpl<BACKEND_TYPE, MaskT = Self> +
                                SimdSetImpl<T, BACKEND_TYPE>,
              Simd<i8, LANES> : SimdConvertImpl<T, LANES, BACKEND_TYPE>
    {
        // SAFETY: Rust's bool has a layout of 1 byte (u8) with a value of
        //     true:    0b_0000_0001
        //     false:   0b_0000_0000
        // Thus, an array of bools is also a valid array of bytes: [u8; N]
        // This would be hypothetically valid as an "in-place" transmute,
        // but these are "dependently-sized" types, so copy elision it is!
        unsafe {
            let bytes: [i8; LANES] = mem::transmute_copy(&array);
            let bools = Simd::<i8, LANES>::from_array(bytes);
            let imm = bools.simd_convert::<T, LANES, BACKEND_TYPE>();
            imm.simd_ne::<BACKEND_TYPE>(&Simd::<T, LANES>::simd_zero::<BACKEND_TYPE>())
        }
    }

    /// Convert a SIMD mask to an array of bools
    pub fn simd_to_array<const BACKEND_TYPE: BackendType>(self) -> [bool; LANES] {
        // This follows mostly the same logic as from_array.
        // SAFETY: Rust's bool has a layout of 1 byte (u8) with a value of
        //     true:    0b_0000_0001
        //     false:   0b_0000_0000
        // Thus, an array of bools is also a valid array of bytes: [u8; N]
        // Since our masks are equal to integers where all bits are set,
        // we can simply convert them to i8s, and then bitand them by the
        // bitpattern for Rust's "true" bool.
        // This would be hypothetically valid as an "in-place" transmute,
        // but these are "dependently-sized" types, so copy elision it is!
        unsafe {
            // let mut bytes : Simd<i8, LANES> = ;
            //bytes &= Simd::splat(1i8);
            //mem::transmute_copy(&bytes);
            todo!()
        }
    }

    /// Converts a vector of integers to a mask, where 0 represents `false` and -1 represents `true`
    /// 
    /// # Safety
    /// All lanes must be either 0 or -1
    pub unsafe fn simd_from_int_unchecked<const BACKEND_TYPE: BackendType>(value: Simd<T, LANES>) -> Self {
        unsafe{ 
            //Self(mask_impl::Mask::from_int_unchecked(value)) 
            if BACKEND_TYPE == BackendType::AVX512 {
                Self{ bitmask: bitmask::Mask::from_int_unchecked(value) }
            } else {
                Self{ full_mask: full_mask::FullMask::from_int_unchecked(value) }
            }
        }
    }

    /// Convert a vector of integers to a mask, where 0 represents `false` and -1 represents `true`
    /// 
    /// # Panics
    /// Panics if any lane is not 0 or -1
    pub fn simd_from_int<const BACKEND_TYPE: BackendType>(value: Simd<T, LANES>) -> Self
        where Simd<T, LANES> : SimdOrImpl<BACKEND_TYPE> + 
                               SimdCmpImpl<BACKEND_TYPE, MaskT = Self> +
                               SimdMaskOpsImpl<BACKEND_TYPE> + 
                               SimdSetImpl<T, BACKEND_TYPE>,
    {
        assert!(Self::is_int_valid::<BACKEND_TYPE>(value));
        unsafe { Self::simd_from_int_unchecked::<BACKEND_TYPE>(value) }
    }

    /// Convert the mask to a vector of integers, where 0 represents `false` and -1 represents `true`
    #[inline]
    #[must_use]
    pub fn simd_to_int<const BACKEND_TYPE: BackendType>(self) -> Simd<T, LANES> {
        if BACKEND_TYPE == BackendType::AVX512 {
            unsafe{ self.bitmask.to_int() }
        } else {
            unsafe{ self.full_mask.to_int() }
        }
    }

    
    /// Test the value of the specific lane
    /// 
    /// # Safety
    /// `lane` must be less than `LANES`
    #[inline]
    #[must_use]
    pub fn simd_test_unchecked<const BACKEND_TYPE: BackendType>(&self, lane: usize) -> bool {
        unsafe {
            if BACKEND_TYPE == BackendType::AVX512 {
                self.bitmask.test_unchecked(lane)
            } else {
                self.full_mask.test_unchecked(lane)
            }
        }
    }

    
    /// Test the value of the specific lane
    /// 
    /// # Panics
    /// Panics if `lane` is greater of equal to the number of lanes in the vector
    #[inline]
    #[must_use]
    pub fn simd_test<const BACKEND_TYPE: BackendType>(&self, lane: usize) -> bool {
        assert!(lane <= LANES, "lane index out of range");
        unsafe{ self.simd_test_unchecked::<BACKEND_TYPE>(lane) }
    }

    /// Sets the value of the specific lane
    /// 
    /// # Safety
    /// `lane` must be less than `LANES`
    #[inline]
    pub fn simd_set_unchecked<const BACKEND_TYPE: BackendType>(&mut self, lane: usize, value: bool) {
        //unsafe{ self.0.set_unchecked(lane, value) }
        unsafe {
            if BACKEND_TYPE == BackendType::AVX512 {
                self.bitmask.set_unchecked(lane, value)
            } else {
                self.full_mask.set_unchecked(lane, value)
            }
        }
    }

    /// Sets the value of the specific lane
    /// 
    /// # Panics
    /// Panics if `lane` is greater of equal to the number of lanes in the vector
    #[inline]
    pub fn simd_set<const BACKEND_TYPE: BackendType>(&mut self, lane: usize, value: bool) {
        assert!(lane <= LANES, "lane index out of range");
        unsafe{ self.simd_set::<BACKEND_TYPE>(lane, value) }
    }

    /// Returns true if any lane is set, or false otherwise
    #[inline]
    #[must_use]
    pub fn simd_any<const BACKEND_TYPE: BackendType>(self) -> bool
        where Simd<T, LANES> : SimdMaskOpsImpl<BACKEND_TYPE>
    {
        unsafe {
            if BACKEND_TYPE == BackendType::AVX512 {
                unsafe{ self.bitmask.any() }
            } else {
                self.full_mask.any::<BACKEND_TYPE>()
            }
        }
    }
    
    /// Returns true if all lanes are set, or false otherwise
    #[inline]
    #[must_use]
    pub fn simd_all<const BACKEND_TYPE: BackendType>(self) -> bool 
        where Simd<T, LANES> : SimdMaskOpsImpl<BACKEND_TYPE>
    {
        unsafe { 
            if BACKEND_TYPE == BackendType::AVX512 {
                self.bitmask.all()
            } else {
                self.full_mask.all::<BACKEND_TYPE>()
            }
        }
    }


    pub fn simd_convert<U, const BACKEND_TYPE: BackendType>(mut self) -> Mask<U, LANES>
        where U : MaskElement,
              Simd<T, LANES> : SimdConvertImpl<U, LANES, BACKEND_TYPE>,
    {
        unsafe {
            if BACKEND_TYPE == BackendType::AVX512 {
                Mask::<U, LANES>{ bitmask: self.bitmask.convert::<U>() }
            } else {
                Mask::<U, LANES>{ full_mask: self.full_mask.convert::<U, BACKEND_TYPE>() }
            }
        }
    }

    pub fn simd_not<const BACKEND_TYPE: BackendType>(self) -> Self
        where Simd<T::Mask, LANES> : SimdNotImpl<BACKEND_TYPE>
    {
        unsafe {
            if BACKEND_TYPE == BackendType::AVX512 {
                todo!()
            } else {
                Self{ full_mask: self.full_mask.not::<BACKEND_TYPE>() }
            }
        }
    }

    pub fn simd_and<const BACKEND_TYPE: BackendType>(self, other: Self) -> Self
        where Simd<T, LANES> : SimdAndImpl<BACKEND_TYPE>
    {
        unsafe {
            if BACKEND_TYPE == BackendType::AVX512 {
                todo!()
            } else {
                Self{ full_mask: self.full_mask.and::<BACKEND_TYPE>(other.full_mask) }
            }
        }
    }

    pub fn simd_xor<const BACKEND_TYPE: BackendType>(self, other: Self) -> Self
        where Simd<T, LANES> : SimdXorImpl<BACKEND_TYPE>
    {
        unsafe {
            if BACKEND_TYPE == BackendType::AVX512 {
                todo!()
            } else {
                Self{ full_mask: self.full_mask.xor::<BACKEND_TYPE>(other.full_mask) }
            }
        }
    }

    pub fn simd_or<const BACKEND_TYPE: BackendType>(self, other: Self) -> Self
        where Simd<T, LANES> : SimdOrImpl<BACKEND_TYPE>
    {
        unsafe {
            if BACKEND_TYPE == BackendType::AVX512 {
                todo!()
            } else {
                Self{ full_mask: self.full_mask.or::<BACKEND_TYPE>(other.full_mask) }
            }
        }
    }

    pub fn simd_eq<const BACKEND_TYPE: BackendType>(&self, other: &Self) -> bool 
        where T : SimdElement<Mask = T>,
              Simd<T, LANES> : SimdCmpImpl<BACKEND_TYPE, MaskT = Self>
    {
        unsafe {
            if BACKEND_TYPE == BackendType::AVX512 {
                self.bitmask == other.bitmask
            } else {
                self.full_mask.eq::<BACKEND_TYPE>(&other.full_mask).all()
            }
        }
    }

    pub fn simd_ne<const BACKEND_TYPE: BackendType>(&self, other: &Self) -> bool 
        where T : SimdElement<Mask = T>,
              Simd<T, LANES> : SimdCmpImpl<BACKEND_TYPE, MaskT = Self>
    {
        unsafe {
            if BACKEND_TYPE == BackendType::AVX512 {
                self.bitmask != other.bitmask
            } else {
                self.full_mask.ne::<BACKEND_TYPE>(&other.full_mask).any()
            }
        }
    }

    fn is_int_valid<const BACKEND_TYPE: BackendType>(values: Simd<T, LANES>) -> bool 
        where Simd<T, LANES> : SimdOrImpl<BACKEND_TYPE> + 
                               SimdCmpImpl<BACKEND_TYPE, MaskT = Self> + 
                               SimdMaskOpsImpl<BACKEND_TYPE> + 
                               SimdSetImpl<T, BACKEND_TYPE>
    {
        values.simd_eq::<BACKEND_TYPE>(&Simd::<T, LANES>::simd_zero::<BACKEND_TYPE>())
            .simd_or(values.simd_eq(&Simd::<T, LANES>::simd_splat::<BACKEND_TYPE>(T::TRUE)))
            .simd_all::<BACKEND_TYPE>()
    }

}

impl<T, const LANES: usize> From<[bool; LANES]> for Mask<T, LANES> 
        where T : MaskElement,
              LaneCount<LANES> : SupportedLaneCount,
              Simd<T , LANES> : SimdCmpImpl<DEF_BACKEND_TYPE, MaskT = Self> + 
                                SimdSetImpl<T, DEF_BACKEND_TYPE>,
              Simd<i8, LANES> : SimdConvertImpl<T, LANES, DEF_BACKEND_TYPE>
{
    fn from(array: [bool; LANES]) -> Self {
        Self::from_array(array)
    }
}

impl<T, const LANES: usize> From<Mask<T, LANES>> for [bool; LANES]
        where T : MaskElement,
          LaneCount<LANES> : SupportedLaneCount
{
    fn from(vector: Mask<T, LANES>) -> Self {
        vector.to_array()
    }
}

impl<T, const LANES: usize> PartialEq for Mask<T, LANES> 
    where T : MaskElement + SimdElement<Mask = T> + PartialEq,
          LaneCount<LANES> : SupportedLaneCount,
          Simd<T, LANES> : SimdCmpImpl<DEF_BACKEND_TYPE, MaskT = Self>
{
    fn eq(&self, other: &Self) -> bool {
        self.simd_eq::<DEF_BACKEND_TYPE>(other)
    }
}

impl<T, const LANES: usize> BitAnd for Mask<T, LANES> 
    where T : MaskElement,
          LaneCount<LANES> : SupportedLaneCount,
          Simd<T, LANES> : SimdAndImpl<DEF_BACKEND_TYPE>
{
    type Output = Self;

    #[inline]
    #[must_use]
    fn bitand(self, rhs: Self) -> Self::Output {
        self.simd_and::<DEF_BACKEND_TYPE>(rhs)
    }
}

impl<T, const LANES: usize> BitAnd<bool> for Mask<T, LANES> 
    where T : MaskElement,
          LaneCount<LANES> : SupportedLaneCount,
          Simd<T, LANES> : SimdAndImpl<DEF_BACKEND_TYPE>
{
    type Output = Self;

    #[inline]
    #[must_use]
    fn bitand(self, rhs: bool) -> Self::Output {
        self & Self::splat(rhs)
    }
}

impl<T, const LANES: usize> BitAnd<Mask<T, LANES>> for bool
    where T : MaskElement,
          LaneCount<LANES> : SupportedLaneCount,
          Simd<T, LANES> : SimdAndImpl<DEF_BACKEND_TYPE>
{
    type Output = Mask<T, LANES>;

    #[inline]
    #[must_use]
    fn bitand(self, rhs: Mask<T, LANES>) -> Self::Output {
        Mask::splat(self) & rhs
    }
}

impl<T, const LANES: usize> BitOr for Mask<T, LANES> 
    where T : MaskElement,
          LaneCount<LANES> : SupportedLaneCount,
          Simd<T, LANES> : SimdOrImpl<DEF_BACKEND_TYPE>
{
    type Output = Self;

    #[inline]
    #[must_use]
    fn bitor(self, rhs: Self) -> Self::Output {
        self.simd_or::<DEF_BACKEND_TYPE>(rhs)
    }
}

impl<T, const LANES: usize> BitOr<bool> for Mask<T, LANES> 
    where T : MaskElement,
          LaneCount<LANES> : SupportedLaneCount,
          Simd<T, LANES> : SimdOrImpl<DEF_BACKEND_TYPE>
{
    type Output = Self;

    #[inline]
    #[must_use]
    fn bitor(self, rhs: bool) -> Self::Output {
        self | Self::splat(rhs)
    }
}

impl<T, const LANES: usize> BitOr<Mask<T, LANES>> for bool
    where T : MaskElement,
          LaneCount<LANES> : SupportedLaneCount,
          Simd<T, LANES> : SimdOrImpl<DEF_BACKEND_TYPE>
{
    type Output = Mask<T, LANES>;

    #[inline]
    #[must_use]
    fn bitor(self, rhs: Mask<T, LANES>) -> Self::Output {
        Mask::splat(self) | rhs
    }
}

impl<T, const LANES: usize> BitXor for Mask<T, LANES> 
    where T : MaskElement,
          LaneCount<LANES> : SupportedLaneCount,
          Simd<T, LANES> : SimdXorImpl<DEF_BACKEND_TYPE>
{
    type Output = Self;

    #[inline]
    #[must_use]
    fn bitxor(self, rhs: Self) -> Self::Output {
        self.simd_xor::<DEF_BACKEND_TYPE>(rhs)
    }
}

impl<T, const LANES: usize> BitXor<bool> for Mask<T, LANES> 
    where T : MaskElement,
          LaneCount<LANES> : SupportedLaneCount,
          Simd<T, LANES> : SimdXorImpl<DEF_BACKEND_TYPE>
{
    type Output = Self;

    #[inline]
    #[must_use]
    fn bitxor(self, rhs: bool) -> Self::Output {
        self ^ Self::splat(rhs)
    }
}

impl<T, const LANES: usize> BitXor<Mask<T, LANES>> for bool
    where T : MaskElement,
          LaneCount<LANES> : SupportedLaneCount,
          Simd<T, LANES> : SimdXorImpl<DEF_BACKEND_TYPE>
{
    type Output = Mask<T, LANES>;

    #[inline]
    #[must_use]
    fn bitxor(self, rhs: Mask<T, LANES>) -> Self::Output {
        Mask::splat(self) ^ rhs
    }
}

impl<T, const LANES: usize> Not for Mask<T, LANES> 
    where T : MaskElement,
          LaneCount<LANES> : SupportedLaneCount,
          Simd<T::Mask, LANES> : SimdNotImpl<DEF_BACKEND_TYPE>
{
    type Output = Self;

    fn not(self) -> Self::Output {
        self.simd_not::<DEF_BACKEND_TYPE>()
    }
}

impl<T, const LANES: usize> BitAndAssign for Mask<T, LANES> 
    where T : MaskElement,
          LaneCount<LANES> : SupportedLaneCount,
          Simd<T, LANES> : SimdAndImpl<DEF_BACKEND_TYPE>
{
    fn bitand_assign(&mut self, rhs: Self) {
        *self = (*self).simd_and::<DEF_BACKEND_TYPE>(rhs)
    }
}

impl<T, const LANES: usize> BitAndAssign<bool> for Mask<T, LANES> 
    where T : MaskElement,
          LaneCount<LANES> : SupportedLaneCount,
          Simd<T, LANES> : SimdAndImpl<DEF_BACKEND_TYPE>
{
    fn bitand_assign(&mut self, rhs: bool) {
        *self &= Mask::splat(rhs)
    }
}

impl<T, const LANES: usize> BitOrAssign for Mask<T, LANES> 
    where T : MaskElement,
          LaneCount<LANES> : SupportedLaneCount,
          Simd<T, LANES> : SimdOrImpl<DEF_BACKEND_TYPE>
{
    fn bitor_assign(&mut self, rhs: Self) {
        *self = (*self).simd_or::<DEF_BACKEND_TYPE>(rhs)
    }
}

impl<T, const LANES: usize> BitOrAssign<bool> for Mask<T, LANES> 
    where T : MaskElement,
          LaneCount<LANES> : SupportedLaneCount,
          Simd<T, LANES> : SimdOrImpl<DEF_BACKEND_TYPE>
{
    fn bitor_assign(&mut self, rhs: bool) {
        *self |= Mask::splat(rhs)
    }
}

impl<T, const LANES: usize> BitXorAssign for Mask<T, LANES> 
    where T : MaskElement,
          LaneCount<LANES> : SupportedLaneCount,
          Simd<T, LANES> : SimdXorImpl<DEF_BACKEND_TYPE>
{
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = (*self).simd_xor::<DEF_BACKEND_TYPE>(rhs)
    }
}

impl<T, const LANES: usize> BitXorAssign<bool> for Mask<T, LANES> 
    where T : MaskElement,
          LaneCount<LANES> : SupportedLaneCount,
          Simd<T, LANES> : SimdXorImpl<DEF_BACKEND_TYPE>
{
    fn bitxor_assign(&mut self, rhs: bool) {
        *self ^= Mask::splat(rhs)
    }
}

impl<T, const LANES: usize> Debug for Mask<T, LANES> 
    where T : MaskElement,
          LaneCount<LANES> : SupportedLaneCount,
          bitmask::Mask<T, LANES> : Debug,
          Simd<T, LANES> : Debug
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("bitmask:" )?;
        unsafe {self.bitmask.fmt(f)? };
        f.write_str("; full_mask: ")?;
        unsafe { self.full_mask.fmt(f) }
    }
}

pub type mask8x8   = Mask<i8, 8>;
pub type mask8x16  = Mask<i8, 16>;
pub type mask8x32  = Mask<i8, 32>;
pub type mask8x64  = Mask<i8, 64>;
pub type mask16x4  = Mask<i16, 4>;
pub type mask16x8  = Mask<i16, 8>;
pub type mask16x16 = Mask<i16, 16>;
pub type mask16x32 = Mask<i16, 32>;
pub type mask32x2  = Mask<i32, 2>;
pub type mask32x4  = Mask<i32, 4>;
pub type mask32x8  = Mask<i32, 8>;
pub type mask32x16 = Mask<i32, 16>;
pub type mask64x2  = Mask<i64, 2>;
pub type mask64x4  = Mask<i64, 4>;
pub type mask64x8  = Mask<i64, 8>;

macro_rules! impl_from {
    { $from:ty => $($to:ty),* } => {
        $(
            impl<const LANES: usize> From<Mask<$from, LANES>>for Mask<$to, LANES>
                where LaneCount<LANES> : SupportedLaneCount,
                      Simd<$from, LANES> : SimdConvertImpl<$to, LANES, DEF_BACKEND_TYPE>
            {
                fn from(value: Mask<$from, LANES>) -> Self {
                    value.convert::<$to>()
                }
            }
        )*
    };
}
impl_from!{ i8 => i16, i32, i64 }
impl_from!{ i16 => i8, i32, i64 }
impl_from!{ i32 => i8, i16, i64 }
impl_from!{ i64 => i8, i16, i32 }