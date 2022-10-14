use core::{any::Any, borrow::*, marker::Unsize, ops::*, ptr::*};
use std::mem::MaybeUninit;

use crate::mem::MEMORY_MANAGER;

use super::{Layout, Allocator, UseAlloc, layout};

/// Representation of allocated memory
#[derive(Debug)]
pub struct Allocation<T: ?Sized>
{
    /// Pointer to memory
    ptr    : NonNull<T>,
    layout : Layout,
}

impl<T: ?Sized> Allocation<T>
{
    /// Create a `Allocation<T>` from a raw pointer and a layout
    /// 
    /// # Panics
    /// 
    /// Panics when the provided pointer is null
    pub fn new(ptr: *mut T, layout: Layout) -> Self {
        Self { ptr: unsafe { NonNull::<_>::new_unchecked(ptr) }, layout }
    }

    /// Get the header
    #[inline]
    pub fn layout(&self) -> &Layout {
        &self.layout
    }

    /// Get the header
    #[inline]
    pub fn layout_mut(&mut self) -> &mut Layout {
        &mut self.layout
    }

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

    /// Cast the `Allocation`  to contain a value of another type
    pub fn cast<U>(self) -> Allocation<U> {
        Allocation { ptr: self.ptr.cast(), layout: self.layout }
    }

    /// Duplicate the `Allocation` 
    /// 
    /// # Safety
    /// 
    /// Duplicating the allocation is unsafe, as it could cause double deallocations
    pub unsafe fn duplicate(&self) -> Self {
        Self::new(self.ptr.as_ptr(), self.layout)
    }

    /// Duplicate the `Allocation`  and cast it
    /// 
    /// # Safety
    /// 
    /// Duplicating the allocation is unsafe, as it could cause double deallocations
    pub unsafe fn duplicate_cast<U>(&self) -> Allocation<U> {
        self.duplicate().cast()
    }

    /// Create an `Allocation` from its raw components
    /// 
    /// # Safety
    /// 
    /// An allocation should only be created from the values given by [`Self::from_raw`]
    pub unsafe fn from_raw(ptr: NonNull<T>, layout: Layout) -> Allocation<T> {
        Self { ptr, layout }
    }

    /// Converts an `Allocation` into its raw components
    /// 
    /// # Safety
    /// 
    /// The components given by this value should only be change when absolutely certain
    pub unsafe fn into_raw(self) -> (NonNull<T>, Layout) {
        (self.ptr, self.layout)
    }
}

impl<T> Allocation<T>
{
    /// Get the pointer as an untyped ptr
    pub fn untyped(this: Self) -> (NonNull<u8>, Layout) {
        (this.ptr.cast::<u8>(), this.layout)
    }

    /// Create a Allocation from an untyped ptr
    /// 
    /// # Panics
    /// 
    /// Panics if the provided pointer is null
    pub fn from_untyped(ptr: *mut u8, layout: Layout) -> Self {
        Self { ptr: unsafe { NonNull::<_>::new_unchecked(ptr.cast::<T>()) }, layout }
    }

    /// Create a null allocation
    /// 
    /// # Safety
    /// 
    /// This function is meant for internal use, as calling anything using it is UB
    pub unsafe fn null() -> Self {
        Self { ptr: NonNull::new_unchecked(null_mut()), layout: Layout::null() }
    }

    /// Create a null heap pointer that store an allocator id for future use
    pub unsafe fn null_alloc(alloc: UseAlloc) -> Self {
        let mut layout = Layout::null();
        match alloc {
            UseAlloc::Default => layout = layout.with_alloc_id(Layout::MAX_ALLOC_ID),
            UseAlloc::Id(id) => layout = layout.with_alloc_id(id)
        }
        Self { ptr: NonNull::new_unchecked(null_mut()), layout }
    }
}

impl<T> Allocation<MaybeUninit<T>> {
    /// Converts to `Allocation<T>`
    /// 
    /// # Safety
    /// 
    /// It's up to the user to guarentee that the value is valid
    pub unsafe fn assume_init(self) -> Allocation<T> {
        Allocation { ptr: self.ptr.cast(), layout: self.layout }
    }

    pub fn write(mut this: Self, value: T) -> Allocation<T> {
        this.get_mut().write(value);
        unsafe { this.assume_init() }
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
        Allocation { ptr: unsafe{ NonNull::new_unchecked(ptr) }, layout: self.layout }
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
        Allocation::<T>{ ptr: self.ptr.cast(), layout: self.layout }
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
        Allocation::<T>{ ptr: self.ptr.cast(), layout: self.layout }
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
        Allocation::<T>{ ptr: self.ptr.cast(), layout: self.layout }
    }
}

impl<T, U> CoerceUnsized<Allocation<U>> for Allocation<T>
where 
    T : Unsize<U> + ?Sized,
    U : ?Sized
{}