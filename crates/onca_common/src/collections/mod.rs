
mod imp;
mod bitset;
mod byte_buffer;

mod dynarr;

use core::alloc::Layout;

pub use bitset::BitSet;
pub use byte_buffer::ByteBuffer;
pub use dynarr::*;

//--------------------------------------------------------------

macro_rules! impl_slice_partial_eq_generic {
    ([$($vars:tt)*] $lhs:ty, $rhs:ty $(where $ty:ty: $bound:ident)?) => {
        impl<T, U, $($vars)*> PartialEq<$rhs> for $lhs  where
            T : PartialEq<U>,
            $($ty: $bound)?
        {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool { self[..] == other[..] }
            #[inline]
            fn ne(&self, other: &$rhs) -> bool { self[..] != other[..] }
        }
    };
}
use impl_slice_partial_eq_generic;

macro_rules! impl_slice_partial_eq {
    ([$($vars:tt)*] $lhs:ty, $rhs:ty $(where $ty:ty: $bound:ident)?) => {
        impl<$($vars)*> PartialEq<$rhs> for $lhs where
            $($ty: $bound)?
        {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool { self[..] == other[..] }
            #[inline]
            fn ne(&self, other: &$rhs) -> bool { self[..] != other[..] }
        }
    };
}
use impl_slice_partial_eq;

//--------------------------------------------------------------

#[derive(Debug)]
pub enum TryReserveError {
    CapacityOverflow,
    AllocError(Layout),
}

//--------------------------------------------------------------

/// A trait used to define a strategy to reserve additional memory for containers.
#[const_trait]
pub trait ReserveStrategy {
    /// Calculate the new capacity for a container.
    /// 
    /// `cur_capacity` represents the current capacity of the container.
    /// 
    /// `min_capacity` represents the minimum required capacity to be able to resize.
    /// 
    /// Returns `Err(())` if the capacity were to overflow
    fn calculate(cur_capacity: usize, min_capacity: usize) -> Result<usize, ()>;
}

/// A reserve strategy that will try to either return double the current capacity, or the minimum required capacity, whichever is bigger.
pub struct DoubleOrMinReserveStrategy;

impl const ReserveStrategy for DoubleOrMinReserveStrategy {
    fn calculate(cur_capacity: usize, min_capacity: usize) -> Result<usize, ()> {
        let double_cap = cur_capacity * 2;
        let new_cap = if double_cap > min_capacity { double_cap } else { min_capacity };
        if new_cap <= isize::MAX as usize { 
            Ok(new_cap)
        } else {
            Err(())
        }
    }
}

/// A reserve strategy that will return a power of 2 capacity
pub struct Pow2ReserveStrategy;

impl const ReserveStrategy for Pow2ReserveStrategy {
    fn calculate(cur_capacity: usize, min_capacity: usize) -> Result<usize, ()> {
        let new_cap = min_capacity.next_power_of_two();
        if new_cap != 0 {
            Ok(new_cap)
        } else {
            Err(())
        }
    }
}

/// A reserve stategy that grows the capacity by 1.5
pub struct ThreeHalvesReserveStrategy;

impl const ReserveStrategy for ThreeHalvesReserveStrategy {
    fn calculate(cur_capacity: usize, min_capacity: usize) -> Result<usize, ()> {
        let mut cap = cur_capacity;
        while cap < min_capacity {
            cap = (cap << 1) - (cap >> 1);
            if cap >= isize::MAX as usize {
                return Err(());
            }
        }
        Ok(cap)
    }
}