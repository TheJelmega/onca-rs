use std::{alloc::{AllocError, Layout}, marker::PhantomData, ptr::{Alignment, NonNull}};
use std::alloc;

use super::{CopyRegion, StorageBase, StorageSingle, StorageSingleSliced, TypedMetadata};





/// Specialized slice handle.
/// 
/// A sliced handle may be dangling, or may be invalid. It is the responsiblilityof the user that the sliced handle is valid when neccesary
pub struct SlicedSingleHandle<T: ?Sized, H> {
    handle:   H,
    _phantom: PhantomData<T>,
}

impl<T, H: Copy> SlicedSingleHandle<T, H> {
    /// Creates a dangling handle.
    /// 
    /// Calls `handle_alloc_error` if the creation of the handle fails.
    pub const fn dangling<S>(storage: &S) -> Self where
        S: ~const StorageBase<Handle = H>
    {
        let Ok(this) = Self::try_dangling(storage) else {
            alloc::handle_alloc_error(Layout::new::<T>());
        };
        this
    }

    /// Tries to create a dangling handle
    /// 
    /// Returns `AllocError` on failure
    pub const fn try_dangling<S>(storage: &S) -> Result<Self, AllocError> where
        S: ~const StorageBase<Handle = H>
    {
        let handle = storage.dangling(Alignment::of::<T>())?;

        Ok(Self { handle, _phantom: PhantomData })
    }

    /// Try to allocate a new sliced handle, with enough space for `count` elements of `T`.
    /// 
    /// Calls `handle_alloc_error` if the creation of the handle fails.
    /// 
    /// The allocated memory is left uninitialized.
    pub const fn allocate<S>(storage: &mut S, count: usize) -> Self where
        S: ~const StorageBase<Handle = H> + ~const StorageSingle<Handle = H>
    {
        let Ok(this) = Self::try_allocate(storage, count) else {
            alloc::handle_alloc_error(Layout::new::<T>())
        };
        this
    }

    /// Try to allocate a new sliced handle, with enough space for `count` elements of `T`.
    /// 
    /// The allocated memory is left uninitialized.
    pub const fn try_allocate<S>(storage: &mut S, count: usize) -> Result<Self, AllocError> where
        S: ~const StorageBase<Handle = H> + ~const StorageSingle<Handle = H>
    {
        if core::mem::size_of::<T>() == 0 {
            let Ok(this) = Self::try_dangling(storage) else {
                alloc::handle_alloc_error(Layout::new::<T>());
            };
            return Ok(this)
        }

        let layout = Self::layout(count)?;
        let (handle, bytes) = storage.allocate(layout)?;

        debug_assert!(bytes >= layout.size());

        Ok(Self { handle, _phantom: PhantomData })
    }

    /// Try to allocate a new sliced handle, with enough space for `count` elements of `T`.
    /// 
    /// Calls `handle_alloc_error` if the creation of the handle fails.
    /// 
    /// The allocated memory is zeroed out.
    pub const fn allocate_zeroed<S>(storage: &mut S, count: usize) -> Self where
        S: ~const StorageBase<Handle = H> + ~const StorageSingle<Handle = H>
    {
        let Ok(this) = Self::try_allocate_zeroed(storage, count) else {
            alloc::handle_alloc_error(Layout::new::<T>())
        };
        this
    }

    /// Try to allocate a new sliced handle, with enough space for `count` elements of `T`.
    /// 
    /// The allocated memory is zeroed out.
    pub const fn try_allocate_zeroed<S>(storage: &mut S, count: usize) -> Result<Self, AllocError> where
        S: ~const StorageBase<Handle = H> + ~const StorageSingle<Handle = H>
    {
        if core::mem::size_of::<T>() == 0 {
            let Ok(this) = Self::try_dangling(storage) else {
                alloc::handle_alloc_error(Layout::new::<T>());
            };
            return Ok(this)
        }

        let layout = Self::layout(count)?;
        let (handle, bytes) = storage.allocate_zeroed(layout)?;

        debug_assert!(bytes >= layout.size());

        Ok(Self { handle, _phantom: PhantomData })
    }

    /// Deallocates the memory associated with the handle.
    /// 
    /// # Safety
    /// 
    /// - `self` must have been allocated by `storage`, or a shared storage.
    /// - `self` must be valid.
    /// - `self` is invalidated alongside any copy of it
    pub const unsafe fn deallocate<S>(&self, storage: &mut S) where 
        S: ~const StorageSingle<Handle = H> + ~const StorageSingleSliced<Handle = H>
    {
        // Safety
        // - `self.handle` was allocated by `storage`, or a shared storage, as per pre-conditions
        // - `self.handle` is still valid, as per pre-conditions
        let (ptr, count) = unsafe { self.resolve_raw(storage) };

        // Safety
        // - The allocation was created by the given properties, so this will be a valid layout.
        let layout = unsafe { Layout::from_size_align_unchecked(count, core::mem::align_of::<T>()) };
        
        // Safety
        // - `self.handle` was allocated by `storage`, as per pre-conditons
        // - `self.handle` is still valid, as per pre-conditions
        // - `layout` fits the block of memory associated with `self.handle`
        unsafe { storage.deallocate(self.handle, layout) };
    }

    /// Returns whether the memory asociated to `self` may not contain any element
    pub const fn is_empty<S>(&self, storage: &S) -> bool where
        S: ~const StorageSingleSliced<Handle = H>
    {
        self.len(storage) == 0
    }

    /// Return the number of element the memory area associated with `self` may contain
    pub const fn len<S>(&self, storage: &S) -> usize where
        S: ~const StorageSingleSliced<Handle = H>
    {
        unsafe { storage.resolve_size(self.handle) / core::mem::size_of::<T>() }
    }
    
    /// Create a handle from raw parts
    /// 
    /// - If `handle` is valid, and associated to a block of memory which fits a slice of `T`, then the resulting typed handle is valid.
    /// - If `handle` is invalid, then the resulting handle is invalid.
    pub const fn from_raw_parts(handle: H) -> Self {
        Self { handle, _phantom: PhantomData }
    }

    /// Decompose a pointer to its raw handle
    pub const fn to_raw_parts(self) -> H {
        self.handle
    }

    /// Resolves the handle to a slice
    /// 
    /// # Safety
    /// 
    /// - `self` must have been allocated by `storage`, or a shared storage.
    /// - `self` must still be valid.
    /// - `self` must be associated to a block of memory containing a valid instance of `T`.
    /// - No access through a mutable slice to this of `T` must overlap with accesses through the result.
    /// - The slice is only guaranteed to be valid as long as `self` is valid.
    /// - The slice is only guaranteed to be valid as long as pointers resolved from `self` are not invalidated.
    ///   Most notably, unless `storage` implements `StoreStable`, any method call on `store` including other `resolve` calls, may invalidate the reference.
    pub const unsafe fn resolve<'a, S>(&self, storage: &'a S) -> &'a [T] where
        S: ~const StorageSingleSliced<Handle = H>
    {
        let (ptr, count) = storage.resolve_sliced(self.handle);

        let count = count / core::mem::size_of::<T>();

        core::slice::from_raw_parts(ptr.cast().as_ptr(), count)
    }

    /// Resolves the handle to a slice
    /// 
    /// # Safety
    /// 
    /// - `self` must have been allocated by `storage`, or a shared storage.
    /// - `self` must still be valid.
    /// - `self` must be associated to a block of memory containing a valid instance of `T`.
    /// - No access through a mutable slice to this of `T` must overlap with accesses through the result.
    /// - The slice is only guaranteed to be valid as long as `self` is valid.
    /// - The slice is only guaranteed to be valid as long as pointers resolved from `self` are not invalidated.
    ///   Most notably, unless `storage` implements `StoreStable`, any method call on `store` including other `resolve` calls, may invalidate the reference.
    pub const unsafe fn resolve_mut<'a, S>(&self, storage: &'a S) -> &'a mut [T] where
        S: ~const StorageSingleSliced<Handle = H>
    {
        let (ptr, count) = storage.resolve_sliced(self.handle);

        let count = count / core::mem::size_of::<T>();

        core::slice::from_raw_parts_mut(ptr.cast().as_ptr(), count)
    }

    /// Resolved the handle to a pointer.
    /// 
    /// # Safety
    /// 
    /// - `self` must have been allocated by `storage`, or a shared storage.
    /// - `self` must still be valid
    /// - The pointer is only guaranteed to be valid as long as `self` is valid.
    /// - The pointer is only guaranteed to be valid as long as pointers resolved from `self` are not invalidated.
    ///   Most notably, unless `storage` implements `StorageStable`, any method call on `storage`, including other `resolve` calls, may invalidate the pointer.
    pub const unsafe fn resolve_raw<S>(&self, storage: &S) -> (NonNull<u8>, usize) where
        S: ~const StorageSingleSliced<Handle = H>
    {
        // Safety
        // - `self.handle` was allocated by `storage` or a shared storage, as per pre-conditions.
        // - `self.handle` is still valid, as per pre-conditions
        let (ptr, size) = storage.resolve_sliced(self.handle);

        (ptr, size / core::mem::size_of::<T>())
    }

    /// Resolved the handle to its size.
    /// 
    /// # Safety
    /// 
    /// - `self` must have been allocated by `storage`, or a shared storage.
    /// - `self` must still be valid
    /// - The size is only guaranteed to be valid as long as `self` is valid.
    /// - The size is only guaranteed to be valid as long as pointers resolved from `self` are not invalidated.
    ///   Most notably, unless `storage` implements `StorageStable`, any method call on `storage`, including other `resolve` calls, may invalidate the poisizenter.
    pub const unsafe fn resolve_size<S>(&self, storage: &S) -> usize where
        S: ~const StorageSingleSliced<Handle = H>
    {
        // Safety
        // - `self.handle` was allocated by `storage` or a shared storage, as per pre-conditions.
        // - `self.handle` is still valid, as per pre-conditions
        let size = storage.resolve_size(self.handle);

        size / core::mem::size_of::<T>()
    }

    /// Grows the block of memory associated with the handle.
    /// 
    /// On success, all the copies of the handle are invalidated, and the extra memory is left uninitialized.
    /// Calls `handle_alloc_error` on failure.
    /// 
    /// # Safety
    /// 
    /// - `self` must have been allocated by `storage`, or a shared storage.
    /// - `self` must still be valid.
    /// - `new_size` must be greater than or euqal to `self.len()`.
    pub const unsafe fn grow<S>(&mut self, new_size: usize, storage: &mut S) where
        S: ~const StorageSingle<Handle = H> + ~const StorageSingleSliced<Handle = H>
    {
        let result = self.try_grow(new_size, storage);
        if result.is_err() {
            alloc::handle_alloc_error(Layout::new::<T>());
        }
    }
    
    /// Tries to grow the block of memory associated with the handle.
    /// 
    /// On success, all the copies of the handle are invalidated, and the extra memory is left uninitialized.
    /// On failure, an error is returned.
    /// 
    /// # Safety
    /// 
    /// - `self` must have been allocated by `storage`, or a shared storage.
    /// - `self` must still be valid.
    /// - `new_size` must be greater than or euqal to `self.len()`.
    pub const unsafe fn try_grow<S>(&mut self, new_size: usize, storage: &mut S) -> Result<(), AllocError> where
        S: ~const StorageSingle<Handle = H> + ~const StorageSingleSliced<Handle = H>
    {
        debug_assert!(new_size >= self.len(storage));

        let old_layout = Self::layout(self.len(storage))?;
        let new_layout = Self::layout(new_size)?;

        // Safety:
        // - `self.handle` was allocated by `storage` or a shared storage, as per pre-conditons
        // - `self.handle` is still valid, as per pre-conditions
        // - `old_layout` fits the block of memory associated to `self.handle`, by construction.
        // - `new_layout`'s size is greater than or equal to the size of `old_layout`, as per pre-conditions.
        let (handle, bytes) = unsafe { storage.grow(self.handle, old_layout, new_layout) }?;

        debug_assert!(bytes >= new_layout.size());

        self.handle = handle;
        Ok(())
    }

    /// Grows the block of memory associated with the handle.
    /// 
    /// On success, all the copies of the handle are invalidated, and the extra memory is zeroed out.
    /// Calls `handle_alloc_error` on failure.
    /// 
    /// # Safety
    /// 
    /// - `self` must have been allocated by `storage`, or a shared storage.
    /// - `self` must still be valid.
    /// - `new_size` must be greater than or euqal to `self.len()`.
    pub const unsafe fn grow_zeroed<S>(&mut self, new_size: usize, storage: &mut S) where
        S: ~const StorageSingle<Handle = H> + ~const StorageSingleSliced<Handle = H>
    {
        let result = self.try_grow_zeroed(new_size, storage);
        if result.is_err() {
            alloc::handle_alloc_error(Layout::new::<T>());
        }
    }
    
    /// Tries to grow the block of memory associated with the handle.
    /// 
    /// On success, all the copies of the handle are invalidated, and the extra memory is zeroed out.
    /// On failure, an error is returned.
    /// 
    /// # Safety
    /// 
    /// - `self` must have been allocated by `storage`, or a shared storage.
    /// - `self` must still be valid.
    /// - `new_size` must be greater than or euqal to `self.len()`.
    pub const unsafe fn try_grow_zeroed<S>(&mut self, new_size: usize, storage: &mut S) -> Result<(), AllocError> where
        S: ~const StorageSingle<Handle = H> + ~const StorageSingleSliced<Handle = H>
    {
        debug_assert!(new_size >= self.len(storage));

        let old_layout = Self::layout(self.len(storage))?;
        let new_layout = Self::layout(new_size)?;

        // Safety:
        // - `self.handle` was allocated by `storage` or a shared storage, as per pre-conditons
        // - `self.handle` is still valid, as per pre-conditions
        // - `old_layout` fits the block of memory associated to `self.handle`, by construction.
        // - `new_layout`'s size is greater than or equal to the size of `old_layout`, as per pre-conditions.
        let (handle, bytes) = unsafe { storage.grow_zeroed(self.handle, old_layout, new_layout) }?;

        debug_assert!(bytes >= new_layout.size());

        self.handle = handle;
        Ok(())
    }

    /// Behaves life `grow`, but also allows specifying a region of elements that should be copied to the new memory.
    /// 
    /// # Safety
    /// 
    /// See `grow`, with the additional contraints of:
    /// 
    /// - `region` must be defined in terms of elements.
    /// - `region` must adhere to its guarantees.
    pub const unsafe fn grow_region<S>(&mut self, new_size: usize, region: CopyRegion, storage: &mut S) where
        S: ~const StorageSingle<Handle = H> + ~const StorageSingleSliced<Handle = H>
    {
        let result = self.try_grow_region(new_size, region, storage);
        if result.is_err() {
            alloc::handle_alloc_error(Layout::new::<T>());
        }
    }
    
    /// Behaves life `try_grow`, but also allows specifying a region of elements that should be copied to the new memory.
    /// 
    /// # Safety
    /// 
    /// See `try_grow`, with the additional contraints of:
    /// 
    /// - `region` must be defined in terms of elements.
    /// - `region` must adhere to its guarantees.
    pub const unsafe fn try_grow_region<S>(&mut self, new_size: usize, region: CopyRegion, storage: &mut S) -> Result<(), AllocError> where
        S: ~const StorageSingle<Handle = H> + ~const StorageSingleSliced<Handle = H>
    {
        let len = self.len(storage);
        debug_assert!(new_size >= len);
        debug_assert!(region.src_offset < len);
        debug_assert!(region.dst_offset < new_size);
        debug_assert!(region.size <= len);
        debug_assert!(region.src_offset + region.size <= len);
        debug_assert!(region.dst_offset + region.size <= new_size);

        let old_layout = Self::layout(len)?;
        let new_layout = Self::layout(new_size)?;

        // Safety:
        // - `region` is defined in terms of element, as per pre-conditions.
        // - `region`'s members can never overflow `isize`, as per pre-conditions
        let region = region.to_typed::<T>();

        // Safety:
        // - `self.handle` was allocated by `storage` or a shared storage, as per pre-conditons
        // - `self.handle` is still valid, as per pre-conditions
        // - `old_layout` fits the block of memory associated to `self.handle`, by construction.
        // - `new_layout`'s size is greater than or equal to the size of `old_layout`, as per pre-conditions.
        let (handle, bytes) = unsafe { storage.grow_region(self.handle, old_layout, new_layout, region) }?;

        debug_assert!(bytes >= new_layout.size());

        self.handle = handle;
        Ok(())
    }

    /// Behaves life `grow_zeroed`, but also allows specifying a region of elements that should be copied to the new memory.
    /// 
    /// # Safety
    /// 
    /// See `grow_zeroed`, with the additional contraints of:
    /// 
    /// - `region` must be defined in terms of elements.
    /// - `region` must adhere to its guarantees.
    pub const unsafe fn grow_region_zeroed<S>(&mut self, new_size: usize, region: CopyRegion, storage: &mut S) where
        S: ~const StorageSingle<Handle = H> + ~const StorageSingleSliced<Handle = H>
    {
        let result = self.try_grow_region_zeroed(new_size, region, storage);
        if result.is_err() {
            alloc::handle_alloc_error(Layout::new::<T>());
        }
    }
    
    /// Behaves life `try_grow_zeroed`, but also allows specifying a region of elements that should be copied to the new memory.
    /// 
    /// # Safety
    /// 
    /// See `try_grow_zeroed`, with the additional contraints of:
    /// 
    /// - `region` must be defined in terms of elements.
    /// - `region` must adhere to its guarantees.
    pub const unsafe fn try_grow_region_zeroed<S>(&mut self, new_size: usize, region: CopyRegion, storage: &mut S) -> Result<(), AllocError> where
        S: ~const StorageSingle<Handle = H> + ~const StorageSingleSliced<Handle = H>
    {
        debug_assert!(new_size >= self.len(storage));

        let old_layout = Self::layout(self.len(storage))?;
        let new_layout = Self::layout(new_size)?;

        // Safety:
        // - `region` is defined in terms of element, as per pre-conditions.
        // - `region`'s members can never overflow `isize`, as per pre-conditions
        let region = region.to_typed::<T>();

        // Safety:
        // - `self.handle` was allocated by `storage` or a shared storage, as per pre-conditons
        // - `self.handle` is still valid, as per pre-conditions
        // - `old_layout` fits the block of memory associated to `self.handle`, by construction.
        // - `new_layout`'s size is greater than or equal to the size of `old_layout`, as per pre-conditions.
        let (handle, bytes) = unsafe { storage.grow_region_zeroed(self.handle, old_layout, new_layout, region) }?;

        debug_assert!(bytes >= new_layout.size());

        self.handle = handle;
        Ok(())
    }

    /// Shrink the block of memory with the handle.
    /// 
    /// On success, all the copies of the handle are invalidated.
    /// Calls `handle_alloc_error` on failure.
    /// 
    /// # Safety
    /// 
    /// - `self` must have been allocated by `storage`, or a shared storage.
    /// - `self` must still be valid.
    /// - `new_size` must be less than or equal to `self.len()`
    pub const unsafe fn shrink<S>(&mut self, new_size: usize, storage: &mut S) where
        S: ~const StorageSingle<Handle = H> + ~const StorageSingleSliced<Handle = H>
    {
        let result = unsafe { self.try_shrink(new_size, storage) };

        if result.is_err() {
            alloc::handle_alloc_error(Layout::new::<T>());
        }
    }

    /// Tries to shrink the block of memory with the handle.
    /// 
    /// On success, all the copies of the handle are invalidated.
    /// On failure, an error is returned.
    /// 
    /// # Safety
    /// 
    /// - `self` must have been allocated by `storage`, or a shared storage.
    /// - `self` must still be valid.
    /// - `new_size` must be less than or equal to `self.len()`
    pub const unsafe fn try_shrink<S>(&mut self, new_size: usize, storage: &mut S) -> Result<(), AllocError> where
        S: ~const StorageSingle<Handle = H> + ~const StorageSingleSliced<Handle = H>
    {
        debug_assert!(new_size <= self.len(storage));

        if core::mem::size_of::<T>() == 0 {
            return Ok(());
        }

        let Ok(old_layout) = Self::layout(self.len(storage)) else { return Err(AllocError) };
        let Ok(new_layout) = Self::layout(new_size) else { return Err(AllocError) };

        let result = unsafe { storage.shrink(self.handle, old_layout, new_layout) };

        let Ok((handle, bytes)) = result else { return Err(AllocError); };

        debug_assert!(bytes >= new_layout.size());

        self.handle = handle;

        Ok(())
    }

    /// Behaves life `shrink`, but also allows specifying a number of elements that should be copied to the new memory.
    /// 
    /// # Safety
    /// 
    /// See `shrink`, with the additional contraints of:
    /// 
    /// - `copy_size` must be less than or equal to `new_size`.
    pub const unsafe fn shrink_region<S>(&mut self, new_size: usize, region: CopyRegion, storage: &mut S) where
        S: ~const StorageSingle<Handle = H> + ~const StorageSingleSliced<Handle = H>
    {
        let result = unsafe { self.try_shrink_region(new_size, region, storage) };

        if result.is_err() {
            alloc::handle_alloc_error(Layout::new::<T>());
        }
    }

    /// Behaves life `try_shrink`, but also allows specifying a number of elements that should be copied to the new memory.
    /// 
    /// # Safety
    /// 
    /// See `try_shrink`, with the additional contraints of:
    /// 
    /// - `copy_size` must be less than or equal to `new_size`.
    pub const unsafe fn try_shrink_region<S>(&mut self, new_size: usize, region: CopyRegion, storage: &mut S) -> Result<(), AllocError> where
        S: ~const StorageSingle<Handle = H> + ~const StorageSingleSliced<Handle = H>
    {
        let len = self.len(storage);
        debug_assert!(new_size <= len);
        debug_assert!(region.src_offset < len);
        debug_assert!(region.dst_offset < new_size);
        debug_assert!(region.size <= len);
        debug_assert!(region.src_offset + region.size <= len);
        debug_assert!(region.dst_offset + region.size <= new_size);

        if core::mem::size_of::<T>() == 0 {
            return Ok(());
        }

        let Ok(old_layout) = Self::layout(len) else { return Err(AllocError) };
        let Ok(new_layout) = Self::layout(new_size) else { return Err(AllocError) };

        // Safety:
        // - `region` is defined in terms of element, as per pre-conditions.
        // - `region`'s members can never overflow `isize`, as per pre-conditions
        let region = region.to_typed::<T>();

        let result = unsafe { storage.shrink_region(self.handle, old_layout, new_layout, region) };

        let Ok((handle, bytes)) = result else { return Err(AllocError); };

        debug_assert!(bytes >= new_layout.size());

        self.handle = handle;

        Ok(())
    }

    /// Behaves life `shrink`, but also allows specifying a number of elements that should be copied to the new memory.
    /// 
    /// On success, any memory that is not copied is zeroed out.
    /// 
    /// # Safety
    /// 
    /// See `shrink`, with the additional contraints of:
    /// 
    /// - `copy_size` must be less than or equal to `new_size`.
    pub const unsafe fn shrink_region_zeroed<S>(&mut self, new_size: usize, region: CopyRegion, storage: &mut S) where
        S: ~const StorageSingle<Handle = H> + ~const StorageSingleSliced<Handle = H>
    {
        let result = unsafe { self.try_shrink_region_zeroed(new_size, region, storage) };

        if result.is_err() {
            alloc::handle_alloc_error(Layout::new::<T>());
        }
    }

    /// Behaves life `try_shrink`, but also allows specifying a number of elements that should be copied to the new memory.
    /// 
    /// On success, any memory that is not copied is zeroed out.
    /// 
    /// # Safety
    /// 
    /// See `try_shrink`, with the additional contraints of:
    /// 
    /// - `copy_size` must be less than or equal to `new_size`.
    pub const unsafe fn try_shrink_region_zeroed<S>(&mut self, new_size: usize, region: CopyRegion, storage: &mut S) -> Result<(), AllocError> where
        S: ~const StorageSingle<Handle = H> + ~const StorageSingleSliced<Handle = H>
    {
        let len = self.len(storage);
        debug_assert!(new_size <= len);
        debug_assert!(region.src_offset < len);
        debug_assert!(region.dst_offset < new_size);
        debug_assert!(region.size <= len);
        debug_assert!(region.src_offset + region.size <= len);
        debug_assert!(region.dst_offset + region.size <= new_size);

        if core::mem::size_of::<T>() == 0 {
            return Ok(());
        }

        let Ok(old_layout) = Self::layout(len) else { return Err(AllocError) };
        let Ok(new_layout) = Self::layout(new_size) else { return Err(AllocError) };

        // Safety:
        // - `region` is defined in terms of element, as per pre-conditions.
        // - `region`'s members can never overflow `isize`, as per pre-conditions
        let region = region.to_typed::<T>();

        let result = unsafe { storage.shrink_region_zeroed(self.handle, old_layout, new_layout, region) };

        let Ok((handle, bytes)) = result else { return Err(AllocError); };

        debug_assert!(bytes >= new_layout.size());

        self.handle = handle;
        Ok(())
    }

    /// Cast the handle to another type.
    /// 
    /// # Safety
    /// 
    /// - The user must ensure that the data stays valid when converting from `T` to `U`.
    pub const fn cast<U>(self) -> SlicedSingleHandle<U, H> {
        SlicedSingleHandle::<U, H>{ handle: self.handle, _phantom: PhantomData }
    }
}



impl<T: ?Sized, H: Copy> Clone for SlicedSingleHandle<T, H> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: ?Sized, H: Copy> Copy for SlicedSingleHandle<T, H> {}

impl<T, H> SlicedSingleHandle<T, H> {
    const fn layout(size: usize) -> Result<Layout, AllocError> {
        match Layout::array::<T>(size) {
            Ok(layout) => Ok(layout),
            Err(_) => Err(AllocError)
        }
    }
}