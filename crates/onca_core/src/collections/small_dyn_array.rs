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
};
use crate::alloc::UseAlloc;

use super::{ExtendFunc, ExtendElement, impl_slice_partial_eq, imp::dyn_array::SliceToImpDynArray};
use super::imp::dyn_array as imp;

use imp::DynArrayBuffer;
use super::dyn_array::DynamicBuffer;

struct StaticBuf<T, const N: usize> {
    buf: MaybeUninit<[T; N]>
}

impl<T, const N: usize> imp::DynArrayBuffer<T> for StaticBuf<T, N> {
    fn new(alloc: UseAlloc) -> Self {
        Self { buf: MaybeUninit::uninit() }
    }

    fn with_capacity(capacity: usize, alloc: UseAlloc) -> Self {
        Self { buf: MaybeUninit::uninit() }
    }

    fn with_capacity_zeroed(capacity: usize, alloc: UseAlloc) -> Self {
        Self { buf: MaybeUninit::uninit() }
    }

    fn reserve(&mut self, len: usize, additional: usize) -> usize {
        N
    }

    fn try_reserve(&mut self, len: usize, additional: usize) -> Result<usize, std::collections::TryReserveError> {
        Ok(N)
    }

    fn reserve_exact(&mut self, len: usize, additional: usize) -> usize {
        N
    }

    fn try_reserve_exact(&mut self, len: usize, additional: usize) -> Result<usize, std::collections::TryReserveError> {
        Ok(N)
    }

    fn shrink_to_fit(&mut self, cap: usize) {
    }

    fn capacity(&self) -> usize {
        N
    }

    fn as_ptr(&self) -> *const T {
        self.buf.as_ptr() as *const T
    }

    fn as_mut_ptr(&mut self) -> *mut T {
        self.buf.as_mut_ptr() as *mut T
    }

    fn allocator_id(&self) -> u16 {
        u16::MAX
    }
}

union SmallBufferData<T, const N: usize> {
    inline  : (ManuallyDrop<MaybeUninit<[T; N]>>, u16),
    dynamic : ManuallyDrop<DynamicBuffer<T>>,
}

impl<T, const N: usize> SmallBufferData<T, N> {
    fn new_inline(alloc_id: u16) -> Self {
        Self { inline: (ManuallyDrop::new(MaybeUninit::uninit()), alloc_id) }
    }

    fn new_dynamic(alloc: UseAlloc) -> Self {
        Self { dynamic: ManuallyDrop::new(DynamicBuffer::new(alloc)) }
    }

    fn new_dynamic_with_capacity(capacity: usize, alloc: UseAlloc) -> Self {
        Self { dynamic: ManuallyDrop::new(DynamicBuffer::with_capacity(capacity, alloc)) }
    }

    fn new_dynamic_with_capacity_zeroed(capacity: usize, alloc: UseAlloc) -> Self {
        Self { dynamic: ManuallyDrop::new(DynamicBuffer::with_capacity_zeroed(capacity, alloc)) }
    }
}

struct SmallBuffer<T, const N: usize> {
    cap  : usize,
    data : SmallBufferData<T, N>
}

impl<T, const N: usize> imp::DynArrayBuffer<T> for SmallBuffer<T, N> {
    fn new(alloc: UseAlloc) -> Self {
        Self { cap: N, data: SmallBufferData::new_inline(alloc.get_id()) }
    }

    fn with_capacity(capacity: usize, alloc: UseAlloc) -> Self {
        if capacity <= N {
            Self::new(alloc)
        } else {
            Self { cap: capacity, data: SmallBufferData::new_dynamic_with_capacity(capacity, alloc) }
        }
    }

    fn with_capacity_zeroed(capacity: usize, alloc: UseAlloc) -> Self {
        if capacity <= N {
            let mut res = Self::new(alloc);
            unsafe { ptr::write_bytes(res.as_mut_ptr(), 0, N) };
            res
        } else {
            Self { cap: capacity, data: SmallBufferData::new_dynamic_with_capacity_zeroed(capacity, alloc) }
        }
    }

    fn reserve(&mut self, len: usize, additional: usize) -> usize {
        self.try_reserve(len, additional).expect("Failed to allocate memory")
    }

    fn try_reserve(&mut self, len: usize, additional: usize) -> Result<usize, std::collections::TryReserveError> {
        if len + additional > self.cap {
            self.cap = if self.cap == N {
                let alloc = UseAlloc::Id(unsafe { self.data.inline.1 });
                let mut data = SmallBufferData::new_dynamic(alloc);

                unsafe {
                    let cap = (*data.dynamic).try_reserve(len, additional)?;
                    ptr::copy_nonoverlapping(self.data.inline.0.as_ptr() as *const T, (*data.dynamic).as_mut_ptr(), self.cap);
                    self.data = data;
                    cap
                }
            } else {
                unsafe { (*self.data.dynamic).try_reserve(len, additional)? }
            };
        }
        Ok(self.cap)
    }

    fn reserve_exact(&mut self, len: usize, additional: usize) -> usize {
        self.try_reserve_exact(len, additional).expect("Failed to allocate memory")
    }

    fn try_reserve_exact(&mut self, len: usize, additional: usize) -> Result<usize, std::collections::TryReserveError> {
        if len + additional > self.cap {
            self.cap = if self.cap == N {
                let alloc = UseAlloc::Id(unsafe { self.data.inline.1 });
                let mut data = SmallBufferData::new_dynamic(alloc);

                unsafe {
                    let cap = (*data.dynamic).try_reserve_exact(len, additional)?;
                    ptr::copy_nonoverlapping(self.data.inline.0.as_ptr() as *const T, (*data.dynamic).as_mut_ptr(), self.cap);
                    self.data = data;
                    cap
                }
            } else {
                unsafe { (*self.data.dynamic).try_reserve_exact(len, additional)? }
            };
        }
        Ok(self.cap)
    }

    fn shrink_to_fit(&mut self, cap: usize) {
        if cap < self.cap {
            if cap <= N {
                unsafe {
                    let alloc_id = (*self.data.dynamic).allocator_id();
                    let mut data = SmallBufferData::new_inline(alloc_id);

                    let dynbuf = ManuallyDrop::take(&mut self.data.dynamic);
                    ptr::copy_nonoverlapping(dynbuf.as_ptr(), (*data.inline.0).as_ptr() as *mut T, self.cap);
                    
                    self.data = data;
                    self.cap = N;
                }
            } else {
                unsafe {
                    (*self.data.dynamic).shrink_to_fit(cap);
                    self.cap = (*self.data.dynamic).capacity();
                }
            }
        }
    }

    fn capacity(&self) -> usize {
        self.cap
    }

    fn as_ptr(&self) -> *const T {
        unsafe {
            if self.cap == N {
                (*self.data.inline.0).as_ptr() as *const T
            } else {
                (*self.data.dynamic).as_ptr()
            }
        }
    }

    fn as_mut_ptr(&mut self) -> *mut T {
        unsafe {
            if self.cap == N {
                (*self.data.inline.0).as_mut_ptr() as *mut T
            } else {
                (*self.data.dynamic).as_mut_ptr()
            }
        }
    }

    fn allocator_id(&self) -> u16 {
        unsafe {
            if self.cap == N {
                self.data.inline.1
            } else {
                (*self.data.dynamic).allocator_id()
            }
        }
    }
}

impl<T, const N: usize> Drop for SmallBuffer<T, N> {
    fn drop(&mut self) {
        unsafe {
            if self.cap > N {
                let _ = ManuallyDrop::take(&mut self.data.dynamic);
            }
        }
    }
}

//------------------------------------------------------------------------------------------------------------------------------

// A [`DynArray`] that exlusively stores its data on the stack, i.e. all elements are stored inline.
pub struct StaticDynArray<T, const N: usize> (imp::DynArray<T, StaticBuf<T, N>>);

impl<T, const N: usize> StaticDynArray<T, N> {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(imp::DynArray::new(UseAlloc::Default))
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.0.capacity()
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
        assert!(new_len <= N);
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
    pub fn append<const M: usize>(&mut self, other: &mut StaticDynArray<T, M>) {
        self.0.append(&mut other.0)
    }

    #[inline]
    pub fn drain<R: RangeBounds<usize>>(&mut self, range: R) -> Drain<'_, T, N> {
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
    pub fn splice<R, I>(&mut self, range: R, replace_with: I) -> Splice<'_, I::IntoIter, N>
    where
        R : RangeBounds<usize>,
        I : IntoIterator<Item = T>
    {
        Splice(self.0.splice(range, replace_with))
    }
}

impl<T: Clone, const N: usize> StaticDynArray<T, N> {
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

impl<T: PartialEq, const N: usize> StaticDynArray<T, N> {
    #[inline]
    pub fn dedup(&mut self) {
        self.0.dedup()
    }
}

//------------------------------------------------------------------------------------------------------------------------------

impl<T, const N: usize> Deref for StaticDynArray<T, N> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &*(self.0)
    }
}

impl<T, const N: usize> DerefMut for StaticDynArray<T, N> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *(self.0)
    }
}

impl<T: Clone, const N: usize> Clone for StaticDynArray<T, N> {
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.0.clone_from(&source.0)
    }
}

impl<T: Hash, const N: usize> Hash for StaticDynArray<T, N> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<T: Hash, I: SliceIndex<[T]>, const N: usize> Index<I> for StaticDynArray<T, N> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        self.0.index(index)
    }
}

impl<T: Hash, I: SliceIndex<[T]>, const N: usize> IndexMut<I> for StaticDynArray<T, N> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

impl<T, const N: usize> FromIterator<T> for StaticDynArray<T, N> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(FromIterator::from_iter(iter))
    }
}

impl<T, const N: usize> IntoIterator for StaticDynArray<T, N> {
    type Item = T;
    type IntoIter = IntoIter<T, N>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.0.into_iter())
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a mut StaticDynArray<T, N> {
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl<T, const N: usize> Extend<T> for StaticDynArray<T, N> {
    #[inline]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.0.extend(iter)
    }
}

impl<'a, T: Copy + 'a, const N: usize> Extend<&'a T> for StaticDynArray<T, N> {
    #[inline]
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.0.extend(iter)
    }
}

impl<T, const N: usize> Default for StaticDynArray<T, N> {
    #[inline]
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: fmt::Debug, const N: usize> fmt::Debug for StaticDynArray<T, N> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl<T, const N: usize> AsRef<StaticDynArray<T, N>> for StaticDynArray<T, N> {
    #[inline]
    fn as_ref(&self) -> &StaticDynArray<T, N> {
        self
    }
}

impl<T, const N: usize> AsMut<StaticDynArray<T, N>> for StaticDynArray<T, N> {
    #[inline]
    fn as_mut(&mut self) -> &mut StaticDynArray<T, N> {
       self 
    }
}

impl<T, const N: usize> AsRef<[T]> for StaticDynArray<T, N> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<T, const N: usize> AsMut<[T]> for StaticDynArray<T, N> {
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
       self 
    }
}

impl<T: Clone, const N: usize> From<&[T]> for StaticDynArray<T, N> {
    #[inline]
    fn from(s: &[T]) -> Self {
        Self(From::from(s))
    }
}

impl<T: Clone, const N: usize> From<&mut [T]> for StaticDynArray<T, N> {
    #[inline]
    fn from(s: &mut [T]) -> Self {
        Self(From::from(s))
    }
}

impl<T, const N: usize> From<[T; N]> for StaticDynArray<T, N> {
    #[inline]
    fn from(s: [T; N]) -> Self {
        Self(From::from(s))
    }
}

impl<const N: usize> From<&str> for StaticDynArray<u8, N> {
    #[inline]
    fn from(s: &str) -> Self {
        Self(From::from(s))
    }
}

impl<T, const N: usize, const M: usize> TryFrom<StaticDynArray<T, N>> for [T; M] {
    type Error = StaticDynArray<T, N>;

    #[inline]
    fn try_from(dynarr: StaticDynArray<T, N>) -> Result<Self, Self::Error> {
        match <[T; M]>::try_from(dynarr.0) {
            Ok(arr) => Ok(arr),
            Err(dynarr) => Err(StaticDynArray(dynarr))
        }
    }
}

//------------------------------------------------------------------------------------------------------------------------------


impl_slice_partial_eq!{ [const N: usize, const M: usize] StaticDynArray<T, N>, StaticDynArray<U, M> }
impl_slice_partial_eq!{ [const N: usize] StaticDynArray<T, N>, [U] }
impl_slice_partial_eq!{ [const N: usize] StaticDynArray<T, N>, &[U] }
impl_slice_partial_eq!{ [const N: usize] StaticDynArray<T, N>, &mut [U] }
impl_slice_partial_eq!{ [const N: usize, const M: usize] StaticDynArray<T, N>, [U; M] }
impl_slice_partial_eq!{ [const N: usize, const M: usize] StaticDynArray<T, N>, &[U; M] }
impl_slice_partial_eq!{ [const N: usize, const M: usize] StaticDynArray<T, N>, &mut [U; M] }
impl_slice_partial_eq!{ [const M: usize] [T], StaticDynArray<U, M> }
impl_slice_partial_eq!{ [const M: usize] &[T], StaticDynArray<U, M> }
impl_slice_partial_eq!{ [const M: usize] &mut [T], StaticDynArray<U, M> }
impl_slice_partial_eq!{ [const N: usize, const M: usize] [T; N], StaticDynArray<U, M> }
impl_slice_partial_eq!{ [const N: usize, const M: usize] &[T; N], StaticDynArray<U, M> }
impl_slice_partial_eq!{ [const N: usize, const M: usize] &mut [T; N], StaticDynArray<U, M> }


impl<T: Eq, const N: usize> Eq for StaticDynArray<T, N> {}

impl<T: PartialOrd, const N: usize> PartialOrd for StaticDynArray<T, N> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T: Ord, const N: usize> Ord for StaticDynArray<T, N> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

//------------------------------------------------------------------------------------------------------------------------------

pub struct IntoIter<T, const N: usize>(imp::IntoIter<T, StaticBuf<T, N>>);

impl<T, const N: usize> IntoIter<T, N> {
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        self.0.as_slice()
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.0.as_mut_slice()
    }
}

impl<T: fmt::Debug, const N: usize> fmt::Debug for IntoIter<T, N> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl<T, const N: usize> Iterator for IntoIter<T, N> {
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

impl<T, const N: usize> DoubleEndedIterator for IntoIter<T, N> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl<T, const N: usize> ExactSizeIterator for IntoIter<T, N> {}
impl<T, const N: usize> FusedIterator for IntoIter<T, N> {}

impl<T: Clone, const N: usize> Clone for IntoIter<T, N> {
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

unsafe impl<T: Send, const N: usize> Send for IntoIter<T, N> {}
unsafe impl<T: Sync, const N: usize> Sync for IntoIter<T, N> {}

//------------------------------------------------------------------------------------------------------------------------------

pub struct Drain<'a, T, const N: usize>(imp::Drain<'a, T, StaticBuf<T, N>>);

impl<T, const N: usize> Drain<'_, T, N> {
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        self.0.as_slice()
    }
}

impl<T: fmt::Debug, const N: usize> fmt::Debug for Drain<'_, T, N> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl<'a, T: 'a, const N: usize> AsRef<[T]> for Drain<'a, T, N> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.0.as_ref()
    }
}

impl<T, const N: usize> Iterator for Drain<'_, T, N> {
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

impl<T, const N: usize> DoubleEndedIterator for Drain<'_, T, N> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl<T, const N: usize> ExactSizeIterator for Drain<'_, T, N> {}

unsafe impl<T: Send, const N: usize> Send for Drain<'_, T, N> {}
unsafe impl<T: Sync, const N: usize> Sync for Drain<'_, T, N> {}

//------------------------------------------------------------------------------------------------------------------------------

pub struct Splice<'a, I, const N: usize>(imp::Splice<'a, I, StaticBuf<I::Item, N>>)
where
    I : Iterator + 'a
;

impl<I: Iterator<Item = T>, T, const N: usize> Iterator for Splice<'_, I, N> {
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

impl<I: Iterator<Item = T>, T, const N: usize> DoubleEndedIterator for Splice<'_, I, N> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl<I: Iterator<Item = T>, T, const N: usize> ExactSizeIterator for Splice<'_, I, N> {}

//------------------------------------------------------------------------------------------------------------------------------

pub trait SliceToStaticDynArray<T: Clone> {
    fn to_static_dynarray<const N: usize>(&self) -> StaticDynArray<T, N>;
}

impl<T: Clone> SliceToStaticDynArray<T> for [T] {
    default fn to_static_dynarray<const N: usize>(&self) -> StaticDynArray<T, N> {
        StaticDynArray(self.to_imp_dynarray::<StaticBuf<T, N>>(UseAlloc::Default))
    }
}

impl<T: Copy> SliceToStaticDynArray<T> for [T] {
    fn to_static_dynarray<const N: usize>(&self) -> StaticDynArray<T, N> {
        StaticDynArray(self.to_imp_dynarray::<StaticBuf<T, N>>(UseAlloc::Default))
    }
}