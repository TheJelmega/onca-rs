extern crate alloc;

use core::{
    ops::{RangeBounds, Index, IndexMut}, 
    cmp::Ordering,
    hash::{Hash, Hasher}
};

use alloc::collections::TryReserveError;
use alloc::collections::vec_deque as alloc_vec_deque;

use crate::{
    alloc::{Allocator, UseAlloc},
    mem::MEMORY_MANAGER
};
use super::DynArray;
use super::collections_alloc::Alloc;

pub struct VecDeque<T>(alloc_vec_deque::VecDeque<T, Alloc>);
type Iter<'a, T> = alloc_vec_deque::Iter<'a, T>;
type IterMut<'a, T> = alloc_vec_deque::IterMut<'a, T>;
type IntoIter<T> = alloc_vec_deque::IntoIter<T, Alloc>;
type Drain<'a, T> = alloc_vec_deque::Drain<'a, T, Alloc>;

impl<T> VecDeque<T> {
    
    pub fn new(alloc: UseAlloc) -> Self {
        Self(alloc_vec_deque::VecDeque::new_in(Alloc::new(alloc)))
    }

    pub fn with_capacity(capacity: usize, alloc: UseAlloc) -> Self {
        Self(alloc_vec_deque::VecDeque::with_capacity_in(capacity, Alloc::new(alloc)))
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.0.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.0.get_mut(index)
    }

    pub fn swap(&mut self, i: usize, j: usize) {
        self.0.swap(i, j)
    }

    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    pub fn reserve_exact(&mut self, additional: usize) {
        self.0.reserve_exact(additional)
    }

    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional)
    }

    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.0.try_reserve_exact(additional)
    }

    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.0.try_reserve(additional)
    }

    pub fn shrink_to_fit(& mut self) {
        self.0.shrink_to_fit();
    }

    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.0.shrink_to(min_capacity)
    }

    pub fn truncate(&mut self, len: usize) {
        self.0.truncate(len)
    }

    pub fn iter(&self) -> Iter<'_, T> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        self.0.iter_mut()
    }

    pub fn as_slices(&self) -> (&[T], &[T]) {
        self.0.as_slices()
    }

    pub fn as_mut_slices(&mut self) -> (&mut [T], &mut [T]) {
        self.0.as_mut_slices()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn range<R: RangeBounds<usize>>(&self, range: R) -> Iter<'_, T> {
        self.0.range(range)
    }

    pub fn range_mut<R: RangeBounds<usize>>(&mut self, range: R) -> IterMut<'_, T> {
        self.0.range_mut(range)
    }

    pub fn drain<R: RangeBounds<usize>>(&mut self, range: R) -> Drain<'_, T> {
        self.0.drain(range)
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }

    pub fn contains(&self, x: &T) -> bool 
        where T : PartialEq
    {
        self.0.contains(x)
    }

    pub fn front(&self) -> Option<&T> {
        self.0.front()
    }

    pub fn front_mut(&mut self) -> Option<&mut T> {
        self.0.front_mut()
    }

    pub fn back(&self) -> Option<&T> {
        self.0.back()
    }
    
    pub fn back_mut(&mut self) -> Option<&mut T> {
        self.0.back_mut()
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.0.pop_front()
    }

    pub fn push_front(&mut self, value: T) {
        self.0.push_front(value)
    }

    pub fn pop_back(&mut self) -> Option<T> {
        self.0.pop_back()
    }

    pub fn push_back(&mut self, value: T) {
        self.0.push_back(value)
    }

    pub fn swap_remove_front(&mut self, index: usize) -> Option<T> {
        self.0.swap_remove_front(index)
    }

    pub fn swap_remove_back(&mut self, index: usize) -> Option<T> {
        self.0.swap_remove_back(index)
    }

    pub fn insert(&mut self, index: usize, value: T) {
        self.0.insert(index, value)
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        self.0.remove(index)
    }

    pub fn split_off(&mut self, at: usize) -> Self {
        Self(self.0.split_off(at))
    }

    pub fn append(&mut self, other: &mut Self) {
        self.0.append(&mut other.0)
    }

    pub fn retain<F>(&mut self, f: F)
        where F : FnMut(&T) -> bool
    {
        self.0.retain(f)
    }

    pub fn retain_mut<F>(&mut self, f: F)
        where F : FnMut(&mut T) -> bool
    {
        self.0.retain_mut(f)
    }

    pub fn resize_with<F>(&mut self, new_len: usize, f: F) 
        where F : FnMut() -> T
    {
        self.0.resize_with(new_len, f)
    }

    pub fn make_contiguous(&mut self) -> &mut [T] {
        self.0.make_contiguous()
    }

    pub fn rotate_left(&mut self, mid: usize) {
        self.0.rotate_left(mid)
    }

    pub fn rotate_right(&mut self, k: usize) {
        self.0.rotate_right(k)
    }

    pub fn binary_search(&self, x: &T) -> Result<usize, usize>
        where T : Ord
    {
        self.0.binary_search(x)
    }

    pub fn binary_search_by<'a, F>(&'a self, f: F) -> Result<usize, usize>
        where F : FnMut(&'a T) -> Ordering
    {
        self.0.binary_search_by(f)
    }

    pub fn binary_search_by_key<'a, B, F>(&'a self, b: &B, f: F) -> Result<usize, usize>
        where F : FnMut(&'a T) -> B,
              B : Ord
    {
        self.0.binary_search_by_key(b, f)
    }

    pub fn partition_point<P>(&self, pred: P) -> usize
        where P: FnMut(&T) -> bool
    {
        self.0.partition_point(pred)
    }

    pub fn allocator_id(&self) -> u16 {
        self.0.allocator().layout().alloc_id()
    }

    pub fn allocator(&mut self) -> &mut dyn Allocator {
        MEMORY_MANAGER.get_allocator(UseAlloc::Id(self.allocator_id())).unwrap()
    }

    pub fn from_iter<I: IntoIterator<Item = T>>(iter: I, alloc: UseAlloc) -> Self {
        let mut deque = VecDeque::new(alloc);
        deque.extend(iter);
        deque
    }
}

impl<T: Clone> VecDeque<T> {
    pub fn resize(&mut self, new_len: usize, value: T) {
        self.0.resize(new_len, value)
    }
}

impl<T: Clone> Clone for VecDeque<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }

    fn clone_from(&mut self, source: &Self)
    {
        self.0.clone_from(&source.0)
    }
}

impl<T> Default for VecDeque<T> {
    fn default() -> Self {
        Self::new(UseAlloc::Default)
    }
}

impl<'a, T: 'a + Copy> Extend<&'a T> for VecDeque<T> {
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.0.extend(iter)
    }

    // feature(extend_one), issue: https://github.com/rust-lang/rust/issues/72631
    /*
    fn extend_one(&mut self, item: &'a T) {
        self.0.extend_one(item)
    }
    */

    // feature(extend_one), issue: https://github.com/rust-lang/rust/issues/72631
    /*
    fn extend_reserve(&mut self, additional: usize) {
        self.0.extend_reserve(additional)
    }
    */
}

impl<T> Extend<T> for VecDeque<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.0.extend(iter)
    }

    // feature(extend_one), issue: https://github.com/rust-lang/rust/issues/72631
    /*
    fn extend_one(&mut self, item: &'a T) {
        self.0.extend_one(item)
    }
    */

    // feature(extend_one), issue: https://github.com/rust-lang/rust/issues/72631
    /*
    fn extend_reserve(&mut self, additional: usize) {
        selfv.extend_reserve(additional)
    }
    */
}

impl<T, const N: usize> From<[T; N]> for VecDeque<T> {
    fn from(arr: [T; N]) -> Self {
        Self::from_iter(arr, UseAlloc::Default)
    }
}

impl<T> From<DynArray<T>> for VecDeque<T> {
    fn from(dyn_arr: DynArray<T>) -> Self {
        Self(<alloc_vec_deque::VecDeque<T, Alloc> as From<Vec<T, Alloc>>>::from(dyn_arr.0))
    }
}

impl<T> From<VecDeque<T>> for DynArray<T> {
    fn from(deque: VecDeque<T>) -> Self {
        DynArray(<alloc::vec::Vec<T, Alloc> as From<alloc_vec_deque::VecDeque<T, Alloc>>>::from(deque.0))
    }
}

impl<T> FromIterator<T> for VecDeque<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from_iter(iter, UseAlloc::Default)
    }
}

impl<T: Hash> Hash for VecDeque<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T> Index<usize> for VecDeque<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

impl<T> IndexMut<usize> for VecDeque<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

impl<T> IntoIterator for VecDeque<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a VecDeque<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}

impl<'a, T> IntoIterator for &'a mut VecDeque<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        (&mut self.0).into_iter()
    }
}

impl<T: PartialEq> PartialEq for VecDeque<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

macro_rules! impl_dyn_arr_slice_eq {
    ([$($vars:tt)*] $lhs: ty, $rhs: ty $(where $ty:ty: $bound:ident)?) => {
        impl<T, U, $($vars)*> PartialEq<$rhs> for $lhs
            where T: PartialEq<U>,
                  $($ty : $bound)?
        {
            #[inline]
            fn eq(&self, other:&$rhs) -> bool 
            {
                if self.len() != other.len() {
                    return false;
                }

                let (sa, sb) = self.as_slices();
                let (oa, ob) = other[..].split_at(sa.len());
                sa == oa && sb == ob
            }
        }
    };
}
impl_dyn_arr_slice_eq!([] VecDeque<T>, [U]);
impl_dyn_arr_slice_eq!([] VecDeque<T>, &[U]);
impl_dyn_arr_slice_eq!([] VecDeque<T>, &mut [U]);
impl_dyn_arr_slice_eq!([const N: usize] VecDeque<T>, [U; N]);
impl_dyn_arr_slice_eq!([const N: usize] VecDeque<T>, &[U; N]);
impl_dyn_arr_slice_eq!([const N: usize] VecDeque<T>, &mut [U; N]);

impl<T: Eq> Eq for VecDeque<T> {}

impl<T: Ord> Ord for VecDeque<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.iter().cmp(other.iter())
    }
}

impl<T: PartialOrd> PartialOrd for VecDeque<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.iter().partial_cmp(other.iter())
    }
}