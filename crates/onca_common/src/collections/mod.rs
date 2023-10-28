mod imp;
mod bitset;
mod byte_buffer;

pub use bitset::BitSet;
pub use byte_buffer::ByteBuffer;

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
