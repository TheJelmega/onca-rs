use core::mem::MaybeUninit;
use core::ops::{Index, IndexMut, Add, Sub, Mul, Div, Rem, BitAnd, BitXor, BitOr, Shr, Shl, Not, Neg};

//#[cfg(test)]
use core::fmt::Debug;

use crate::{
    backend::*,
    lanes::*,
    mask::*,
    DEF_BACKEND_TYPE
};

mod sealed {
    pub trait Sealed {}
}
use sealed::Sealed;

/// Marker trait for types that may be used as SIMD register elements.
///
/// # Safety
/// This trait, when implemented, asserts the compiler can monomorphize
/// `#[repr(simd)]` structs with the marked type as an element.
/// Strictly, it is valid to impl if the vector will not be miscompiled.
/// Practically, it is user-unfriendly to impl it if the vector won't compile,
/// even when no soundness guarantees are broken by allowing the user to try.
pub unsafe trait SimdElement : Sealed + Copy + PartialEq + PartialOrd + Default
{
    /// The mask element type corresponding to this element type.
    type Mask : MaskElement;
    type IdxT;
}

impl Sealed for i8 {}
impl Sealed for i16 {}
impl Sealed for i32 {}
impl Sealed for i64 {}
impl Sealed for u8 {}
impl Sealed for u16 {}
impl Sealed for u32 {}
impl Sealed for u64 {}
impl Sealed for f32 {}
impl Sealed for f64 {}

unsafe impl SimdElement for i8 {
    type Mask = i8;
    type IdxT = i8;
}
unsafe impl SimdElement for i16 {
    type Mask = i16;
    type IdxT = i16;
}
unsafe impl SimdElement for i32 {
    type Mask = i32;
    type IdxT = i32;
}
unsafe impl SimdElement for i64 {
    type Mask = i64;
    type IdxT = i64;
}
unsafe impl SimdElement for u8 {
    type Mask = i8;
    type IdxT = i8;
}
unsafe impl SimdElement for u16 {
    type Mask = i16;
    type IdxT = i16;
}
unsafe impl SimdElement for u32 {
    type Mask = i32;
    type IdxT = i32;
}
unsafe impl SimdElement for u64 {
    type Mask = i64;
    type IdxT = i64;
}
unsafe impl SimdElement for f32 {
    type Mask = i32;
    type IdxT = i32;
}
unsafe impl SimdElement for f64 {
    type Mask = i64;
    type IdxT = i64;
}

/// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
/// 
/// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
/// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
/// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | |  S  |  S  | | 1x  | 2x  | 4x  
/// |  SSE   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
/// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 1x  | 2x  
/// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 2x  | | TBD | TBD | | 1x  | 1x  | 2x  
/// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | | TBD | TBD | | 1x  | 1x  | 1x  
/// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
/// 
/// SSE means all SSE extension, up to and including SSE4.2
/// 
/// S : Scalar (Unknown latency, as this partially depends on the compiler)

#[repr(simd)]
pub struct Simd<T, const LANES: usize>([T; LANES])
    where T : SimdElement,
          LaneCount<LANES> : SupportedLaneCount;

impl<T, const LANES: usize> Simd<T, LANES>
    where T : SimdElement,
          LaneCount<LANES> : SupportedLaneCount
{
    /// Number of lanes in this register
    pub const LANES: usize = LANES;

    /// Get the number of lanes in this register
    pub const fn lanes(&self) -> usize {
        LANES
    }

    /// Construct a SIMD register by setting all lanes to 0
    #[must_use]
    #[inline]
    pub fn zero() -> Simd<T, LANES>
        where Self : SimdSetImpl<T, DEF_BACKEND_TYPE>
    {
        Self::simd_zero::<DEF_BACKEND_TYPE>()
    }

    /// Construct a SIMD register by setting all lanes to the given value
    #[must_use]
    #[inline]
    pub fn splat(value: T) -> Self
        where Self : SimdSetImpl<T, DEF_BACKEND_TYPE>
    {
        Self::simd_splat::<DEF_BACKEND_TYPE>(value)
    }

    /// Load a SIMD register from memory
    #[must_use]
    #[inline]
    pub fn load(mem: *const T) -> Self
        where Self : SimdLoadStoreImpl<T, DEF_BACKEND_TYPE>
    {
        Self::simd_load::<DEF_BACKEND_TYPE>(mem)
    }

    /// Store the content from a SIMD register to memory
    #[inline]
    pub fn store(self, mem: *mut T)
        where Self : SimdLoadStoreImpl<T, DEF_BACKEND_TYPE>
    {
        self.simd_store::<DEF_BACKEND_TYPE>(mem)
    }

    /// Returns an array reference containing the entire SIMD register
    pub const fn as_array(&self) -> &[T; LANES] {
        &self.0
    }

    /// Returns a mutable array reference containing the entire SIMD register
    pub fn as_mut_array(&mut self) -> &mut [T; LANES] {
        &mut self.0
    }

    /// Converts an array to a SIMD register
    pub const fn from_array(array: [T; LANES]) -> Self {
        Self(array)
    }

    /// Converts a SIMD register to an array
    pub const fn to_array(self) -> [T; LANES] {
        self.0
    }

    /// Converts a slice to a SIMD register containing `slice[..LANES]`
    /// #Panics
    /// `from_slice` will panic if the slice's `len` is less than the register's `Simd::LANES`
    #[must_use]
    pub const fn from_slice(slice: &[T]) -> Self {
        assert!(slice.len() >= LANES, "slice length must be at least the number of lanes");
        let mut array = [slice[0]; LANES];
        let mut i = 0;
        while i < LANES {
            array[i] = slice[i];
            i += 1;
        }
        Self(array)
    }

    /// Performs lanewise conversion of a SIMD register's elements to another SIMD-valid type.
    /// This follows the semantics of Rust's`as` conversion for casting integers to unsigned integers (interpresitng as the other type, so `-1` to `MAX`),
    /// and from floats to integers (truncating, or saturating at the limits) for each lane, or vice versa
    #[must_use]
    #[inline]
    pub fn cast<U: SimdElement>(self) -> Simd<U, LANES> {
        todo!()
    }

    /// Reads from potential discontiguous indices in `slice` to construct666 a SIMD register.
    /// If an index is out-of-bounds, the lane is instead selected from the `or` register
    #[must_use]
    #[inline]
    pub fn gather_or(slice: &[T], idxs: Simd<u64, LANES>, or: Self) -> Self
        where Self : SimdSetImpl<T, DEF_BACKEND_TYPE>
    {
        Self::simd_gather_or::<DEF_BACKEND_TYPE>(slice, idxs, or)
    }

    /// Reads from potentially discontiguous indices in `slice` to construct a SIMD register
    /// If an index is out-of-bounds, the lane is set to the default value for the type
    #[must_use]
    #[inline]
    pub fn gather_or_default(slice: &[T], idxs: Simd<u64, LANES>) -> Self 
        where T : Default,
              Self : SimdSetImpl<T, DEF_BACKEND_TYPE>
    {
        Self::simd_gather_or_default::<DEF_BACKEND_TYPE>(slice, idxs)
    }

    /// Reads from potentially dicontiguous indices in `slice` to construct a SIMD register.
    /// The mask `enable`s all `true` lanes and disables all `false` lanes
    /// If an index is disabled or is out-of-bounds, the lane is selected from the `or` vector
    #[must_use]
    #[inline]
    pub fn gather_select(slice: &[T], enable: Mask<i64, LANES>, idxs: Simd<u64, LANES>, or: Self) -> Self {
        Self::simd_gather_select::<DEF_BACKEND_TYPE>(slice, enable, idxs, or)
    }

    /// Reads from potentially dscontiguous indices in `slice` to construct a SIMD register
    /// The mask `enable`s all `true` lanes and disables all `false` lanes
    /// If an index is disabled, the lane is selected from the `or` vector
    #[must_use]
    #[inline]
    pub unsafe fn gather_select_unchecked(slice: &[T], enable: Mask<i64, LANES>, idxs: Simd<u64, LANES>, or: Self) -> Self {
        Self::simd_gather_select_unchecked::<DEF_BACKEND_TYPE>(slice, enable, idxs, or)
    }

    /// Writes the values in a SIMD register to potentially discontiguous indices in `slice`
    /// If two lanes in the scattered vector would write to the same index, only the last lane is guaranteed to actually be written
    #[inline]
    pub fn scatter(self, slice: &mut [T], idxs: Simd<u64, LANES>) 
        where Self : SimdSetImpl<T, DEF_BACKEND_TYPE>
    {
        self.simd_scatter::<DEF_BACKEND_TYPE>(slice, idxs)
    }

    /// Writes the values in a SIMD register to multiple potentially discontiguous indices in `slice`
    /// The mask `enable`s all `true` lanes and disables all `false` lanes
    /// If an enable index is out-of-bounds, the lane is not written
    /// If two enabled lanes in the scattered vector would write to the same index, only the last lane is guearanteed to actually be written
    #[inline]
    pub fn scatter_select(self, slice: &mut [T], enable: Mask<i64, LANES>, idxs: Simd<u64, LANES>) {
        self.simd_scatter_select::<DEF_BACKEND_TYPE>(slice, enable, idxs)
    }

    /// Writes the values in a SIMD register to multiple potentially discontiguous indices in `slice`
    /// The mask `enable`s all `true` lanes and disables all `false` lanes
    /// If two enabled lanes in the scattered vector would write to the same index, only the last lane is guearanteed to actually be written
    #[inline]
    pub fn scatter_select_unchecked(self, slice: &mut [T], enable: Mask<i64, LANES>, idxs: Simd<u64, LANES>) {
        self.simd_scatter_select_unchecked::<DEF_BACKEND_TYPE>(slice, enable, idxs)
    }


    /// Convert to a register of a given element and size
    #[inline]
    pub fn convert<U, const TO_LANES: usize>(self) -> Simd<U, TO_LANES>
        where Self : SimdConvertImpl<U, TO_LANES, DEF_BACKEND_TYPE>,
              U : SimdElement,
              LaneCount<TO_LANES> : SupportedLaneCount
    {
        self.simd_convert::<U, TO_LANES, DEF_BACKEND_TYPE>()
    }

    /// Convert to a register of a given element and size, and saturate any values falling out of the current range
    #[inline]
    pub fn convert_saturate<U, const TO_LANES: usize>(self) -> Simd<U, TO_LANES>
        where Self : SimdConvertImpl<U, TO_LANES, DEF_BACKEND_TYPE>,
              U : SimdElement,
              LaneCount<TO_LANES> : SupportedLaneCount
    {
        self.simd_convert_saturate::<U, TO_LANES, DEF_BACKEND_TYPE>()
    }

    /// Extend the lower half into a full register, doubling the number of bits per element
    #[inline]
    pub fn extend_lower(self) -> <Self as SimdExtendCompressImpl<DEF_BACKEND_TYPE>>::ExtendedType
        where Self : SimdExtendCompressImpl<DEF_BACKEND_TYPE>
    {
        self.simd_extend_lower::<DEF_BACKEND_TYPE>()
    }

    /// Extend the upper half into a full register, doubling the number of bits per element
    #[inline]
    pub fn extend_upper(self) -> <Self as SimdExtendCompressImpl<DEF_BACKEND_TYPE>>::ExtendedType
        where Self : SimdExtendCompressImpl<DEF_BACKEND_TYPE>
    {
        self.simd_extend_upper::<DEF_BACKEND_TYPE>()
    }

    /// 'Compress' 2 registers into 1 same sized register, halbing the number of bits per element
    #[inline]
    pub fn compress(a: <Self as SimdExtendCompressImpl<DEF_BACKEND_TYPE>>::ExtendedType,
                    b: <Self as SimdExtendCompressImpl<DEF_BACKEND_TYPE>>::ExtendedType) -> Self
        where Self : SimdExtendCompressImpl<DEF_BACKEND_TYPE>
    {
        Self::simd_compress::<DEF_BACKEND_TYPE>(a, b)
    }

    /// Element-wise equals of 2 registers
    /// 
    /// For more detail about performance, check [`simd_eq`]
    #[inline]
    pub fn eq(&self, other: &Self) -> Mask<T::Mask, LANES>
        where Simd<T, LANES> : SimdCmpImpl<DEF_BACKEND_TYPE, MaskT = Mask<T::Mask, LANES>>
    {
        self.simd_eq::<DEF_BACKEND_TYPE>(other)
    }

    /// Element-wise not-equals of 2 registers
    /// 
    /// For more detail about performance, check [`simd_ne`]
    #[inline]
    pub fn ne(&self, other: &Self) -> Mask<T::Mask, LANES> 
        where Simd<T, LANES> : SimdCmpImpl<DEF_BACKEND_TYPE, MaskT = Mask<T::Mask, LANES>>
    {
        self.simd_ne::<DEF_BACKEND_TYPE>(other)
    }

    /// Element-wise less-than of 2 registers
    /// 
    /// For more detail about performance, check [`simd_lt`]
    #[inline]
    pub fn lt(&self, other: &Self) -> Mask<T::Mask, LANES> 
        where Simd<T, LANES> : SimdCmpImpl<DEF_BACKEND_TYPE, MaskT = Mask<T::Mask, LANES>>
    {
        self.simd_lt::<DEF_BACKEND_TYPE>(other)
    }

    /// Element-wise less-or-equal-to of 2 registers
    /// 
    /// For more detail about performance, check [`simd_le`]
    #[inline]
    pub fn le(&self, other: &Self) -> Mask<T::Mask, LANES> 
        where Simd<T, LANES> : SimdCmpImpl<DEF_BACKEND_TYPE, MaskT = Mask<T::Mask, LANES>>
    {
        self.simd_le::<DEF_BACKEND_TYPE>(other)
    }

    /// Element-wise greater-than of 2 registers
    /// 
    /// For more detail about performance, check [`simd_gt`]
    #[inline]
    pub fn gt(&self, other: &Self) -> Mask<T::Mask, LANES> 
        where Simd<T, LANES> : SimdCmpImpl<DEF_BACKEND_TYPE, MaskT = Mask<T::Mask, LANES>>
    {
        self.simd_gt::<DEF_BACKEND_TYPE>(other)
    }
    /// Element-wise less-or-equal-to of 2 registers
    /// 
    /// For more detail about performance, check [`simd_ge`]
    #[inline]
    pub fn ge(&self, other: &Self) -> Mask<T::Mask, LANES> 
        where Simd<T, LANES> : SimdCmpImpl<DEF_BACKEND_TYPE, MaskT = Mask<T::Mask, LANES>>
    {
        self.simd_ge::<DEF_BACKEND_TYPE>(other)
    }

    /// Calculate the element-wise max of 2 registers
    /// 
    /// For more detail about performance, check [`simd_max`]
    #[inline]
    pub fn max(self, other: Self) -> Self 
        where Simd<T, LANES> : SimdCmpImpl<DEF_BACKEND_TYPE>
    {
        self.simd_max::<DEF_BACKEND_TYPE>(other)
    }

    /// Calculate the element-wise min of 2 registers
    /// 
    /// For more detail about performance, check [`simd_min`]
    #[inline]
    pub fn min(self, other: Self) -> Self 
        where Simd<T, LANES> : SimdCmpImpl<DEF_BACKEND_TYPE>
    {
        self.simd_min::<DEF_BACKEND_TYPE>(other)
    }

    /// Element-wise clamp a register between the values in a `min` and `max` register
    /// 
    /// For more detail about performance, check [`simd_min`]
    #[inline]
    pub fn clamp(self, min: Self, max: Self) -> Self 
        where Simd<T, LANES> : SimdCmpImpl<DEF_BACKEND_TYPE>
    {
        self.simd_clamp::<DEF_BACKEND_TYPE>(min, max)
    }

    /// Element-wise clamp a register between the values in a `min` and `max` register
    /// 
    /// For more detail about performance, check [`simd_andnot`]
    #[inline]
    pub fn andnot(self, other: Self) -> Self 
        where Self : SimdAndNotImpl<DEF_BACKEND_TYPE>
    {
        self.simd_andnot::<DEF_BACKEND_TYPE>(other)
    }

    /// Logical shift right by a simd register
    ///
    /// For more detail about performance, check [`simd_shrl`]
    #[inline]
    pub fn shrl(self, other: Self) -> Simd<T, LANES>
        where Self : SimdShiftImpl<DEF_BACKEND_TYPE>
    {
        self.simd_shrl::<DEF_BACKEND_TYPE>(other)
    }

    /// Arithmatic shift right by a simd register
    ///
    /// For more detail about performance, check [`simd_shra`]
    #[inline]
    pub fn shra(self, other: Self) -> Simd<T, LANES> 
        where Self : SimdShiftImpl<DEF_BACKEND_TYPE>
    {
        self.simd_shra::<DEF_BACKEND_TYPE>(other)
    }

    /// Shift left by a scalar value
    ///
    /// For more detail about performance, check [`simd_shl_scalar`]
    #[inline]
    pub fn shl_scalar(self, shift: u8) -> Simd<T, LANES> 
        where Self : SimdShiftImpl<DEF_BACKEND_TYPE>
    {
        self.simd_shl_scalar::<DEF_BACKEND_TYPE>(shift)
    }

    /// Logical shift right by a scalar value
    ///
    /// For more detail about performance, check [`simd_shrl_scalar`]
    #[inline]
    pub fn shrl_scalar(self, shift: u8) -> Simd<T, LANES> 
        where Self : SimdShiftImpl<DEF_BACKEND_TYPE>
    {
        self.simd_shrl_scalar::<DEF_BACKEND_TYPE>(shift)
    }

    /// Arithmatic shift right by a scalar value
    ///
    /// For more detail about performance, check [`simd_shra_scalar`]
    #[inline]
    pub fn shra_scalar(self, shift: u8) -> Simd<T, LANES> 
        where Self : SimdShiftImpl<DEF_BACKEND_TYPE>
    {
        self.simd_shra_scalar::<DEF_BACKEND_TYPE>(shift)
    }

    /// Element-wise floor
    ///
    /// For more detail about performance, check [`simd_floor`]
    #[inline]
    pub fn floor(self) -> Simd<T, LANES> 
        where Self : SimdFloorImpl<DEF_BACKEND_TYPE>
    {
        self.simd_floor::<DEF_BACKEND_TYPE>()
    }

    /// Element-wise ceil
    ///
    /// For more detail about performance, check [`simd_ceil`]
    #[inline]
    pub fn ceil(self) -> Simd<T, LANES> 
        where Self : SimdCeilImpl<DEF_BACKEND_TYPE>
    {
        self.simd_ceil::<DEF_BACKEND_TYPE>()
    }

    /// Element-wise round
    ///
    /// For more detail about performance, check [`simd_round`]
    #[inline]
    pub fn round(self) -> Simd<T, LANES> 
        where Self : SimdRoundImpl<DEF_BACKEND_TYPE>
    {
        self.simd_round::<DEF_BACKEND_TYPE>()
    }

    /// Element-wise sqaure root
    ///
    /// For more detail about performance, check [`simd_sqrt`]
    #[inline]
    pub fn sqrt(self) -> Simd<T, LANES> 
        where Self : SimdSqrtImpl<DEF_BACKEND_TYPE>
    {
        self.simd_sqrt::<DEF_BACKEND_TYPE>()
    }

    /// Element-wise reciprocal square root
    ///
    /// For more detail about performance, check [`simd_rsqrt`]
    #[inline]
    pub fn rsqrt(self) -> Simd<T, LANES> 
        where Self : SimdRsqrtImpl<DEF_BACKEND_TYPE>
    {
        self.simd_rsqrt::<DEF_BACKEND_TYPE>()
    }

    /// Element-wise reciprocal square root (aproximated, i.e. less precision but could be faster)
    ///
    /// For more detail about performance, check [`simd_rsqrt`]
    #[inline]
    pub fn rsqrt_approx(self) -> Simd<T, LANES> 
        where Self : SimdRsqrtImpl<DEF_BACKEND_TYPE>
    {
        self.simd_rsqrt::<DEF_BACKEND_TYPE>()
    }

    /// Element-wise reciprocal
    ///
    /// For more detail about performance, check [`simd_rcp`]
    #[inline]
    pub fn rcp(self) -> Simd<T, LANES> 
        where Self : SimdRcpImpl<DEF_BACKEND_TYPE>
    {
        self.simd_rcp::<DEF_BACKEND_TYPE>()
    }

    /// Element-wise reciprocal (aproximated, i.e. less precision but could be faster)
    ///
    /// For more detail about performance, check [`simd_rcp_approc`]
    #[inline]
    pub fn rcp_approx(self) -> Simd<T, LANES> 
        where Self : SimdRcpImpl<DEF_BACKEND_TYPE>
    {
        self.simd_rcp::<DEF_BACKEND_TYPE>()
    }

    /// Element-wise absolute value
    ///
    /// For more detail about performance, check [`simd_abs`]
    #[inline]
    pub fn abs(self) -> Simd<T, LANES> 
        where Self : SimdAbsImpl<DEF_BACKEND_TYPE>
    {
        self.simd_abs::<DEF_BACKEND_TYPE>()
    }
}

impl<T, const LANES: usize> Simd<T, LANES>
    where T : SimdElement,
          LaneCount<LANES> : SupportedLaneCount
{
    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    /// 
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | |  S  |  S  | | 1x  | 2x  | 4x  
    /// |  SSE   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 1x  | 2x  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 2x  | | TBD | TBD | | 1x  | 1x  | 2x  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | | TBD | TBD | | 1x  | 1x  | 1x  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    #[inline]
    pub fn simd_zero<const BACKEND_TYPE: BackendType>() -> Self
        where Self : SimdSetImpl<T, BACKEND_TYPE>
    {
        <Self as SimdSetImpl<T, BACKEND_TYPE>>::simd_zero_impl()
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    /// 
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | |  S  |  S  | | 1x  | 2x  | 4x  
    /// |  SSE   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 1x  | 2x  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 2x  | | TBD | TBD | | 1x  | 1x  | 2x  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | | TBD | TBD | | 1x  | 1x  | 1x  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    #[inline]
    pub fn simd_splat<const BACKEND_TYPE: BackendType>(val: T) -> Self
        where Self : SimdSetImpl<T, BACKEND_TYPE>
    {
        <Self as SimdSetImpl<T, BACKEND_TYPE>>::simd_splat_impl(val)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    /// 
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | |  S  |  S  | | 1x  | 2x  | 4x  
    /// |  SSE   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 1x  | 2x  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 2x  | | TBD | TBD | | 1x  | 1x  | 2x  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | | TBD | TBD | | 1x  | 1x  | 1x  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    #[inline]
    pub fn simd_load<const BACKEND_TYPE: BackendType>(mem: *const T) -> Self
        where Self : SimdLoadStoreImpl<T, BACKEND_TYPE>
    {
        <Self as SimdLoadStoreImpl<T, BACKEND_TYPE>>::simd_load_impl(mem)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    /// 
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | |  S  |  S  | | 1x  | 2x  | 4x  
    /// |  SSE   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 1x  | 2x  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 2x  | | TBD | TBD | | 1x  | 1x  | 2x  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | | TBD | TBD | | 1x  | 1x  | 1x  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    #[inline]
    pub fn simd_store<const BACKEND_TYPE: BackendType>(self, mem: *mut T)
        where Self : SimdLoadStoreImpl<T, BACKEND_TYPE>
    {
        <Self as SimdLoadStoreImpl<T, BACKEND_TYPE>>::simd_store_impl(self, mem)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    /// 
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | |  S  |  S  | | 1x  | 2x  | 4x  
    /// |  SSE   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 1x  | 2x  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 2x  | | TBD | TBD | | 1x  | 1x  | 2x  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | | TBD | TBD | | 1x  | 1x  | 1x  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    #[must_use]
    #[inline]
    pub fn simd_gather_or<const BACKEND_TYPE: BackendType>(slice: &[T], idxs: Simd<u64, LANES>, or: Self) -> Self 
        where Self : SimdSetImpl<T, BACKEND_TYPE>
    {
        Self::simd_gather_select::<BACKEND_TYPE>(slice, Mask::simd_splat::<BACKEND_TYPE>(true), idxs, or)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    /// 
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | |  S  |  S  | | 1x  | 2x  | 4x  
    /// |  SSE   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 1x  | 2x  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 2x  | | TBD | TBD | | 1x  | 1x  | 2x  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | | TBD | TBD | | 1x  | 1x  | 1x  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    #[must_use]
    #[inline]
    pub fn simd_gather_or_default<const BACKEND_TYPE: BackendType>(slice: &[T], idxs: Simd<u64, LANES>) -> Self 
        where T : Default,
              Self : SimdSetImpl<T, BACKEND_TYPE>
    {
        Self::simd_gather_or::<BACKEND_TYPE>(slice, idxs, Self::simd_splat::<BACKEND_TYPE>(T::default()))
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    /// 
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | |  S  |  S  | | 1x  | 2x  | 4x  
    /// |  SSE   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 1x  | 2x  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 2x  | | TBD | TBD | | 1x  | 1x  | 2x  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | | TBD | TBD | | 1x  | 1x  | 1x  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    #[must_use]
    #[inline]
    pub fn simd_gather_select<const BACKEND_TYPE: BackendType>(slize: &[T], enable: Mask<i64, LANES>, idxs: Simd<u64, LANES>, or: Self) -> Self {
        todo!()
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    /// 
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | |  S  |  S  | | 1x  | 2x  | 4x  
    /// |  SSE   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 1x  | 2x  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 2x  | | TBD | TBD | | 1x  | 1x  | 2x  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | | TBD | TBD | | 1x  | 1x  | 1x  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    #[must_use]
    #[inline]
    pub unsafe fn simd_gather_select_unchecked<const BACKEND_TYPE: BackendType>(slice: &[T], enable: Mask<i64, LANES>, idxs: Simd<u64, LANES>, or: Self) -> Self {
        todo!()
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    /// 
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | |  S  |  S  | | 1x  | 2x  | 4x  
    /// |  SSE   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 1x  | 2x  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 2x  | | TBD | TBD | | 1x  | 1x  | 2x  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | | TBD | TBD | | 1x  | 1x  | 1x  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    #[inline]
    pub fn simd_scatter<const BACKEND_TYPE: BackendType>(self, slice: &mut [T], idxs: Simd<u64, LANES>) 
        where Self : SimdSetImpl<T, BACKEND_TYPE>
    {
        self.simd_scatter_select::<BACKEND_TYPE>(slice, Mask::simd_splat::<BACKEND_TYPE>(true), idxs)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    /// 
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | |  S  |  S  | | 1x  | 2x  | 4x  
    /// |  SSE   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 1x  | 2x  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 2x  | | TBD | TBD | | 1x  | 1x  | 2x  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | | TBD | TBD | | 1x  | 1x  | 1x  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    #[inline]
    pub fn simd_scatter_select<const BACKEND_TYPE: BackendType>(self, slice: &mut [T], enable: Mask<i64, LANES>, idxs: Simd<u64, LANES>) {
        todo!()
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    /// 
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | |  S  |  S  | | 1x  | 2x  | 4x  
    /// |  SSE   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 1x  | 2x  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 2x  | | TBD | TBD | | 1x  | 1x  | 2x  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | | TBD | TBD | | 1x  | 1x  | 1x  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    #[inline]
    pub fn simd_scatter_select_unchecked<const BACKEND_TYPE: BackendType>(self, slice: &mut [T], enable: Mask<i64, LANES>, idxs: Simd<u64, LANES>) {
        todo!()
    }



    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    /// 
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | |  S  |  S  | | 1x  | 2x  | 4x  
    /// |  SSE   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 1x  | 2x  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 2x  | | TBD | TBD | | 1x  | 1x  | 2x  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | | TBD | TBD | | 1x  | 1x  | 1x  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    #[inline]
    pub fn simd_convert<To, const TO_LANES: usize, const BACKEND_TYPE: BackendType>(self) -> Simd<To, TO_LANES> 
        where Self : SimdConvertImpl<To, TO_LANES, BACKEND_TYPE>,
              To: SimdElement,
              LaneCount<TO_LANES> : SupportedLaneCount
    {
        <Self as SimdConvertImpl<To, TO_LANES, BACKEND_TYPE>>::simd_convert_impl(self)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    /// 
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | |  S  |  S  | | 1x  | 2x  | 4x  
    /// |  SSE   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 1x  | 2x  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 2x  | | TBD | TBD | | 1x  | 1x  | 2x  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | | TBD | TBD | | 1x  | 1x  | 1x  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    #[inline]
    pub fn simd_convert_saturate<To, const TO_LANES: usize, const BACKEND_TYPE: BackendType>(self) -> Simd<To, TO_LANES> 
        where Self : SimdConvertImpl<To, TO_LANES, BACKEND_TYPE>,
              To: SimdElement,
              LaneCount<TO_LANES> : SupportedLaneCount
    {
        <Self as SimdConvertImpl<To, TO_LANES, BACKEND_TYPE>>::simd_convert_saturate_impl(self)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    /// 
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  SSE   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | TBD | TBD | | N/A | N/A | | N/A | N/A | N/A  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | TBD | TBD | | N/A | N/A | | N/A | N/A | N/A  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    #[inline]
    pub fn simd_extend_lower<const BACKEND_TYPE: BackendType>(self) -> <Self as SimdExtendCompressImpl<BACKEND_TYPE>>::ExtendedType
        where Self : SimdExtendCompressImpl<BACKEND_TYPE>
    {
        <Self as SimdExtendCompressImpl<BACKEND_TYPE>>::simd_extend_lower_impl(self)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    /// 
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  SSE   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | TBD | TBD | | N/A | N/A | | N/A | N/A | N/A  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | TBD | TBD | | N/A | N/A | | N/A | N/A | N/A  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    #[inline]
    pub fn simd_extend_upper<const BACKEND_TYPE: BackendType>(self) -> <Self as SimdExtendCompressImpl<BACKEND_TYPE>>::ExtendedType
        where Self : SimdExtendCompressImpl<BACKEND_TYPE>
    {
        <Self as SimdExtendCompressImpl<BACKEND_TYPE>>::simd_extend_upper_impl(self)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    /// 
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  SSE   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | TBD | TBD | | N/A | N/A | | N/A | N/A | N/A  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | TBD | TBD | | N/A | N/A | | N/A | N/A | N/A  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    #[inline]
    pub fn simd_compress<const BACKEND_TYPE: BackendType>(a: <Self as SimdExtendCompressImpl<BACKEND_TYPE>>::ExtendedType,
                                                          b: <Self as SimdExtendCompressImpl<BACKEND_TYPE>>::ExtendedType) -> Self
        where Self : SimdExtendCompressImpl<BACKEND_TYPE>
    {
        <Self as SimdExtendCompressImpl<BACKEND_TYPE>>::simd_compress_impl(a, b)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    /// 
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | |  S  |  S  | | 1x  | 2x  | 4x  
    /// |  SSE   |  1  |  1  |  1  |  1  |  1  |  1  |  1  |  1  | | 1x  | 2x  | 4x  | | 3-4 | 3-4 | | 1x  | 2x  | 4x  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | 3-4 | 3-4 | | 1x  | 1x  | 2x  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 2x  | | 3-4 | 3-4 | | 1x  | 1x  | 2x  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | | 3-4 | 3-4 | | 1x  | 1x  | 1x  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    #[inline]
    pub fn simd_eq<const BACKEND_TYPE: BackendType>(&self, other: &Self) -> Mask<T::Mask, LANES>
        where Self : SimdCmpImpl<BACKEND_TYPE, MaskT = Mask<T::Mask, LANES>>
    {
        <Self as SimdCmpImpl<BACKEND_TYPE>>::simd_eq_impl(self, other)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    /// 
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | |  S  |  S  | | 1x  | 2x  | 4x  
    /// |  SSE   |  1  |  1  |  1  |  1  |  1  |  1  |  1  |  1  | | 1x  | 2x  | 4x  | | 3-4 | 3-4 | | 1x  | 2x  | 4x  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | 3-4 | 3-4 | | 1x  | 1x  | 2x  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 2x  | | 3-4 | 3-4 | | 1x  | 1x  | 2x  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | | 3-4 | 3-4 | | 1x  | 1x  | 1x  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    #[inline]
    pub fn simd_ne<const BACKEND_TYPE: BackendType>(&self, other: &Self) -> Mask<T::Mask, LANES>
        where Self : SimdCmpImpl<BACKEND_TYPE, MaskT = Mask<T::Mask, LANES>>
    {
        <Self as SimdCmpImpl<BACKEND_TYPE>>::simd_ne_impl(self, other)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    /// 
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | |  S  |  S  | | 1x  | 2x  | 4x  
    /// |  SSE   |  3  |  3  |  3  |  5  |  3  |  3  |  3  |  5  | | 1x  | 2x  | 4x  | | 3-4 | 3-4 | | 1x  | 2x  | 4x  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | 1-5 | | 1x  | 2x  | 4x  | | 3-4 | 3-4 | | 1x  | 1x  | 2x  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | 1-5 | | 1x  | 1x  | 2x  | | 3-4 | 3-4 | | 1x  | 1x  | 2x  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | 1-5 | | 1x  | 1x  | 1x  | | 3-4 | 3-4 | | 1x  | 1x  | 1x  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    #[inline]
    pub fn simd_lt<const BACKEND_TYPE: BackendType>(&self, other: &Self) -> Mask<T::Mask, LANES>
        where Self : SimdCmpImpl<BACKEND_TYPE, MaskT = Mask<T::Mask, LANES>>
    {
        <Self as SimdCmpImpl<BACKEND_TYPE>>::simd_lt_impl(self, other)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    /// 
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | |  S  |  S  | | 1x  | 2x  | 4x  
    /// |  SSE   |  1  |  1  |  1  | 1-5 |  1  |  1  |  1  | 1-5 | | 1x  | 2x  | 4x  | | 3-4 | 3-4 | | 1x  | 2x  | 4x  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | 3-4 | 3-4 | | 1x  | 1x  | 2x  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 2x  | | 3-4 | 3-4 | | 1x  | 1x  | 2x  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | | 3-4 | 3-4 | | 1x  | 1x  | 1x  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    #[inline]
    pub fn simd_le<const BACKEND_TYPE: BackendType>(&self, other: &Self) -> Mask<T::Mask, LANES>
        where Self : SimdCmpImpl<BACKEND_TYPE, MaskT = Mask<T::Mask, LANES>>
    {
        <Self as SimdCmpImpl<BACKEND_TYPE>>::simd_le_impl(self, other)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    /// 
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | |  S  |  S  | | 1x  | 2x  | 4x  
    /// |  SSE   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | 3-4 | 3-4 | | 1x  | 2x  | 4x  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | 3-4 | 3-4 | | 1x  | 1x  | 2x  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 2x  | | 3-4 | 3-4 | | 1x  | 1x  | 2x  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | | 3-4 | 3-4 | | 1x  | 1x  | 1x  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    #[inline]
    pub fn simd_gt<const BACKEND_TYPE: BackendType>(&self, other: &Self) -> Mask<T::Mask, LANES>
        where Self : SimdCmpImpl<BACKEND_TYPE, MaskT = Mask<T::Mask, LANES>>
    {
        <Self as SimdCmpImpl<BACKEND_TYPE>>::simd_gt_impl(self, other)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    /// 
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | |  S  |  S  | | 1x  | 2x  | 4x  
    /// |  SSE   |  1  |  1  |  1  | 1-5 |  1  |  1  |  1  | 1-5 | | 1x  | 2x  | 4x  | | 3-4 | 3-4 | | 1x  | 2x  | 4x  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | 3-4 | 3-4 | | 1x  | 1x  | 2x  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 2x  | | 3-4 | 3-4 | | 1x  | 1x  | 2x  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | | 3-4 | 3-4 | | 1x  | 1x  | 1x  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    #[inline]
    pub fn simd_ge<const BACKEND_TYPE: BackendType>(&self, other: &Self) -> Mask<T::Mask, LANES>
        where Self : SimdCmpImpl<BACKEND_TYPE, MaskT = Mask<T::Mask, LANES>>
    {
        <Self as SimdCmpImpl<BACKEND_TYPE>>::simd_ge_impl(self, other)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    /// 
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | |  S  |  S  | | 1x  | 2x  | 4x  
    /// |  SSE   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 1x  | 2x  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 2x  | | TBD | TBD | | 1x  | 1x  | 2x  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | | TBD | TBD | | 1x  | 1x  | 1x  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    #[inline]
    pub fn simd_max<const BACKEND_TYPE: BackendType>(self, other: Self) -> Self 
        where Self : SimdCmpImpl<BACKEND_TYPE>
    {
        <Self as SimdCmpImpl<BACKEND_TYPE>>::simd_max_impl(self, other)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    /// 
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | |  S  |  S  | | 1x  | 2x  | 4x  
    /// |  SSE   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 1x  | 2x  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 2x  | | TBD | TBD | | 1x  | 1x  | 2x  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | | TBD | TBD | | 1x  | 1x  | 1x  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    #[inline]
    pub fn simd_min<const BACKEND_TYPE: BackendType>(self, other: Self) -> Self 
        where Self : SimdCmpImpl<BACKEND_TYPE>
    {
        <Self as SimdCmpImpl<BACKEND_TYPE>>::simd_min_impl(self, other)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    /// 
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | |  S  |  S  | | 1x  | 2x  | 4x  
    /// |  SSE   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 1x  | 2x  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 2x  | | TBD | TBD | | 1x  | 1x  | 2x  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | | TBD | TBD | | 1x  | 1x  | 1x  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    #[inline]
    pub fn simd_clamp<const BACKEND_TYPE: BackendType>(self, min: Self, max: Self) -> Self 
        where Self : SimdCmpImpl<BACKEND_TYPE>
    {
        <Self as SimdCmpImpl<BACKEND_TYPE>>::simd_clamp_impl(self, min, max)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    /// 
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | |  S  |  S  | | 1x  | 2x  | 4x  
    /// |  SSE   |  2  |  2  |  2  |  2  |  2  |  2  |  2  |  2  | | 1x  | 2x  | 4x  | | 4-5 | 4-5 | | 1x  | 2x  | 4x  
    /// |  AVX   |  2  |  2  |  2  |  2  |  2  |  2  |  2  |  2  | | 1x  | 2x  | 4x  | | 4-5 | 4-5 | | 1x  | 1x  | 2x  
    /// |  AVX2  |  2  |  2  |  2  |  2  |  2  |  2  |  2  |  2  | | 1x  | 1x  | 2x  | | 4-5 | 4-5 | | 1x  | 1x  | 2x  
    /// | AVX512 |  2  |  2  |  2  |  2  |  2  |  2  |  2  |  2  | | 1x  | 1x  | 1x  | |  5  |  5  | | 1x  | 1x  | 1x  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    #[inline]
    pub fn simd_neg<const BACKEND_TYPE: BackendType>(self) -> Self 
        where Self : SimdNegImpl<BACKEND_TYPE>
    {
        <Self as SimdNegImpl<BACKEND_TYPE>>::simd_neg_impl(self)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    /// 
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | |  S  |  S  | | 1x  | 2x  | 4x  
    /// |  SSE   |  1  |  1  |  1  |  1  |  1  |  1  |  1  |  1  | | 1x  | 2x  | 4x  | | 3-4 | 3-4 | | 1x  | 2x  | 4x  
    /// |  AVX   |  1  |  1  |  1  |  1  |  1  |  1  |  1  |  1  | | 1x  | 2x  | 4x  | | 3-4 | 3-4 | | 1x  | 1x  | 2x  
    /// |  AVX2  |  1  |  1  |  1  |  1  |  1  |  1  |  1  |  1  | | 1x  | 1x  | 2x  | | 3-4 | 3-4 | | 1x  | 1x  | 2x  
    /// | AVX512 |  1  |  1  |  1  |  1  |  1  |  1  |  1  |  1  | | 1x  | 1x  | 1x  | |  4  |  4  | | 1x  | 1x  | 1x  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    #[inline]
    pub fn simd_add<const BACKEND_TYPE: BackendType>(self, other: Self) -> Self 
        where Self : SimdAddImpl<BACKEND_TYPE>
    {
        <Self as SimdAddImpl<BACKEND_TYPE>>::simd_add_impl(self, other)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    /// 
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | |  S  |  S  | | 1x  | 2x  | 4x  
    /// |  SSE   |  1  |  1  |  1  |  1  |  1  |  1  |  1  |  1  | | 1x  | 2x  | 4x  | | 3-4 | 3-4 | | 1x  | 2x  | 4x  
    /// |  AVX   |  1  |  1  |  1  |  1  |  1  |  1  |  1  |  1  | | 1x  | 2x  | 4x  | | 3-4 | 3-4 | | 1x  | 1x  | 2x  
    /// |  AVX2  |  1  |  1  |  1  |  1  |  1  |  1  |  1  |  1  | | 1x  | 1x  | 2x  | | 3-4 | 3-4 | | 1x  | 1x  | 2x  
    /// | AVX512 |  1  |  1  |  1  |  1  |  1  |  1  |  1  |  1  | | 1x  | 1x  | 1x  | | 3-4 | 3-4 | | 1x  | 1x  | 1x  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    #[inline]
    pub fn simd_sub<const BACKEND_TYPE: BackendType>(self, other: Self) -> Self
        where Self : SimdSubImpl<BACKEND_TYPE>
    {
        <Self as SimdSubImpl<BACKEND_TYPE>>::simd_sub_impl(self, other)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    /// 
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | |  S  |  S  | | 1x  | 2x  | 4x  
    /// |  SSE   | TBD | TBD |  5  | TBD | TBD | TBD |  5  | TBD | | 1x  | 2x  | 4x  | | 3-5 | 4-5 | | 1x  | 2x  | 4x  
    /// |  AVX   | TBD | TBD |  5  | TBD | TBD | TBD |  5  | TBD | | 1x  | 2x  | 4x  | | 3-5 | 3-4 | | 1x  | 1x  | 2x  
    /// |  AVX2  | TBD | TBD |  5  | TBD | TBD | TBD |  5  | TBD | | 1x  | 1x  | 2x  | | 3-5 | 3-4 | | 1x  | 1x  | 2x  
    /// | AVX512 | TBD | TBD |  5  | TBD | TBD | TBD |  5  | TBD | | 1x  | 1x  | 1x  | | 3-5 | 3-5 | | 1x  | 1x  | 1x  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    #[inline]
    pub fn simd_mul<const BACKEND_TYPE: BackendType>(self, other: Self) -> Self
        where Self : SimdMulImpl<BACKEND_TYPE>
    {
        <Self as SimdMulImpl<BACKEND_TYPE>>::simd_mul_impl(self, other)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    /// 
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | |  S  |  S  | | 1x  | 2x  | 4x  
    /// |  SSE   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | |11-14|14-20| | 1x  | 2x  | 4x  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | TBD | TBD | |11-21|14-35| | 1x  | 1x  | 2x  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | TBD | TBD | |11-21|14-35| | 1x  | 1x  | 2x  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | |11-23|14-35| | 1x  | 1x  | 1x  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    #[inline]
    pub fn simd_div<const BACKEND_TYPE: BackendType>(self, other: Self) -> Self
        where Self : SimdDivImpl<BACKEND_TYPE>
    {
        <Self as SimdDivImpl<BACKEND_TYPE>>::simd_div_impl(self, other)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    /// 
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | |  S  |  S  | | 1x  | 2x  | 4x  
    /// |  SSE   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | TBD | TBD | | TBD | TBD | | 1x  | 1x  | 2x  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | TBD | TBD | | TBD | TBD | | 1x  | 1x  | 2x  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | | TBD | TBD | | 1x  | 1x  | 1x  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | TBD | TBD | | 1x  | 2x  | 4x  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    #[inline]
    pub fn simd_rem<const BACKEND_TYPE: BackendType>(self, other: Self) -> Self
        where Self : SimdRemImpl<BACKEND_TYPE>
    {
        <Self as SimdRemImpl<BACKEND_TYPE>>::simd_rem_impl(self, other)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    /// 
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | |  S  |  S  | | 1x  | 2x  | 4x  
    /// |  SSE   | 2-7 | 2-7 | 2-7 | 2-7 | 2-7 | 2-7 | 2-7 | 2-7 | | 1x  | 2x  | 4x  | | N/A | N/A | | 1x  | 2x  | 4x  
    /// |  AVX   | 2-7 | 2-7 | 2-7 | 2-7 | 2-7 | 2-7 | 2-7 | 2-7 | | 1x  | 2x  | 4x  | | N/A | N/A | | 1x  | 1x  | 2x  
    /// |  AVX2  | 2-7 | 2-7 | 2-7 | 2-7 | 2-7 | 2-7 | 2-7 | 2-7 | | 1x  | 1x  | 2x  | | N/A | N/A | | 1x  | 1x  | 2x  
    /// | AVX512 | 2-7 | 2-7 | 2-7 | 2-7 | 2-7 | 2-7 | 2-7 | 2-7 | | 1x  | 1x  | 1x  | | N/A | N/A | | 1x  | 1x  | 1x  
    /// |  NEON  | 2-7 | 2-7 | 2-7 | 2-7 | 2-7 | 2-7 | 2-7 | 2-7 | | 1x  | 2x  | 4x  | | N/A | N/A | | 1x  | 2x  | 4x  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    #[inline]
    pub fn simd_not<const BACKEND_TYPE: BackendType>(self) -> Self
        where Self : SimdNotImpl<BACKEND_TYPE>
    {
        <Self as SimdNotImpl<BACKEND_TYPE>>::simd_not_impl(self)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    /// 
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A 
    /// |  SSE   |  1  |  1  |  1  |  1  |  1  |  1  |  1  |  1  | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A 
    /// |  AVX   |  1  |  1  |  1  |  1  |  1  |  1  |  1  |  1  | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A 
    /// |  AVX2  |  1  |  1  |  1  |  1  |  1  |  1  |  1  |  1  | | 1x  | 1x  | 2x  | | N/A | N/A | | N/A | N/A | N/A 
    /// | AVX512 |  1  |  1  |  1  |  1  |  1  |  1  |  1  |  1  | | 1x  | 1x  | 1x  | | N/A | N/A | | N/A | N/A | N/A 
    /// |  NEON  |  1  |  1  |  1  |  1  |  1  |  1  |  1  |  1  | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A 
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    #[inline]
    pub fn simd_and<const BACKEND_TYPE: BackendType>(self, other: Self) -> Self
        where Self : SimdAndImpl<BACKEND_TYPE>
    {
        <Self as SimdAndImpl<BACKEND_TYPE>>::simd_and_impl(self, other)
    }

    // Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    /// 
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A 
    /// |  SSE   |  1  |  1  |  1  |  1  |  1  |  1  |  1  |  1  | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A 
    /// |  AVX   |  1  |  1  |  1  |  1  |  1  |  1  |  1  |  1  | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A 
    /// |  AVX2  |  1  |  1  |  1  |  1  |  1  |  1  |  1  |  1  | | 1x  | 1x  | 2x  | | N/A | N/A | | N/A | N/A | N/A 
    /// | AVX512 |  1  |  1  |  1  |  1  |  1  |  1  |  1  |  1  | | 1x  | 1x  | 1x  | | N/A | N/A | | N/A | N/A | N/A 
    /// |  NEON  |  1  |  1  |  1  |  1  |  1  |  1  |  1  |  1  | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A 
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    #[inline]
    pub fn simd_xor<const BACKEND_TYPE: BackendType>(self, other: Self) -> Self
        where Self : SimdXorImpl<BACKEND_TYPE>
    {
        <Self as SimdXorImpl<BACKEND_TYPE>>::simd_xor_impl(self, other)
    }

    // Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    /// 
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A 
    /// |  SSE   |  1  |  1  |  1  |  1  |  1  |  1  |  1  |  1  | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A 
    /// |  AVX   |  1  |  1  |  1  |  1  |  1  |  1  |  1  |  1  | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A 
    /// |  AVX2  |  1  |  1  |  1  |  1  |  1  |  1  |  1  |  1  | | 1x  | 1x  | 2x  | | N/A | N/A | | N/A | N/A | N/A 
    /// | AVX512 |  1  |  1  |  1  |  1  |  1  |  1  |  1  |  1  | | 1x  | 1x  | 1x  | | N/A | N/A | | N/A | N/A | N/A 
    /// |  NEON  |  1  |  1  |  1  |  1  |  1  |  1  |  1  |  1  | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A 
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    #[inline]
    pub fn simd_or<const BACKEND_TYPE: BackendType>(self, other: Self) -> Self
        where Self : SimdOrImpl<BACKEND_TYPE>
    {
        <Self as SimdOrImpl<BACKEND_TYPE>>::simd_or_impl(self, other)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    /// 
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A 
    /// |  SSE   |  1  |  1  |  1  |  1  |  1  |  1  |  1  |  1  | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A 
    /// |  AVX   |  1  |  1  |  1  |  1  |  1  |  1  |  1  |  1  | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A 
    /// |  AVX2  |  1  |  1  |  1  |  1  |  1  |  1  |  1  |  1  | | 1x  | 1x  | 2x  | | N/A | N/A | | N/A | N/A | N/A 
    /// | AVX512 |  1  |  1  |  1  |  1  |  1  |  1  |  1  |  1  | | 1x  | 1x  | 1x  | | N/A | N/A | | N/A | N/A | N/A 
    /// |  NEON  |  1  |  1  |  1  |  1  |  1  |  1  |  1  |  1  | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A 
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    #[inline]
    pub fn simd_andnot<const BACKEND_TYPE: BackendType>(self, other: Self) -> Self
        where Self : SimdAndNotImpl<BACKEND_TYPE>
    {
        <Self as SimdAndNotImpl<BACKEND_TYPE>>::simd_andnot_impl(self, other)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    /// 
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  SSE   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | TBD | TBD | | N/A | N/A | | N/A | N/A | N/A  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | TBD | TBD | | N/A | N/A | | N/A | N/A | N/A  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    #[inline]
    pub fn simd_shl<const BACKEND_TYPE: BackendType>(self, other: Self) -> Self
        where Self : SimdShiftImpl<BACKEND_TYPE>
    {
        <Self as SimdShiftImpl<BACKEND_TYPE>>::simd_shl_impl(self, other)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    /// 
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  SSE   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | TBD | TBD | | N/A | N/A | | N/A | N/A | N/A  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | TBD | TBD | | N/A | N/A | | N/A | N/A | N/A  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    pub fn simd_shrl<const BACKEND_TYPE: BackendType>(self, other: Self) -> Self
        where Self : SimdShiftImpl<BACKEND_TYPE>
    {
        <Self as SimdShiftImpl<BACKEND_TYPE>>::simd_shrl_impl(self, other)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    /// 
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  SSE   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | TBD | TBD | | N/A | N/A | | N/A | N/A | N/A  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | TBD | TBD | | N/A | N/A | | N/A | N/A | N/A  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    pub fn simd_shra<const BACKEND_TYPE: BackendType>(self, other: Self) -> Self
        where Self : SimdShiftImpl<BACKEND_TYPE>
    {
        <Self as SimdShiftImpl<BACKEND_TYPE>>::simd_shra_impl(self, other)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    /// 
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  SSE   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | TBD | TBD | | N/A | N/A | | N/A | N/A | N/A  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | TBD | TBD | | N/A | N/A | | N/A | N/A | N/A  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    pub fn simd_shl_scalar<const BACKEND_TYPE: BackendType>(self, other: u8) -> Self
        where Self : SimdShiftImpl<BACKEND_TYPE>
    {
        <Self as SimdShiftImpl<BACKEND_TYPE>>::simd_shl_scalar_impl(self, other)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    /// 
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  SSE   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | TBD | TBD | | N/A | N/A | | N/A | N/A | N/A  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | TBD | TBD | | N/A | N/A | | N/A | N/A | N/A  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    pub fn simd_shrl_scalar<const BACKEND_TYPE: BackendType>(self, other: u8) -> Self
        where Self : SimdShiftImpl<BACKEND_TYPE>
    {
        <Self as SimdShiftImpl<BACKEND_TYPE>>::simd_shrl_scalar_impl(self, other)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    ///
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  SSE   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | TBD | TBD | | N/A | N/A | | N/A | N/A | N/A  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | TBD | TBD | | N/A | N/A | | N/A | N/A | N/A  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    pub fn simd_shra_scalar<const BACKEND_TYPE: BackendType>(self, other: u8) -> Self
        where Self : SimdShiftImpl<BACKEND_TYPE>
    {
        <Self as SimdShiftImpl<BACKEND_TYPE>>::simd_shra_scalar_impl(self, other)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    ///
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  SSE   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | TBD | TBD | | N/A | N/A | | N/A | N/A | N/A  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | TBD | TBD | | N/A | N/A | | N/A | N/A | N/A  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    pub fn simd_floor<const BACKEND_TYPE: BackendType>(self) -> Self
        where Self : SimdFloorImpl<BACKEND_TYPE>
    {
        <Self as SimdFloorImpl<BACKEND_TYPE>>::simd_floor_impl(self)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    ///
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  SSE   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | TBD | TBD | | N/A | N/A | | N/A | N/A | N/A  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | TBD | TBD | | N/A | N/A | | N/A | N/A | N/A  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    pub fn simd_ceil<const BACKEND_TYPE: BackendType>(self) -> Self
        where Self : SimdCeilImpl<BACKEND_TYPE>
    {
        <Self as SimdCeilImpl<BACKEND_TYPE>>::simd_ceil_impl(self)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    ///
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  SSE   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | TBD | TBD | | N/A | N/A | | N/A | N/A | N/A  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | TBD | TBD | | N/A | N/A | | N/A | N/A | N/A  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    pub fn simd_round<const BACKEND_TYPE: BackendType>(self) -> Self
        where Self : SimdRoundImpl<BACKEND_TYPE>
    {
        <Self as SimdRoundImpl<BACKEND_TYPE>>::simd_round_impl(self)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    ///
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  SSE   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | TBD | TBD | | N/A | N/A | | N/A | N/A | N/A  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | TBD | TBD | | N/A | N/A | | N/A | N/A | N/A  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    pub fn simd_sqrt<const BACKEND_TYPE: BackendType>(self) -> Self
        where Self : SimdSqrtImpl<BACKEND_TYPE>
    {
        <Self as SimdSqrtImpl<BACKEND_TYPE>>::simd_sqrt_impl(self)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    ///
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  SSE   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | TBD | TBD | | N/A | N/A | | N/A | N/A | N/A  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | TBD | TBD | | N/A | N/A | | N/A | N/A | N/A  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    pub fn simd_rsqrt<const BACKEND_TYPE: BackendType>(self) -> Self
        where Self : SimdRsqrtImpl<BACKEND_TYPE>
    {
        <Self as SimdRsqrtImpl<BACKEND_TYPE>>::simd_rsqrt_impl(self)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    ///
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  SSE   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | TBD | TBD | | N/A | N/A | | N/A | N/A | N/A  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | TBD | TBD | | N/A | N/A | | N/A | N/A | N/A  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    pub fn simd_rsqrt_approx<const BACKEND_TYPE: BackendType>(self) -> Self
        where Self : SimdRsqrtImpl<BACKEND_TYPE>
    {
        <Self as SimdRsqrtImpl<BACKEND_TYPE>>::simd_rsqrt_approx_impl(self)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    ///
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  SSE   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | TBD | TBD | | N/A | N/A | | N/A | N/A | N/A  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | TBD | TBD | | N/A | N/A | | N/A | N/A | N/A  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    pub fn simd_rcp<const BACKEND_TYPE: BackendType>(self) -> Self
        where Self : SimdRcpImpl<BACKEND_TYPE>
    {
        <Self as SimdRcpImpl<BACKEND_TYPE>>::simd_rcp_impl(self)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    ///
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  SSE   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | TBD | TBD | | N/A | N/A | | N/A | N/A | N/A  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | TBD | TBD | | N/A | N/A | | N/A | N/A | N/A  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    pub fn simd_rcp_approx<const BACKEND_TYPE: BackendType>(self) -> Self
        where Self : SimdRcpImpl<BACKEND_TYPE>
    {
        <Self as SimdRcpImpl<BACKEND_TYPE>>::simd_rcp_approx_impl(self)
    }

    /// Performance (in cycles, numbers represent estimated latency, not including throughput, and are therefore not 100% accurate and are meant as a guide)
    ///
    /// | intrin | u8  | u16 | u32 | u64 | i8  | i16 | i32 | i64 | | 128 | 256 | 512 | | f32 | f64 | | 128 | 256 | 512 
    /// |--------|-----|-----|-----|-----|-----|-----|-----|-----|-|-----|-----|-----|-|-----|-----|-|-----|-----|-----
    /// | scalar |  S  |  S  |  S  |  S  |  S  |  S  |  S  |  S  | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  SSE   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  AVX   | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | TBD | TBD | | N/A | N/A | | N/A | N/A | N/A  
    /// |  AVX2  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | TBD | TBD | | N/A | N/A | | N/A | N/A | N/A  
    /// | AVX512 | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 1x  | 1x  | | N/A | N/A | | N/A | N/A | N/A  
    /// |  NEON  | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | | 1x  | 2x  | 4x  | | N/A | N/A | | N/A | N/A | N/A  
    /// 
    /// SSE means all SSE extension, up to and including SSE4.2
    /// 
    /// S : Scalar (Unknown latency, as this partially depends on the compiler)
    pub fn simd_abs<const BACKEND_TYPE: BackendType>(self) -> Self
        where Self : SimdAbsImpl<BACKEND_TYPE>
    {
        <Self as SimdAbsImpl<BACKEND_TYPE>>::simd_abs_impl(self)
    }
}

macro_rules! impl_split_2 {
    ($([$ty:ty, $from_lanes:literal => $to_lanes:literal])*) => {
        $(
            impl Simd<$ty, $from_lanes> {
                /// Split either a 256-bit register into 2 128-bit registers, or a 512-bit register into 2 256-bit registers
                #[must_use]
                #[inline]
                pub fn split_2(self) -> [Simd<$ty, $to_lanes>; 2] {
                    unsafe{ core::mem::transmute_copy(&self) }
                }
            }
        )*
    };
}
impl_split_2!{
    [i8 , 32 => 16]
    [i8 , 64 => 32]
    [i16, 16 => 8 ]
    [i16, 32 => 16]
    [i32, 8  => 4 ]
    [i32, 16 => 8 ]
    [i64, 4  => 2 ]
    [i64, 8  => 4 ]
    [u8 , 32 => 16]
    [u8 , 64 => 32]
    [u16, 16 => 8 ]
    [u16, 32 => 16]
    [u32, 8  => 4 ]
    [u32, 16 => 8 ]
    [u64, 4  => 2 ]
    [u64, 8  => 4 ]
    [f32, 8  => 4 ]
    [f32, 16 => 8 ]
    [f64, 4  => 2 ]
    [f64, 8  => 4 ]
}

macro_rules! impl_split_4 {
    ($([$ty:ty, $from_lanes:literal => $to_lanes:literal])*) => {
        $(
            impl Simd<$ty, $from_lanes> {
                /// Split a 512-bit register into 4 128-bit registers
                #[must_use]
                #[inline]
                pub fn split_4(self) -> [Simd<$ty, $to_lanes>; 4] {
                    unsafe{ core::mem::transmute_copy(&self) }
                }
            }
        )*
    };
}
impl_split_4!{
    [i8 , 64 => 16]
    [i16, 32 => 8 ]
    [i32, 16 => 4 ]
    [i64, 8  => 2 ]
    [u8 , 64 => 16]
    [u16, 32 => 8 ]
    [u32, 16 => 4 ]
    [u64, 8  => 2 ]
    [f32, 16 => 4 ]
    [f64, 8  => 2 ]
}

macro_rules! impl_combine_2 {
    ($([$ty:ty, $from_lanes:literal => $to_lanes:literal])*) => {
        $(
            impl Simd<$ty, $to_lanes> {
                // Combines either 2 128-bit registers into a 256-bit register or 2 256-bit registers into a 512-bit register
                #[must_use]
                #[inline]
                pub fn combine_2(from: [Simd<$ty, $from_lanes>; 2]) -> Self {
                    unsafe{ core::mem::transmute_copy(&from) }
                }
            }

            impl From<[Simd<$ty, $from_lanes>; 2]> for Simd<$ty, $to_lanes> {
                fn from(arr: [Simd<$ty, $from_lanes>; 2]) -> Self {
                    Self::combine_2(arr)
                }
            }
        )*
    };
}
impl_combine_2!{
    [i8 , 16 => 32]
    [i8 , 32 => 64]
    [i16, 8  => 16]
    [i16, 16 => 32]
    [i32, 4  => 8 ]
    [i32, 8  => 16]
    [i64, 2  => 4 ]
    [i64, 4  => 8 ]
    [u8 , 16 => 32]
    [u8 , 32 => 64]
    [u16, 8  => 16]
    [u16, 16 => 32]
    [u32, 4  => 8 ]
    [u32, 8  => 16]
    [u64, 2  => 4 ]
    [u64, 4  => 8 ]
    [f32, 4  => 8 ]
    [f32, 8  => 16]
    [f64, 2  => 4 ]
    [f64, 4  => 8 ]
}

macro_rules! impl_combine_4 {
    ($([$ty:ty, $from_lanes:literal => $to_lanes:literal])*) => {
        $(
            impl Simd<$ty, $to_lanes> {
                // Combines 4 128-bit registers into a 512-bit register
                #[must_use]
                #[inline]
                pub fn combine_4(from: [Simd<$ty, $from_lanes>; 4]) -> Self {
                    unsafe{ core::mem::transmute_copy(&from) }
                }
            }

            impl From<[Simd<$ty, $from_lanes>; 4]> for Simd<$ty, $to_lanes> {
                fn from(arr: [Simd<$ty, $from_lanes>; 4]) -> Self {
                    Self::combine_4(arr)
                }
            }
        )*
    };
}
impl_combine_4!{
    [i8 , 16 => 64]
    [i16, 8  => 32]
    [i32, 4  => 16]
    [i64, 2  => 8 ]
    [u8 , 16 => 64]
    [u16, 8  => 32]
    [u32, 4  => 16]
    [u64, 2  => 8 ]
    [f32, 4  => 16]
    [f64, 2  => 8 ]
}

impl<T, const LANES: usize> Copy for Simd<T, LANES>
    where T : SimdElement,
          LaneCount<LANES> : SupportedLaneCount
{
}

impl<T, const LANES: usize> Clone for Simd<T, LANES>
    where T : SimdElement,
          LaneCount<LANES> : SupportedLaneCount
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, const LANES: usize> Default for Simd<T, LANES>
    where T : SimdElement + Default,
          LaneCount<LANES> : SupportedLaneCount,
          Self : SimdSetImpl<T, DEF_BACKEND_TYPE>
{
    #[inline]
    fn default() -> Self {
        Self::splat(T::default())
    }
}

impl<T, const LANES: usize> PartialEq for Simd<T, LANES>
    where T : SimdElement + PartialEq,
          LaneCount<LANES> : SupportedLaneCount,
          Simd<T::Mask, LANES> : SimdMaskOpsImpl<DEF_BACKEND_TYPE>,
          Simd<T, LANES> : SimdCmpImpl<DEF_BACKEND_TYPE, MaskT = Mask<T::Mask, LANES>>
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.eq(other).all()
    }

    #[inline]
    fn ne(&self, other: &Self) -> bool {
        self.ne(other).any()
    }
}

impl<T, const LANES: usize> Eq for Simd<T, LANES>
    where T : SimdElement + Eq,
          LaneCount<LANES> : SupportedLaneCount,
          Simd<T::Mask, LANES> : SimdMaskOpsImpl<DEF_BACKEND_TYPE>,
          Simd<T, LANES> : SimdCmpImpl<DEF_BACKEND_TYPE, MaskT = Mask<T::Mask, LANES>>
{
}

impl<T, const LANES: usize> core::hash::Hash for Simd<T, LANES>
    where T : SimdElement + core::hash::Hash,
          LaneCount<LANES> : SupportedLaneCount
{
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.as_array().hash(state)
    }
}

impl<T, const LANES: usize> AsRef<[T; LANES]> for Simd<T, LANES>
    where T : SimdElement,
          LaneCount<LANES> : SupportedLaneCount
{
    #[inline]
    fn as_ref(&self) -> &[T; LANES] {
        &self.0
    }
}

impl<T, const LANES: usize> AsMut<[T; LANES]> for Simd<T, LANES>
    where T : SimdElement,
          LaneCount<LANES> : SupportedLaneCount
{
    #[inline]
    fn as_mut(&mut self) -> &mut [T; LANES] {
        &mut self.0
    }
}

impl<T, const LANES: usize> AsRef<[T]> for Simd<T, LANES>
    where T : SimdElement,
          LaneCount<LANES> : SupportedLaneCount
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

impl<T, const LANES: usize> AsMut<[T]> for Simd<T, LANES>
    where T : SimdElement,
          LaneCount<LANES> : SupportedLaneCount
{
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        &mut self.0
    }
}

impl<T, const LANES: usize> From<[T; LANES]> for Simd<T, LANES>
    where T : SimdElement,
          LaneCount<LANES> : SupportedLaneCount
{
    fn from(array: [T; LANES]) -> Self {
        Self(array)
    }
}

impl<T, const LANES: usize> From<Simd<T, LANES>> for [T; LANES]
    where T : SimdElement,
          LaneCount<LANES> : SupportedLaneCount
{
    fn from(vector: Simd<T, LANES>) -> Self {
        vector.to_array()
    }
}

impl<T, I, const LANES: usize> Index<I> for Simd<T, LANES>
    where T : SimdElement,
          I : core::slice::SliceIndex<[T]>,
          LaneCount<LANES> : SupportedLaneCount
{
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.as_array()[index]
    }
}

impl<T, I, const LANES: usize> IndexMut<I> for Simd<T, LANES>
    where T : SimdElement,
          I : core::slice::SliceIndex<[T]>,
          LaneCount<LANES> : SupportedLaneCount
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.as_mut_array()[index]
    }
}

impl<T, const LANES: usize> Neg for Simd<T, LANES> 
    where T : SimdElement,
          LaneCount<LANES> : SupportedLaneCount,
          Self : SimdNegImpl<DEF_BACKEND_TYPE>
{
    type Output = Simd<T, LANES>;

    fn neg(self) -> Self::Output {
        self.simd_neg::<DEF_BACKEND_TYPE>()
    }
}

impl<T, const LANES: usize> Add for Simd<T, LANES> 
    where T : SimdElement,
          LaneCount<LANES> : SupportedLaneCount,
          Self : SimdAddImpl<DEF_BACKEND_TYPE>
{
    type Output = Simd<T, LANES>;

    fn add(self, rhs: Self) -> Self::Output {
        self.simd_add::<DEF_BACKEND_TYPE>(rhs)
    }
}

impl<T, const LANES: usize> Sub for Simd<T, LANES> 
    where T : SimdElement,
          LaneCount<LANES> : SupportedLaneCount,
          Self : SimdSubImpl<DEF_BACKEND_TYPE>
{
    type Output = Simd<T, LANES>;

    fn sub(self, rhs: Self) -> Self::Output {
        self.simd_sub::<DEF_BACKEND_TYPE>(rhs)
    }
}

impl<T, const LANES: usize> Mul for Simd<T, LANES> 
    where T : SimdElement,
          LaneCount<LANES> : SupportedLaneCount,
          Self : SimdMulImpl<DEF_BACKEND_TYPE>
{
    type Output = Simd<T, LANES>;

    fn mul(self, rhs: Self) -> Self::Output {
        self.simd_mul::<DEF_BACKEND_TYPE>(rhs)
    }
}

impl<T, const LANES: usize> Div for Simd<T, LANES> 
    where T : SimdElement,
          LaneCount<LANES> : SupportedLaneCount,
          Self : SimdDivImpl<DEF_BACKEND_TYPE>
{
    type Output = Simd<T, LANES>;

    
    fn div(self, rhs: Self) -> Self::Output {
        self.simd_div::<DEF_BACKEND_TYPE>(rhs)
    }
}

impl<T, const LANES: usize> Rem for Simd<T, LANES> 
    where T : SimdElement,
          LaneCount<LANES> : SupportedLaneCount,
          Self : SimdRemImpl<DEF_BACKEND_TYPE>
{
    type Output = Simd<T, LANES>;
    
    fn rem(self, rhs: Self) -> Self::Output {
        self.simd_rem::<DEF_BACKEND_TYPE>(rhs)
    }
}

impl<T, const LANES: usize> Not for Simd<T, LANES> 
    where T : SimdElement,
          LaneCount<LANES> : SupportedLaneCount,
          Self : SimdNotImpl<DEF_BACKEND_TYPE>
{
    type Output = Simd<T, LANES>;

    fn not(self) -> Self::Output {
        self.simd_not::<DEF_BACKEND_TYPE>()
    }
}

impl<T, const LANES: usize> BitAnd for Simd<T, LANES> 
    where T : SimdElement,
          LaneCount<LANES> : SupportedLaneCount,
          Self : SimdAndImpl<DEF_BACKEND_TYPE>
{
    type Output = Simd<T, LANES>;

    fn bitand(self, rhs: Self) -> Self::Output {
        self.simd_and::<DEF_BACKEND_TYPE>(rhs)
    }
}

impl<T, const LANES: usize> BitXor for Simd<T, LANES> 
    where T : SimdElement,
          LaneCount<LANES> : SupportedLaneCount,
          Self : SimdXorImpl<DEF_BACKEND_TYPE>
{
    type Output = Simd<T, LANES>;
    
    fn bitxor(self, rhs: Self) -> Self::Output {
        self.simd_xor::<DEF_BACKEND_TYPE>(rhs)
    }
}

impl<T, const LANES: usize> BitOr for Simd<T, LANES> 
    where T : SimdElement,
          LaneCount<LANES> : SupportedLaneCount,
          Self : SimdOrImpl<DEF_BACKEND_TYPE>
{
    type Output = Simd<T, LANES>;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.simd_or::<DEF_BACKEND_TYPE>(rhs)
    }
}

impl<T, const LANES: usize> Shl for Simd<T, LANES> 
    where T : SimdElement,
          LaneCount<LANES> : SupportedLaneCount,
          Self : SimdShiftImpl<DEF_BACKEND_TYPE>
{
    type Output = Simd<T, LANES>;

    fn shl(self, rhs: Self) -> Self::Output {
        self.simd_shl::<DEF_BACKEND_TYPE>(rhs)
    }
}

macro_rules! impl_shr_a {
    {$($ty:ty)*} => {
        $(
            impl<const LANES: usize> Shr for Simd<$ty, LANES> 
                where LaneCount<LANES> : SupportedLaneCount,
                      Self : SimdShiftImpl<DEF_BACKEND_TYPE>
            {
                type Output = Simd<$ty, LANES>;

                fn shr(self, rhs: Self) -> Self::Output {
                    self.shra(rhs)
                }
            }
        )*
    };
}
impl_shr_a!{
    i8
    i16
    i32
    i64
}

macro_rules! impl_shr_l {
    {$($ty:ty)*} => {
        $(
            impl<const LANES: usize> Shr for Simd<$ty, LANES> 
                where LaneCount<LANES> : SupportedLaneCount,
                      Self : SimdShiftImpl<DEF_BACKEND_TYPE>
            {
                type Output = Simd<$ty, LANES>;
                fn shr(self, rhs: Self) -> Self::Output {
                    self.shrl(rhs)
                }
            }
        )*
    };
}
impl_shr_l!{
    u8
    u16
    u32
    u64
}


////////////////////////////////////////////////////////////////

//#[cfg(test)]
impl<T, const LANES: usize> Debug for Simd<T, LANES>
    where T : SimdElement + Debug,
          LaneCount<LANES> : SupportedLaneCount
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Simd").field(&self.0).finish()
    }
}