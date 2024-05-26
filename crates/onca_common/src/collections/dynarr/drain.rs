use core::{
    mem::ManuallyDrop,
    ptr::{self, NonNull},
    fmt,
    slice,
    mem,
};
use std::iter::{FusedIterator, TrustedLen};

use crate::{
    collections::{DoubleOrMinReserveStrategy, ReserveStrategy},
    mem::StorageSingleSliced,
};

use super::DynArr;





/// A draining iterator for `DynArr<T, ...>`.
/// 
/// This `struct` is created by [`DynArr::drain`].
/// See its documentation for more.
/// 
/// # Example
pub struct Drain<'a, T: 'a, S: StorageSingleSliced, R: ReserveStrategy = DoubleOrMinReserveStrategy> {
    /// Index of tail to preserve.
    pub(super) tail_start: usize,
    /// Lenght of the tail.
    pub(super) tail_len: usize,
    /// Current remaining range to remove.
    pub(super) iter: slice::Iter<'a, T>,
    pub(super) arr: NonNull<DynArr<T, S, R>>,
}

impl<T: fmt::Debug, S: StorageSingleSliced, R: ReserveStrategy> fmt::Debug for Drain<'_, T, S, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Drain").field(&self.iter.as_slice()).finish()
    }
}

impl<T, S: StorageSingleSliced, R: ReserveStrategy> Drain<'_, T, S, R> {
    /// Returns the remaining items of this iterator as a slice.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let mut arr = dynarr!['a', 'b', 'c'];
    /// let mut drain = arr.drain(..);
    /// assert_eq!(drain.as_slice(), &['a', 'b', 'c']);
    /// let _ = drain.next().unwrap();
    /// assert_eq!(drain.as_slice(), &['b', 'c']);
    /// ```
    #[must_use]
    pub fn as_slice(&self) -> &[T] {
        self.iter.as_slice()
    }

    /// Keep unyielded elements in the source `DynArr`.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let mut arr = dynarr!['a', 'b', 'c'];
    /// let mut drain = arr.drain(..);
    /// 
    /// assert_eq!(drain.next().unwrap(), 'a');
    /// 
    /// // This call keeps 'b' and 'c' in the array.
    /// drain.keep_rest();
    /// 
    /// // If we wouldn't call `keep_rest()`, `arr` would be empty.
    /// assert_eq!(arr, ['b', 'c']);
    /// ```
    pub fn keep_rest(self) {
        // A this moment layout looks like this:
        //
        // [head] [yielded by next] [unyielded] [yielded by next_back] [tail]
        //        ^-- start         \_________/-- unyieldable_len      \____/-- self.tail_len
        //                           ^-- unyielded_ptr                  ^-- tail
        //
        // Normally `Drop` impl would drop [unyielded] adn then move [tail] to the `start`.
        // Here we want to
        // 1. Move [unyielded] to `start`
        // 2. Move [tail] to a new start at `start`
        // 3. Update length of the original array t o`len(head) + len(unyielded) + len(tail)`
        //    a. In case of ZST, this is the only thing we want to do
        // 4. Do *not* drop self, as everything is put in a consistent state already, there is nothing to do
        let mut this = ManuallyDrop::new(self);

        unsafe {
            let source_arr = this.arr.as_mut();

            let start = source_arr.len;
            let tail = this.tail_start;

            let unyielded_len = this.iter.len();
            let unyielded_ptr = this.iter.as_slice().as_ptr();

            // ZST have no identity
            if core::mem::size_of::<T>() != 0 {
                let start_ptr = source_arr.as_mut_ptr().add(start);

                // memove back unyielded elements
                if unyielded_ptr != start_ptr {
                    ptr::copy(unyielded_ptr, start_ptr, unyielded_len);
                }

                // memmove back untouched tail
                if tail != (start + unyielded_len) {
                    let src = source_arr.as_ptr().add(tail);
                    let dst = start_ptr.add(unyielded_len);
                    ptr::copy(src, dst, this.tail_len);
                }
            }

            source_arr.set_len(start + unyielded_len + this.tail_len);
        }
    }
}

impl<'a, T, S: StorageSingleSliced, R: ReserveStrategy> AsRef<[T]> for Drain<'a, T, S, R> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

unsafe impl<'a, T: Sync, S: Sync + StorageSingleSliced, R: ReserveStrategy> Sync for Drain<'a, T, S, R> {}
unsafe impl<'a, T: Send, S: Send + StorageSingleSliced, R: ReserveStrategy> Send for Drain<'a, T, S, R> {}

impl<T, S: StorageSingleSliced, R: ReserveStrategy> Iterator for Drain<'_, T, S, R> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|elt| unsafe { unsafe { ptr::read(elt as *const _) } })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T, S: StorageSingleSliced, R: ReserveStrategy> DoubleEndedIterator for Drain<'_, T, S, R> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|elt| unsafe { ptr::read(elt as *const _) })
    }
}

impl<T, S: StorageSingleSliced, R: ReserveStrategy> Drop for Drain<'_, T, S, R> {
    fn drop(&mut self) {
        // Moves back the un-`Drain`ed elements to restore the original `DynArr`
        struct DropGuard<'r, 'a, T, S: StorageSingleSliced, R: ReserveStrategy>(&'r mut Drain<'a, T, S, R>);

        impl<'r, 'a, T, S: StorageSingleSliced, R: ReserveStrategy> Drop for DropGuard<'r, 'a, T, S, R> {
            fn drop(&mut self) {
                if self.0.tail_len > 0 {
                    unsafe {
                        let source_arr = self.0.arr.as_mut();
                        // memmove back untouched tail, update to new lenght
                        let start = source_arr.len;
                        let tail = self.0.tail_start;
                        if tail != start {
                            let src = source_arr.as_ptr().add(tail);
                            let dst = source_arr.as_mut_ptr().add(start);
                            ptr::copy(src, dst, self.0.tail_len);
                        }
                        source_arr.set_len(start + self.0.tail_len);
                    }
                }
            }
        }

        let iter = mem::take(&mut self.iter);
        let drop_len = iter.len();

        let mut arr = self.arr;

        if mem::size_of::<T>() == 0 {
            // ZST have no identity, so we don't need to move them around, we only need to drop the correct amount.
            // this can be achieved by manipulationg the DynArr length instead of moving value out from `iter`
            unsafe {
                let arr = arr.as_mut();
                let old_len = arr.len;
                arr.set_len(old_len + drop_len + self.tail_len);
                arr.truncate(old_len + self.tail_len);
            }

            return;
        }

        // ensure elements are moved back into their appropriate places, even when drop_in_place panics
        let _guard = DropGuard(self);

        if drop_len == 0 {
            return;
        }

        // as_slice() must only be called when iter.len() is > 0 because it also gets touched by dynarr::Splice which may turn it into a dangling pointer
        // which ould make it and the dynarr pointer point to different allocations which would lead to invalid pointer arithmetic below.
        let drop_ptr = iter.as_slice().as_ptr();

        unsafe {
            // drop_ptr comes from a slice::Iter which only gives us a &[T] out of drop_in_place a pointer with mutable provenance is necessary.
            // Therefore we must reconstruct it from the original dynarr but also avoid creating a &mut to the fron
            // since that could invalidate raw pointers to it which some unsafe code might rely one
            let arr_ptr = arr.as_mut().as_mut_ptr();
            let drop_offset = drop_ptr.sub_ptr(arr_ptr);
            let to_drop = ptr::slice_from_raw_parts_mut(arr_ptr.add(drop_offset), drop_len);
            ptr::drop_in_place(to_drop);
        }
    }
}

impl<T, S: StorageSingleSliced, R: ReserveStrategy> ExactSizeIterator for Drain<'_, T, S, R> {
    fn is_empty(&self) -> bool {
        self.iter.is_empty()
    }
}

unsafe impl<T, S: StorageSingleSliced, R: ReserveStrategy> TrustedLen for Drain<'_, T, S ,R> {}

impl<T, S: StorageSingleSliced, R: ReserveStrategy> FusedIterator for Drain<'_, T, S, R> {}