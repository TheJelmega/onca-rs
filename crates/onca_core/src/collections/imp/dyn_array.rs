use core::{
    fmt,
    slice::{self, SliceIndex},
    iter,
    iter::FusedIterator,
    mem::{self, MaybeUninit, ManuallyDrop},
    ops::{RangeBounds, Range, Deref, DerefMut, Index, IndexMut},
    ptr::{self, NonNull},
    hash::{Hash, Hasher},
    array,
    cmp,
    any::Any,
    marker::PhantomData,
};
use std::collections::TryReserveError;
use crate::{
    alloc::{Layout, ScopedAlloc, UseAlloc},
    collections::{ExtendFunc, ExtendElement, ExtendWith, SetLenOnDrop, SpecExtendFromWithin, SpecCloneFrom, SpecExtend, IsZero, SpecFromIterNested, SpecFromIter, impl_slice_partial_eq_generic},
};


/// Trait representing the internal storage for any DynamicArray implementation
pub trait DynArrayBuffer<T> {
    fn new() -> Self;
    fn with_capacity(capacity: usize) -> Self;
    fn with_capacity_zeroed(capacity: usize) -> Self;

    fn reserve(&mut self, len: usize, additional: usize) -> usize;
    fn try_reserve(&mut self, len: usize, additional: usize) -> Result<usize, TryReserveError>;

    fn reserve_exact(&mut self, len: usize, additional: usize) -> usize;
    fn try_reserve_exact(&mut self, len: usize, additional: usize) -> Result<usize, TryReserveError>;

    fn shrink_to_fit(&mut self, cap: usize);

    fn capacity(&self) -> usize;

    fn as_ptr(&self) -> *const T;
    fn as_mut_ptr(&mut self) -> *mut T;

    fn layout(&self) -> Layout;
    fn allocator_id(&self) -> u16;
}


// A [`DynArray`] that exlusively stores its data on the stack, i.e. all elements are stored inline.
pub struct DynArray<T, B: DynArrayBuffer<T>> {
    len            : usize,
    pub(crate) buf : B,
    _p             : PhantomData<T>
}

impl<T, B: DynArrayBuffer<T>> DynArray<T, B> {
    #[inline]
    pub fn new() -> Self {
        Self {len: 0, buf: B::new(), _p: PhantomData }
    }

    #[inline]
    pub const fn const_new(buf: B) -> Self {
        Self {len: 0, buf, _p: PhantomData }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self { len: 0, buf: B::with_capacity(capacity), _p: PhantomData }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.buf.capacity()
    }

    #[inline]
    pub fn reserve(&mut self, additional: usize) -> usize {
        self.buf.reserve(self.len, additional)
    }

    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<usize, TryReserveError> {
        self.buf.try_reserve(self.len, additional)
    }

    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) -> usize {
        self.buf.reserve_exact(self.len, additional)
    }

    #[inline]
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<usize, TryReserveError> {
        self.buf.try_reserve_exact(self.len, additional)
    }

    pub fn shrink_to_fit(&mut self) {
        if self.capacity() > self.len {
            self.buf.shrink_to_fit(self.len);
        }
    }

    pub fn shrink_to(&mut self, min_capacity: usize) {
        if self.capacity() > min_capacity {
            self.buf.shrink_to_fit(cmp::min(self.len, min_capacity))
        }
    }

    pub fn truncate(&mut self, len: usize) {
        if len >= self.len {
            return;
        }

        // This is safe because:
        // * The slice passed to `drop_in_place` is valid; the `len > self.len` case avoids creating an invalid slice, and
        // * the `len` of the dynarray is shrunk before calling `dop_in_place`, such that no value will be dropped twice in case `drop_in_place` were to panic once (if it panics twice, the program aborts).
        unsafe {
            let remaining_len = self.len - len;
            let s = ptr::slice_from_raw_parts_mut(self.as_mut_ptr().add(len), remaining_len);
            self.len = len;
            ptr::drop_in_place(s);
        }
    }

    #[inline]
    pub fn as_slice(&self) -> &[T] {
        self
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self
    }

    pub fn as_ptr(&self) -> *const T {
        self.buf.as_ptr() as *const T
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.buf.as_mut_ptr() as *mut T
    }

    pub unsafe fn set_len(&mut self, new_len: usize) {
        assert!(new_len <= self.capacity());
        self.len = new_len;
    }

    pub fn swap_remove(&mut self, index: usize) -> T {
        let len = self.len();
        assert!(index < self.len, "swap_remove index (is {index}) should be < len (is {len})");

        unsafe {
            // We replace self[index] with the last element.
            // Note that if the bounds check above succeeds, there must be a last element (which can be self[index] itself)
            let value = ptr::read(self.as_ptr().add(index));
            let base_ptr = self.as_mut_ptr();
            ptr::copy(base_ptr.add(self.len - 1), base_ptr.add(index), 1);
            self.len -= 1;
            value
        }
    }

    pub fn insert(&mut self, index: usize, element: T) {
        let len = self.len;
        assert!(index <= len, "insert index (is {index}) should be <= len (is {len})");

        let new_cap = self.buf.reserve(self.len, 1);
        
        // If the buffer is not resizable, we will just panic, as that's something that the user should make sure never happens
        if len == new_cap {
            panic!("Tried to reserve space in a GenericDynArray that doesn't have a resizable buffer");
        }

        unsafe {
            let ptr = self.as_mut_ptr().add(index);

            if index < self.len {
                // Shift everyting to make sapce. (Duplicating the `index`th element into two consecutive places.)
                ptr::copy(ptr, ptr.add(1), len - index);
            } else if index > self.len {
                panic!("insert index (is {index}) should be <= len (is {len})");
            }

            // Write it in, overwriting the first copy of the `index`th element
            ptr::write(ptr, element);
            self.len += 1;
        }
    }

    pub fn remove(&mut self, index: usize) -> T {
        let len = self.len;
        assert!(index < len);
        unsafe {
            let ret : T;
            {
                // The place we are taking from
                let ptr = self.as_mut_ptr().add(index);

                // Copy it out, unsafely having a copy of the value on the stack and in the vector at the same time
                ret = ptr::read(ptr);
                
                // Shift everything to fill in tha spot
                ptr::copy(ptr.add(1), ptr, len - index - 1);
            }
            self.len = len - 1;
            ret
        }
    }

    pub fn remove_first_if<F>(&mut self, mut f: F) -> Option<T> where
        F: FnMut(&T) -> bool
    {
        let mut idx = None;
        for (id, val) in self.iter().enumerate() {
            if f(val) {
                idx = Some(id);
                break;
            }
        }
        idx.map(|idx| self.remove(idx))
    }

    pub fn retain<F>(&mut self, mut pred: F)
    where
        F : FnMut(&T) -> bool
    {
        self.retain_mut(|elem| pred(elem))
    }

    pub fn retain_mut<F>(&mut self, mut pred: F)
    where
        F : FnMut(&mut T) -> bool
    {
        let original_len = self.len();
        // Avoid double drop if the drop guard is not executed, since we may make some holes during the process.
        self.len = 0;

        // DynArr: [Kept, Kept, Hole, Hole, Hole, Hole, Unchecked, Unchecked]
        //         |<-             processed_len    ->| ^- next to check
        //                    |<-  deleted_count   -> |
        //         |<-             original_len                           ->|
        // Kept: Elements which predicate returns tru on
        // Hole: Moved or dropped element slot.
        // Unchecked: Unchecked valid elements.
        //
        // THis drop queard will be invoked whan predicate or `drop` of element panicked.
        // It shifts unchecked elements to cover holes and sets 'set_len' to the correct length.
        // In cases when the predicate and `drop` never panic, it will be optimized out.
        struct BackshiftOnDrop<'a, T, B: DynArrayBuffer<T>> {
            dynarr        : &'a mut DynArray<T, B>,
            processed_len : usize,
            deleted_count : usize,
            original_len  : usize,
        }

        impl<T, B: DynArrayBuffer<T>> Drop for BackshiftOnDrop<'_, T, B> {
            fn drop(&mut self) {
                if self.deleted_count > 0 {
                    // SAFETY: Trailing unchecked items must be valid since we never touch them.
                    unsafe {
                        ptr::copy(
                            self.dynarr.as_ptr().add(self.processed_len),
                            self.dynarr.as_mut_ptr().add(self.processed_len - self.deleted_count),
                            self.original_len - self.processed_len
                        );
                    }
                }

                // SAFETY: After filling holes, all items are in contiguous memory.
                unsafe {
                    self.dynarr.set_len(self.original_len - self.deleted_count);
                }
            }
        }

        let mut g = BackshiftOnDrop { dynarr: self, processed_len: 0, deleted_count: 0, original_len };

        fn process_loop<F, T, B: DynArrayBuffer<T>, const DELETED: bool>(
            original_len: usize,
            f: &mut F,
            g: &mut BackshiftOnDrop<'_, T, B>
        ) where
            F : FnMut(&mut T) -> bool
        {
            while g.processed_len != original_len {
                // SAFETY: Unchecked element must be valid
                let cur = unsafe { &mut *g.dynarr.as_mut_ptr().add(g.processed_len) };
                if !f(cur) {
                    // Advance early to avoid double drop if `drop_in_place` panicked.
                    g.processed_len += 1;
                    g.deleted_count += 1;
                    // SAFETY: We never touch this element again after we dropped it.
                    unsafe { ptr::drop_in_place(cur) };
                    // We alreaady advanced the counter.
                    if DELETED {
                        continue;
                    } else {
                        break;
                    }
                }
                if DELETED {
                    // SAFETY: `deleted_count` > 0, so the hole slot must not overlap with the current elements.
                    // We use copy for move, and never touch this element again.
                    unsafe {
                        let hole_slot = g.dynarr.as_mut_ptr().add(g.processed_len - g.deleted_count);
                        ptr::copy_nonoverlapping(cur, hole_slot, 1);
                    }
                }
                g.processed_len += 1;
            }
        }

        // Stage 1: Nothing was deleted
        process_loop::<F, T, B, false>(original_len, &mut pred, &mut g);

        // Stage 2: Some elements wer deleted
        process_loop::<F, T, B, true>(original_len, &mut pred, &mut g);

        // This should be able to be optimized to `set_len` by LLVM (according the rust's Vec implementation)
        drop(g);
    }

    pub fn dedup_by_key<F, K>(&mut self, mut key: F)
    where
        F : FnMut(&mut T) -> K,
        K : PartialEq<K>
    {
        self.dedup_by(|a, b| key(a) == key(b))
    }

    pub fn dedup_by<F>(&mut self, mut same_bucket: F) 
    where
        F : FnMut(&mut T, &mut T) -> bool
    {
        let len = self.len;
        if len <= 1 {
            return;
        }

        // INVARIANT: dynarr.len > read > write - 1 >= 0
        struct FillGapOnDrop<'a, T, B: DynArrayBuffer<T>> {
            // Offset of the element we want to check if it is a duplicate
            read   : usize,
            // Offset of the place where we want to place teh non-duplicate when we find it.
            write  : usize,
            // The StaticDynArray tha would need correction if `same_bucket` panicked.
            dynarr : &'a mut DynArray<T, B>
        }

        impl<'a, T, B: DynArrayBuffer<T>> Drop for FillGapOnDrop<'a, T, B> {
            fn drop(&mut self) {
                // This code gets executed when `same_bucket` panics.
                
                // SAFETY: invariant guarantees that `read - write` and `len - read` never overflow and that the copy is always in-bounds.
                unsafe {
                    let ptr = self.dynarr.as_mut_ptr();
                    let len = self.dynarr.len();

                    // How many iterms were left when `same_bucket` panicked.
                    // Basically dynarr[read..].len()
                    let items_left = len.wrapping_sub(self.read);

                    // Pointer to first item in dynarr[write..write+items_left] slice
                    let dropped_ptr = ptr.add(self.write);
                    // Pointer to first item in dynarr[read..] slice
                    let valid_ptr = ptr.add(self.read);

                    // Copy `dynarr[read..]` to `dynarr[write..write+items_left]`.
                    // The slices can overlp, so `copy_nonoverlapping` cannot be used.
                    ptr::copy(valid_ptr, dropped_ptr, items_left);

                    // How many items have been already dropped.
                    // Basicxally dynarr[read..write].len()
                    let dropped = self.read.wrapping_sub(self.write);

                    self.dynarr.set_len(len - dropped)
                }
            }
        }

        let mut gap = FillGapOnDrop { read: 1, write: 1, dynarr: self };
        let ptr = gap.dynarr.as_mut_ptr();

        // Drop items while going through StaticDynArray, it should be more efficient than doing slice partition_dedup + truncate

        // SAFETY: Becausee of the invariant, read_ptr, prev_ptr, and write_ptr are always in-bounds and read_ptr never aliases prev_ptr
        unsafe {
            while gap.read < len {
                let read_ptr = ptr.add(gap.read);
                let prev_ptr = ptr.add(gap.write.wrapping_sub(1));

                if same_bucket(&mut *read_ptr, &mut *prev_ptr) {
                    // Increase `gap_read` now since th drop may panic.
                    gap.read += 1;
                    // We have fond a duptlicate, frop it in place
                    ptr::drop_in_place(read_ptr);
                } else {
                    let write_ptr = ptr.add(gap.write);

                    // Because `read_ptr` can be equal to `write_ptr`, we have to use `copy`
                    ptr::copy(read_ptr, write_ptr, 1);

                    // We have filled that place, so go further
                    gap.write += 1;
                    gap.read += 1;
                }
            }

            // Technically we could let `gap` clea up with its Drop, but when `same_bucket` is guaranteed to not panic, this bloats the code get a little, so we just do it manually
            gap.dynarr.set_len(gap.write);
            mem::forget(gap);
        }
    }

    #[inline]
    pub fn push(&mut self, value: T) {
        let new_cap = self.reserve(1);

        // If the buffer is not resizable, we will just panic, as that's something that the user should make sure never happens
        if self.len == new_cap {
            panic!("Tried to reserve space in a GenericDynArray that doesn't have a resizable buffer");
        }

        unsafe { 
            let end = self.as_mut_ptr().add(self.len);
            ptr::write(end, value); 
            self.len += 1;
        }
    }

    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            unsafe {
                self.len -= 1;
                Some(ptr::read(self.as_ptr().add(self.len)))
            }
        }
    }

    pub fn append<B2: DynArrayBuffer<T>>(&mut self, other: &mut DynArray<T, B2>) {
        unsafe {
            self.append_elements(other.as_slice() as _);
            other.set_len(0);
        }
    }

    unsafe fn append_elements(&mut self, other: *const [T]) {
        let count = (*other).len();
        if count == 0 {
            return;
        }

        let new_cap = self.reserve(count);

        // If the buffer is not resizable, we will just panic, as that's something that the user should make sure never happens
        if self.len == new_cap {
            panic!("Tried to reserve space in a GenericDynArray that doesn't have a resizable buffer");
        }

        let len = self.len;
        ptr::copy_nonoverlapping(other as *const T, self.as_mut_ptr().add(len), count);
        self.len += count;
    }

    pub fn drain<R: RangeBounds<usize>>(&mut self, range: R) -> Drain<'_, T, B> {
        // Memeory safety
        //
        // Whne the Drain is first created, it shortens the length of the source dynarr to make suer no uninitialized or moved-from elements are accessible at all, if the Drain's destructor never gets to run.
        //
        // Drain will prt::read out the values to remove.
        // When finished, remaining tail of the dynarr is copied back to cover the hole, and the dynarr length is restored to the new lenght
        let len = self.len();
        let Range { start, end } = slice::range(range, ..len);

        unsafe {
            // Set self.len to start, to be safe in case Drain is leaked.
            self.set_len(start);
            // Use the borrow in the IterMut to indicate borrowing behavior of the whole Drain iterator (like &mut T)
            let range_slice = slice::from_raw_parts_mut(self.as_mut_ptr().add(start), end - start);
            Drain { 
                tail_start: end, 
                tail_len: len - end, 
                iter: range_slice.iter(), 
                dynarr: NonNull::from(self)
            }
        }
    }

    pub fn clear(&mut self) {
        let elems: *mut [T] = self.as_mut_slice();

        // SAFETY:
        // - `elems` comes directly from `as_mut_slice` and is therefore valid.
        // - Setting `self.len` before calling `drop_in_place` means that, if an element's `Drop` impl panics, the dynarr's `Drop` impl will do nothing (leaking the rest of the elements) instead of dropping some twice
        unsafe {
            self.len = 0;
            ptr::drop_in_place(elems);
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }
    
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn layout(&self) -> Layout {
        self.buf.layout()
    }

    #[inline]
    pub fn allocator_id(&self) -> u16 {
        self.buf.allocator_id()
    }

    pub fn split_off(&mut self, at: usize) -> Self {
        assert!(at < self.len);

        let _scope_alloc = ScopedAlloc::new(UseAlloc::Id(self.allocator_id()));

        if at == 0 {
            return mem::replace(
                self, 
                Self::with_capacity(self.capacity())  
            );
        }

        let other_len = self.len - at;
        let mut other = DynArray::with_capacity(self.capacity());

        // Unsafely `set_len` and copy items to `other`.
        unsafe {
            self.set_len(at);
            other.set_len(other_len);

            ptr::copy_nonoverlapping(self.as_ptr().add(at), other.as_mut_ptr(), other.len);
        }
        other
    }

    pub fn resize_with<F>(&mut self, new_len: usize, f: F)
    where
        F : FnMut() -> T
    {
        let len = self.len;
        if new_len > len {
            self.extend_with(new_len - len, ExtendFunc(f));
        } else {
            self.truncate(new_len);
        }
    }

    pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<T>] {
        // Note:
        // This method is not implementated in terms of 'split_at_spare_mut`, to prevent invalidation of pointer to the buffer
        unsafe {
            slice::from_raw_parts_mut(
                self.as_mut_ptr().add(self.len) as *mut MaybeUninit<T>,
                self.buf.capacity() - self.len
            )
        }
    }

    #[inline]
    pub fn split_at_spare_mut(&mut self) -> (&mut [T], &mut [MaybeUninit<T>]) {
        // SAFETY :
        // - len is ignored and so never changed
        let (init, spare, _) = unsafe { self.split_at_spare_mut_with_len() };
        (init, spare)
    }

    unsafe fn split_at_spare_mut_with_len(&mut self) -> (&mut [T], &mut [MaybeUninit<T>], &mut usize) {
        let ptr = self.as_mut_ptr();
        // SAFETY:
        // - `ptr` is quaranteed to be valid for `self.len` elements
        //- but the alloction extends out ot `self.buf.capacity()` elements, possibley uninitialized
        let spare_ptr = unsafe { ptr.add(self.len) };
        let spare_ptr = spare_ptr.cast::<MaybeUninit<T>>();
        let spare_len = self.buf.capacity() - self.len;

        // SAFETY:
        // - `ptr` is guaranteed to be valid for `self.len` elements
        // - `spare_ptr` is pointing one element past the buffer, so it doesn't overlap with `initialized`
        unsafe {
            let initialized = slice::from_raw_parts_mut(ptr, self.len);
            let spare = slice::from_raw_parts_mut(spare_ptr, spare_len);

            (initialized, spare, &mut self.len)
        }
    }

    fn extend_with<E: ExtendWith<T>>(&mut self, n: usize, mut value: E){
        let cap = self.reserve(n);

        // Clamp `n` to not overflow in case the array has a static size
        let n = n.min(cap - self.len);

        unsafe {
            let mut ptr = self.as_mut_ptr().add(self.len);
            // Use SetLenOnDrop to work around a bug where compiler might not realize the store through `ptr` through self.set_len() don't alias
            // NOTE(jel): not 100% sure which bug this refers to, so just expect this bug to still exists
            let mut local_len = SetLenOnDrop::new(&mut self.len);

            // Write all elements except the last one
            for _ in 1..n {
                ptr::write(ptr, value.next());
                ptr = ptr.add(1);
                // Increment th length in every step in case next() panics
                local_len.increment_len(1);
            }

            if n > 0 {
                // We can write last element directly without cloning needlessly
                ptr::write(ptr, value.last());
                local_len.increment_len(1);
            }

            // len set by scope guard
        }
    }

    /// Note that if after removing the range, the replace_with iterator would contain more elements than there is space in the dynarr, the dynarr will be filled and any aditional elements will be dropped.
    pub fn splice<R, I>(&mut self, range: R, replace_with: I) -> Splice<'_, I::IntoIter, B>
    where
        R : RangeBounds<usize>,
        I : IntoIterator<Item = T>,
    {
        Splice { drain: self.drain(range), replace_with: replace_with.into_iter() }
    }

    // Leaf method to  which various SpecFrom/SpecExtend implementations delegate when the y have no furter optimizations to apply
    // Ignores remaining elements when array is empty
    fn extend_desugared<I: Iterator<Item = T>>(&mut self, mut iterator: I) {
        // This is the case for a general iterator.
        //
        // This function should be the moral equivalent of:
        //
        //      for item in iterator {
        //          self.push(item);
        //      }
        while let Some(element) = iterator.next() {
            let len = self.len();
            let old_cap = self.capacity();
            if len == old_cap {
                let (lower, _) = iterator.size_hint();
                let new_cap = self.buf.reserve(self.len, lower);

                // Buffer can't resize, so just bail
                if new_cap == old_cap {
                    return;
                }
            }

            unsafe {
                ptr::write(self.as_mut_ptr().add(len), element);
                // Since next() executes user code which can panic, we have to bump the length after each step
                self.set_len(len + 1);
            }
        }
    }
}

impl<T: Clone, B: DynArrayBuffer<T>> DynArray<T, B> {
    pub fn resize(&mut self, new_len: usize, value: T) {
        if new_len > self.len {
            self.extend_with(new_len - self.len, ExtendElement(value));
        } else {
            self.truncate(new_len);
        }
    }

    pub fn extend_from_slice(&mut self, other: &[T]) {
        self.spec_extend(other.iter())
    }

    pub fn extend_from_within<R>(&mut self, src: R)
    where
        R : RangeBounds<usize>
    {
        let range = slice::range(src, ..self.len);
        self.reserve(range.len());

        // SAFETY:
        // - `slice::range` guarantees that the given reange is valid for indexing self
        unsafe {
            self.spec_extend_from_within(range);
        }
    }
}

impl<T: PartialEq, B: DynArrayBuffer<T>> DynArray<T, B> {
    #[inline]
    pub fn dedup(&mut self) {
        self.dedup_by(|a, b| a == b)
    }
}

//------------------------------------------------------------------------------------------------------------------------------

impl<T, I, B> SpecExtend<T, I> for DynArray<T, B> 
where
    I : Iterator<Item = T>,
    B : DynArrayBuffer<T>
{
    default fn spec_extend(&mut self, iter: I) {
        self.extend_desugared(iter)
    }
}

/*
impl<T, I, const N: usize> SpecExtend<T, I> for StaticDynArray<T, N> 
where
    I : TrustedLen<Item = T>
{
    default fn spec_extend(&mut self, iter: T) {
        // THis is the case for a TrustedLen iterator
        let (low, high) = iter.size_hint();
        unsafe {
            let mut ptr = self.as_mut_ptr().add(self.len());
            let mut local_len = SetLenOnDrop::new(&mut self.len);
            let take_count = high.and_then(|x| core::cmp::min(x, N)).unwrap_or(N);

            iter.take(take_count).for_each(move |element| {
                ptr::write(ptr, element);
                ptr = ptr.add(1);
                // Since the loop executes user code which can panic, we have to bump the pointer after each step.
                local_len.increment_len(1);
            });
        }
    }
}
*/

impl<T, B: DynArrayBuffer<T>, C: DynArrayBuffer<T>> SpecExtend<T, IntoIter<T, C>> for DynArray<T, B> {
    fn spec_extend(&mut self, mut iter: IntoIter<T, C>) {
        let old_len = self.len;
        unsafe {
            self.append_elements(iter.as_slice() as _);
        }
        iter.forget_or_drop_remaining_elements(self.len - old_len);
    }
}

impl<'a, T: 'a, I, B: DynArrayBuffer<T>> SpecExtend<&'a T, I> for DynArray<T, B>
where
    I : Iterator<Item = &'a T>,
    T : Clone
{
    default fn spec_extend(&mut self, iter: I) {
        self.spec_extend(iter.cloned())
    }
}

impl<'a, T: 'a + Copy, B: DynArrayBuffer<T>> SpecExtend<&'a T, slice::Iter<'a, T>> for DynArray<T, B> {
    fn spec_extend(&mut self, iter: slice::Iter<'a, T>) {
        let slice = iter.as_slice();
        unsafe { self.append_elements(slice) }
    }
}

//--------------------------------------------------------------

pub trait SpecFromElem<B: DynArrayBuffer<Self>> : Sized {
    fn from_elem(elem: Self, n: usize) -> DynArray<Self, B>;
}

impl<T: Clone, B: DynArrayBuffer<Self>> SpecFromElem<B> for T{
    default fn from_elem(elem: Self, n: usize) -> DynArray<Self, B> {
        let mut dynarr = DynArray::new();
        dynarr.extend_with(n, ExtendElement(elem));
        dynarr
    }
}

impl<T: Clone + IsZero, B: DynArrayBuffer<Self>> SpecFromElem<B> for T {
    #[inline]
    default fn from_elem(elem: Self, n: usize) -> DynArray<Self, B> {
        if elem.is_zero() {
            return DynArray{ len: n, buf: B::with_capacity_zeroed(n), _p: PhantomData };
        }
        let mut dynarr = DynArray::new();
        dynarr.extend_with(n, ExtendElement(elem));
        dynarr
    }
}

impl<B: DynArrayBuffer<i8>> SpecFromElem<B> for i8 {
    #[inline]
    fn from_elem(elem: Self, n: usize) -> DynArray<Self, B> {
        if elem == 0 {
            return DynArray{ len: n, buf: B::with_capacity_zeroed(n), _p: PhantomData };
        }
        unsafe {
            let mut dynarr = DynArray::new();
            ptr::write_bytes(dynarr.as_mut_ptr(), elem as u8, n);
            dynarr.set_len(n);
            dynarr
        }
    }
}

impl<B: DynArrayBuffer<u8>> SpecFromElem<B> for u8 {
    #[inline]
    fn from_elem(elem: Self, n: usize) -> DynArray<Self, B> {
        if elem == 0 {
            return DynArray{ len: n, buf: B::with_capacity_zeroed(n), _p: PhantomData };
        }
        unsafe {
            let mut dynarr = DynArray::new();
            ptr::write_bytes(dynarr.as_mut_ptr(), elem, n);
            dynarr.set_len(n);
            dynarr
        }
    }
}

//--------------------------------------------------------------

impl<T, I, B> SpecFromIterNested<T, I> for DynArray<T, B> 
where
    I : Iterator<Item = T>,
    B : DynArrayBuffer<T>
{
    default fn from_iter(mut iter: I) -> Self {
        // Unroll the first iteration, as the dynarr is going to be expanded on this iteration in every case when the iterable is not empyt, 
        // but the loop is extend_desugared() is not going to see the vector being full in the few subsequent loop iterations.
        // So we get better branch prediction.
        let mut dynarr = match iter.next() {
            None => return DynArray::new(),
            Some(element) => {
                let (lower, _) = iter.size_hint();
                const MIN_CAP : usize = 8;
                let initial_capacity = cmp::max(MIN_CAP, lower.saturating_add(1));
                let mut dynarr = DynArray::with_capacity(initial_capacity);
                // The rare case where the backend has no memory
                if dynarr.capacity() == 0 {
                    return dynarr;
                }

                unsafe {
                    // SAFETY: We requested capacity for at least 1 element
                    ptr::write(dynarr.as_mut_ptr(), element);
                    dynarr.set_len(1);
                }
                dynarr
            }
        };

        // must delegate to spec_extend() since extend itself delegates to spec_from for empty DynArrays
        <DynArray<T, B> as SpecExtend<T, I>>::spec_extend(&mut dynarr, iter);
        dynarr
    }
}

//--------------------------------------------------------------

impl<T, I, B> SpecFromIter<T, I> for DynArray<T, B>
where
    I : Iterator<Item = T>,
    B : DynArrayBuffer<T>
{
    default fn from_iter(iter: I) -> Self {
        SpecFromIterNested::from_iter(iter)
    }
}

// NOTE(jel): Because of some limiations on `min_specialization`, specifically using `B` twice, i.e. in `IntoIter<T, B>` and `GenericDynArray<T, B>`,
//            is not allowed atm in `min_specialization`, even though they are unrelated to the actual specialization
impl<T, B: DynArrayBuffer<T>> SpecFromIter<T, IntoIter<T, B>> for DynArray<T, B> {
    fn from_iter(iter: IntoIter<T, B>) -> Self {
        // A common case is passing a dynarr into a function which immediately re-collects inot a dynarray.
        // We can short circuit this if the IntoIter has not been advanced at all.
        // When it has been advanced, we can also reuse the memeory and move the data to the fron.
        // But we only do so when the resutling dynarr wouldn't have more used capacity than creating it thhrough the generic fromIterator implementation would.
        // That limitaition is not strictly necessary as DynArray's alloction behavior is intentionally unspecified.
        // But it is a conservative choice.

        let mut dynarr = DynArray::new();

        let has_advanced = iter.buf.as_ptr() as *const _ != iter.ptr;
        if !has_advanced || iter.len() >= iter.buf.capacity() / 2 {
            unsafe {
                let mut it = ManuallyDrop::new(iter);
                if has_advanced {
                    let len = it.len();
                    ptr::copy(it.ptr, it.buf.as_mut_ptr(), len);
                    it.buf.set_len(len);
                }
                return unsafe { mem::transmute_copy(&ManuallyDrop::take(&mut it.buf)) };
            }
        }

        // must delegate to spec_extend() since extend() itself delegate to spec_from for emtpy dynarrs
        dynarr.spec_extend(iter);
        dynarr
    }
}

//--------------------------------------------------------------

impl<T: Clone, B: DynArrayBuffer<T>> SpecExtendFromWithin for DynArray<T, B> {
    default unsafe fn spec_extend_from_within(&mut self, src: Range<usize>) {
        // SAFETY: len is increased only after initializing elements
        let (this, spare, len) = unsafe { self.split_at_spare_mut_with_len() };

        // SAFETY: caller quarantees that src is a valid index
        let to_clone = unsafe { this.get_unchecked(src) };

        iter::zip(to_clone, spare)
            .map(|(src, dst)| dst.write(src.clone()))
            // Note:
            // - Element was just initialized with `MaybeUninit::write`, so it's ok to inearese len
            .for_each(|_| *len += 1);
            
    }
}

impl<T: Copy, B: DynArrayBuffer<T>> SpecExtendFromWithin for DynArray<T, B> {
    unsafe fn spec_extend_from_within(&mut self, src: Range<usize>) {
        let count = self.len;
        {
            let (init, spare) = self.split_at_spare_mut();

            // SAFETY: caller guarantees that `src` is a valid index
            let source = unsafe { init.get_unchecked(src) };

            // SAFETY:
            // - Both pointer are created from unique slice references (`&mut [_]`), so they are valid and do not overlap
            // - Elements are :Copy, so it's OK to copy them, without doing anything with the original values
            // - `count` is equal to the len of `sourece`, so sourve is valid for `count` reads
            // - `.reserve(count)` guarantees tha `spare.len() >= count`, so spare is valid for `count` writes
            unsafe { ptr::copy_nonoverlapping(source.as_ptr(), spare.as_mut_ptr() as _, count) };
        }

        // SAFETY: The elements were just initialize by `copy_nonoverlapping`
        self.len += count;
    }

}

//--------------------------------------------------------------

impl<T: Clone, B: DynArrayBuffer<T>> SpecCloneFrom for DynArray<T, B> {
    default fn clone_from(this: &mut Self, other: &Self) {
        // drop anything that will not be overwritten
        this.truncate(other.len);

        // self.len <= other.len dueue to the trucate above, so the slices her are alwyas in-bound
        let (init, tail) = other.split_at(this.len());

        this.clone_from_slice(init);
        this.extend_from_slice(tail);
    }
}

impl<T: Copy, B: DynArrayBuffer<T>> SpecCloneFrom for DynArray<T, B> {
    fn clone_from(this: &mut Self, other: &Self) {
        this.clear();
        this.extend_from_slice(other);
    }
}

//------------------------------------------------------------------------------------------------------------------------------

impl<T, B: DynArrayBuffer<T>> Deref for DynArray<T, B> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.as_ptr(), self.len()) }
    }
}

impl<T, B: DynArrayBuffer<T>> DerefMut for DynArray<T, B> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len()) }
    }
}

impl<T: Clone, B: DynArrayBuffer<T>> Clone for DynArray<T, B> {
    #[inline]
    fn clone(&self) -> Self {
        let _scope_alloc = ScopedAlloc::new(UseAlloc::Id(self.buf.allocator_id()));

        <[T]>::to_imp_dynarray(&**self)
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        SpecCloneFrom::clone_from(self, source)
    }
}

impl<T: Hash, B: DynArrayBuffer<T>> Hash for DynArray<T, B> {
    /// The hash of a `StaticDynArray` is the same as that of the corerspoonding slice, as required by the `coree::borrow::Borrow` implementation
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&**self, state)
    }
}

impl<T, I: SliceIndex<[T]>, B: DynArrayBuffer<T>> Index<I> for DynArray<T, B> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        Index::index(&**self, index)
    }
}

impl<T, I: SliceIndex<[T]>, B: DynArrayBuffer<T>> IndexMut<I> for DynArray<T, B> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(&mut **self, index)
    }
}

impl<T, B: DynArrayBuffer<T>> FromIterator<T> for DynArray<T, B> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        <Self as SpecFromIter<T, I::IntoIter>>::from_iter(iter.into_iter())
    }
}

impl<T, B: DynArrayBuffer<T>> IntoIterator for DynArray<T, B> {
    type Item = T;
    type IntoIter = IntoIter<T, B>;

    fn into_iter(self) -> Self::IntoIter {
        unsafe {
            let mut me = ManuallyDrop::new(self);
            let begin = me.as_mut_ptr();
            let end = if mem::size_of::<T>() == 0 {
                (begin as *const i8).add(me.len()) as *const T
            } else {
                begin.add(me.len())
            };

            IntoIter {
                buf: me,
                ptr: begin,
                end,
            }
        }
    }
}

impl<'a, T, B: DynArrayBuffer<T>> IntoIterator for &'a DynArray<T, B> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T, B: DynArrayBuffer<T>> IntoIterator for &'a mut DynArray<T, B> {
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T, B: DynArrayBuffer<T>> Extend<T> for DynArray<T, B> {
    #[inline]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        <Self as SpecExtend<T, I::IntoIter>>::spec_extend(self, iter.into_iter())
    }

    //#[inline]
    //fn extend_one(&mut self, item: T) {
    //    self.push(item);
    //}
}

impl<'a, T: Copy + 'a, B: DynArrayBuffer<T>> Extend<&'a T> for DynArray<T, B> {
    #[inline]
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.spec_extend(iter.into_iter())
    }
    
    //#[inline]
    //fn extend_one(&mut self, item: &'a T) {
    //    self.push(item);
    //}
}

impl<T, B: DynArrayBuffer<T>> Drop for DynArray<T, B> {
    fn drop(&mut self) {
        unsafe {
            // use drop for [T]
            ptr::drop_in_place(ptr::slice_from_raw_parts_mut(self.as_mut_ptr(), self.len))
        }
        // Buffer should handle cleaning up the memory
    }
}

impl<T, B: DynArrayBuffer<T>> Default for DynArray<T, B> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: fmt::Debug, B: DynArrayBuffer<T>> fmt::Debug for DynArray<T, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T, B: DynArrayBuffer<T>> AsRef<DynArray<T, B>> for DynArray<T, B> {
    fn as_ref(&self) -> &DynArray<T, B> {
        self
    }
}

impl<T, B: DynArrayBuffer<T>> AsMut<DynArray<T, B>> for DynArray<T, B> {
    fn as_mut(&mut self) -> &mut DynArray<T, B> {
        self
    }
}

impl<T, B: DynArrayBuffer<T>> AsRef<[T]> for DynArray<T, B> {
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<T, B: DynArrayBuffer<T>> AsMut<[T]> for DynArray<T, B> {
    fn as_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<T: Clone, B: DynArrayBuffer<T>> From<&[T]> for DynArray<T, B> {
    fn from(s: &[T]) -> Self {
        s.to_imp_dynarray::<B>()
    }
}

impl<T: Clone, B: DynArrayBuffer<T>> From<&mut [T]> for DynArray<T, B> {
    fn from(s: & mut[T]) -> Self {
        s.to_imp_dynarray::<B>()
    }
}

impl<T, B: DynArrayBuffer<T>, const N: usize> From<[T; N]> for DynArray<T, B> {
    /// Truncates the given array if it's larger than the StaticDynArray
    fn from(arr: [T; N]) -> Self {
        let mut dynarr = Self::new();
        dynarr.extend(arr);
        dynarr
    }
}

impl<B: DynArrayBuffer<u8>> From<&str> for DynArray<u8, B> {
    fn from(s: &str) -> Self {
        Self::from(s.as_bytes())
    }
}

impl<T, B: DynArrayBuffer<T>, const N: usize> TryFrom<DynArray<T, B>> for [T; N] {
    type Error = DynArray<T, B>;

    fn try_from(mut dynarr: DynArray<T, B>) -> Result<Self, Self::Error> {
        if dynarr.len() != N {
            return Err(dynarr);
        }

        // SAFETY: `.set_len(0)` is always sound
        unsafe { dynarr.set_len(0) };

        let array = unsafe { ptr::read(dynarr.as_ptr() as *const [T; N]) };
        Ok(array)
    }
}

//------------------------------------------------------------------------------------------------------------------------------

impl_slice_partial_eq_generic!{ [B: DynArrayBuffer<T>, C: DynArrayBuffer<U>] DynArray<T, B>, DynArray<U, C> }
impl_slice_partial_eq_generic!{ [B: DynArrayBuffer<T>] DynArray<T, B>, [U] }
impl_slice_partial_eq_generic!{ [B: DynArrayBuffer<T>] DynArray<T, B>, &[U] }
impl_slice_partial_eq_generic!{ [B: DynArrayBuffer<T>] DynArray<T, B>, &mut [U] }
impl_slice_partial_eq_generic!{ [B: DynArrayBuffer<T>, const N: usize] DynArray<T, B>, [U; N] }
impl_slice_partial_eq_generic!{ [B: DynArrayBuffer<T>, const N: usize] DynArray<T, B>, &[U; N] }
impl_slice_partial_eq_generic!{ [B: DynArrayBuffer<T>, const N: usize] DynArray<T, B>, &mut [U; N] }
impl_slice_partial_eq_generic!{ [B: DynArrayBuffer<U>] [T], DynArray<U, B> }
impl_slice_partial_eq_generic!{ [B: DynArrayBuffer<U>] &[T], DynArray<U, B> }
impl_slice_partial_eq_generic!{ [B: DynArrayBuffer<U>] &mut [T], DynArray<U, B> }
impl_slice_partial_eq_generic!{ [B: DynArrayBuffer<U>, const N: usize] [T; N], DynArray<U, B> }
impl_slice_partial_eq_generic!{ [B: DynArrayBuffer<U>, const N: usize] &[T; N], DynArray<U, B> }
impl_slice_partial_eq_generic!{ [B: DynArrayBuffer<U>, const N: usize] &mut [T; N], DynArray<U, B> }

impl<T: Eq, B: DynArrayBuffer<T>> Eq for DynArray<T, B> {}

impl<T: PartialOrd, B: DynArrayBuffer<T>> PartialOrd for DynArray<T, B> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }
}

impl<T: Ord, B: DynArrayBuffer<T>> Ord for DynArray<T, B> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        Ord::cmp(&**self, &**other)
    }
}

//------------------------------------------------------------------------------------------------------------------------------

pub struct IntoIter<T, B: DynArrayBuffer<T>> {
    buf     : ManuallyDrop<DynArray<T, B>>,
    ptr     : *const T,
    end     : *const T,
}

impl<T, B: DynArrayBuffer<T>> IntoIter<T, B> {
    pub fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.ptr, self.len()) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { &mut *self.as_raw_mut_slice() }
    }

    pub fn allocator_id(&self) -> u16 {
        self.buf.allocator_id()
    }

    fn as_raw_mut_slice(&mut self) -> *mut [T] {
        ptr::slice_from_raw_parts_mut(self.ptr as *mut T, self.len())
    }

    fn forget_allocataion_drop_remaining(&mut self) {
        let remaining = self.as_raw_mut_slice();
        
        let _scope_alloc = ScopedAlloc::new(UseAlloc::Id(self.allocator_id()));


        self.buf = ManuallyDrop::new(DynArray::new());
        self.ptr = self.buf.as_ptr();
        self.end = self.buf.as_ptr();

        unsafe { ptr::drop_in_place(remaining) }
    }

    fn forget_or_drop_remaining_elements(&mut self, drop_start: usize) {
        if self.buf.len > drop_start {
            unsafe {
                let slice = slice::from_raw_parts_mut(self.buf.as_mut_ptr().add(drop_start), self.buf.len - drop_start);
                ptr::drop_in_place(slice);
            }
        }
        self.ptr = self.end;
    }
}

impl<T: fmt::Debug, B: DynArrayBuffer<T>> fmt::Debug for IntoIter<T, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("IntoIter").field(&self.as_slice()).finish()
    }
}

impl<T, B: DynArrayBuffer<T>> Iterator for IntoIter<T, B> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr == self.end {
            None
        } else if mem::size_of::<T>() == 0 {
            // purposefully don't use `ptr.offset` because for dynarrs with 0-size elements this would return the same pointer
            self.ptr = unsafe { (self.ptr as *const u8).add(1) as *mut T };

            // Make up a value fo this ZST
            Some(unsafe { mem::zeroed() })
        } else {
            let old = self.ptr;
            self.ptr = unsafe { self.ptr.add(1) };

            Some(unsafe { ptr::read(old) })
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let exact = if mem::size_of::<T>() == 0 {
            self.end.addr().wrapping_sub(self.ptr.addr())
        } else {
            unsafe { self.end.sub_ptr(self.ptr) }
        };
        (exact, Some(exact))
    }

    #[inline]
    fn count(self) -> usize {
        self.len()
    }
}

impl<T, B: DynArrayBuffer<T>> DoubleEndedIterator for IntoIter<T, B> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.end == self.ptr {
            None
        } else if mem::size_of::<T>() == 0 {
            self.end = (self.end as *const i8).wrapping_sub(1) as *mut T;
            // Make up a value of this ZST
            Some(unsafe { mem::zeroed() })
        } else {
            self.end = unsafe { self.end.sub(1) };

            Some(unsafe { ptr::read(self.end) })
        }
    }
}

impl<T, B: DynArrayBuffer<T>> ExactSizeIterator for IntoIter<T, B> {}
impl<T, B: DynArrayBuffer<T>> FusedIterator for IntoIter<T, B> {}


impl<T: Clone, B: DynArrayBuffer<T>> Clone for IntoIter<T, B> {
    fn clone(&self) -> Self {
        let _scope_alloc = ScopedAlloc::new(UseAlloc::Id(self.buf.allocator_id()));

        self.as_slice().to_imp_dynarray::<B>().into_iter()
    }
}

impl<T, B: DynArrayBuffer<T>> Drop for IntoIter<T, B> {
    fn drop(&mut self) {
        struct DropGuard<'a, T, B: DynArrayBuffer<T>>(&'a mut IntoIter<T, B>);

        impl<T, B: DynArrayBuffer<T>> Drop for DropGuard<'_, T, B> {
            fn drop(&mut self) {
                unsafe {
                    // Dyn array buffer handles deallocation
                    let _ = mem::replace(&mut self.0.buf.buf, B::new());
                }
            }
        }

        let guard = DropGuard(self);
        // Destroy the remainin elements
        unsafe { 
            ptr::drop_in_place(guard.0.as_raw_mut_slice());
        }
        // now `guard` will be dropped and do the rest
    }
}

unsafe impl<T: Send, B: DynArrayBuffer<T>> Send for IntoIter<T, B> {}
unsafe impl<T: Sync, B: DynArrayBuffer<T>> Sync for IntoIter<T, B> {}

//------------------------------------------------------------------------------------------------------------------------------

pub struct Drain<'a, T: 'a, B: DynArrayBuffer<T>> {
    tail_start : usize,
    tail_len   : usize,
    iter       : slice::Iter<'a, T>,
    dynarr     : NonNull<DynArray<T, B>>
}

impl<'a, T: 'a, B: DynArrayBuffer<T>> Drain<'a, T, B> {
    #[must_use]
    pub fn as_slice(&self) -> &[T] {
        self.iter.as_slice()
    }

    unsafe fn fill<I: Iterator<Item = T>>(&mut self, replace_with: &mut I) -> bool {
        let dynarr = unsafe { self.dynarr.as_mut() };
        let range_start = dynarr.len;
        let range_end = self.tail_start;
        let range_len = range_end - range_start;

        // No more space, so early exit, this will only happen when the buffer is not resizable
        if range_len == 0 {
            return true;
        }

        let range_slice = unsafe {
            slice::from_raw_parts_mut(dynarr.as_mut_ptr().add(range_start), range_len)
        };

        for place in range_slice {
            if let Some(new_item) = replace_with.next() {
                unsafe { ptr::write(place, new_item) };
                dynarr.len += 1;
            } else {
                return false;
            }
        }
        true
    }

    unsafe fn move_tail(&mut self, additional: usize) {
        let dynarr = unsafe { self.dynarr.as_mut() };
        let len = self.tail_start + self.tail_len;
        let cap = dynarr.buf.reserve(dynarr.len, additional);

        // Limit additional elements, as we can't grow in size, in case the buffer is not resizable
        let additional = core::cmp::min(additional, cap - dynarr.len);
        
        let new_tail_start = self.tail_start + additional;
        unsafe {
            let src = dynarr.as_ptr().add(self.tail_start);
            let dst = dynarr.as_mut_ptr().add(new_tail_start);
            ptr::copy(src, dst, self.tail_len);
        }
        self.tail_start = new_tail_start;
    }
}

impl<T: fmt::Debug, B: DynArrayBuffer<T>> fmt::Debug for Drain<'_, T, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Drain")
            .field(&self.iter.as_slice())
        .finish()
    }
}

impl<'a, T: 'a, B: DynArrayBuffer<T>> AsRef<[T]> for Drain<'a, T, B> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, B: DynArrayBuffer<T>> Iterator for Drain<'_, T, B> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|elt| unsafe { ptr::read(elt as *const _) })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T, B: DynArrayBuffer<T>> DoubleEndedIterator for Drain<'_, T, B> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|elt| unsafe { ptr::read(elt as *const _) })
    }
}

impl<T, B: DynArrayBuffer<T>> Drop for Drain<'_, T, B> {
    fn drop(&mut self) {
        /// Moves back the un-`Drain`ed elements to restore the original `StaticDynArray`
        struct DropGuard<'r, 'a, T, B: DynArrayBuffer<T>>(&'r mut Drain<'a, T, B>);

        impl<'r, 'a, T, B: DynArrayBuffer<T>> Drop for DropGuard<'r, 'a, T, B> {
            fn drop(&mut self) {
                if self.0.tail_len > 0 {
                    unsafe {
                        let source_dynarr = self.0.dynarr.as_mut();
                        // memmove back untouched tail, update to new length
                        let start = source_dynarr.len();
                        let tail = self.0.tail_start;
                        if tail != start {
                            let src = source_dynarr.as_ptr().add(tail);
                            let dst = source_dynarr.as_mut_ptr().add(start);
                            ptr::copy(src, dst, self.0.tail_len);
                        }
                        source_dynarr.set_len(start + self.0.tail_len);
                    }
                }
            }
        }

        let iter = mem::replace(&mut self.iter, (&mut []).iter());
        let drop_len = iter.len();

        let mut dynarr = self.dynarr;

        if mem::size_of::<T>() == 0 {
            // ZSTs have no identity, so we don't need to move them around , we only need to drop the current amount.
            // This can be achievend by manipulating the StaticDynArray length instead of moving values out of `iter`.
            unsafe {
                let dynarr = dynarr.as_mut();
                let old_len = dynarr.len();
                dynarr.set_len(old_len + drop_len + self.tail_len);
                dynarr.truncate(old_len + self.tail_len);
            }

            return;
        }

        // ensuer elements are moved back into their appropriate places, even when drop_in_place panics
        let _guard = DropGuard(self);

        if drop_len == 0 {
            return;
        }

        let drop_ptr = iter.as_slice().as_ptr();

        unsafe {
            // drop_ptr comes from a slice::Iter, which only gives us a &[T], but for drop_in)lace, a pointer with mutable provenance is necessary.
            // Therefroe we must reconstruct it from the original dynarr, but also avoid createing a &mut to the front, since that could invalidate raw pointers to it which some unsafe code might rely on.
            let dynarr_ptr = dynarr.as_mut().as_mut_ptr();
            let drop_offset = drop_ptr.sub_ptr(dynarr_ptr);
            let to_drop = ptr::slice_from_raw_parts_mut(dynarr_ptr.add(drop_offset), drop_len);
            ptr::drop_in_place(to_drop);
        }
    }
}

impl<T, B: DynArrayBuffer<T>> ExactSizeIterator for Drain<'_, T, B> {
    //#[inline]
    //fn is_empty(&self) -> bool {
    //    self.iter.is_empty()
    //}
}

impl<T, B: DynArrayBuffer<T>> FusedIterator for Drain<'_, T, B> {}

unsafe impl<T: Sync, B: DynArrayBuffer<T>> Sync for Drain<'_, T, B> {}
unsafe impl<T: Send, B: DynArrayBuffer<T>> Send for Drain<'_, T, B> {}

//------------------------------------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub struct Splice<'a, I: Iterator + 'a, B: DynArrayBuffer<I::Item>> {
    drain        : Drain<'a, I::Item, B>,
    replace_with : I
}

impl<I: Iterator, B: DynArrayBuffer<I::Item>> Iterator for Splice<'_, I, B> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.drain.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.drain.size_hint()
    }
}

impl<I: Iterator, B: DynArrayBuffer<I::Item>> DoubleEndedIterator for Splice<'_, I, B> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.drain.next_back()
    }
}

impl<I: Iterator, B: DynArrayBuffer<I::Item>> ExactSizeIterator for Splice<'_, I, B> {}

impl<I: Iterator, B: DynArrayBuffer<I::Item>> Drop for Splice<'_, I, B> {
    fn drop(&mut self) {
        self.drain.by_ref().for_each(drop);

        unsafe {
            if self.drain.tail_len == 0 {
                self.drain.dynarr.as_mut().extend(self.replace_with.by_ref());
                return;
            }

            // First fill the range left by drain()
            if !self.drain.fill(&mut self.replace_with) {
                return;
            }

            // There may be more elelemts. Use the lowe bound as an estimate
            let (lower_bound, _upper_bound) = self.replace_with.size_hint();
            if lower_bound > 0 {
                self.drain.move_tail(lower_bound);
                if !self.drain.fill(&mut self.replace_with) {
                    return;
                }
            }

            // Collect any remaining elements.
            // This is a zero-length vector which does not allocate if `lower_bound` was exact
            let mut collected = self.replace_with.by_ref().collect::<DynArray<I::Item, B>>().into_iter();
            // Now we have an "exact" amount
            if collected.len() > 0 {
                self.drain.move_tail(collected.len());
                let filled = self.drain.fill(&mut collected);
                debug_assert!(filled);
                debug_assert_eq!(collected.len(), 0);
            }
        }
        // Let `Drain::drop` move the tail back if necessary and restore `vec.len`
    }
}

//------------------------------------------------------------------------------------------------------------------------------

pub trait SliceToImpDynArray<T: Clone> {
    fn to_imp_dynarray<B: DynArrayBuffer<T>>(&self) -> DynArray<T, B>;
}

impl<T: Clone> SliceToImpDynArray<T> for [T] {
    default fn to_imp_dynarray<B: DynArrayBuffer<T>>(&self) -> DynArray<T, B> {
        struct DropGuard<'a, T, B: DynArrayBuffer<T>> {
            dynarr: &'a mut DynArray<T, B>,
            num_init: usize
        }
        impl<'a, T, B: DynArrayBuffer<T>> Drop for DropGuard<'a, T, B> {
            #[inline]
            fn drop(&mut self) {
                // SAFETY: items were marked initialized in the loop below
                unsafe {
                    self.dynarr.set_len(self.num_init);
                }
            }
        }

        let mut dynarr = DynArray::with_capacity(self.len());
        let mut guard = DropGuard{ dynarr: &mut dynarr, num_init: 0 };
        let slots = guard.dynarr.spare_capacity_mut();
        // .take(slots.len()) is necessary for LLVM to remove bounds checks and has better code gen than zip (mentioned by rust's implementation)
        // slots.len() can also not be greater than the capacity, even with static buffers
        for (i, b) in self.iter().enumerate().take(slots.len()) {
            guard.num_init = i;
            slots[i].write(b.clone());
        }
        core::mem::forget(guard);
        // SAFETY: The dynarr was allocated and initialized above to at least this length
        unsafe {
            dynarr.set_len(self.len());
        }
        dynarr
    }
}

impl<T: Copy> SliceToImpDynArray<T> for [T] {
    #[inline]
    fn to_imp_dynarray<B: DynArrayBuffer<T>>(&self) -> DynArray<T, B> {
        let mut dynarr = DynArray::with_capacity(self.len());
        let n = cmp::min(self.len(), dynarr.capacity());
        // SAfETY: allocated above with the capacity of`n`, and initialized to `n` is ptr:copy_to_non_overlapping below
        unsafe {
            ptr::copy_nonoverlapping(self.as_ptr(), dynarr.as_mut_ptr(), n);
            dynarr.set_len(n);
        }
        dynarr
    }
}