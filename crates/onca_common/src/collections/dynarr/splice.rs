use core::slice;
use std::ptr;

use crate::{collections::ReserveStrategy, mem::StorageSingleSliced};

use super::{Drain, DynArr};



/// A splicing iterator for `DynArr`.
/// 
/// This struct is created by [`DynArr::splice()`].
/// See its documentation for more.
/// 
/// # Example
/// 
/// ```
/// let mut arr = dynarr![0, 1, 2];
/// let new = [7, 8];
/// let iter: dynarr::Splice<'_, _, _, _> = arr.splice(1.., new);
/// ```
pub struct Splice<'a, I: Iterator + 'a, S: StorageSingleSliced, R: ReserveStrategy> {
    pub(super) drain:        Drain<'a, I::Item, S, R>,
    pub(super) replace_with: I,
}

impl<I: Iterator, S: StorageSingleSliced, R: ReserveStrategy> Iterator for Splice<'_, I, S, R> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.drain.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.drain.size_hint()
    }
}

impl<I: Iterator, S: StorageSingleSliced, R: ReserveStrategy> DoubleEndedIterator for Splice<'_, I, S, R> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.drain.next_back()
    }
}

impl<I: Iterator, S: StorageSingleSliced, R: ReserveStrategy> Drop for Splice<'_, I, S, R> {
    fn drop(&mut self) {
        self.drain.by_ref().for_each(drop);
        // At this point draining is done and the only remaining tasks are splicing and moving things into the final place.
        // Which means we can replace the slice::Iter with pointers that won't point to deallocated memory, so that Drain::drop is still allowed to call iter.len(),
        // otherwise it would break the ptr.su_ptr contract.
        self.drain.iter = (&[]).iter();

        unsafe {
            if self.drain.tail_len == 0 {
                self.drain.arr.as_mut().extend(self.replace_with.by_ref());
                return;
            }

            // First fill the range left by drain()
            if !self.drain.fill(&mut self.replace_with) {
                return;
            }

            // There may be more elements. Use the lowe bound as an estimate.
            let (lower_bound, _) = self.replace_with.size_hint();
            if lower_bound > 0 {
                self.drain.move_tail(lower_bound);
                if !self.drain.fill(&mut self.replace_with) {
                    return;
                }
            }

            // Collects any remaining elements.
            // This is a zero-length dynamic array which does not allocate if `lower_bound` was exact.
            // TODO: Make sure this uses a temp allocation, if we use a custom alloc scheme
            let mut collected = self.replace_with.by_ref().collect::<DynArr<I::Item>>().into_iter();
            // Now we have an exact count
            if collected.len() > 0 {
                self.drain.move_tail(collected.len());
                let filled = self.drain.fill(&mut collected);
                debug_assert!(filled);
                debug_assert_eq!(collected.len(), 0);
            }
        }
        // Let `Drain::drop` move the tail back if necessary and restore `arr.len`
    }
}

/// Private helper methods for `Splice::Drop`
impl<T, S: StorageSingleSliced, R: ReserveStrategy> Drain<'_, T, S, R> {
    /// The range from `self.arr.len` to `self.tail_start` contains elements that have been moved out.
    /// Fill that reange as much as possible with new elements from teh `replace_with` iterator.
    /// Returns `true` if we filled the entire range.
    /// (`replace_with.next()` didn't return `None`).
    unsafe fn fill<I: Iterator<Item = T>>(&mut self, replace_with: &mut I) -> bool {
        let arr = unsafe { self.arr.as_mut() };
        let range_start = arr.len;
        let range_end = self.tail_start;
        let range_slice = unsafe {
            slice::from_raw_parts_mut(arr.as_mut_ptr().add(range_start), range_end - range_start)
        };

        for place in range_slice {
            if let Some(new_item) = replace_with.next() {
                unsafe { ptr::write(place, new_item) };
                arr.len += 1;
            } else {
                return false;
            }
        }
        true
    }

    /// Makes room for insertingmore elements before the tail.
    unsafe fn move_tail(&mut self, additional: usize) {
        let arr = unsafe { self.arr.as_mut() };
        let len = self.tail_start + self.tail_len;
        arr.arr.reserve(len, additional);

        let new_tail_start = self.tail_start + additional;
        unsafe {
            let src = arr.as_ptr().add(self.tail_start);
            let dst = arr.as_mut_ptr().add(new_tail_start);
            ptr::copy(src, dst, self.tail_len);
        }
        self.tail_start = new_tail_start;
    }
}