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
};
use crate::{alloc::{UseAlloc, Allocation, Layout, self}, KiB, mem::{MEMORY_MANAGER, HeapPtr}};

use super::{ExtendFunc, ExtendElement, impl_slice_partial_eq, imp::dyn_array::SliceToImpDynArray};
use super::imp::dyn_array as imp;
use imp::DynArrayBuffer;

extern crate alloc as rs_alloc;

// TODO: move into allocators
enum AllocInit {
    /// The contents of the new memeory are uninitialized
    Uninitialized,
    /// The new memory is guaranteed to be zeroed
    Zeroed,
}

// Even if we wanted to, we can't exactly wrap alloc::vec::RawVec as it isn't public
pub(super) struct DynamicBuffer<T> {
    ptr : Allocation<T>,
    cap : usize
}

impl<T> DynamicBuffer<T> {
    // Tiny Buffers are dumb, skip to:
    // - 8 if the element size is 1, because any heap allocator is likely to round up a request of less than 8 bytes to at least 8 bytes.
    // - 4 if elements are moderate-size (<= `KiB).
    // - 1 otherwise, to acoid wastin too much space for very short dynarrs
    const MIN_NON_ZERO_CAP : usize = if mem::size_of::<T>() == 1 {
        8
    } else if mem::size_of::<T>() <= KiB(1) {
        4
    } else {
        1
    };

    fn allocate(capacity: usize, init: AllocInit, alloc: UseAlloc) -> Self {
        if mem::size_of::<T>() == 0 || capacity == 0 {
            Self::new(alloc)
        } else {
            let layout = Layout::array::<T>(capacity);
            let res = match init {
                AllocInit::Uninitialized => MEMORY_MANAGER.alloc_raw(alloc, layout),
                AllocInit::Zeroed => MEMORY_MANAGER.alloc_raw_zeroed(alloc, layout),
            };
            let ptr = match res {
                Some(ptr) => ptr,
                None => panic!("Failed to allocate memory")
            }.cast();

            Self { ptr, cap:capacity }
        }
    }

    fn needs_to_grow(&self, len: usize, additional: usize) -> bool {
        additional > self.cap.wrapping_sub(len)
    }

    fn grow_amortized(&mut self, len: usize, additional: usize) -> Result<usize, std::collections::TryReserveError> {
        // This is ensured by the calling contexts.
        debug_assert!(additional > 0);

        if mem::size_of::<T>() == 0 {
            // Since we return a capacity of `usize::MAX` when `elem_size` is 0, getting to here necessarily means that `DynamicBuffer` is overfull
            return Err(std::collections::TryReserveErrorKind::CapacityOverflow.into());
        }

        // Nothing we can really do about these checks, sadly
        let required_cap = len.checked_add(additional).ok_or(std::collections::TryReserveErrorKind::CapacityOverflow)?;

        // This guarantees exponential growth. The doubling cannot overflow because `cap <= isize::MAX` and the type of `cap` is usize.
        // While rust increases the capacity by 2x, we will increase it by 1.5x, so we don't get to a run-away capacity as fast
        // PERF(jel): What impact does 1.5x have compared to 2x?
        let cap = cmp::max(self.cap + self.cap / 2, required_cap);
        let cap = cmp::max(Self::MIN_NON_ZERO_CAP, cap);

        let new_layout = Layout::array::<T>(cap);

        self.finish_grow(new_layout, cap)
    }

    fn grow_exact(&mut self, len: usize, additional: usize) -> Result<usize, std::collections::TryReserveError> {
        if mem::size_of::<T>() == 0 {
            // Since we return a capacity of `usize::MAX` when `elem_size` is 0, getting to here necessarily means that `DynamicBuffer` is overfull
            return Err(std::collections::TryReserveErrorKind::CapacityOverflow.into());
        }

        let cap = len.checked_add(additional).ok_or(std::collections::TryReserveErrorKind::CapacityOverflow)?;

        let new_layout = Layout::array::<T>(cap);


        self.finish_grow(new_layout, cap)
    }

    pub fn finish_grow(&mut self, new_layout: Layout, new_cap: usize) -> Result<usize, std::collections::TryReserveError> {
        if self.cap == 0 {
            let res = MEMORY_MANAGER.alloc_raw(UseAlloc::Id(self.ptr.layout().alloc_id()), new_layout);
            self.ptr = match res {
                Some(ptr) => ptr.cast(),
                None => {
                    let rs_layout = unsafe { std::alloc::Layout::from_size_align_unchecked(new_layout.size(), new_layout.align()) };
                    let err_kind = std::collections::TryReserveErrorKind::AllocError { layout: rs_layout, non_exhaustive: () };
                    return Err(err_kind.into());
                }
            };
        } else {
            self.ptr = match MEMORY_MANAGER.grow(mem::replace(&mut self.ptr, unsafe { Allocation::null() }), new_layout) {
                Ok(ptr) => ptr,
                Err(_) => {
                    let rs_layout = unsafe { std::alloc::Layout::from_size_align_unchecked(new_layout.size(), new_layout.align()) };
                    let err_kind = std::collections::TryReserveErrorKind::AllocError { layout: rs_layout, non_exhaustive: () };
                    return Err(err_kind.into());
                }
            };
        }
        self.cap = new_cap;
        Ok(new_cap)
    }

}

impl<T> imp::DynArrayBuffer<T> for DynamicBuffer<T> {
    fn new(alloc: UseAlloc) -> Self {
        Self { ptr: unsafe { Allocation::null_alloc(alloc) }, cap: 0 }
    }

    fn with_capacity(capacity: usize, alloc: UseAlloc) -> Self {
        Self::allocate(capacity, AllocInit::Uninitialized, alloc)
    }

    fn with_capacity_zeroed(capacity: usize, alloc: UseAlloc) -> Self {
        Self::allocate(capacity, AllocInit::Zeroed, alloc)
    }

    fn reserve(&mut self, len: usize, additional: usize) -> usize {
        if self.needs_to_grow(len, additional) {
            self.grow_amortized(len, additional).expect("Failed to allocate memory");
        }
        self.cap
    }

    fn try_reserve(&mut self, len: usize, additional: usize) -> Result<usize, std::collections::TryReserveError> {
        if self.needs_to_grow(len, additional) {
            self.grow_amortized(len, additional)
        } else {
            Ok(self.cap)
        }       
    }

    fn reserve_exact(&mut self, len: usize, additional: usize) -> usize {
        self.try_reserve_exact(len, additional).expect("Failed to allocate memory")
    }

    fn try_reserve_exact(&mut self, len: usize, additional: usize) -> Result<usize, std::collections::TryReserveError> {
        if self.needs_to_grow(len, additional) {
            self.grow_exact(len, additional)
        } else {
            Ok(self.cap)
        }
    }

    fn shrink_to_fit(&mut self, cap: usize) {
        assert!(cap < self.cap, "Tried to shrink to a larger capacity");

        if self.cap == 0 {
            return;
        }

        let new_layout = Layout::array::<T>(cap);
        self.ptr = match MEMORY_MANAGER.shrink(mem::replace(&mut self.ptr, unsafe { Allocation::null() }), new_layout) {
            Ok(ptr) => ptr,
            Err(_) => {
                //let rs_layout = unsafe { std::alloc::Layout::from_size_align_unchecked(new_layout.size(), new_layout.align()) };
                //let err_kind = std::collections::TryReserveErrorKind::AllocError { layout: rs_layout, non_exhaustive: () };
                //return Err(err_kind.into());
                panic!("Could not shrink buffer")
            }
        };
        self.cap = cap;
    }

    fn capacity(&self) -> usize {
        if mem::size_of::<T>() == 0 {
            usize::MAX
        } else {
            self.cap
        }
    }

    fn as_ptr(&self) -> *const T {
        self.ptr.ptr()
    }

    fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr.ptr_mut()
    }

    fn allocator_id(&self) -> u16 {
        self.ptr.layout().alloc_id()
    }
}

impl<T> Drop for DynamicBuffer<T> {
    fn drop(&mut self) {
        if self.cap > 0 {
            MEMORY_MANAGER.dealloc(mem::replace(&mut self.ptr, unsafe { Allocation::null() }))
        }
    }
}

//------------------------------------------------------------------------------------------------------------------------------

// A [`DynArray`] that exlusively stores its data on the stack, i.e. all elements are stored inline.
pub struct DynArray<T> (imp::DynArray<T, DynamicBuffer<T>>);

impl<T> DynArray<T> {
    #[inline]
    #[must_use]
    pub fn new(alloc: UseAlloc) -> Self {
        Self(imp::DynArray::new(alloc))
    }

    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize, alloc: UseAlloc) -> Self {
        Self(imp::DynArray::with_capacity(capacity, alloc))
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    #[inline]
    pub fn reserve(&mut self, additional:usize) {
        self.0.reserve(additional);
    }

    #[inline]
    pub fn try_reserve(&mut self, additional:usize) -> Result<(), std::collections::TryReserveError> {
        self.0.try_reserve(additional).map(|_| ())
    }

    #[inline]
    pub fn reserve_exact(&mut self, additional:usize) {
        self.0.reserve_exact(additional);
    }

    #[inline]
    pub fn try_reserve_exact(&mut self, additional:usize) -> Result<(), std::collections::TryReserveError> {
        self.0.try_reserve_exact(additional).map(|_| ())
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit()
    }

    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.0.shrink_to(min_capacity)
    }

    #[inline]
    pub fn truncate(&mut self, len: usize) {
        self.0.truncate(len)
    }

    #[inline]
    pub fn as_slice(&self) -> &[T] {
        self
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self
    }

    #[inline]
    pub fn as_ptr(&self) -> *const T {
        self.0.as_ptr()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.0.as_mut_ptr()
    }

    #[inline]
    pub unsafe fn set_len(&mut self, new_len: usize) {
        self.0.set_len(new_len)
    }

    #[inline]
    pub fn swap_remove(&mut self, index: usize) -> T {
        self.0.swap_remove(index)
    }

    #[inline]
    pub fn insert(&mut self, index: usize, element: T) {
        self.0.insert(index, element)   
    }

    #[inline]
    pub fn remove(&mut self, index: usize) -> T {
        self.0.remove(index)
    }

    #[inline]
    pub fn retain<F>(&mut self, pred: F) 
    where
        F : FnMut(&T) -> bool
    {
        self.0.retain(pred)
    }

    #[inline]
    pub fn retain_mut<F>(&mut self, pred: F)
    where
        F : FnMut(&mut T) -> bool
    {
        self.0.retain_mut(pred)
    }

    #[inline]
    pub fn dedup_by_key<F, K>(&mut self, mut key: F)
    where
        F : FnMut(&mut T) -> K,
        K : PartialEq<K>
    {
        self.0.dedup_by_key(key)
    }

    #[inline]
    pub fn dedup_by<F>(&mut self, same_bucket: F)
    where
        F : FnMut(&mut T, &mut T) -> bool
    {
        self.0.dedup_by(same_bucket)
    }

    #[inline]
    pub fn push(&mut self, value: T) {
        self.0.push(value)
    }

    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        self.0.pop()
    }

    #[inline]
    pub fn append(&mut self, other: &mut DynArray<T>) {
        self.0.append(&mut other.0)
    }

    #[inline]
    pub fn drain<R: RangeBounds<usize>>(&mut self, range: R) -> Drain<'_, T> {
        Drain(self.0.drain(range))
    }

    #[inline]
    pub fn clear(&mut self) {
        self.0.clear()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn split_off(&mut self, at: usize) -> Self {
        Self(self.0.split_off(at))
    }

    #[inline]
    pub fn resize_with<F>(&mut self, new_len: usize, f: F)
    where
        F : FnMut() -> T
    {
        self.0.resize_with(new_len, f)
    }

    #[inline]
    pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<T>] {
        self.0.spare_capacity_mut()
    }

    #[inline]
    pub fn split_at_spare_mut(&mut self) -> (&mut [T], &mut [MaybeUninit<T>]) {
        self.0.split_at_spare_mut()
    }

    #[inline]
    pub fn splice<R, I>(&mut self, range: R, replace_with: I) -> Splice<'_, I::IntoIter>
    where
        R : RangeBounds<usize>,
        I : IntoIterator<Item = T>
    {
        Splice(self.0.splice(range, replace_with))
    }

    #[inline]
    pub fn allocator_id(&self) -> u16 {
        self.0.allocator_id()
    }

    #[inline]
    pub fn into_heap_slice(self) -> HeapPtr<[T]> {
        unsafe {
            let mut me = ManuallyDrop::new(self);
            let slice_len = me.capacity();
            let alloc = &mut me.0.buf.ptr;
            let ptr = slice::from_raw_parts_mut(alloc.ptr_mut(), slice_len);
            HeapPtr::from_raw_components(NonNull::new_unchecked(ptr), *alloc.layout())
        }
    }
}

impl<T: Clone> DynArray<T> {
    #[inline]
    pub fn resize(&mut self, new_len: usize, value: T) {
        self.0.resize(new_len, value)
    }

    #[inline]
    pub fn extend_from_slice(&mut self, other: &[T]) {
        self.0.extend_from_slice(other)
    }

    #[inline]
    pub fn extend_from_within<R: RangeBounds<usize>>(&mut self, src: R) {
        self.0.extend_from_within(src)
    }
}

impl<T: PartialEq> DynArray<T> {
    #[inline]
    pub fn dedup(&mut self) {
        self.0.dedup()
    }
}

//------------------------------------------------------------------------------------------------------------------------------

impl<T> Deref for DynArray<T> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &*(self.0)
    }
}

impl<T> DerefMut for DynArray<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *(self.0)
    }
}

impl<T: Clone> Clone for DynArray<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.0.clone_from(&source.0)
    }
}

impl<T: Hash> Hash for DynArray<T> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<T: Hash, I: SliceIndex<[T]>> Index<I> for DynArray<T> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        self.0.index(index)
    }
}

impl<T: Hash, I: SliceIndex<[T]>> IndexMut<I> for DynArray<T> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

impl<T> FromIterator<T> for DynArray<T> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(FromIterator::from_iter(iter))
    }
}

impl<T> IntoIterator for DynArray<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.0.into_iter())
    }
}

impl<'a, T> IntoIterator for &'a mut DynArray<T> {
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl<T> Extend<T> for DynArray<T> {
    #[inline]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.0.extend(iter)
    }
}

impl<'a, T: Copy + 'a> Extend<&'a T> for DynArray<T> {
    #[inline]
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.0.extend(iter)
    }
}

impl<T> Default for DynArray<T> {
    #[inline]
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: fmt::Debug> fmt::Debug for DynArray<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl<T> AsRef<DynArray<T>> for DynArray<T> {
    #[inline]
    fn as_ref(&self) -> &DynArray<T> {
        self
    }
}

impl<T> AsMut<DynArray<T>> for DynArray<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut DynArray<T> {
       self 
    }
}

impl<T> AsRef<[T]> for DynArray<T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<T> AsMut<[T]> for DynArray<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
       self 
    }
}

impl<T: Clone> From<&[T]> for DynArray<T> {
    #[inline]
    fn from(s: &[T]) -> Self {
        Self(From::from(s))
    }
}

impl<T: Clone> From<&mut [T]> for DynArray<T> {
    #[inline]
    fn from(s: &mut [T]) -> Self {
        Self(From::from(s))
    }
}

impl<T, const N: usize> From<[T; N]> for DynArray<T> {
    #[inline]
    fn from(s: [T; N]) -> Self {
        Self(From::from(s))
    }
}

impl<> From<&str> for DynArray<u8> {
    #[inline]
    fn from(s: &str) -> Self {
        Self(From::from(s))
    }
}

impl<T, const N: usize> TryFrom<DynArray<T>> for [T; N] {
    type Error = DynArray<T>;

    #[inline]
    fn try_from(dynarr: DynArray<T>) -> Result<Self, Self::Error> {
        match <[T; N]>::try_from(dynarr.0) {
            Ok(arr) => Ok(arr),
            Err(dynarr) => Err(DynArray(dynarr))
        }
    }
}

//------------------------------------------------------------------------------------------------------------------------------


impl_slice_partial_eq!{ [] DynArray<T>, DynArray<U> }
impl_slice_partial_eq!{ [] DynArray<T>, [U] }
impl_slice_partial_eq!{ [] DynArray<T>, &[U] }
impl_slice_partial_eq!{ [] DynArray<T>, &mut [U] }
impl_slice_partial_eq!{ [const M: usize] DynArray<T>, [U; M] }
impl_slice_partial_eq!{ [const M: usize] DynArray<T>, &[U; M] }
impl_slice_partial_eq!{ [const M: usize] DynArray<T>, &mut [U; M] }
impl_slice_partial_eq!{ [] [T], DynArray<U> }
impl_slice_partial_eq!{ [] &[T], DynArray<U> }
impl_slice_partial_eq!{ [] &mut [T], DynArray<U> }
impl_slice_partial_eq!{ [const N: usize] [T; N], DynArray<U> }
impl_slice_partial_eq!{ [const N: usize] &[T; N], DynArray<U> }
impl_slice_partial_eq!{ [const N: usize] &mut [T; N], DynArray<U> }


impl<T: Eq> Eq for DynArray<T> {}

impl<T: PartialOrd> PartialOrd for DynArray<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T: Ord> Ord for DynArray<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

//------------------------------------------------------------------------------------------------------------------------------

pub struct IntoIter<T>(imp::IntoIter<T, DynamicBuffer<T>>);

impl<T> IntoIter<T> {
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        self.0.as_slice()
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.0.as_mut_slice()
    }
}

impl<T: fmt::Debug> fmt::Debug for IntoIter<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    #[inline]
    fn count(self) -> usize{
        self.0.count()
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {}
impl<T> FusedIterator for IntoIter<T> {}

impl<T: Clone> Clone for IntoIter<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

unsafe impl<T: Send> Send for IntoIter<T> {}
unsafe impl<T: Sync> Sync for IntoIter<T> {}

//------------------------------------------------------------------------------------------------------------------------------

pub struct Drain<'a, T>(imp::Drain<'a, T, DynamicBuffer<T>>);

impl<T> Drain<'_, T> {
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        self.0.as_slice()
    }
}

impl<T: fmt::Debug> fmt::Debug for Drain<'_, T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl<'a, T: 'a> AsRef<[T]> for Drain<'a, T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.0.as_ref()
    }
}

impl<T> Iterator for Drain<'_, T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<T> DoubleEndedIterator for Drain<'_, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl<T> ExactSizeIterator for Drain<'_, T> {}

unsafe impl<T: Send> Send for Drain<'_, T> {}
unsafe impl<T: Sync> Sync for Drain<'_, T> {}

//------------------------------------------------------------------------------------------------------------------------------

pub struct Splice<'a, I>(imp::Splice<'a, I, DynamicBuffer<I::Item>>)
where
    I : Iterator + 'a
;

impl<I: Iterator<Item = T>, T> Iterator for Splice<'_, I> {
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<I: Iterator<Item = T>, T> DoubleEndedIterator for Splice<'_, I> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl<I: Iterator<Item = T>, T> ExactSizeIterator for Splice<'_, I> {}

//------------------------------------------------------------------------------------------------------------------------------

pub trait SliceToDynArray<T: Clone> {
    fn to_static_dynarray(&self) -> DynArray<T>;
}

impl<T: Clone> SliceToDynArray<T> for [T] {
    default fn to_static_dynarray(&self) -> DynArray<T> {
        DynArray(self.to_imp_dynarray::<DynamicBuffer<T>>(UseAlloc::Default))
    }
}

impl<T: Copy> SliceToDynArray<T> for [T] {
    fn to_static_dynarray(&self) -> DynArray<T> {
        DynArray(self.to_imp_dynarray::<DynamicBuffer<T>>(UseAlloc::Default))
    }
}