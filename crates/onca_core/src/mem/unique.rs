use core::{
    any::Any,
    borrow::*,
    convert::{AsRef, AsMut},
    cmp::*,
    hash::{Hash, Hasher},
    iter::*,
    marker::PhantomData,
    mem::{MaybeUninit, ManuallyDrop},
    ops::{Deref, DerefMut},
    pin::*,
    ptr::drop_in_place,
};
use std::{ops::CoerceUnsized, marker::Unsize, f32::consts::E};
use crate::alloc::{Allocator, Allocation, Layout, UseAlloc, MemTag};
use super::{MEMORY_MANAGER, HeapPtr};


pub struct Unique<T: ?Sized> {
    ptr : HeapPtr<T>
}

impl<T: ?Sized> Unique<T> {
    /// Create a `Unique<T>` from an allocation
    /// 
    /// #Safety
    /// 
    /// The user needs to guarantee that the given allocation will not be deallocate by anything else, otherwise it results in UB
    #[inline]
    pub unsafe fn from_raw(ptr: Allocation<T>) -> Self {
        Unique { ptr: HeapPtr::from_raw(ptr) }
    }
    
    /// Pin the `Unique<T>`, if T does not implement 'Unpin', then x will be pinned in memory and unable to move
    // TODO(jel): if allocators would be able to defragment, this needs to notify it
    #[inline]
    pub fn pin(this: Self) -> Pin<Unique<T>> {
        unsafe { Pin::new_unchecked(this) }
    }

    /// Leak the underlying allocation
    #[inline]
    pub fn leak(this: Self) -> Allocation<T> {
        HeapPtr::leak(this.ptr)
    }

    /// Get the layout
    #[inline]
    pub fn layout(this: &Self) -> Layout {
        HeapPtr::layout(&this.ptr)
    }

    /// Get the allocator id
    #[inline]
    pub fn allocator_id(this: &Self) -> u16 {
        HeapPtr::allocator_id(&this.ptr)
    }

    /// Get the allocator
    #[inline]
    pub fn allocator(this: &Self) -> &mut dyn Allocator {
        HeapPtr::allocator(&this.ptr)
    }

    /// Get the memory tag
    #[inline]
    pub fn mem_tag(this: &Self) -> MemTag {
        HeapPtr::mem_tag(&this.ptr)
    }
}

impl<T> Unique<T> {
    
    /// Create a new `Unique<T>` using the given allocator
    #[inline]
    pub fn new(x: T, alloc: UseAlloc, mem_tag: MemTag) -> Self {
        Self::try_new(x, alloc, mem_tag).expect("Failed to allocate memory")
    }

    /// Try to create a new `Unique<T>` using the given allocator
    pub fn try_new(x: T, alloc: UseAlloc, mem_tag: MemTag) -> Option<Self> {
        let heap = HeapPtr::<T>::try_new(x, alloc, mem_tag);
        match heap {
            None => None,
            Some(ptr) => Some(Self{ ptr })
        }
    }

    /// Creates new `Unique<T>` with an uninitialized value, using the given allocator
    #[inline]
    pub fn new_uninit(alloc: UseAlloc, mem_tag: MemTag) -> Unique<MaybeUninit<T>> {
        Self::try_new_uninit(alloc, mem_tag).expect("Failed to allocate memory")
    }

    /// Try to create a new `Unique<T>` with an uninitialized value, using the given allocator
    pub fn try_new_uninit(alloc: UseAlloc, mem_tag: MemTag) -> Option<Unique<MaybeUninit<T>>> {
        let ptr = HeapPtr::<T>::try_new_uninit(alloc, mem_tag);
        match ptr {
            None => None,
            Some(ptr) => {
                Some(Unique::<MaybeUninit<T>>{ ptr })
            }
        }
    }

}

impl<T> Unique<MaybeUninit<T>> {

    /// Converts to `Unique<T>`
    /// 
    /// # Safety
    /// 
    /// It's up to the user to guarentee that the value is valid
    #[inline]
    pub unsafe fn assume_init(self) -> Unique<T> {
        Unique { ptr: self.ptr.assume_init() }
    }

    #[inline]
    pub fn write(this: Self, value: T) -> Unique<T> {
        Unique { ptr: HeapPtr::<_>::write(this.ptr, value) }
    }
}

impl Unique<dyn Any>
{
    /// Try to downcast to a concrete type, if the conversion failed, the original value will be found in the Err value
    pub fn downcast<T: Any>(self) -> Result<Unique<T>, Unique<dyn Any>>
    {
        match self.ptr.downcast() {
            Ok(ptr) => Ok(Unique::<T>{ ptr }),
            Err(ptr) => Err(Unique::<dyn Any>{ ptr })
        }
    }

    /// Downcast to a concrete type, calling this on an invalid downcasted type will result in UB
    #[inline]
    pub unsafe fn downcast_unchecked<T: Any>(self) -> Unique<T> {
        Unique::<T> { ptr: self.ptr.downcast_unchecked() }
    }
}

impl Unique<dyn Any + Send>
{
    /// Try to downcast to a concrete type, if the conversion failed, the original value will be found in the Err value
    pub fn downcast<T: Any>(self) -> Result<Unique<T>, Unique<dyn Any + Send>>
    {
        match self.ptr.downcast() {
            Ok(ptr) => Ok(Unique::<T>{ ptr }),
            Err(ptr) => Err(Unique::<dyn Any + Send>{ ptr })
        }
    }

    /// Downcast to a concrete type, calling this on an invalid downcasted type will result in UB
    #[inline]
    pub unsafe fn downcast_unchecked<T: Any>(self) -> Unique<T> {
        Unique::<T> { ptr: self.ptr.downcast_unchecked() }
    }
}

impl Unique<dyn Any + Send + Sync>
{
    /// Try to downcast to a concrete type, if the conversion failed, the original value will be found in the Err value
    pub fn downcast<T: Any>(self) -> Result<Unique<T>, Unique<dyn Any + Send + Sync>>
    {
        match self.ptr.downcast() {
            Ok(ptr) => Ok(Unique::<T>{ ptr }),
            Err(ptr) => Err(Unique::<dyn Any + Send + Sync>{ ptr })
        }
    }

    /// Downcast to a concrete type, calling this on an invalid downcasted type will result in UB
    #[inline]
    pub unsafe fn downcast_unchecked<T: Any>(self) -> Unique<T> {
        Unique::<T> { ptr: self.ptr.downcast_unchecked() }
    }
}

impl<T: ?Sized> AsMut<T> for Unique<T> {
    fn as_mut(&mut self) -> &mut T {
        self.ptr.as_mut()
    }
}

impl<T: ?Sized> AsRef<T> for Unique<T> {
    fn as_ref(&self) -> &T {
        self.ptr.as_ref()
    }
}

impl<T: ?Sized> Borrow<T> for Unique<T> {
    fn borrow(&self) -> &T {
        self.ptr.borrow()
    }
}

impl<T: ?Sized> BorrowMut<T> for Unique<T> {
    fn borrow_mut(&mut self) -> &mut T {
        self.ptr.borrow_mut()
    }
}

impl<T: ?Sized> Deref for Unique<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.ptr.deref()
    }
}

impl<T: ?Sized> DerefMut for Unique<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ptr.deref_mut()
    }
}

impl<T: Iterator + ?Sized> Iterator for Unique<T> {
    type Item = <T as Iterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.as_mut().next()
    }
}

impl<T: DoubleEndedIterator + ?Sized> DoubleEndedIterator for Unique<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.as_mut().next_back()
    }
}

impl<T: ExactSizeIterator + ?Sized> ExactSizeIterator for Unique<T> {
    fn len(&self) -> usize {
        self.as_ref().len()
    }
}

impl<T: FusedIterator + ?Sized> FusedIterator for Unique<T> {}

impl<T: Hash + ?Sized> Hash for Unique<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state)
    }
}

impl<T: Hasher + ?Sized> Hasher for Unique<T> {
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

impl <T: PartialEq + ?Sized> PartialEq for Unique<T> {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref().eq(other.as_ref())
    }

    fn ne(&self, other: &Self) -> bool {
        self.as_ref().ne(other.as_ref())
    }
}

impl <T: Eq + ?Sized> Eq for Unique<T> {}

impl<T: PartialOrd + ?Sized> PartialOrd for Unique<T> {
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

impl<T: Ord + ?Sized> Ord for Unique<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}

impl<T: ?Sized> Unpin for Unique<T> {}

impl<T, U> CoerceUnsized<Unique<U>> for Unique<T>
    where T : Unsize<U> + ?Sized,
          U : ?Sized
{}