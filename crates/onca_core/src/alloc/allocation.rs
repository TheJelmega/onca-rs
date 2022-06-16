use core::{any::Any, borrow::*, marker::Unsize, ops::*, ptr::*};

use super::{Layout, Allocator};

/// Representation of allocated memory
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

    /// Cast the allocation to contain a value of another type
    pub fn cast<U>(self) -> Allocation<U> {
        Allocation { ptr: self.ptr.cast(), layout: self.layout }
    }

    /// Duplicate the allocation
    /// 
    /// # Safety
    /// 
    /// Duplicating the allocation is unsafe, as it could cause double deallocations
    pub unsafe fn duplicate(&self) -> Self {
        Self::new(self.ptr.as_ptr(), self.layout)
    }

    /// Duplicate the allocation and cast it
    /// 
    /// # Safety
    /// 
    /// Duplicating the allocation is unsafe, as it could cause double deallocations
    pub unsafe fn duplicate_cast<U>(&self) -> Allocation<U> {
        self.duplicate().cast()
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
    where T : Unsize<U> + ?Sized,
          U : ?Sized
{}