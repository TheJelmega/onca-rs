mod collections_alloc;
mod imp;

mod static_dyn_array;
mod dyn_array;
mod small_dyn_array;

mod btree_map;
mod btree_set;
mod linked_list;
mod vec_deque;
mod hash_map;
mod hash_set;

mod bitset;


pub use static_dyn_array::*;
pub use dyn_array::*;
pub use small_dyn_array::*;

pub use btree_map::*;
pub use btree_set::*;
pub use linked_list::*;
pub use vec_deque::*;
pub use hash_map::*;
pub use hash_set::*;

pub use bitset::*;

use core::ops::Range;

//--------------------------------------------------------------

macro_rules! impl_slice_partial_eq {
    ([$($vars:tt)*] $lhs:ty, $rhs:ty $(where $ty:ty: $bound:ident)?) => {
        impl<T, U, $($vars)*> PartialEq<$rhs> for $lhs
        where
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
use impl_slice_partial_eq;

//--------------------------------------------------------------

trait SpecExtend<T, I> {
    fn spec_extend(&mut self, iter: I);
}

trait SpecExtendFromWithin {
    /// # Safety
    /// 
    /// - `src` need to be a valid index
    /// - `self.capacity() - self.len()` must be >= `src.len()`
    unsafe fn spec_extend_from_within(&mut self, src: Range<usize>);
}

trait SpecCloneFrom {
    fn clone_from(this: &mut Self, other: &Self);
}

trait ExtendWith<T> {
    fn next(&mut self) -> T;
    fn last(self) -> T;
}

trait SpecFromIterNested<T, I> {
    fn from_iter(iter: I) -> Self;
}

trait SpecFromIter<T, I> {
    fn from_iter(iter: I) -> Self;
}

//--------------------------------------------------------------

struct ExtendElement<T>(T);
impl<T: Clone> ExtendWith<T> for ExtendElement<T> {
    fn next(&mut self) -> T {
        self.0.clone()
    }

    fn last(self) -> T {
        self.0
    }
}

struct ExtendFunc<F>(F);
impl<T, F: FnMut() -> T> ExtendWith<T> for ExtendFunc<F> {
    fn next(&mut self) -> T {
        (self.0)()
    }

    fn last(mut self) -> T {
        (self.0)()
    }
}

//--------------------------------------------------------------

struct SetLenOnDrop<'a> {
    len       : &'a mut usize,
    local_len : usize,
}

impl<'a> SetLenOnDrop<'a> {
    #[inline]
    fn new(len: &'a mut usize) -> Self {
        SetLenOnDrop { local_len: *len, len }
    }

    #[inline]
    fn increment_len(&mut self, increment: usize) {
        self.local_len += increment;
    }
}

impl Drop for SetLenOnDrop<'_> {
    #[inline]
    fn drop(&mut self) {
        *self.len = self.local_len;
    }
}

//--------------------------------------------------------------

#[rustc_specialization_trait]
pub unsafe trait IsZero {
    /// Whether this value's representation is all zeroes
    fn is_zero(&self) -> bool;
}

macro_rules! impl_is_zero {
    ($t:ty, $is_zero:expr) => {
        unsafe impl IsZero for $t {
            #[inline]
            fn is_zero(&self) -> bool {
                $is_zero(*self)
            }
        }
    };
}
impl_is_zero!(i8, |x| x == 0);
impl_is_zero!(i16, |x| x == 0);
impl_is_zero!(i32, |x| x == 0);
impl_is_zero!(i64, |x| x == 0);
impl_is_zero!(i128, |x| x == 0);
impl_is_zero!(isize, |x| x == 0);

impl_is_zero!(u8, |x| x == 0);
impl_is_zero!(u16, |x| x == 0);
impl_is_zero!(u32, |x| x == 0);
impl_is_zero!(u64, |x| x == 0);
impl_is_zero!(u128, |x| x == 0);
impl_is_zero!(usize, |x| x == 0);

impl_is_zero!(bool, |x| x == false);
impl_is_zero!(char, |x| x == '\0');

impl_is_zero!(f32, |x: f32| x.to_bits() == 0);
impl_is_zero!(f64, |x: f64| x.to_bits() == 0);

unsafe impl<T> IsZero for *const T {
    #[inline]
    fn is_zero(&self) -> bool {
        (*self).is_null()
    }
}

unsafe impl<T> IsZero for *mut T {
    #[inline]
    fn is_zero(&self) -> bool {
        (*self).is_null()
    }
}

unsafe impl<T: IsZero, const N: usize> IsZero for [T; N] {
    #[inline]
    fn is_zero(&self) -> bool {
        // Because this is generated as a runtime check, it's not obvious that it's worth doing if the array is really long.  
        // The threshold here is largely arbitrary, but was picked because as of 2022-07-01 LLVM fails to const-fold the check in `vec![[1; 32]; n]`
        // See https://github.com/rust-lang/rust/pull/97581#issuecomment-1166628022
        // Feel free to tweak if you have better evidence.
        N <= 16 && self.iter().all(IsZero::is_zero)
    }
}

// This is a recursive macro
macro_rules! impl_for_tuples {
    // Stopper
    () => {
        // No use for implementing for an empty tuple because it is ZST
    };
    ($first_arg:ident $(,$rest:ident)*) => {
        unsafe impl <$first_arg: IsZero, $($rest: IsZero,)*> IsZero for ($first_arg, $($rest,)*) {
            #[inline]
            fn is_zero(&self) -> bool {
                // Destructure tuple to N references
                // Ruest allows to hide generic params by locacl variable names
                #[allow(non_snake_case)]
                let ($first_arg, $($rest,)*) = self;

                $first_arg.is_zero() $(&& $rest.is_zero())*
            }
        }

        impl_for_tuples!($($rest),*);
    };
}
impl_for_tuples!(A, B, C, D, E, F, G, H);

// Option<&T> are guaranteeed to represetn `None` as null.
// For fat pointers, teh bytes that would be the pointer metadata in the `Some` variant are padded in the `None` variant, so ignoring them and zero-initializing instead is ok.
// `Option<&mut T>` never implements `Clone`, so tehre's no need for an impl or `SpecFromElem`

unsafe impl<T: ?Sized> IsZero for Option<&T> {
    #[inline]
    fn is_zero(&self) -> bool {
        self.is_none()
    }
}

// `Option<NonZeroU32>` and other similar options have a representation that guarantees they're the same size as the corresponding `u32` type, 
// as well as a guarantee that transmuting between `NonZeroU32` and `Option<num::NonZeroU32>` works.
// While the documentation officially makes it UB to transmute from `None`, we're are basing out implementation directly on the standard library, 
// we can make extra inferences, and we know that the only niche available to represent `None` is the one that's all zeros.
macro_rules! impl_is_zero_option_of_nonzero {
    ($($t:ident,)*) => {
        $(
            unsafe impl IsZero for Option<core::num::$t> {
                #[inline]
                fn is_zero(&self) -> bool {
                    self.is_none()
                }
            }
        )*
    };
}
impl_is_zero_option_of_nonzero!(
    NonZeroU8,
    NonZeroU16,
    NonZeroU32,
    NonZeroU64,
    NonZeroU128,
    NonZeroI8,
    NonZeroI16,
    NonZeroI32,
    NonZeroI64,
    NonZeroI128,
    NonZeroUsize,
    NonZeroIsize,
);