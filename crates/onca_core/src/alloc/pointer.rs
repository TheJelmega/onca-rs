use core::{any::Any, borrow::*, marker::Unsize, ops::*, ptr::*};

use super::{Layout, Allocator};

/// Representation of allocated memory
pub struct MemPointer<T: ?Sized>
{
    /// Pointer to memory
    ptr    : *mut T,
    layout : Layout,
}

impl<T: ?Sized> MemPointer<T>
{
    /// Create a `MemPointer<T>` from a raw pointer and a layout
    pub fn new(ptr: *mut T, layout: Layout) -> Self {
        Self { ptr, layout }
    }

    /// Get the header
    pub fn layout(&self) -> &Layout {
        &self.layout
    }

    /// Get the header
    pub fn layout_mut(&mut self) -> &mut Layout {
        &mut self.layout
    }

    /// Get the contained pointer
    pub fn ptr(&self) -> *const T {
        self.ptr
    }

    /// Get the contained pointer
    pub fn ptr_mut(&self) -> *mut T {
        self.ptr
    }

    /// Get a reference to the data pointed at by the `MemPointer<T>`
    pub fn get_ref(&self) -> &T {
        unsafe { &*self.ptr() }
    }

    /// Get a mutable reference to the data pointed at by the `MemPointer<T>`
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.ptr_mut() }
    }
}

impl<T> MemPointer<T>
{
    /// Create a null `MemPointer<T>`
    pub fn null() -> Self {
        Self { ptr: null_mut::<T>(), layout: Layout::null() }
    }

    pub fn is_null(&self) -> bool {
        self.ptr == null_mut()
    }

    /// Get the pointer as an untyped ptr
    pub fn untyped(this: Self) -> (*mut u8, Layout) {
        (this.ptr.cast::<u8>(), this.layout)
    }

    /// Create a MemPointer from an untyped ptr
    pub fn from_untyped(ptr: *mut u8, layout: Layout) -> Self {
        Self { ptr: ptr.cast::<T>(), layout }
    }
}

impl MemPointer<dyn Any>
{
    /// Try to downcast to a concrete type, if the conversion failed, the original value will be found in the Err value
    pub fn downcast<T: Any>(self) -> Result<MemPointer<T>, MemPointer<dyn Any>>
    {
        if unsafe { (*self.ptr).is::<T>() } {
            Ok(unsafe { self.downcast_unchecked() })
        } else {
            Err(self)
        }
    }

    /// Downcast to a concrete type, calling this on an invalid downcasted type will result in UB
    pub unsafe fn downcast_unchecked<T: Any>(self) -> MemPointer<T> {
        debug_assert!(unsafe { (*self.ptr).is::<T>() });
        MemPointer::<T>{ ptr: self.ptr.cast::<T>(), layout: self.layout }
    }
}

impl MemPointer<dyn Any + Send>
{
    /// Try to downcast to a concrete type, if the conversion failed, the original value will be found in the Err value
    pub fn downcast<T: Any>(self) -> Result<MemPointer<T>, MemPointer<dyn Any>>
    {
        if unsafe { (*self.ptr).is::<T>() } {
            Ok(unsafe { self.downcast_unchecked() })
        } else {
            Err(self)
        }
    }

    /// Downcast to a concrete type, calling this on an invalid downcasted type will result in UB
    pub unsafe fn downcast_unchecked<T: Any>(self) -> MemPointer<T> {
        debug_assert!(unsafe { (*self.ptr).is::<T>() });
        MemPointer::<T>{ ptr: self.ptr.cast::<T>(), layout: self.layout }
    }
}

impl MemPointer<dyn Any + Send + Sync>
{
    /// Try to downcast to a concrete type, if the conversion failed, the original value will be found in the Err value
    pub fn downcast<T: Any>(self) -> Result<MemPointer<T>, MemPointer<dyn Any>>
    {
        if unsafe { (*self.ptr).is::<T>() } {
            Ok(unsafe { self.downcast_unchecked() })
        } else {
            Err(self)
        }
    }

    /// Downcast to a concrete type, calling this on an invalid downcasted type will result in UB
    pub unsafe fn downcast_unchecked<T: Any>(self) -> MemPointer<T> {
        debug_assert!(unsafe { (*self.ptr).is::<T>() });
        MemPointer::<T>{ ptr: self.ptr.cast::<T>(), layout: self.layout }
    }
}

impl<T, U> CoerceUnsized<MemPointer<U>> for MemPointer<T>
    where T : Unsize<U> + ?Sized,
          U : ?Sized
{}