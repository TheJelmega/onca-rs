use core::{
    marker::Unsize,
    ops::{Deref, DerefMut, Index, IndexMut},
    slice::{self, SliceIndex}
};

use crate::{alloc::{UseAlloc, MemTag, Layout, CoreMemTag, ScopedMemTag, ScopedAlloc}, mem::HeapPtr};

use super::DynArray;


/// Handle into a callback array
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CallbackHandle(usize);

pub type CallbackEntry<F> = (CallbackHandle, HeapPtr<F>);

// PERF(jel): Check if there would be a good performance reason to switch to a HashMap<usize, HeapPtr<...>>
/// Dynamic array for storing callbacks
pub struct CallbackArray<F: ?Sized>{
    arr    : DynArray<CallbackEntry<F>>,
    cur_id : usize
}

impl<F: ?Sized> CallbackArray<F> {
    /// Create a new callback array
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        let _scope_mem_tag = ScopedMemTag::new(CoreMemTag::callbacks());
        Self {
            arr: DynArray::new(),
            cur_id: 0
        }
    }

    /// Create a new callback array with a given capacity
    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        let _scope_mem_tag = ScopedMemTag::new(CoreMemTag::callbacks());
        Self {
            arr: DynArray::with_capacity(capacity),
            cur_id: 0
        }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.arr.capacity()
    }

    #[inline]
    pub fn reserve(&mut self, additional:usize) {
        self.arr.reserve(additional);
    }

    #[inline]
    pub fn try_reserve(&mut self, additional:usize) -> Result<(), std::collections::TryReserveError> {
        self.arr.try_reserve(additional)
    }

    #[inline]
    pub fn reserve_exact(&mut self, additional:usize) {
        self.arr.reserve_exact(additional);
    }

    #[inline]
    pub fn try_reserve_exact(&mut self, additional:usize) -> Result<(), std::collections::TryReserveError> {
        self.arr.try_reserve_exact(additional)
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.arr.shrink_to_fit()
    }

    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.arr.shrink_to(min_capacity)
    }

    /// Remove a callback using its handle
    pub fn remove(&mut self, handle: CallbackHandle) {
        let idx = self.arr.binary_search_by_key(&handle, |val| val.0);
        if let Ok(idx) = idx {
            self.arr.remove(idx);
        }
    }

    #[must_use = "If not stored, the callback will not be able to be removed separately. If discarded, the callback will only be able to be removed using `clear()`."]
    pub fn push<G>(&mut self, callback: G) -> CallbackHandle
    where
        G : Unsize<F>
    {
        let _scope_alloc = ScopedAlloc::new(UseAlloc::Id(self.arr.allocator_id()));
        let _scope_mem_tag = ScopedMemTag::new(CoreMemTag::callbacks());

        let handle = CallbackHandle(self.cur_id);
        let heap_ptr = HeapPtr::new(callback);

        self.arr.push((handle, heap_ptr));
        self.cur_id += 1;

        handle
    }
    
    #[inline]
    pub fn clear(&mut self) {
        self.arr.clear()
    }

    pub fn get(&self, handle: CallbackHandle) -> Option<&HeapPtr<F>> {
        let idx = self.arr.binary_search_by_key(&handle, |val| val.0);
        match idx {
            Ok(idx) => Some(&self.arr[idx].1),
            Err(_) => None,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.arr.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.arr.is_empty()
    }
    
    #[inline]
    #[must_use]
    pub fn layout(&self) -> Layout {
        self.arr.layout()
    }

    #[inline]
    #[must_use]
    pub fn allocator_id(&self) -> u16 {
        self.arr.allocator_id()
    }

    #[inline]
    #[must_use]
    pub fn mem_tag(&self) -> MemTag {
        self.arr.mem_tag()
    }
}

impl<F: ?Sized> Deref for CallbackArray<F> {
    type Target = [CallbackEntry<F>];

    fn deref(&self) -> &Self::Target {
        &*(self.arr)
    }
}

impl<F: ?Sized> DerefMut for CallbackArray<F> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *(self.arr)
    }
}

impl<F: ?Sized> AsRef<CallbackArray<F>> for CallbackArray<F> {
    fn as_ref(&self) -> &CallbackArray<F> {
        self
    }
}

impl<F: ?Sized> AsMut<CallbackArray<F>> for CallbackArray<F> {
    fn as_mut(&mut self) -> &mut CallbackArray<F> {
        self
    }
}

impl<F: ?Sized> AsRef<[CallbackEntry<F>]> for CallbackArray<F> {
    fn as_ref(&self) -> &[CallbackEntry<F>] {
        self
    }
}

impl<F: ?Sized> AsMut<[CallbackEntry<F>]> for CallbackArray<F> {
    fn as_mut(&mut self) -> &mut [CallbackEntry<F>] {
        self
    }
}


impl<F: ?Sized> IntoIterator for CallbackArray<F> {
    type Item = CallbackEntry<F>;
    type IntoIter = super::dyn_array::IntoIter<CallbackEntry<F>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.arr.into_iter()
    }
}

impl<'a, F: ?Sized> IntoIterator for &'a CallbackArray<F> {
    type Item = &'a CallbackEntry<F>;
    type IntoIter = slice::Iter<'a, CallbackEntry<F>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.arr.iter()
    }
}

impl<'a, F: ?Sized> IntoIterator for &'a mut CallbackArray<F> {
    type Item = &'a mut CallbackEntry<F>;
    type IntoIter = slice::IterMut<'a, CallbackEntry<F>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.arr.iter_mut()
    }
}
