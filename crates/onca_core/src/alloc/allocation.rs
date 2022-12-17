use core::{any::Any, borrow::*, marker::Unsize, ops::*, ptr::*};
use std::mem::MaybeUninit;

use crate::mem::MEMORY_MANAGER;

use super::{Layout, Allocator, UseAlloc, layout};

#[cfg(feature = "memory_tracking")]
use super::mem_tag::MemTag;

/// Representation of allocated memory
#[derive(Debug)]
pub struct Allocation<T: ?Sized>
{
    ptr     : NonNull<T>,
    layout  : Layout,
    mem_tag : MemTag,
}

impl<T: ?Sized> Allocation<T>
{
    /// Get the contained pointer
    #[inline]
    pub fn ptr(&self) -> *const T {
        self.ptr.as_ptr()
    }

    /// Get the contained pointer
    #[inline]
    pub fn ptr_mut(&self) -> *mut T {
        self.ptr.as_ptr()
    }

    /// Get the layout
    #[inline]
    pub fn layout(&self) -> Layout {
        self.layout
    }

    /// Get the layout
    #[inline]
    pub fn layout_mut(&mut self) -> &mut Layout {
        &mut self.layout
    }

    /// Get the memory tag
    #[inline]
    pub fn mem_tag(&self) -> MemTag {
        self.mem_tag
    }
    
    /// Get the memory tag
    #[inline]
    pub fn mem_tag_mut(&mut self) -> &mut MemTag {
        &mut self.mem_tag
    }
    
    /// Get a reference to the data pointed at by the `Allocation<T>`
    #[inline]
    pub fn get_ref(&self) -> &T {
        unsafe { self.ptr.as_ref() }
    }

    /// Get a mutable reference to the data pointed at by the `Allocation<T>`
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.ptr_mut() }
    }

    /// Duplicate the `Allocation`  and cast it
    /// 
    /// # Safety
    /// 
    /// Duplicating the allocation is unsafe, as it could cause double deallocations
    #[inline]
    pub unsafe fn duplicate_cast<U>(&self) -> Allocation<U> {
        self.duplicate().cast()
    }

    /// Converts an `Allocation` into its raw components
    /// 
    /// # Safety
    /// 
    /// The components given by this value should only be change when absolutely certain
    #[inline]
    pub unsafe fn into_raw(self) -> (NonNull<T>, Layout, MemTag) {
        (self.ptr, self.layout, self.mem_tag)
    }
    
    /// Cast the `Allocation`  to contain a value of another type
    #[inline]
    pub fn cast<U>(self) -> Allocation<U> {
        unsafe { self.with_ptr(self.ptr.as_ptr().cast()) }
    }

    
    /// Duplicate the `Allocation` 
    /// 
    /// # Safety
    /// 
    /// Duplicating the allocation is unsafe, as it could cause double deallocations
    #[inline]
    pub unsafe fn duplicate(&self) -> Self {
        self.with_ptr(self.ptr.as_ptr())
    }

    /// Create a `Allocation<T>` from a raw pointer and a layout
    /// 
    /// # Panics
    /// 
    /// Panics when the provided pointer is null
    #[inline]
    pub fn new(ptr: *mut T, layout: Layout, mem_tag: MemTag) -> Self {
        Self { ptr: unsafe { NonNull::<_>::new_unchecked(ptr) }, layout, mem_tag }
    }

    /// Create an `Allocation` from its raw components
    /// 
    /// # Safety
    /// 
    /// An allocation should only be created from the values given by [`Self::from_raw`]
    #[inline]
    pub unsafe fn from_raw(ptr: NonNull<T>, layout: Layout, mem_tag: MemTag) -> Allocation<T> {
        Self { ptr, layout, mem_tag }
    }

    /// Replace the pointer with the given pointer
    /// 
    /// # Safety
    /// 
    /// The user has to make sure that the previous allocation has been deallocated correctly and that the memory matches the current layout
    #[inline]
    pub unsafe fn with_ptr<U: ?Sized>(&self, ptr: *mut U) -> Allocation<U> {
        Allocation { ptr: unsafe { NonNull::new_unchecked(ptr) }, layout: self.layout, mem_tag: self.mem_tag }
    } 
}

impl<T> Allocation<T>
{
    /// Get the pointer as an untyped ptr
    pub fn untyped(this: Self) -> (NonNull<u8>, Layout, MemTag) {
        (this.ptr.cast::<u8>(), this.layout, this.mem_tag)
    }

    /// Create a Allocation from an untyped ptr
    /// 
    /// # Panics
    /// 
    /// Panics if the provided pointer is null
    #[inline]
    pub fn from_untyped(ptr: *mut u8, layout: Layout, mem_tag: MemTag) -> Self {
        Self { ptr: unsafe { NonNull::<_>::new_unchecked(ptr.cast::<T>()) }, layout, mem_tag }
    }

    /// Create a null allocation
    /// 
    /// # Safety
    /// 
    /// This function is meant for internal use, calling anything using it is UB
    #[inline]
    pub const unsafe fn null() -> Self {
        Self { ptr: NonNull::new_unchecked(null_mut()), layout: Layout::null(), mem_tag: MemTag::unknown(0) }
    }

    
    /// Create a null heap pointer that store an allocator id for future use
    pub unsafe fn null_alloc(alloc: UseAlloc) -> Self {
        let layout = Layout::null().with_alloc_id(alloc.get_id());
        Self { ptr: NonNull::new_unchecked(null_mut()), layout, mem_tag: MemTag::default() }
    }

    /// Create a null heap pointer that store an allocator id and memory tag for future use
    pub unsafe fn null_alloc_tag(alloc: UseAlloc, mem_tag: MemTag) -> Self {
        let layout = Layout::null().with_alloc_id(alloc.get_id());
        Self { ptr: NonNull::new_unchecked(null_mut()), layout, mem_tag }
    }
}

impl<T> Allocation<MaybeUninit<T>> {
    pub fn write(mut this: Self, value: T) -> Allocation<T> {
        this.get_mut().write(value);
        unsafe { this.assume_init() }
    }

    /// Converts to `Allocation<T>`
    /// 
    /// # Safety
    /// 
    /// It's up to the user to guarentee that the value is valid
    pub unsafe fn assume_init(self) -> Allocation<T> {
        self.with_ptr(self.ptr.as_ptr().cast())
    }
}

impl<T> Allocation<[MaybeUninit<T>]> {
    /// Converts to `Allocation<T>`
    /// 
    /// # Safety
    /// 
    /// It's up to the user to guarentee that the value is valid
    pub unsafe fn assume_init(self) -> Allocation<[T]> {
        let ptr = core::ptr::slice_from_raw_parts_mut(self.ptr.as_ptr() as *mut T, self.ptr.len());
        self.with_ptr(ptr)
    }
}

impl Allocation<dyn Any>
{
    /// Try to downcast to a concrete type, if the conversion failed, the original value will be found in the Err value
    pub fn downcast<T: Any>(self) -> Result<Allocation<T>, Allocation<dyn Any>>
    {
        if unsafe { self.ptr.as_ref().is::<T>() } {
            Ok(unsafe { self.downcast_unchecked() })
        } else {
            Err(self)
        }
    }

    /// Downcast to a concrete type, calling this on an invalid downcasted type will result in UB
    pub unsafe fn downcast_unchecked<T: Any>(self) -> Allocation<T> {
        debug_assert!(unsafe { self.ptr.as_ref().is::<T>() });
        self.cast()
    }
}

impl Allocation<dyn Any + Send>
{
    /// Try to downcast to a concrete type, if the conversion failed, the original value will be found in the Err value
    pub fn downcast<T: Any>(self) -> Result<Allocation<T>, Allocation<dyn Any>>
    {
        if unsafe { self.ptr.as_ref().is::<T>() } {
            Ok(unsafe { self.downcast_unchecked() })
        } else {
            Err(self)
        }
    }

    /// Downcast to a concrete type, calling this on an invalid downcasted type will result in UB
    pub unsafe fn downcast_unchecked<T: Any>(self) -> Allocation<T> {
        debug_assert!(unsafe { self.ptr.as_ref().is::<T>() });
        self.cast()
    }
}

impl Allocation<dyn Any + Send + Sync>
{
    /// Try to downcast to a concrete type, if the conversion failed, the original value will be found in the Err value
    pub fn downcast<T: Any>(self) -> Result<Allocation<T>, Allocation<dyn Any>>
    {
        if unsafe { self.ptr.as_ref().is::<T>() } {
            Ok(unsafe { self.downcast_unchecked() })
        } else {
            Err(self)
        }
    }

    /// Downcast to a concrete type, calling this on an invalid downcasted type will result in UB
    pub unsafe fn downcast_unchecked<T: Any>(self) -> Allocation<T> {
        debug_assert!(unsafe { self.ptr.as_ref().is::<T>() });
        self.cast()
    }
}

impl<T, U> CoerceUnsized<Allocation<U>> for Allocation<T>
where 
    T : Unsize<U> + ?Sized,
    U : ?Sized
{}