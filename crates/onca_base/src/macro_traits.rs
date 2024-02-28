//! Contains traits used by onca macros


/// Trait to get the number of elements in an enum
pub trait EnumCountT {
    /// Count or number of element in an enum
    const COUNT : usize;
}

/// Trait to get an enum from a given index
pub trait EnumFromIndexT: Sized {
    /// Try to convert an index to an enum
    fn from_idx(idx: usize) -> Option<Self>;

    /// Try to convert an index to an enum, if it couldn't convert it, return a default value
    fn from_idx_or(idx: usize, default: Self) -> Self;

    /// Convert an index to an enum, without checking bounds
    /// 
    /// # SAFETY
    /// 
    /// The user is required to make sure that the index is an index of a valid enum variant
    unsafe fn from_idx_unchecked(idx: usize) -> Self;
}

pub trait EnumFromNameT: Sized {
    /// Try to parse the enum from a string slice.
    fn parse(s: &str) -> Option<Self>;
}