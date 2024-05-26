use std::{
    iter::*,
    marker::PhantomData,
    mem::{self, ManuallyDrop, MaybeUninit},
    num::NonZero,
    ptr::{self, NonNull},
    fmt,
    slice,
};

use crate::{
    collections::{imp::array::RawArray, ReserveStrategy},
    mem::{SlicedSingleHandle, Storage, StorageSingleSliced},
};
use super::{
    to_dynarr::*,
    in_place_collect::AsDynArrIntoIter,
};

macro_rules! non_null {
    (mut $place:expr, $ty:ident) => {
        #[allow(unused_unsafe)] // We're sometimes used wthin an unsafe block
        unsafe { &mut *(ptr::addr_of_mut!($place) as *mut NonNull<$ty>) }
    };
    ($place:expr, $ty:ident) => {
        #[allow(unused_unsafe)] // We're sometimes used wthin an unsafe block
        unsafe { *(ptr::addr_of!($place) as *const NonNull<$ty>) }
    };
}

pub struct IntoIter<T, S: StorageSingleSliced, R: ReserveStrategy> {
    pub(super) phantom: PhantomData<(T, R)>,
    /// The drop impl reconstructs a RawArray from handle, to avoid dropping the storage twice, we need to wrap it inot ManuallyDrop
    pub(super) handle: SlicedSingleHandle<T, S::Handle>,
    pub(super) storage: ManuallyDrop<S>,
    pub(super) ptr: NonNull<T>,
    /// If T is a ZST, this is acutally ptr + len.
    /// This encoding is picked so that ptr == end is a quick test for the Iterator being empty, that works for both ZST and non-ZST.
    /// For non-ZSTs the pointer is treated as `NonNull<T>`
    pub(super) end: *const T,
}

impl<T: fmt::Debug, S: StorageSingleSliced, R: ReserveStrategy> fmt::Debug for IntoIter<T, S, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("IntoIter").field(&self.as_slice()).finish()
    }
}

impl<T, S: StorageSingleSliced, R: ReserveStrategy> IntoIter<T, S, R> {
    /// Returns the remaining items of this iteratar as a slice.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let arr = dynarr!['a', 'b', 'c'];
    /// let mut into_iter = arr.into_iter();
    /// assert_eq!(into_iter.as_slice(), &['a', 'b', 'c']);
    /// let _ = into_iter.next().unwrap();
    /// assert_eq!(into_iter.as_slice(), &['b', 'c']);
    /// ```
    pub fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len()) }
    }

    /// Returns the remaining items of this iteratar as a mutable slice.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let arr = dynarr!['a', 'b', 'c'];
    /// let mut into_iter = arr.into_iter();
    /// assert_eq!(into_iter.as_slice(), &['a', 'b', 'c']);
    /// into_iter.as_mut_slice()[2] = 'z';
    /// assert_eq!(into_iter.next().unwrap(), 'a');
    /// assert_eq!(into_iter.next().unwrap(), 'b');
    /// assert_eq!(into_iter.next().unwrap(), 'z');
    /// ```
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len()) }
    }

    fn as_raw_mut_slice(&mut self) -> *mut [T] {
        ptr::slice_from_raw_parts_mut(self.ptr.as_ptr(), self.len())
    }

    /// Drops the remaining elements and relinquishes the backing allocation.
    /// This method guarantees it won't panic before the backing allocation.
    /// 
    /// This is roughly equivalent to the following.
    /// 
    /// ```
    /// let mut into_iter = DynArr::<u8>::with_capacity(10).into_iter();
    /// let mut inot_iter = std::mem::replace(&mut into_iter, DynArr::new().into_iter());
    /// (&mut into_iter).foreach_drop();
    /// std::mem::forget(into_iter);
    /// ```
    /// 
    /// This method is used by in-place iteration, refer to the dynarr::in_place_collect documentation for an overview.
    pub(super) fn forget_allocation_drop_remaining(&mut self) {
        let remaining = self.as_raw_mut_slice();

        // Overwrite the individual fields instead of creating a new struct and then over wirting &mut self.
        // This creates less assembly
        self.ptr = unsafe { SlicedSingleHandle::<T, _>::dangling(&*self.storage).resolve_raw(&*self.storage).0.cast() };
        self.end = self.ptr.as_ptr();

        // Dropping the raminaing elemens can panic, so this needs to be done only after updating the other fields
        unsafe {
            ptr::drop_in_place(remaining);
        }
    }

    pub(crate) fn forget_remaining_elements(&mut self) {
        // For the ZST case, it is curcial that we mutabe `end` here, not `ptr`.
        // `ptr` must stay aligned, while `end` may be unaligned
        self.end = self.ptr.as_ptr();
    }

    // TODO
    //pub(crate) fn into_deque();
}

impl<T, S: StorageSingleSliced, R: ReserveStrategy> AsRef<[T]> for IntoIter<T, S, R> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

unsafe impl<T: Send, S: StorageSingleSliced + Send, R: ReserveStrategy> Send for IntoIter<T, S, R> {}
unsafe impl<T: Sync, S: StorageSingleSliced + Sync, R: ReserveStrategy> Sync for IntoIter<T, S, R> {}

impl<T, S: StorageSingleSliced, R: ReserveStrategy> Iterator for IntoIter<T, S, R> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let ptr = if mem::size_of::<T>() == 0 {
            if self.ptr.as_ptr() == self.end as *mut T {
                return None;
            }

            // `ptr` has to stay where it is to remain aligned, so we reduce the lenght by 1 by reducing the end
            self.end = self.end.wrapping_byte_sub(1);
            self.ptr
        } else {
            if self.ptr == non_null!(self.end, T) {
                return None;
            }
            let old = self.ptr;
            self.ptr = unsafe { old.add(1) };
            old
        };
        Some(unsafe { ptr.read() })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let exact = if mem::size_of::<T>() == 0 {
            self.end.addr().wrapping_sub(self.ptr.as_ptr().addr())
        } else {
            unsafe { non_null!(self.end, T).sub_ptr(self.ptr) }
        };
        (exact, Some(exact))
    }

    #[inline]
    fn advance_by(&mut self, n: usize) -> Result<(), core::num::NonZero<usize>> {
        let step_size = self.len().min(n);
        let to_drop = ptr::slice_from_raw_parts_mut(self.ptr.as_ptr(), step_size);
        if mem::size_of::<T>() == 0 {
            // See `next` for why we sub `end` here
            self.end = self.end.wrapping_byte_sub(1);
        } else {
            self.ptr = unsafe { self.ptr.add(step_size) };
        }
        // SAFETY: the min() above ensures that step_size is in bounds
        unsafe {
            ptr::drop_in_place(to_drop);
        }
    
        NonZero::new(n - step_size).map_or(Ok(()), Err)
    }

    #[inline]
    fn count(self) -> usize {
        self.len()
    }

    fn next_chunk<const N: usize>(&mut self) -> Result<[Self::Item; N], core::array::IntoIter<Self::Item, N>> {
        let mut raw_ary = MaybeUninit::uninit_array();

        let len = self.len();

        if mem::size_of::<T>() == 0 {
            if len < N {
                self.forget_remaining_elements();
                // SAFETY: ZSTs can be conjured ex nihilo, only the amounnt has to be correct
                return Err(unsafe { core::array::IntoIter::new_unchecked(raw_ary, 0..len) });
            }

            self.end = self.end.wrapping_byte_sub(N);
            // SAFETY: ditto
            return Ok(unsafe { raw_ary.transpose().assume_init() });
        }

        if len < N {
            // SAFETY: `len` indicates that this many elements are available and we jst checked that it fits into the array.
            unsafe {
                ptr::copy_nonoverlapping(self.ptr.as_ptr(), raw_ary.as_mut_ptr() as *mut T, len);
                self.forget_remaining_elements();
                return Err(core::array::IntoIter::new_unchecked(raw_ary, 0..len));
            }
        }

        // SAFETY: `len` is larger than the array size. Copy a fixed amount here to fully initialized.
        unsafe {
            ptr::copy_nonoverlapping(self.ptr.as_ptr(), raw_ary.as_mut_ptr() as *mut T, N);
            self.ptr = self.ptr.add(N);
            Ok(raw_ary.transpose().assume_init())
        }
    }

    unsafe fn __iterator_get_unchecked(&mut self, idx: usize) -> Self::Item where
            Self: std::iter::TrustedRandomAccessNoCoerce
    {
        // SAFETY: the caller must guarantee that `i` is in bounds of the `DynArr<T>`, so `i` cannot overflow and `isize`,
        // and the `self.ptr.add(i)` is guaranteed to point to an element of the `DynArr<T>` and thus guaranteed to be valid to dereference.
        //
        // Also note the implementation of `Self: TrustedRandomAccess` requires that `T: Copy` so reading elements from the buffer does not invalidate them for `Drop`.
        unsafe { self.ptr.add(idx).read() }
    }
}

impl<T, S: StorageSingleSliced, R: ReserveStrategy> DoubleEndedIterator for IntoIter<T, S, R> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if mem::size_of::<T>() == 0 {
            if self.ptr.as_ptr() == self.end as *mut _ {
                return None;
            }
            // See above why `ptr.offset` isn't used
            self.end = self.end.wrapping_byte_sub(1);
            // Note that even though this is next_back(), we're reading from `self.ptr`, not `self.end`.
            // We track our length using the byte offset from `self.ptr` to `self.end`, so the end pointer may not be suitably aligned for T
            Some(unsafe { ptr::read(self.ptr.as_ptr()) })
        } else {
            if self.ptr == non_null!(self.end, T) {
                return None;
            }
            unsafe {
                self.end = self.end.sub(1);
                Some(ptr::read(self.end))
            }
        }
    }

    fn advance_back_by(&mut self, n: usize) -> Result<(), NonZero<usize>> {
        let step_size = self.len().min(n);
        if mem::size_of::<T>() == 0 {
            // SAFETY: same as advance_by()
            self.end = self.end.wrapping_byte_sub(step_size);
        } else {
            // SAFETY: same as advance_by()
            self.end = unsafe { self.end.sub(step_size) };
        }
        let to_drop = ptr::slice_from_raw_parts_mut(self.end as *mut T, step_size);
        // SAFETY: same as advance_by()
        unsafe {
            ptr::drop_in_place(to_drop);
        }
        NonZero::new(n - step_size).map_or(Ok(()), Err)
    }
}

impl<T, S: StorageSingleSliced, R: ReserveStrategy> ExactSizeIterator for IntoIter<T, S, R> {
    fn is_empty(&self) -> bool {
        if mem::size_of::<T>() == 0 {
            self.ptr.as_ptr() == self.end as *mut _
        } else {
            self.ptr == non_null!(self.end, T)
        }
    }
}

impl<T, S: StorageSingleSliced, R: ReserveStrategy> FusedIterator for IntoIter<T, S, R> {}
unsafe impl<T, S: StorageSingleSliced, R: ReserveStrategy> TrustedFused for IntoIter<T, S, R> {}
unsafe impl<T, S: StorageSingleSliced, R: ReserveStrategy> TrustedLen for IntoIter<T, S, R> {}

impl<T, S: StorageSingleSliced + Default, R: ReserveStrategy> Default for IntoIter<T, S, R> {
    /// Creates an empty `DynArr::IntoIter`.
    /// 
    /// ```
    /// use onca_common::collections::DynArr;
    /// let iter: DynArr::IntoIter<u8> = Default::default();
    /// assert_eq!(iter.len(), 0);
    /// assert_eq!(iter.as_slice, &[]);
    /// ```
    fn default() -> Self {
        super::DynArr::new().into_iter()
    }
}

pub trait NonDrop {}

// T: Copy as approximation for !Drop sicne get_unchecked does not advance self.ptr and thus we can't implement drop-handling
impl<T: Copy> NonDrop for T {}

unsafe impl<T, S: StorageSingleSliced, R: ReserveStrategy> TrustedRandomAccessNoCoerce for IntoIter<T, S, R> {
    const MAY_HAVE_SIDE_EFFECT: bool = false;
}

impl<T: Clone, S: StorageSingleSliced + Clone, R: ReserveStrategy> Clone for IntoIter<T, S, R> {
    fn clone(&self) -> Self {
        self.as_slice().to_dynarr_in((*self.storage).clone()).into_iter()
    }
}

unsafe impl<#[may_dangle] T, S: StorageSingleSliced, R: ReserveStrategy> Drop for IntoIter<T, S, R> {
    fn drop(&mut self) {
        struct DropGuard<'a, T, S: StorageSingleSliced, R: ReserveStrategy>(&'a mut IntoIter<T, S, R>);

        impl<T, S: StorageSingleSliced, R: ReserveStrategy> Drop for DropGuard<'_, T, S, R> {
            fn drop(&mut self) {
                unsafe {
                    /// `IntoIter::storage` is not used anymore after this and will be dropped by RawArray
                    let storage = ManuallyDrop::take(&mut self.0.storage);
                    let _ = RawArray::<T, S ,R>::from_raw_parts(self.0.handle, storage);
                }
            }
        }

        let guard = DropGuard(self);
        // destroy the remaining elements
        unsafe {
            ptr::drop_in_place(guard.0.as_raw_mut_slice());
        }
        // now `guard` will be dropped and do the rest
    }
}

unsafe impl<T, S: StorageSingleSliced, R: ReserveStrategy> InPlaceIterable for IntoIter<T, S, R> {
    const EXPAND_BY: Option<NonZero<usize>> = NonZero::new(1);
    const MERGE_BY: Option<NonZero<usize>> = NonZero::new(1);
}

unsafe impl<T, S: StorageSingleSliced, R: ReserveStrategy> SourceIter for IntoIter<T, S, R> {
    type Source = Self;

    #[inline]
    unsafe fn as_inner(&mut self) -> &mut Self::Source {
        self
    }
}

unsafe impl<T, S: StorageSingleSliced, R: ReserveStrategy> AsDynArrIntoIter for IntoIter<T, S, R> {
    type Item = T;
    type Store = S;
    type Reserve = R;

    fn as_into_iter(&mut self) -> &mut super::IntoIter<Self::Item, S, R> {
        self
    }
    
    
}