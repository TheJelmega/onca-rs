use core::{
    any::Any,
    borrow::*,
    convert::{AsRef, AsMut},
    cmp::*,
    hash::{Hash, Hasher},
    iter::*,
    marker::{PhantomData, Unsize},
    mem::{MaybeUninit, ManuallyDrop, size_of},
    ops::{Deref, DerefMut, CoerceUnsized},
    pin::*,
    ptr::{drop_in_place, null, NonNull},
};
use std::{
    
};
use crate::{
    alloc::{Allocation, Layout, UseAlloc, MemTag, ScopedAlloc, ScopedMemTag},
    mem::MEMORY_MANAGER,
};

use super::AllocInitState;

#[derive(Debug)]
pub struct HeapPtr<T: ?Sized> {
    ptr      : Allocation<T>,
    _phantom : PhantomData<T>
}

impl<T: ?Sized> HeapPtr<T> {

    /// Create a `HeapPtr` from an allocation
    /// 
    /// #Safety
    /// 
    /// The user needs to guarantee that the given allocation will not be deallocate by anything else, otherwise it results in UB
    #[inline]
    pub unsafe fn from_raw(ptr: Allocation<T>) -> Self {
        HeapPtr { ptr, _phantom: PhantomData }
    }

    /// Create a `HeapPtr` from a pointer and a layour
    /// 
    /// #Safety
    /// 
    /// The user needs to guarantee that the given allocation will not be deallocate by anything else, otherwise it results in UB
    #[inline]
    pub unsafe fn from_raw_components(ptr: *mut T, layout: Layout, mem_tag: MemTag) -> Self {
        HeapPtr { ptr: Allocation::from_raw(ptr, layout, mem_tag), _phantom: PhantomData }
    }

    /// Cast a `HeapPtr` to another `HeapPtr` that holds a different type
    /// 
    /// # Safety
    /// 
    /// The user is fully resposible to make sure the resulting cast is valid
    pub unsafe fn cast<U>(self) -> HeapPtr<U> {
        let alloc = Self::leak(self);
        HeapPtr::<U>::from_raw(alloc.cast::<U>())
    }
    
    /// Pin the `HeapPtr<T>`, if T does not implement 'Unpin', then x will be pinned in memory and unable to move
    // TODO(jel): if allocators would be able to defragment, this needs to notify it
    pub fn pin(this: Self) -> Pin<HeapPtr<T>> {
        unsafe { Pin::<_>::new_unchecked(this) }
    }

    /// Leak the underlying allocation
    pub fn leak(this: Self) -> Allocation<T> {
        let manual_drop = ManuallyDrop::new(this);
        unsafe { manual_drop.ptr.duplicate() }
    }

    /// Get a pointer to the underlying memory
    #[inline]
    pub fn ptr(&self) -> *const T {
        self.ptr.ptr()
    }

    /// Get a mutable pointer to the underlying memory
    #[inline]
    pub fn ptr_mut(&self) -> *mut T {
        self.ptr.ptr_mut()
    }

    /// Get the layout
    #[inline]
    pub fn layout(this: &Self) -> Layout {
        this.ptr.layout()
    }

    /// Get the allocator id
    #[inline]
    pub fn allocator_id(this: &Self) -> u16 {
        this.ptr.layout().alloc_id()
    }

    /// Get the memory tag
    #[inline]
    pub fn mem_tag(this: &Self) -> MemTag {
        this.ptr.mem_tag()
    }
}

impl<T> HeapPtr<T> {
    
    /// Create a new `HeapPtr<T>` using the given allocator
    #[inline]
    pub fn new(x: T) -> Self {
        Self::try_new(x).expect("Failed to allocate memory")
    }

    /// Try to create a new `HeapPtr<T>` using the given allocator
    pub fn try_new(x: T) -> Option<Self> {
        let uninit = Self::try_new_uninit();
        match uninit {
            None => None,
            Some(uninit) => Some(HeapPtr::<MaybeUninit<T>>::write(uninit, x))
        }
    }

    /// Creates new `HeapPtr<T>` with an uninitialized value, using the given allocator
    #[inline]
    pub fn new_uninit() -> HeapPtr<MaybeUninit<T>> {
        Self::try_new_uninit().expect("Failed to allocate memory")
    }

    /// Try to create a new `HeapPtr<T>` with an uninitialized value, using the given allocator
    pub fn try_new_uninit() -> Option<HeapPtr<MaybeUninit<T>>> {
        let uninit = if size_of::<MaybeUninit<T>>() == 0 {
            Some(unsafe { Allocation::const_null() })
        } else {
            MEMORY_MANAGER.alloc::<MaybeUninit<T>>(AllocInitState::Uninitialized)
        };
        match uninit {
            None => None,
            Some(ptr) => Some(HeapPtr::<MaybeUninit<T>>{ ptr: ptr.cast(), _phantom: PhantomData })
        }
    }

    /// Creates new `HeapPtr<[T]>` with an uninitialized value, using the given allocator
    #[inline]
    pub fn new_uninit_slice(len: usize) -> HeapPtr<[MaybeUninit<T>]> {
        Self::try_new_uninit_slice(len).expect("Failed to allocate memory")
    }

    /// Try to create a new `HeapPtr<[T]>` with an uninitialized value, using the given allocator
    pub fn try_new_uninit_slice(len: usize) -> Option<HeapPtr<[MaybeUninit<T>]>> {
        let layout = Layout::array::<MaybeUninit<T>>(len);
        let uninit = MEMORY_MANAGER.alloc_raw(AllocInitState::Uninitialized, layout);
        unsafe {
            match uninit {
                None => None,
                Some(ptr) => {
                    let mem = core::slice::from_raw_parts_mut(ptr.ptr_mut() as *mut MaybeUninit<T>, len);
                    Some(HeapPtr::<[MaybeUninit<T>]>{ ptr: ptr.with_ptr(mem), _phantom: PhantomData })
                }
            }
        }
    }

    /// Convert a `HeapPtr<T>` into `HeapPtr<[T]>`
    pub fn into_heap_slice(this: Self) -> HeapPtr<[T]> {
        let tmp = ManuallyDrop::new(this);
        HeapPtr { ptr: unsafe{ tmp.ptr.duplicate_cast::<[T; 1]>() }, _phantom: PhantomData }
    }

    pub fn null() -> HeapPtr<T> {
        HeapPtr { ptr: unsafe{ Allocation::null() }, _phantom: PhantomData }
    }
}

impl<T> HeapPtr<MaybeUninit<T>> {

    /// Converts to `HeapPtr<T>`
    /// 
    /// # Safety
    /// 
    /// It's up to the user to guarentee that the value is valid
    pub unsafe fn assume_init(self) -> HeapPtr<T> {
        let this = ManuallyDrop::new(self);
        let ptr = unsafe{ this.ptr.duplicate_cast::<T>() };
        HeapPtr { ptr, _phantom: PhantomData }
    }

    pub fn write(mut this: Self, value: T) -> HeapPtr<T> {
        this.ptr.get_mut().write(value);
        unsafe { this.assume_init() }
    }
}

impl<T> HeapPtr<[MaybeUninit<T>]> {

    /// Converts to `HeapPtr<T>`
    /// 
    /// # Safety
    /// 
    /// It's up to the user to guarentee that the value is valid
    pub unsafe fn assume_init(self) -> HeapPtr<[T]> {
        let this = ManuallyDrop::new(self);
        let ptr = unsafe{ this.ptr.duplicate().assume_init() };
        HeapPtr { ptr, _phantom: PhantomData }
    }
}


impl HeapPtr<dyn Any>
{
    /// Try to downcast to a concrete type, if the conversion failed, the original value will be found in the Err value
    pub fn downcast<T: Any>(self) -> Result<HeapPtr<T>, Self>
    {
        if self.is::<T>() {
            Ok(unsafe { self.downcast_unchecked() })
        } else {
            Err(self)
        }
    }

    /// Downcast to a concrete type, calling this on an invalid downcasted type will result in UB
    pub unsafe fn downcast_unchecked<T: Any>(self) -> HeapPtr<T> {
        debug_assert!(unsafe { self.ptr.get_ref().is::<T>() });
        HeapPtr::<T>{ ptr: HeapPtr::<_>::leak(self).cast(), _phantom: PhantomData }
    }
}

impl HeapPtr<dyn Any + Send>
{
    /// Try to downcast to a concrete type, if the conversion failed, the original value will be found in the Err value
    pub fn downcast<T: Any>(self) -> Result<HeapPtr<T>, Self>
    {
        if self.is::<T>() {
            Ok(unsafe { self.downcast_unchecked() })
        } else {
            Err(self)
        }
    }

    /// Downcast to a concrete type, calling this on an invalid downcasted type will result in UB
    pub unsafe fn downcast_unchecked<T: Any>(self) -> HeapPtr<T> {
        debug_assert!(unsafe { self.ptr.get_ref().is::<T>() });
        HeapPtr::<T>{ ptr: HeapPtr::<_>::leak(self).cast(), _phantom: PhantomData }
    }
}

impl HeapPtr<dyn Any + Send + Sync>
{
    /// Try to downcast to a concrete type, if the conversion failed, the original value will be found in the Err value
    pub fn downcast<T: Any>(self) -> Result<HeapPtr<T>, Self>
    {
        if self.is::<T>() {
            Ok(unsafe { self.downcast_unchecked() })
        } else {
            Err(self)
        }
    }

    /// Downcast to a concrete type, calling this on an invalid downcasted type will result in UB
    pub unsafe fn downcast_unchecked<T: Any>(self) -> HeapPtr<T> {
        debug_assert!(unsafe { self.ptr.get_ref().is::<T>() });
        HeapPtr::<T>{ ptr: HeapPtr::<_>::leak(self).cast(), _phantom: PhantomData }
    }
}


impl<T: ?Sized> AsMut<T> for HeapPtr<T> {
    fn as_mut(&mut self) -> &mut T {
        self.ptr.get_mut()
    }
}

impl<T: ?Sized> AsRef<T> for HeapPtr<T> {
    fn as_ref(&self) -> &T {
        self.ptr.get_ref()
    }
}

impl<T: ?Sized> Borrow<T> for HeapPtr<T> {
    fn borrow(&self) -> &T {
        self.ptr.get_ref()
    }
}

impl<T: ?Sized> BorrowMut<T> for HeapPtr<T> {
    fn borrow_mut(&mut self) -> &mut T {
        self.ptr.get_mut()
    }
}

impl<T: ?Sized> Deref for HeapPtr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.ptr.get_ref()
    }
}

impl<T: ?Sized> DerefMut for HeapPtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ptr.get_mut()
    }
}

impl<T> HeapPtr<T> {
    /// Deref and move the value out of the HeapPtr
    /// 
    /// Memory will also be deallocated
    pub fn deref_move(self) -> T {
        let me = ManuallyDrop::new(self);
        let val = unsafe{ core::ptr::read(me.ptr()) };
        MEMORY_MANAGER.dealloc(unsafe{ me.ptr.duplicate() });
        val
    }
}

impl<T: ?Sized> Drop for HeapPtr<T> {
    fn drop(&mut self) {
        if self.ptr.ptr() as *const u8 != null() {
            unsafe { drop_in_place(self.ptr.ptr_mut()) };
            MEMORY_MANAGER.dealloc(unsafe{ self.ptr.duplicate() });
        }
    }
}

impl<T: Clone> Clone for HeapPtr<T> {
    fn clone(&self) -> Self {
        let _scope_alloc = ScopedAlloc::new(UseAlloc::Id(Self::allocator_id(self)));
        let _scope_mem_tag = ScopedMemTag::new(Self::mem_tag(self));
        let new = Self::new_uninit();
        HeapPtr::<_>::write(new, self.as_ref().clone())
    }

    fn clone_from(&mut self, source: &Self)
    {
        (**self).clone_from(&**source);
    }
}

impl<T: Iterator + ?Sized> Iterator for HeapPtr<T> {
    type Item = <T as Iterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.as_mut().next()
    }
}

impl<T: DoubleEndedIterator + ?Sized> DoubleEndedIterator for HeapPtr<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.as_mut().next_back()
    }
}

impl<T: ExactSizeIterator + ?Sized> ExactSizeIterator for HeapPtr<T> {
    fn len(&self) -> usize {
        self.as_ref().len()
    }
}

impl<T: FusedIterator + ?Sized> FusedIterator for HeapPtr<T> {}

impl<T: Hash + ?Sized> Hash for HeapPtr<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state)
    }
}

impl<T: Hasher + ?Sized> Hasher for HeapPtr<T> {
    fn finish(&self) -> u64 {
        self.as_ref().finish()
    }

    fn write(&mut self, bytes: &[u8]) {
        self.as_mut().write(bytes);
    }

    fn write_u8(&mut self, i: u8) {
        self.as_mut().write_u8(i);
    }

    fn write_u16(&mut self, i: u16) {
        self.as_mut().write_u16(i);
    }

    fn write_u32(&mut self, i: u32) {
        self.as_mut().write_u32(i);
    }

    fn write_u64(&mut self, i: u64) {
        self.as_mut().write_u64(i);
    }

    fn write_u128(&mut self, i: u128) {
        self.as_mut().write_u128(i);
    }

    fn write_usize(&mut self, i: usize) {
        self.as_mut().write_usize(i);
    }

    fn write_i8(&mut self, i: i8) {
        self.as_mut().write_i8(i);
    }

    fn write_i16(&mut self, i: i16) {
        self.as_mut().write_i16(i);
    }

    fn write_i32(&mut self, i: i32) {
        self.as_mut().write_i32(i);
    }

    fn write_i64(&mut self, i: i64) {
        self.as_mut().write_i64(i);
    }

    fn write_i128(&mut self, i: i128) {
        self.as_mut().write_i128(i);
    }

    fn write_isize(&mut self, i: isize) {
        self.as_mut().write_isize(i);
    }
}

impl <T: PartialEq + ?Sized> PartialEq for HeapPtr<T> {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref().eq(other.as_ref())
    }

    fn ne(&self, other: &Self) -> bool {
        self.as_ref().ne(other.as_ref())
    }
}

impl <T: Eq + ?Sized> Eq for HeapPtr<T> {}

impl<T: PartialOrd + ?Sized> PartialOrd for HeapPtr<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.as_ref().partial_cmp(other.as_ref())
    }

    fn lt(&self, other: &Self) -> bool {
        self.as_ref().lt(other.as_ref())
    }

    fn le(&self, other: &Self) -> bool {
        self.as_ref().le(other.as_ref())
    }

    fn gt(&self, other: &Self) -> bool {
        self.as_ref().gt(other.as_ref())
    }

    fn ge(&self, other: &Self) -> bool {
        self.as_ref().ge(other.as_ref())
    }
}

impl<T: Ord + ?Sized> Ord for HeapPtr<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}

impl<T: ?Sized> Unpin for HeapPtr<T> {}

impl<T, U> CoerceUnsized<HeapPtr<U>> for HeapPtr<T>
where 
    T : Unsize<U> + ?Sized,
    U : ?Sized
{}