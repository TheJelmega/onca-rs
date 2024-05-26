use core::{
    cmp, iter::{self, TrustedLen}, ops::Range, ptr, slice
};
use std::{mem::ManuallyDrop, ptr::NonNull};

use crate::{collections::{imp::array::RawArray, ReserveStrategy}, mem::StorageSingleSliced};

use super::{is_zero::IsZero, DynArr, IntoIter};



pub(super) trait SpecExtend<T, I> {
    fn spec_extend(&mut self, iter: I);
}

impl<T, I: Iterator<Item = T>, S: StorageSingleSliced, R: ReserveStrategy> SpecExtend<T, I> for DynArr<T, S, R> {
    default fn spec_extend(&mut self, iter: I) {
        self.extend_desugared(iter)
    }
}

impl<T, I: TrustedLen<Item = T>, S: StorageSingleSliced, R: ReserveStrategy> SpecExtend<T, I> for DynArr<T, S, R> {
    default fn spec_extend(&mut self, iter: I) {
        self.extend_trusted(iter)
    }
}

impl<T, S: StorageSingleSliced, R: ReserveStrategy> SpecExtend<T, IntoIter<T, S, R>> for DynArr<T, S, R> {
    fn spec_extend(&mut self, mut iter: IntoIter<T, S, R>) {
        unsafe {
            self.append_elements(iter.as_slice() as _);
        }
        iter.forget_remaining_elements();
    }
}

impl<'a, T: 'a + Clone, I: Iterator<Item = &'a T>, S: StorageSingleSliced, R: ReserveStrategy> SpecExtend<&'a T, I> for DynArr<T, S, R> {
    default fn spec_extend(&mut self, iter: I) {
        self.spec_extend(iter.cloned())
    }
}

impl<'a, T: 'a + Copy, S: StorageSingleSliced, R: ReserveStrategy> SpecExtend<&'a T, slice::Iter<'a, T>> for DynArr<T, S, R> {
    fn spec_extend(&mut self, iter: slice::Iter<T>) {
        let slice = iter.as_slice();
        unsafe { self.append_elements(slice) };
    }
}


//--------------------------------------------------------------

pub(super) trait SpecFromElem: Sized {
    fn from_elem<S: StorageSingleSliced, R: ReserveStrategy>(elem: Self, n:usize, storage: S) -> DynArr<Self, S, R>;
}

impl<T: Clone> SpecFromElem for T {
    default fn from_elem<S: StorageSingleSliced, R: ReserveStrategy>(elem: Self, n:usize, storage: S) -> DynArr<Self, S, R> {
        let mut arr = DynArr::with_capacity_in(n, storage);
        arr.extend_with(n, elem);
        arr
    }
}

impl<T: Clone + IsZero> SpecFromElem for T {
    #[inline]
    default fn from_elem<S: StorageSingleSliced, R: ReserveStrategy>(elem: Self, n:usize, storage: S) -> DynArr<Self, S, R> {
        if elem.is_zero() {
            return DynArr { arr: RawArray::with_capacity_zeroed_in(n, storage), len: n }
        }
        let mut arr = DynArr::with_capacity_in(n, storage);
        arr.extend_with(n, elem);
        arr
    }
}

impl SpecFromElem for i8 {
    #[inline]
    fn from_elem<S: StorageSingleSliced, R: ReserveStrategy>(elem: Self, n:usize, storage: S) -> DynArr<Self, S, R> {
        if elem == 0 {
            return DynArr { arr: RawArray::with_capacity_zeroed_in(n, storage), len: n }
        }
        let mut arr = DynArr::with_capacity_in(n, storage);
        unsafe {
            ptr::write_bytes(arr.as_mut_ptr(), elem as u8, n);
            arr.set_len(n);
        }
        arr
    }
}

impl SpecFromElem for u8 {
    #[inline]
    fn from_elem<S: StorageSingleSliced, R: ReserveStrategy>(elem: Self, n:usize, storage: S) -> DynArr<Self, S, R> {
        if elem == 0 {
            return DynArr { arr: RawArray::with_capacity_zeroed_in(n, storage), len: n }
        }
        let mut arr = DynArr::with_capacity_in(n, storage);
        unsafe {
            ptr::write_bytes(arr.as_mut_ptr(), elem, n);
            arr.set_len(n);
        }
        arr
    }
}

// A better way would be to implement this for all ZSTs which are `Copy` and have trivial `CLone`, but the latter cannot be detected currently
impl SpecFromElem for () {
    #[inline]
    fn from_elem<S: StorageSingleSliced, R: ReserveStrategy>(elem: Self, n:usize, storage: S) -> DynArr<Self, S, R> {
        let mut arr = DynArr::with_capacity_in(n, storage);
        // Safety: The capacity has just bee set to `n` and `()` is a ZST with trivial `Clone` implementation
        unsafe {
            arr.set_len(n);
        }
        arr
    }
}

//--------------------------------------------------------------

pub trait ExtendFromWithinSpec {
    /// # Safety
    /// 
    /// - `src` needs to be a valid index.
    /// - `self.capacity() - self.len()` must be `>= src.len()`.
    unsafe fn spec_extend_from_within(&mut self, src: Range<usize>);
}

impl<T: Clone, S: StorageSingleSliced, R: ReserveStrategy> ExtendFromWithinSpec for DynArr<T, S, R> {
    default unsafe fn spec_extend_from_within(&mut self, src: Range<usize>) {
        // SAFETY:
        // - len is increased only after initializing elements
        let (this, spare, len) = unsafe { self.split_at_spare_mut_with_len() };

        // SAFETY:
        // - caller guarantees that src is a valid index
        let to_clone = unsafe { this.get_unchecked(src) };

        iter::zip(to_clone, spare)
            .map(|(src, dst)| dst.write(src.clone()))
            // Note:
            // - Element was just initialized with `MaybeUninit::write`, so it's ok to increase len
            // - len is increased after each element to prevent leaks
            .for_each(|_| *len += 1);
    }
}

impl<T: Copy, S: StorageSingleSliced, R: ReserveStrategy> ExtendFromWithinSpec for DynArr<T, S, R> {
    unsafe fn spec_extend_from_within(&mut self, src: Range<usize>) {
        let count = src.len();
        {
            let (init, spare) = self.split_at_spare_mut();

            // SAFETY:
            // - caller guarantees that `src` is a valid index
            let source = unsafe { init.get_unchecked(src) };

            // SAFETY:
            // - Both pointer s are created from unique slice references (`&mut [_]`) so the yar evalid and do not overlap.
            // - Elements are :Copy so it's OK to copy them, without doing anything with the original values.
            // - `count` is euqal to the len of `source`, so source is valid for `count` reads.
            // - `.reserve(count)` guarantees that `spare.len() >= count` so spare is valid for `count` writes.
            unsafe { ptr::copy_nonoverlapping(source.as_ptr(), spare.as_mut_ptr() as _, count) };
        }

        // SAFETY:
        // - The elements were just initialized by `copy_nonoverlapping`
        self.len += count;
    }
}

//--------------------------------------------------------------

pub(super) trait SpecCloneIntoDynArray<T, S: StorageSingleSliced, R: ReserveStrategy> {
    fn clone_into(&self, target: &mut DynArr<T, S, R>);
}

impl<T: Clone, S: StorageSingleSliced, R: ReserveStrategy> SpecCloneIntoDynArray<T, S, R> for [T] {
    default fn clone_into(&self, target: &mut DynArr<T, S, R>) {
        // drop anything in target that will not be overwritten.
        target.truncate(self.len());

        // target.len <= self.len due to the truncate above, so the slices here are always in-bounds.
        let (init, tail) = self.split_at(target.len);

        // reuse the contained values' allocations/resources.
        target.clone_from_slice(init);
        target.extend_from_slice(tail);
    }
}

impl<T: Copy, S: StorageSingleSliced, R: ReserveStrategy> SpecCloneIntoDynArray<T, S, R> for DynArr<T, S, R> {
    fn clone_into(&self, target: &mut DynArr<T, S, R>) {
        target.clear();
        target.extend_from_slice(self);
    }
}

//--------------------------------------------------------------

/// Specialization trait used for DynArr::from_iter
/// 
/// ## The delegation graph:
/// 
/// +--------------+
/// | FromIterator |
/// +--------------+
///      |
///      v
/// +----+------------------------------+  +----------------------+
/// | SpecFromIter                    +--->+ SpecFromIterNexted   |
/// | where I:                        | |  | where I:             |
/// |   Iterator (default)------------+ |  |   Iterator (default) |
/// |   vec::IntoIter                 | |  |   TrustedLen         |
/// |   InPlaceCollect--(fallback to)-+ |  +----------------------+
/// +-----------------------------------+
pub(super) trait SpecFromIter<T, I> {
    fn from_iter(iter: I) -> Self;
}

impl<T, I: Iterator<Item = T>, S: StorageSingleSliced + Default, R: ReserveStrategy> SpecFromIter<T, I> for DynArr<T, S, R> where
    I: Iterator<Item = T>
{
    default fn from_iter(iter: I) -> Self {
        SpecFromIterNested::from_iter(iter)
    }
}

impl<T, S: StorageSingleSliced + Default, R: ReserveStrategy> SpecFromIter<T, IntoIter<T, S, R>> for DynArr<T, S, R> {
    fn from_iter(iter: IntoIter<T, S, R>) -> Self {
        // A common case is passing a dynamic array into a functio which immediately re-collects into a dynamic array.
        // We can short cirguit this if hte IntoIter has not been advanced at all.
        // When it has been advances, we can also reuse the memeory and move the data to the front.
        // But we only do so when the resulting DynArr wouldn't have amore unused capacity then creating it through the generic FromIterator implementation would.
        // That limitiation is not strictly necessary as DynArr's allocation behavior is intendtionally unspecified.
        // But it is a conservative choice.
        let (base_ptr, cap) = unsafe { iter.handle.resolve_raw(&*iter.storage) };
        let base_ptr: NonNull<T> = base_ptr.cast();
        let has_advanced = base_ptr != iter.ptr;
        if !has_advanced || iter.len() >= cap / 2 {
            unsafe {
                let it = ManuallyDrop::new(iter);
                if has_advanced {
                    ptr::copy(it.ptr.as_ptr(), base_ptr.as_ptr(), it.len());
                }
                return DynArr::from_raw_parts_in(it.handle, ptr::read(&*it.storage), it.len());
            }
        }

        let mut arr = DynArr::new();
        // must delegate to spec_extend(), since extend() itself delegates to spec_from for emty DynArrs
        arr.spec_extend(iter);
        arr
    }
}

//--------------------------------------------------------------

/// Another specialization trait for DynArr::from_iter necessary to manually prioritize overlappig specializations
/// see [`SpecFromIter`](super::SpecFromIter) for details.
pub(super) trait SpecFromIterNested<T, I> {
    fn from_iter(iter: I) -> Self;
}

impl<T, I: Iterator<Item = T>, S: StorageSingleSliced + Default, R: ReserveStrategy> SpecFromIterNested<T, I> for DynArr<T, S, R> {
    default fn from_iter(mut iter: I) -> Self {
        // Unroll the firs iteration, as the dynamic array is going to be expanded on this iteration in every case when the iterable is not emtpy,
        // but the loop is extend_desugared() is not going to see the dynamic array bing full in the few subsequent loop iterations.
        // So we get better branch predicition.
        let mut arr = match iter.next() {
            None => return DynArr::new(),
            Some(elem) => {
                let (lower, _) = iter.size_hint();
                let initial_capacity = cmp::max(RawArray::<T, S, R>::MIN_NON_ZERO_CAP, lower.saturating_add(1));
                let mut arr = DynArr::with_capacity(initial_capacity);
                unsafe {
                    // SAFETY: We requested capcity at least 1
                    ptr::write(arr.as_mut_ptr(), elem);
                    arr.set_len(1);
                }
                arr
            }
        };
        // Must delegate to spec_extend() since extend() itself delegates to spec_from for empty DynArr
        <DynArr<T, S, R> as SpecExtend<T, I>>::spec_extend(&mut arr, iter);
        arr
    }
}

impl<T, I: TrustedLen<Item = T>, S: StorageSingleSliced + Default, R: ReserveStrategy> SpecFromIterNested<T, I> for DynArr<T, S, R> {
    fn from_iter(mut iter: I) -> Self {
        let mut arr = match iter.size_hint() {
            (_, Some(upper)) => DynArr::with_capacity(upper),
            // TrustedLen contract guarantees that `size_hing() == (_, None)` means there are more than `usize::MAX` elements.
            // Since the previous branch would eagerly panic if the capacity is too large
            // (via `with_capacity`) we do the same here.
            _ => panic!("capacity overflow"),
        };
        // reuse extend specialization for TrustedLen
        arr.spec_extend(iter);
        arr
    }
}