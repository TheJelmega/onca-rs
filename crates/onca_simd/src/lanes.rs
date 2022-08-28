mod sealed {
    pub trait Sealed {}
}
use core::{ops::{BitAnd, BitXor, BitOr, Not}, fmt::Debug};

use sealed::Sealed;

pub struct LaneCount<const LANES: usize>;

impl<const LANES: usize> LaneCount<LANES> {
    pub const BITMASK_LEN : usize = (LANES + 7) / 8;
}

impl<const LANES: usize> Sealed for LaneCount<LANES> {}

pub trait SupportedLaneCount : Sealed {
    type BitMask : Copy + Default + AsRef<[u8]> + AsMut<[u8]> + Debug + PartialEq;
    type MaskInt : Copy + Default + BitAnd + BitXor + BitOr + Not;
}

impl SupportedLaneCount for LaneCount<2> {
    type BitMask = [u8; Self::BITMASK_LEN];
    type MaskInt = i8;
}

impl SupportedLaneCount for LaneCount<4> {
    type BitMask = [u8; Self::BITMASK_LEN];
    type MaskInt = i8;
}

impl SupportedLaneCount for LaneCount<8> {
    type BitMask = [u8; Self::BITMASK_LEN];
    type MaskInt = i8;
}

impl SupportedLaneCount for LaneCount<16> {
    type BitMask = [u8; Self::BITMASK_LEN];
    type MaskInt = i16;
}

impl SupportedLaneCount for LaneCount<32> {
    type BitMask = [u8; Self::BITMASK_LEN];
    type MaskInt = i32;
}

impl SupportedLaneCount for LaneCount<64> {
    type BitMask = [u8; Self::BITMASK_LEN];
    type MaskInt = i64;
}