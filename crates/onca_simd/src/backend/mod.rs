use core::{marker::PhantomData, convert};
use crate::{SimdElement, Simd, LaneCount, SupportedLaneCount, Mask, MaskElement};


#[derive(PartialEq, Eq)]
pub enum BackendType {
    Scalar,

    // x86-64
    SSE,
    AVX,
    AVX2,
    AVX512,

    // AArch64
    NEON,
}

pub trait SimdSetImpl<T: SimdElement, const BACKEND_TYPE: BackendType>
{
    /// Set all elements to 0
    fn simd_zero_impl() -> Self;

    /// Set all elements of the register to `val`
    fn simd_splat_impl(val: T) -> Self;
}

pub trait SimdLoadStoreImpl<T: SimdElement, const BACKEND_TYPE: BackendType> {
    // Load all elements from memory
    fn simd_load_impl(mem: *const T) -> Self;

    // Store all elements into the given memory
    fn simd_store_impl(self, mem: *mut T);
}

pub trait SimdGatherImpl<T: SimdElement, const LANES: usize, const BACKEND_TYPE: BackendType>
    where LaneCount<LANES> : SupportedLaneCount
{
    /// Gather all element from their given indices
    fn simd_gather_impl(mem: *const T, idxs: Simd<T::Idx, LANES>) -> Self;
    
    /// Gather all element from their given indices if the mask is set, otherwise get the value of the corresponding `or` element
    fn simd_gather_select_impl(mem: *const T, idxs: Simd<T::Idx, LANES>, mask: Mask<T::Mask, LANES>, or: Self) -> Self;

    /// Gather all element from their given indices if the mask is set, otherwise get the value of the corresponding `or` element
    /// If the index is out of range (larger than `max_idx`), the `or` element will be picked
    fn simd_gather_select_clamped_impl(mem: *const T, idxs: Simd<T::Idx, LANES>, mask: Mask<T::Mask, LANES>, or: Self, max_idx: usize) -> Self;

    /// Gather all element from their given indices, with the indexes being 32-bit values
    fn simd_gather_idx32_impl(mem: *const T, idxs: [u32; LANES]) -> Self;
    
    /// Gather all element from their given indices if the mask is set, otherwise get the value of the corresponding `or` element, with the indexes being 32-bit values
    fn simd_gather_idx32_select_impl(mem: *const T, idxs: [u32; LANES], mask: Mask<T::Mask, LANES>, or: Self) -> Self;

    /// Gather all element from their given indices if the mask is set, otherwise get the value of the corresponding `or` element, with the indexes being 32-bit values
    /// If the index is out of range (larger than `max_idx`), the `or` element will be picked
    fn simd_gather_idx32_select_clamped_impl(mem: *const T, idxs: [u32; LANES], mask: Mask<T::Mask, LANES>, or: Self, max_idx: usize) -> Self;

    /// Gather all element from their given indices, with the indexes being 64-bit values
    fn simd_gather_idx64_impl(mem: *const T, idxs: [u64; LANES]) -> Self;
    
    /// Gather all element from their given indices if the mask is set, otherwise get the value of the corresponding `or` element, with the indexes being 64-bit values
    fn simd_gather_idx64_select_impl(mem: *const T, idxs: [u64; LANES], mask: Mask<T::Mask, LANES>, or: Self) -> Self;
    
    /// Gather all element from their given indices if the mask is set, otherwise get the value of the corresponding `or` element, with the indexes being 64-bit values
    /// If the index is out of range (larger than `max_idx`), the `or` element will be picked
    fn simd_gather_idx64_select_clamped_impl(mem: *const T, idxs: [u64; LANES], mask: Mask<T::Mask, LANES>, or: Self, max_idx: usize) -> Self;
}

/// Trait for converting between same sized simd registers with same sized element types
/// 
/// int <-> uint: bitcast
/// int/uint -> float: integral as float, with precision truncated to the float's matissa bits
/// float -> int/uint: original value without decimals, UB if the float value is out of range
pub trait SimdConvertImpl<T, const LANES: usize, const BACKEND_TYPE: BackendType> : Sized
    where T : SimdElement,
          LaneCount<LANES> : SupportedLaneCount
{
    /// Convert the element type
    fn simd_convert_impl(self) -> Simd<T, LANES>;

    /// Convert the type, if the type does not fit into the new type, saturate it
    fn simd_convert_saturate_impl(self) -> Simd<T, LANES> {
        Self::simd_convert_impl(self)
    }
}

pub trait SimdExtendCompressImpl<const BACKEND_TYPE: BackendType>
{
    type ExtendedType;

    /// Extend the lower elements
    fn simd_extend_lower_impl(self) -> Self::ExtendedType;

    /// Extend the upper elements
    fn simd_extend_upper_impl(self) -> Self::ExtendedType;

    /// Compress 2 registers
    fn simd_compress_impl(a: Self::ExtendedType, b: Self::ExtendedType) -> Self;
}


/// Trait to equality
pub trait SimdCmpImpl<const BACKEND_TYPE: BackendType>
{
    type MaskT;

    fn simd_eq_impl(&self, other: &Self) -> Self::MaskT;
    fn simd_ne_impl(&self, other: &Self) -> Self::MaskT;
    fn simd_lt_impl(&self, other: &Self) -> Self::MaskT;
    fn simd_le_impl(&self, other: &Self) -> Self::MaskT;
    fn simd_gt_impl(&self, other: &Self) -> Self::MaskT;
    fn simd_ge_impl(&self, other: &Self) -> Self::MaskT;

    fn simd_max_impl(self, other: Self) -> Self;
    fn simd_min_impl(self, other: Self) -> Self;
    fn simd_clamp_impl(self, min: Self, max: Self) -> Self;
}

/// Trait for mask-specific operations
pub trait SimdMaskOpsImpl<const BACKEND_TYPE: BackendType> {
    /// Check if all elements are masked
    fn simd_all_impl(self) -> bool;

    /// Check if any element is masked
    fn simd_any_impl(self) -> bool;
}

pub trait SimdAddImpl<const BACKEND_TYPE: BackendType> {
    /// Per element add
    fn simd_add_impl(self, other: Self) -> Self;
}

pub trait SimdSubImpl<const BACKEND_TYPE: BackendType> {
    /// Per element subtract
    fn simd_sub_impl(self, other: Self) -> Self;
}

pub trait SimdMulImpl<const BACKEND_TYPE: BackendType> {
    /// Per element multiplication
    fn simd_mul_impl(self, other: Self) -> Self;
}

pub trait SimdDivImpl<const BACKEND_TYPE: BackendType> {
    /// Per element division
    fn simd_div_impl(self, other: Self) -> Self;
}

pub trait SimdRemImpl<const BACKEND_TYPE: BackendType> {
    /// Per element remainder
    fn simd_rem_impl(self, other: Self) -> Self;
}

pub trait SimdNegImpl<const BACKEND_TYPE: BackendType> {
    /// Per element negate
    fn simd_neg_impl(self) -> Self;
}

pub trait SimdNotImpl<const BACKEND_TYPE: BackendType> {
    /// Per element not
    fn simd_not_impl(self) -> Self;
}

pub trait SimdAndImpl<const BACKEND_TYPE: BackendType> {
    /// Per element and
    fn simd_and_impl(self, other: Self) -> Self;
}

pub trait SimdXorImpl<const BACKEND_TYPE: BackendType> {
    /// Per element xor
    fn simd_xor_impl(self, other: Self) -> Self;
}

pub trait SimdOrImpl<const BACKEND_TYPE: BackendType> {
    /// Per element or
    fn simd_or_impl(self, other: Self) -> Self;
}

pub trait SimdAndNotImpl<const BACKEND_TYPE: BackendType> {
    /// Per element bit and with the not of `other`
    fn simd_andnot_impl(self, other: Self) -> Self;
}

// NOTE(jel): For any shift value smaller than the number of bits in the sub-type, the register will be shifted by that amount,
//            For any shift larger than that, the element will be 0
pub trait SimdShiftImpl<const BACKEND_TYPE: BackendType>
{
    /// Per element bit shift left
    fn simd_shl_impl(self, other: Self) -> Self;

    /// Per element bit shift right (logical shift, i.e. zero extend)
    /// 
    /// Any shift by a value >= type's bitsize will result in the element being set to 0
    fn simd_shrl_impl(self, other: Self) -> Self;

    // Per element bit shift right (arithmetic shift, i.e. sign extend)
    /// 
    /// Any shift by a value >= type's bitsize will result in the element being set to 0
    fn simd_shra_impl(self, other: Self) -> Self;

    /// Shift each element bit left by the `shift` bits
    /// 
    /// Any shift by a value >= type's bitsize will result in the element being set to 0
    fn simd_shl_scalar_impl(self, shift: u8) ->Self;

    /// Shift each element bit right by the `shift` bits (logical shift, i.e. zero extend)
    /// 
    /// Any shift by a value >= type's bitsize will result in the element being set to 0
    fn simd_shrl_scalar_impl(self, shift: u8) -> Self;

    /// Shift each element bit right by the `shift` bits (arithmetic shift, i.e. sign extend)
    /// 
    /// Any shift by a value >= type's bitsize will result in the element being set to 0
    fn simd_shra_scalar_impl(self, shift: u8) -> Self;
}

pub trait SimdFloorImpl<const BACKEND_TYPE: BackendType> {
    /// Per element `floor`
    fn simd_floor_impl(self) -> Self;
}

pub trait SimdCeilImpl<const BACKEND_TYPE: BackendType> {
    /// Per element `ceil`
    fn simd_ceil_impl(self) -> Self;
}

pub trait SimdRoundImpl<const BACKEND_TYPE: BackendType> {
    /// Per element `round` (round to nearest)
    fn simd_round_impl(self) -> Self;
}

pub trait SimdSqrtImpl<const BACKEND_TYPE: BackendType> {
    /// Per element square root
    fn simd_sqrt_impl(self) -> Self;
}

pub trait SimdRsqrtImpl<const BACKEND_TYPE: BackendType> : Sized {
    /// Per element reverse square root
    fn simd_rsqrt_impl(self) -> Self;

    /// Per element reverse square root (aproximated, i.e. less precision but could be faster)
    fn simd_rsqrt_approx_impl(self) -> Self {
        Self::simd_rsqrt_impl(self)
    }
}

pub trait SimdRcpImpl<const BACKEND_TYPE: BackendType> : Sized {
    /// Per element reciprical
    fn simd_rcp_impl(self) -> Self;
    
    /// Per element reciprical (aproximated, i.e. less precision but could be faster)
    fn simd_rcp_approx_impl(self) -> Self {
        Self::simd_rcp_impl(self)
    }
}

pub trait SimdAbsImpl<const BACKEND_TYPE: BackendType> {
    /// Per element absolute value
    fn simd_abs_impl(self) -> Self;
}



macro_rules! from_transmute {
    { unsafe $a:ty => $b:ty } => {
        from_transmute!{ @impl $a => $b }
        from_transmute!{ @impl $b => $a }
    };
    { @impl $from:ty => $to:ty } => {
        impl core::convert::From<$from> for $to {
            #[inline]
            fn from(value: $from) -> $to {
                unsafe { core::mem::transmute(value) }
            }
        }
    }
}

mod scalar;

#[cfg(target_feature = "sse")]
mod x86_64;