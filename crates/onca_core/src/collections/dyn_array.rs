extern crate alloc;

use crate::{alloc::{UseAlloc, Allocator}, mem::MEMORY_MANAGER};
use super::collections_alloc::Alloc;
use core::{
    mem::MaybeUninit, 
    borrow::{Borrow, BorrowMut},
    ops::{RangeBounds, Deref, DerefMut, Index, IndexMut},
    hash::Hash,
    slice::{SliceIndex, Iter, IterMut}
};
use alloc::{
    collections::TryReserveError,
    vec::Vec,
    borrow::Cow
};

/// Dynamically size array
/// 
/// Currently this is a wrapper around alloc::vec::Vec, but this will not always be the case
/// 
/// For information about the functions, check https://doc.rust-lang.org/std/vec/struct.Vec.html
pub struct DynArray<T>(pub(crate) Vec<T, Alloc>);

pub type Drain<'a, T> = alloc::vec::Drain<'a, T, Alloc>;
pub type Splice<'a, I> = alloc::vec::Splice<'a, I, Alloc>;
pub type IntoIter<T> = alloc::vec::IntoIter<T, Alloc>;

// feature(drain_filter ), issue: https://github.com/rust-lang/rust/issues/43244
// pub type DrainFilter<'a, T, F> = alloc::vec::DrainFilter<'a, T, F, Alloc>;

impl<T> DynArray<T> {
    
    #[must_use]
    pub fn new(alloc: UseAlloc) -> Self {
        Self(Vec::new_in(Alloc::new(alloc)))
    }

    pub fn with_capacity(capacity: usize, alloc: UseAlloc) -> Self {
        Self(Vec::with_capacity_in(capacity, Alloc::new(alloc)))
    }

    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional)
    }

    pub fn reserve_exact(&mut self, additional: usize) {
        self.0.reserve_exact(additional)
    }

    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.0.try_reserve(additional)
    }

    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.0.try_reserve_exact(additional)
    }

    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit()
    }

    pub fn truncate(&mut self, len: usize) {
        self.0.truncate(len)
    }

    pub fn as_slice(&self) -> &[T] {
        self.0.as_slice()
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.0.as_mut_slice()
    }

    pub fn as_ptr(&self) -> *const T {
        self.0.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.0.as_mut_ptr()
    }

    pub unsafe fn set_len(&mut self, new_len: usize) {
        self.0.set_len(new_len)
    }

    pub fn swap_remove(&mut self, index: usize) -> T {
        self.0.swap_remove(index)
    }

    pub fn insert(&mut self, index: usize, element: T) {
        self.0.insert(index, element)        
    }

    pub fn remove(&mut self, index: usize) -> T {
        self.0.remove(index)
    }

    pub fn retain<F>(&mut self, pred: F)
        where F: FnMut(&T) -> bool
    {
        self.0.retain(pred)
    }

    pub fn retain_mut<F>(&mut self, pred: F)
        where F: FnMut(&mut T) -> bool
    {
        self.0.retain_mut(pred)
    }

    pub fn dedup_by_key<F, K>(&mut self, key: F) 
        where F : FnMut(&mut T) -> K,
              K : PartialEq<K>
    {
        self.0.dedup_by_key(key)
    }

    pub fn dedup_by<F>(&mut self, same_bucket: F)
        where F : FnMut(&mut T, &mut T) -> bool
    {
        self.0.dedup_by(same_bucket)
    }

    pub fn push(&mut self, value: T)
    {
        self.0.push(value)
    }

    pub fn pop(&mut self) -> Option<T> {
        self.0.pop()
    }

    pub fn append(&mut self, other: &mut DynArray<T>) {
        self.0.append(&mut other.0)
    }

    pub fn drain<R: RangeBounds<usize>>(&mut self, range: R) -> Drain<'_, T> {
        self.0.drain(range)
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn split_off(&mut self, at: usize) -> DynArray<T> {
        Self(self.0.split_off(at))
    }

    pub fn resize_with<F>(&mut self, new_len: usize, f: F) 
        where F : FnMut() -> T
    {
        self.0.resize_with(new_len, f)
    }

    pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<T>] {
        self.0.spare_capacity_mut()
    }

    // feature(vec_split_at_spare), issue: https://github.com/rust-lang/rust/issues/81944
    /* 
    pub fn split_at_spare_mut(&mut self) -> (&mut [T], &mut [MaybeUninit<T>]) {
        self.0.split_at_spare_mut()
    }
    */

    pub fn splice<R, I>(&mut self, range: R, replace_with: I) -> Splice<'_, <I as IntoIterator>::IntoIter>
        where R : RangeBounds<usize>,
              I : IntoIterator<Item = T>
    {
        self.0.splice(range, replace_with)
    }

    // feature(drain_filter ), issue: https://github.com/rust-lang/rust/issues/43244
    /*
    pub fn drain_filter<F>(&mut self, filter: F) -> DrainFilter<'_, T, F>
        where F : FnMut(&mut T) -> bool
    {
        self.0.drain_filter(filter)
    }
    */

    pub fn from_str(val: &str, alloc: UseAlloc) -> DynArray<u8> {
        DynArray::from_slice(val.as_bytes(), alloc)
    }

    pub fn from_array<const N: usize>(val: [T; N], alloc: UseAlloc) -> Self {
        Self(<[T]>::into_vec(Box::new_in(val, Alloc::new(alloc))))
    }

    pub fn from_iter<I: IntoIterator<Item = T>>(iter: I, alloc: UseAlloc) -> Self {
        let mut arr = Self::new(alloc);
        arr.extend(iter);
        arr
    }

    pub fn allocator(&mut self) -> &mut dyn Allocator {
        MEMORY_MANAGER.get_allocator(UseAlloc::Id(self.allocator_id())).unwrap()
    }

    pub fn allocator_id(&self) -> u16 {
        self.0.allocator().layout().alloc_id()
    }

    pub unsafe fn get_underlying_container(&self) -> &alloc::vec::Vec<T, Alloc> {
        &self.0
    }

    pub unsafe fn get_underlying_container_mut(&mut self) -> &mut alloc::vec::Vec<T, Alloc> {
        &mut self.0
    }
}

impl<T: Clone> DynArray<T> {
    pub fn resize(&mut self, new_len: usize, value: T) {
        self.0.resize(new_len, value)
    }

    pub fn extend_from_slice(&mut self, other: &[T]) {
        self.0.extend_from_slice(other)
    }

    pub fn extend_from_within<R>(&mut self, src: R) 
        where R : RangeBounds<usize>
    {
        self.0.extend_from_within(src)
    }

    pub fn from_slice(val: &[T], alloc: UseAlloc) -> Self {
        Self(val.to_vec_in(Alloc::new(alloc)))
    }
}

impl<T, const N: usize> DynArray<[T; N]> {
    // feature(slice_flatten), issue: https://github.com/rust-lang/rust/issues/95629
    /*
    pub fn into_flattened(self) -> DynArray<T> {
        DynArray::<_>(self.0.into_flattened())
    }
    */
}

impl<T: PartialEq<T>> DynArray<T> {
    pub fn dedup(&mut self) {
        self.0.dedup()
    }
}

impl<T> AsMut<[T]> for DynArray<T> {
    fn as_mut(&mut self) -> &mut [T] {
        self.0.as_mut()
    }
}

impl<T> AsMut<DynArray<T>> for DynArray<T> {
    fn as_mut(&mut self) -> &mut DynArray<T> {
        self
    }
}

impl<T> AsRef<[T]> for DynArray<T> {
    fn as_ref(&self) -> &[T] {
        self.0.as_ref()
    }
}

impl<T> AsRef<DynArray<T>> for DynArray<T> {
    fn as_ref(&self) -> &DynArray<T> {
        self
    }
}

impl<T> Borrow<[T]> for DynArray<T> {
    fn borrow(&self) -> &[T] {
        self.0.borrow()
    }
}

impl<T> BorrowMut<[T]> for DynArray<T> {
    fn borrow_mut(&mut self) -> &mut [T] {
        self.0.borrow_mut()
    }
}

impl<T: Clone> Clone for DynArray<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Default for DynArray<T> {
    fn default() -> Self {
        Self::new(UseAlloc::Default)
    }
}

impl<T> Deref for DynArray<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<T> DerefMut for DynArray<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}

impl<'a, T: Copy + 'a> Extend<&'a T> for DynArray<T> {
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.0.extend(iter)
    }

    // feature(extend_one , issue: https://github.com/rust-lang/rust/issues/72631
    /*
    fn extend_one(&mut self, item: &'a T) {
        self.extend_one(item)
    }
    */

    // feature(extend_one), issue: https://github.com/rust-lang/rust/issues/72631
    /*
    fn extend_reserve(&mut self, additional: usize) {
        self.extend_reserve(additional)
    }
    */
}

impl<T> Extend<T> for DynArray<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.0.extend(iter)
    }

    // feature(extend_one), issue: https://github.com/rust-lang/rust/issues/72631
    /*
    fn extend_one(&mut self, item: &'a T) {
        self.extend_one(item)
    }
    */

    // feature(extend_one), issue: https://github.com/rust-lang/rust/issues/72631
    /*
    fn extend_reserve(&mut self, additional: usize) {
        self.extend_reserve(additional)
    }
    */
}

impl<T: Clone> From<&[T]> for DynArray<T> {
    fn from(val: &[T]) -> Self {
        Self::from_slice(val, UseAlloc::Default)
    }
}

impl<T: Clone> From<&mut [T]> for DynArray<T> {
    fn from(val: &mut [T]) -> Self {
        Self::from_slice(val, UseAlloc::Default)
    }
}

impl From<&str> for DynArray<u8> {
    fn from(val: &str) -> Self {
        Self::from_str(val, UseAlloc::Default)
    }
}

impl<'a, T: Clone> From<&'a DynArray<T>> for Cow<'a, [T]> {
    fn from(arr: &'a DynArray<T>) -> Self {
        Cow::Borrowed(arr.as_slice())
    }
}

impl<T, const N: usize> From<[T; N]> for DynArray<T> {
    fn from(val: [T; N]) -> Self {
        Self::from_array(val, UseAlloc::Default)
    }
}

impl<T> FromIterator<T> for DynArray<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from_iter(iter, UseAlloc::Default)
    }
}

impl<T: Hash> Hash for DynArray<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Hash::hash(&**self, state)
    }
}

impl<T, I: SliceIndex<[T]>> Index<I> for DynArray<T> {
    type Output = <I as SliceIndex<[T]>>::Output;

    fn index(&self, index: I) -> &Self::Output {
        Index::index(&**self, index)
    }
}

impl<T, I: SliceIndex<[T]>> IndexMut<I> for DynArray<T> {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(&mut **self, index)
    }
}

impl<T> IntoIterator for DynArray<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a mut DynArray<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        (&mut self.0).into_iter()
    }
}

impl<'a, T> IntoIterator for &'a DynArray<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}

macro_rules! impl_dyn_arr_slice_eq {
    ([$($vars:tt)*] $lhs: ty, $rhs: ty $(where $ty:ty: $bound:ident)?) => {
        impl<T, U, $($vars)*> PartialEq<$rhs> for $lhs
            where T: PartialEq<U>,
                  $($ty : $bound)?
        {
            #[inline]
            fn eq(&self, other:&$rhs) -> bool { self[..] == other[..] }
            #[inline]
            fn ne(&self, other:&$rhs) -> bool { self[..] != other[..] }
        }
    };
}
impl_dyn_arr_slice_eq!([] DynArray<T>, DynArray<U>);
impl_dyn_arr_slice_eq!([] DynArray<T>, [U]);
impl_dyn_arr_slice_eq!([] DynArray<T>, &[U]);
impl_dyn_arr_slice_eq!([] DynArray<T>, &mut [U]);
impl_dyn_arr_slice_eq!([] [T], DynArray<U>);
impl_dyn_arr_slice_eq!([] &[T], DynArray<U>);
impl_dyn_arr_slice_eq!([] &mut [T], DynArray<U>);
impl_dyn_arr_slice_eq!([const N: usize] DynArray<T>, [U; N]);
impl_dyn_arr_slice_eq!([const N: usize] DynArray<T>, &[U; N]);
impl_dyn_arr_slice_eq!([const N: usize] DynArray<T>, &mut [U; N]);
impl_dyn_arr_slice_eq!([const N: usize] [T; N], DynArray<U>);
impl_dyn_arr_slice_eq!([const N: usize] &[T; N], DynArray<U>);
impl_dyn_arr_slice_eq!([const N: usize] &mut [T; N], DynArray<U>);

impl<T: Eq> Eq for DynArray<T> {}

impl<T: PartialOrd> PartialOrd for DynArray<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }
}

impl<T: Ord> Ord for DynArray<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        Ord::cmp(&**self, &**other)
    }
}

impl<T, const N: usize> TryFrom<DynArray<T>> for [T; N] {
    type Error = DynArray<T>;

    fn try_from(mut value: DynArray<T>) -> Result<Self, Self::Error> {
        if value.len() != N {
            Err(value)
        } else {
            unsafe { value.set_len(0) }
            let array = unsafe { core::ptr::read(value.as_ptr() as *const [T; N]) };
            Ok(array)
        }
    }
}

