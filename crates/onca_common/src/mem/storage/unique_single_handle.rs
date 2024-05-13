use core::alloc::{Layout, AllocError};

use super::{CopyRegion, StorageSingle, StorageBase, TypedSingleHandle, TypedMetadata};

use std::{alloc, ptr::NonNull};

/// A typed, unique handle.
pub struct UniqueSingleHandle<T: ?Sized, H>(TypedSingleHandle<T, H>);

impl<T, H: Copy> UniqueSingleHandle<T, H> {
    /// Create a dangling handle.
    /// 
    /// Calls `handle_alloc_error` on allocation failure
    pub const fn dangling<S>(storage: &S) -> Self where 
        S: ~const StorageBase<Handle = H>
    {
        let Ok(handle) = Self::try_dangling(storage) else {
            alloc::handle_alloc_error(Layout::new::<T>())
        };
        handle
    }

    /// Try to create a dangling handle.
    /// 
    /// Returns an error on allocation failure.
    pub const fn try_dangling<S>(storage: &S) -> Result<Self, AllocError> where
        S: ~const StorageBase<Handle = H>
    {
        let handle = TypedSingleHandle::try_dangling(storage)?;
        Ok(Self(handle))
    }

    /// Create a new handle, pointing at a `T`.
    /// 
    /// Calls `handle_alloc_error` on failure
    pub fn new<S>(value: T, storage: &mut S) -> Self where
        S: StorageSingle<Handle = H>
    {
        Self(TypedSingleHandle::new(value, storage))
    }

    /// Try to create a new handle, pointing at a `T`.
    pub fn try_new<S>(value: T, storage: &mut S) -> Result<Self, AllocError> where
        S: StorageSingle<Handle = H>
    {
        TypedSingleHandle::try_new(value, storage).map(Self)
    }

    /// Allocate a new handle, with enough space for a `T`.
    /// 
    /// The allocated memory is left uninitialized.
    /// 
    /// Calls `handle_alloc_error` on failure.
    pub const fn allocate<S>(storage: &mut S) -> Self where
        S: ~const StorageSingle<Handle = H>
    {
        Self(TypedSingleHandle::allocate(storage))
    }

    /// Try to allocate a new handle, with enough space for `T`.
    /// 
    /// The allocated memory is left uninitialized.
    pub const fn try_allocate<S>(storage: &mut S) -> Result<Self, AllocError> where
        S: ~const StorageSingle<Handle = H>
    {
        let handle = TypedSingleHandle::try_allocate(storage)?;
        Ok(Self(handle))
    }

    /// Allocate a new handle, with enough space for a `T`.
    /// 
    /// The allocated memory is zeroed out.
    /// 
    /// Calls `handle_alloc_error` on failure.
    pub const fn allocate_zeroed<S>(storage: &mut S) -> Self where
        S: ~const StorageSingle<Handle = H>
    {
        let Ok(this) = Self::try_allocate_zeroed(storage) else {
            alloc::handle_alloc_error(Layout::new::<T>());
        };
        this
    }

    /// Try to allocate a new handle, with enough space for `T`.
    /// 
    /// The allocated memory is zeroed out.
    pub const fn try_allocate_zeroed<S>(storage: &mut S) -> Result<Self, AllocError> where
        S: ~const StorageSingle<Handle = H>
    {
        let handle = TypedSingleHandle::try_allocate_zeroed(storage)?;
        Ok(Self(handle))
    }
}

impl<T: ?Sized, H: Copy> UniqueSingleHandle<T, H> {
    /// Creates a handle from raw parts
    /// 
    /// - If `handle` is valid, and associated to a block of memory that fits an instance of `T`, then ther resulting unique handle is valid.
    /// - If `handle` is invalid, then the resulting unique handle is invalid.
    /// - If `handle` is valid and `metadata` does not fit the block of memory associated with it, the the resulting typed handle is invalid.
    /// 
    /// # Safety
    /// 
    /// - No copy of `handle` must be used henceforth.
    pub const unsafe fn from_raw_parts(handle: H, metadata: TypedMetadata<T>) -> Self {
        Self(TypedSingleHandle::from_raw_parts(handle, metadata))
    }

    /// Decompose a (possibly wide) pointer into its handle and metadata components.
    pub const unsafe fn to_raw_parts(self) -> (H, TypedMetadata<T>) {
        self.0.to_raw_parts()
    }

    /// Deallocates the memory associated with the handle.
    /// 
    /// # Safety
    /// 
    /// - `self` must have been allocated by `storage`, or a shared storage.
    /// - `self` must still be valid. 
    pub const unsafe fn deallocate<S>(self, storage: &mut S) where
        S: ~const StorageSingle<Handle = H>
    {
        self.0.deallocate(storage)
    }

    /// Resolved the handle to a reference, borrowing the handle.
    /// 
    /// # Safety
    /// 
    /// - `self` must have been allocated by `storage`, or a shared storage.
    /// - `self` must still be valid.
    /// - `self` must be associated to a block of memory containing a valid instance of `T`.
    /// - The reference is only guaranteed to be valid as long as `self` is valid.
    /// - The reference is only guaranteed to be valid as long as pointers resovled from `self` are not invalidated.
    ///   Most notably, unless `storage` implements `StoreStable`, any method call on `store` including other `resolve` calls, may invalidate the reference.
    pub const unsafe fn resolve<'a, S>(&'a self, storage: &'a S) -> &'a T where
        S: ~const StorageSingle<Handle = H>
    {
        // Safety
        // - `self.handle` was allocated by `storage`, or a shared storage, as per pre-conditions.
        // - `self.handle` is still valid, as per pre-conditions.
        // - `self.handle` is associated with a block of memory containing a live instance of `T`, as per pre-condition.
        // - The resulting reference borrows `self` immutably, guaranteeing that no mutable reference exists, not can be created during its lifetime.
        // - The resulting reference borrows `storate` immutable, guaranteeing it won't invalidated by moving or destroying `storage`, though it may still be invalidated by allocating.
        self.0.resolve(storage)
    }

    /// Resolved the handle to a reference, borrowing the handle.
    /// 
    /// # Safety
    /// 
    /// - `self` must have been allocated by `storage`, or a shared storage.
    /// - `self` must still be valid.
    /// - `self` must be associated to a block of memory containing a valid instance of `T`.
    /// - The reference is only guaranteed to be valid as long as `self` is valid.
    /// - The reference is only guaranteed to be valid as long as pointers resovled from `self` are not invalidated.
    ///   Most notably, unless `storage` implements `StoreStable`, any method call on `store` including other `resolve` calls, may invalidate the reference.
    pub const unsafe fn resolve_mut<'a, S>(&'a mut self, storage: &'a mut S) -> &'a mut T where 
        S: ~const StorageSingle<Handle = H>
    {
        // Safety
        // - `self.handle` was allocated by `storage`, or a shared storage, as per pre-conditions.
        // - `self.handle` is still valid, as per pre-conditions.
        // - `self.handle` is associated with a block of memory containing a live instance of `T`, as per pre-condition.
        // - The resulting reference borrows `self` immutably, guaranteeing that no mutable reference exists, not can be created during its lifetime.
        // - The resulting reference borrows `storate` immutable, guaranteeing it won't invalidated by moving or destroying `storage`, though it may still be invalidated by allocating.
        self.0.resolve_mut(storage)
    }

    /// Resolved the handle to non-null pointer.
    /// 
    /// # Safety
    /// 
    /// - `self` must have been allocated by `storage`, or a shared storage.
    /// - `self` must still be valid.
    /// - The pointer is only guaranteed to be valid as long as `self` is valid.
    /// - The poitner is only guaranteed to be valid as long as pointers resolved from `self` are not invalidated.
    ///   Most notably, unless `storage` implements `StorageStable`, any method call on `storage`, indlucing other `resolve` calls, may invalidte the pointer.
    pub const unsafe fn resolve_raw<S>(&self, storage: &S) -> NonNull<T> where
        S: ~const StorageSingle<Handle = H>
    {
        // Safety:
        // - `self.handle` was allcoated by `storage`, or a shared storage, as per pre-conditions.
        // - `self.handle` is still valid, as per pre-conditions
        self.0.resolve_raw(storage)
    }
}

impl <T, H: Copy> UniqueSingleHandle<[T], H> {
    /// Create a dangling handle.
    /// 
    /// Call `handle_alloc_error` on allocation failure.
    pub const fn dangling_slice<S>(storage: &S) -> Self where
        S: ~const StorageBase<Handle = H>
    {
        let Ok(this) = Self::try_dangling_slice(storage) else {
            alloc::handle_alloc_error(Layout::new::<T>());
        };
        this
    }

    /// Tries to create a dangling handle
    /// 
    /// Returns an error on allocation failure
    pub const fn try_dangling_slice<S>(storage: &S) -> Result<Self, AllocError> where
        S: ~const StorageBase<Handle = H>
    {
        let handle = TypedSingleHandle::try_dangling_slice(storage)?;
        Ok(Self(handle))
    }

    /// Allocate a new handle, with enough space for `size` elements of type `T`.
    /// 
    /// The allocated memory is left uninitialized.
    /// 
    /// Calls `handle_alloc_error` on allocation failure.
    pub const fn allocate_sliced<S>(size: usize, storage: &mut S) -> Self where
        S:~const StorageBase<Handle = H> + ~const StorageSingle<Handle = H>
    {
        Self(TypedSingleHandle::allocate_sliced(size, storage))
    }

    /// Try to allocate a new handle, with enough space for `size` elements of type `T`.
    /// 
    /// The allocated memory is left uninitialized.
    pub const fn try_allocate_sliced<S>(size: usize, storage: &mut S) -> Result<Self, AllocError> where
        S:~const StorageBase<Handle = H> + ~const StorageSingle<Handle = H>
    {
        let handle = TypedSingleHandle::try_allocate_sliced(size, storage)?;
        Ok(Self(handle))
    }

    /// Allocate a new handle, with enough space for `size` elements of type `T`.
    /// 
    /// The allocated memory is zeroed out.
    /// 
    /// Calls `handle_alloc_error` on allocation failure.
    pub const fn allocate_zeroed_sliced<S>(size: usize, storage: &mut S) -> Self where
        S:~const StorageBase<Handle = H> + ~const StorageSingle<Handle = H>
    {
        Self(TypedSingleHandle::allocate_zeroed_sliced(size, storage))
    }

    /// Try to allocate a new handle, with enough space for `size` elements of type `T`.
    /// 
    /// The allocated memory is zeroed out.
    pub const fn try_allocate_zeroed_sliced<S>(size: usize, storage: &mut S) -> Result<Self, AllocError> where
        S:~const StorageBase<Handle = H> + ~const StorageSingle<Handle = H>
    {
        let handle = TypedSingleHandle::try_allocate_zeroed_sliced(size, storage)?;
        Ok(Self(handle))
    }

    /// Returns whether the memory area associated to `self` may not contain any element.
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the nmber of elements the memory area associated to `self` may contain.
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    /// Grows the block of memory associated with the handle.
    /// 
    /// On success, the extra memory is left uninitialized.
    /// Calls `handle_alloc_error` on failure.
    /// 
    /// # Safety
    /// 
    /// - `self` must have been allocated by `storage`, or a shared storage.
    /// - `self` must still be valid.
    /// - `new_size` must be greater than or euqal to `self.len()`.
    pub const unsafe fn grow<S>(&mut self, new_size: usize, storage: &mut S) where
        S: ~const StorageSingle<Handle = H>
    {
        // Safety
        // - `self.0` has been allocted by `store` or a shared storage, as per pre-conditions.
        // - `self.0` is still valid, as per pre-conditions
        // - `new_size` is greater than or equal to `self.0.len()`, as per pre-conditions.
        TypedSingleHandle::grow(&mut self.0, new_size, storage)
    }
    
    /// Tries to grow the block of memory associated with the handle.
    /// 
    /// On success, the extra memory is left uninitialized.
    /// On failure, an error is returned.
    /// 
    /// # Safety
    /// 
    /// - `self` must have been allocated by `storage`, or a shared storage.
    /// - `self` must still be valid.
    /// - `new_size` must be greater than or euqal to `self.len()`.
    pub const unsafe fn try_grow<S>(&mut self, new_size: usize, storage: &mut S) -> Result<(), AllocError> where
        S: ~const StorageSingle<Handle = H>
    {
        // Safety
        // - `self.0` has been allocted by `store` or a shared storage, as per pre-conditions.
        // - `self.0` is still valid, as per pre-conditions
        // - `new_size` is greater than or equal to `self.0.len()`, as per pre-conditions.
        TypedSingleHandle::try_grow(&mut self.0, new_size, storage)
    }

    
    /// Grows the block of memory associated with the handle.
    /// 
    /// On success, the extra memory is zeroed out.
    /// Calls `handle_alloc_error` on failure.
    /// 
    /// # Safety
    /// 
    /// - `self` must have been allocated by `storage`, or a shared storage.
    /// - `self` must still be valid.
    /// - `new_size` must be greater than or euqal to `self.len()`.
    pub const unsafe fn grow_zeroed<S>(&mut self, new_size: usize, storage: &mut S) where
        S: ~const StorageSingle<Handle = H>
    {
        // Safety
        // - `self.0` has been allocted by `store` or a shared storage, as per pre-conditions.
        // - `self.0` is still valid, as per pre-conditions
        // - `new_size` is greater than or equal to `self.0.len()`, as per pre-conditions.
        TypedSingleHandle::grow_zeroed(&mut self.0, new_size, storage)
    }
    
    /// Tries to grow the block of memory associated with the handle.
    /// 
    /// On success, the extra memory is zeroed out.
    /// On failure, an error is returned.
    /// 
    /// # Safety
    /// 
    /// - `self` must have been allocated by `storage`, or a shared storage.
    /// - `self` must still be valid.
    /// - `new_size` must be greater than or euqal to `self.len()`.
    pub const unsafe fn try_grow_zeroed<S>(&mut self, new_size: usize, storage: &mut S) -> Result<(), AllocError> where
        S: ~const StorageSingle<Handle = H>
    {
        // Safety
        // - `self.0` has been allocted by `store` or a shared storage, as per pre-conditions.
        // - `self.0` is still valid, as per pre-conditions
        // - `new_size` is greater than or equal to `self.0.len()`, as per pre-conditions.
        TypedSingleHandle::try_grow(&mut self.0, new_size, storage)
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
        S: ~const StorageSingle<Handle = H>
    {
        // Safety
        // - `self.0` has been allocted by `store` or a shared storage, as per pre-conditions.
        // - `self.0` is still valid, as per pre-conditions
        // - `new_size` is greater than or equal to `self.0.len()`, as per pre-conditions.
        // - `region` is defined in elements, as per pre-conditions.
        // - `region` adheres to its guarantees, as per pre-conditions.
        TypedSingleHandle::grow_region(&mut self.0, new_size, region, storage)
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
        S: ~const StorageSingle<Handle = H>
    {
        // Safety
        // - `self.0` has been allocted by `store` or a shared storage, as per pre-conditions.
        // - `self.0` is still valid, as per pre-conditions
        // - `new_size` is greater than or equal to `self.0.len()`, as per pre-conditions.
        // - `region` is defined in elements, as per pre-conditions.
        // - `region` adheres to its guarantees, as per pre-conditions.
        TypedSingleHandle::try_grow_region(&mut self.0, new_size, region, storage)
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
        S: ~const StorageSingle<Handle = H>
    {
        // Safety
        // - `self.0` has been allocted by `store` or a shared storage, as per pre-conditions.
        // - `self.0` is still valid, as per pre-conditions
        // - `new_size` is greater than or equal to `self.0.len()`, as per pre-conditions.
        // - `region` is defined in elements, as per pre-conditions.
        // - `region` adheres to its guarantees, as per pre-conditions.
        TypedSingleHandle::grow_region_zeroed(&mut self.0, new_size, region, storage)
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
        S: ~const StorageSingle<Handle = H>
    {
        // Safety
        // - `self.0` has been allocted by `store` or a shared storage, as per pre-conditions.
        // - `self.0` is still valid, as per pre-conditions
        // - `new_size` is greater than or equal to `self.0.len()`, as per pre-conditions.
        // - `region` is defined in elements, as per pre-conditions.
        // - `region` adheres to its guarantees, as per pre-conditions.
        TypedSingleHandle::try_grow_region_zeroed(&mut self.0, new_size, region, storage)
    }

    /// Shrink the block of memory with the handle.
    /// 
    /// Calls `handle_alloc_error` on failure.
    /// 
    /// # Safety
    /// 
    /// - `self` must have been allocated by `storage`, or a shared storage.
    /// - `self` must still be valid.
    /// - `new_size` must be less than or equal to `self.len()`
    pub const unsafe fn shrink<S>(&mut self, new_size: usize, storage: &mut S) where
        S: ~const StorageSingle<Handle = H>
    {
        // Safety
        // - `self.0` has been allocted by `store` or a shared storage, as per pre-conditions.
        // - `self.0` is still valid, as per pre-conditions
        // - `new_size` is greater than or equal to `self.0.len()`, as per pre-conditions.
        TypedSingleHandle::shrink(&mut self.0, new_size, storage)
    }

    /// Tries to shrink the block of memory with the handle.
    /// 
    /// On failure, an error is returned.
    /// 
    /// # Safety
    /// 
    /// - `self` must have been allocated by `storage`, or a shared storage.
    /// - `self` must still be valid.
    /// - `new_size` must be less than or equal to `self.len()`
    pub const unsafe fn try_shrink<S>(&mut self, new_size: usize, storage: &mut S) -> Result<(), AllocError> where
        S: ~const StorageSingle<Handle = H>
    {
        // Safety
        // - `self.0` has been allocted by `store` or a shared storage, as per pre-conditions.
        // - `self.0` is still valid, as per pre-conditions
        // - `new_size` is greater than or equal to `self.0.len()`, as per pre-conditions.
        TypedSingleHandle::try_shrink(&mut self.0, new_size, storage)
    }

    /// Behaves life `shrink`, but also allows specifying a number of elements that should be copied to the new memory.
    /// 
    /// # Safety
    /// 
    /// See `shrink`, with the additional contraints of:
    /// 
    /// - `copy_size` must be less than or equal to `new_size`.
    pub const unsafe fn shrink_region<S>(&mut self, new_size: usize, region: CopyRegion, storage: &mut S) where
        S: ~const StorageSingle<Handle = H>
    {
        // Safety
        // - `self.0` has been allocted by `store` or a shared storage, as per pre-conditions.
        // - `self.0` is still valid, as per pre-conditions
        // - `new_size` is greater than or equal to `self.0.len()`, as per pre-conditions.
        // - `region` is defined in elements, as per pre-conditions.
        // - `region` adheres to its guarantees, as per pre-conditions.
        TypedSingleHandle::shrink_region(&mut self.0, new_size, region, storage)
    }

    /// Behaves life `try_shrink`, but also allows specifying a number of elements that should be copied to the new memory.
    /// 
    /// # Safety
    /// 
    /// See `try_shrink`, with the additional contraints of:
    /// 
    /// - `copy_size` must be less than or equal to `new_size`.
    pub const unsafe fn try_shrink_region<S>(&mut self, new_size: usize, region: CopyRegion, storage: &mut S) -> Result<(), AllocError> where
        S: ~const StorageSingle<Handle = H>
    {
        // Safety
        // - `self.0` has been allocted by `store` or a shared storage, as per pre-conditions.
        // - `self.0` is still valid, as per pre-conditions
        // - `new_size` is greater than or equal to `self.0.len()`, as per pre-conditions.
        // - `region` is defined in elements, as per pre-conditions.
        // - `region` adheres to its guarantees, as per pre-conditions.
        TypedSingleHandle::try_shrink_region(&mut self.0, new_size, region, storage)
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
        S: ~const StorageSingle<Handle = H>
    {
        // Safety
        // - `self.0` has been allocted by `store` or a shared storage, as per pre-conditions.
        // - `self.0` is still valid, as per pre-conditions
        // - `new_size` is greater than or equal to `self.0.len()`, as per pre-conditions.
        // - `region` is defined in elements, as per pre-conditions.
        // - `region` adheres to its guarantees, as per pre-conditions.
        TypedSingleHandle::shrink_region_zeroed(&mut self.0, new_size, region, storage)
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
        S: ~const StorageSingle<Handle = H>
    {
        // Safety
        // - `self.0` has been allocted by `store` or a shared storage, as per pre-conditions.
        // - `self.0` is still valid, as per pre-conditions
        // - `new_size` is greater than or equal to `self.0.len()`, as per pre-conditions.
        // - `region` is defined in elements, as per pre-conditions.
        // - `region` adheres to its guarantees, as per pre-conditions.
        TypedSingleHandle::try_shrink_region_zeroed(&mut self.0, new_size, region, storage)
    }
}