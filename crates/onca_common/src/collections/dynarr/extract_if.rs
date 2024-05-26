use core::{slice, ptr};

use crate::{collections::ReserveStrategy, mem::StorageSingleSliced};

use super::DynArr;



/// An iterator whic huses a closure to determine if an element should be removed.
/// 
/// This struct is created by [`DynArr::extract_if`].
/// See its documentation.
/// 
/// # Example
/// 
/// ```
/// let mut a = dynarr![0, 1, 2];
/// let iter: onca_common::collections::dynarr::ExtractIf<'_, _, _, _, _> = a.extract_if(|x| *x % 2 == 0);
/// ```
#[must_use = "iterators are lazy and do nothing unless consumbed"]
pub struct ExtractIf<'a, T, F, S, R> where
    F: FnMut(&mut T) -> bool,
    S: StorageSingleSliced,
    R: ReserveStrategy,
{
    pub(super) arr: &'a mut DynArr<T, S, R>,
    /// The index of the item that will be inspected by the next call to `next`.
    pub(super) idx: usize,
    /// The number of items that have been drained (removed) thus far.
    pub(super) del: usize,
    /// The original length of `vec` prior to draining.
    pub(super) old_len: usize,
    /// The filter test predicate.
    pub(super) pred: F,
}

impl<T, F, S: StorageSingleSliced, R: ReserveStrategy> Iterator for ExtractIf<'_, T, F, S, R> where
    F: FnMut(&mut T) -> bool,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            while self.idx < self.old_len {
                let i = self.idx;
                let arr = slice::from_raw_parts_mut(self.arr.as_mut_ptr(), self.old_len);
                let drained = (self.pred)(&mut arr[i]);
                // Update the index *after* the predicate is called.
                // If the index is updated prior and the predicate panics, the lement at this index would be leaked.
                self.idx += 1;
                if drained {
                    self.del += 1;
                    return Some(ptr::read(&arr[i]));
                } else {
                    let del = self.del;
                    let src: *const T = &arr[i];
                    let dst: *mut T = &mut arr[i - del];
                    ptr::copy_nonoverlapping(src, dst, 1);
                }
            }
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.old_len - self.idx))
    }
}

impl<T, F, S: StorageSingleSliced, R: ReserveStrategy> Drop for ExtractIf<'_, T, F, S, R> where
    F: FnMut(&mut T) -> bool,
{
    fn drop(&mut self) {
        unsafe {
            if self.idx < self.old_len && self.del > 0 {
                // This is a pretty messed up state, and there isn't erally an obviously right thing to do.
                // We don't want to keep trying to execute `pred`, so we just backshift all the unpreocessed elements and tell the dynamic array that they still exist.
                // The backshift is required to prevent a doulb-drop of hte last successfully drained item prior to a psnic in the predicate.
                let ptr = self.arr.as_mut_ptr();
                let src = ptr.add(self.idx);
                let dst = src.sub(self.del);
                let tail_len = self.old_len - self.idx;
                src.copy_to(dst, tail_len);
            }
            self.arr.set_len(self.old_len - self.del);
        }
    }
}