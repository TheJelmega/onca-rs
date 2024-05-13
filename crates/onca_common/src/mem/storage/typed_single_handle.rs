use core::{
    marker::Unsize,
    mem,
    ptr::{Alignment, NonNull}
};

use std::alloc::{self, AllocError, Layout};

use super::{CopyRegion, StorageSingle, StorageBase, TypedMetadata};


/// Arbitrary types handle, for type safety, and coercion
/// 
/// A typed handle may be dangling, or may be invalid. It is the responsibility of the user that the typed handle is valid when neccesary
pub struct TypedSingleHandle<T: ?Sized, H> {
    handle:   H,
    metadata: TypedMetadata<T>
}

impl<T, H: Copy> TypedSingleHandle<T, H> {
    /// Creates a dangling handle
    /// 
    /// Calls `handle_alloc_error` if the creation of the handle fails
    pub const fn dangling<S>(storage: &S) -> Self where
        S: ~const StorageBase<Handle = H>
    {
        let Ok(this) = Self::try_dangling(storage) else {
            alloc::handle_alloc_error(Layout::new::<T>())
        };
        this
    }


    /// Tries to create a dangling handle.
    /// 
    /// Returns `AllocError` on failure.
    pub const fn try_dangling<S>(storage: &S) -> Result<Self, AllocError> where
        S: ~const StorageBase<Handle = H>
    {
        let handle = storage.dangling(Alignment::of::<T>())?;
        let metadata = TypedMetadata::new();

        Ok(Self { handle, metadata })
    }

    /// Create a new handle, poinint to a `T`
    /// 
    /// Calls `handle_alloc_error` if the creation of the handle fails.
    pub fn new<S>(value: T, storage: &mut S) -> Self where
    S: StorageSingle<Handle = H>
    {
        let Ok(this) = Self::try_new(value, storage) else {
            alloc::handle_alloc_error(Layout::new::<T>());
        };
        this
    }
    
    /// Try to create a new handle, pointing to a `T`.
    pub fn try_new<S>(value: T, storage: &mut S) -> Result<Self, AllocError> where
        S: StorageSingle<Handle = H>
    {
        /// Safety
        let (handle, _) = storage.allocate(Layout::new::<T>())?;

        // Safety
        // - `handle` was just allocated by `store`
        // - `handle` is still valid, as no other operation occured on `store`
        let ptr = unsafe { storage.resolve_mut(handle) };

        // Safety
        // - `ptr` points to writable memory
        // - `ptr` points to a sufficient aligned and sized memory arena
        // - `ptr` has exclusive access to the memory area it points to
        unsafe { core::ptr::write(ptr.cast().as_ptr(), value) };

        Ok(Self {
            handle,
            metadata: TypedMetadata::new(),
        })
    }

    /// Allocate a new handle, with enough space for `T`.
    /// 
    /// Calls `handle_alloc_error` if the creation of the handle fails.
    /// 
    /// The allocated memory is left uninitialized.
    pub const fn allocate<S>(storage: &mut S) -> Self where
        S: ~const StorageSingle<Handle = H>
    {
        let Ok(this) = Self::try_allocate(storage) else {
            alloc::handle_alloc_error(Layout::new::<T>());
        };
        this
    }

    /// Try to allocate a new handle, with enough space for `T`.
    /// 
    /// The allocated memory is left uninitialized.
    pub const fn try_allocate<S>(storage: &mut S) -> Result<Self, AllocError> where
        S: ~const StorageSingle<Handle = H>
    {
        let (handle, _) = storage.allocate(Layout::new::<T>())?;

        Ok(Self {
            handle,
            metadata: TypedMetadata::new(),
        })
    }

    /// Allocate a new handle, with enough space for `T`.
    /// 
    /// Calls `handle_alloc_error` if the creation of the handle fails.
    /// 
    /// The allocated memory is zeroed out.
    pub const fn allocate_zeroed<S>(storage: &mut S) -> Self where
        S: ~const StorageSingle<Handle = H>
    {
        let Ok(this) = Self::try_allocate(storage) else {
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
        let (handle, _) = storage.allocate_zeroed(Layout::new::<T>())?;

        Ok(Self {
            handle,
            metadata: TypedMetadata::new(),
        })
    }
}

impl<T: ?Sized, H: Copy> TypedSingleHandle<T, H> {
    /// Creates a handle from raw parts
    /// 
    /// - If `handle` is valid, and associated to a block of memory which fits an instance of `T`, then the resulting typed handle is valid.
    /// - If `handle` is invalid, then the resulting types handle is invalid.
    /// - If `handle` is valid and `metadata` does not fit the block of memory associated with it, the resuling typed handle is invalid
    pub const fn from_raw_parts(handle: H, metadata: TypedMetadata<T>) -> Self {
        Self { handle, metadata }
    }

    /// Decomposes a (possibly wide) pointer into its (raw) handle and metadata components
    pub const fn to_raw_parts(self) -> (H, TypedMetadata<T>) {
        (self.handle, self.metadata)
    }

    /// Deallocates the memory associated with the handle.
    /// 
    /// # Safety
    /// 
    /// - `self` must have been allocated by `storage`, or a shared storage.
    /// - `self` must be valid.
    /// - `self` is invalidated alongside any copy of it
    pub const unsafe fn deallocate<S>(&self, storage: &mut S) where 
        S: ~const StorageSingle<Handle = H>
    {
        // Safety
        // - `self.handle` was allocated by `storage`, or a shared storage, as per pre-conditions
        // - `self.handle` is still valid, as per pre-conditions
        let ptr = unsafe { self.resolve_raw(storage) };

        // Safety
        // - `pointer` has valid metadata for `T`
        let layout = unsafe { Layout::for_value_raw(ptr.as_ptr() as *const T) };
        
        // Safety
        // - `self.handle` was allocated by `storage`, as per pre-conditons
        // - `self.handle` is still valid, as per pre-conditions
        // - `layout` fits the block of memory associated with `self.handle`
        unsafe { storage.deallocate(self.handle, layout) };
    }

    /// Resolved the handle to a reference
    /// 
    /// # Safety
    /// 
    /// - `self` must have been allocated by `storage`, or a shared storage.
    /// - `self` must still be valid.
    /// - `self` must be associated to a block of memory containing a valid instance of `T`.
    /// - No access through a mutable reference to this of `T` must overlap with accesses through the result.
    /// - The reference is only guaranteed to be valid as long as `self` is valid.
    /// - The reference is only guaranteed to be valid as long as pointers resovled from `self` are not invalidated.
    ///   Most notably, unless `storage` implements `StoreStable`, any method call on `store` including other `resolve` calls, may invalidate the reference.
    pub const unsafe fn resolve<'a, S>(&self, storage: &'a S) -> &'a T where
        S: ~const StorageSingle<Handle = H>
    {
        // Safety
        // - `self.handle` was allocated by `storage` or a shared storage, as per pre-conditions.
        // - `self.handle` is still valid, as per pre-conditions
        let ptr = self.resolve_raw(storage);

        // Safety
        // - `ptr` points to a live instance of `T`, as per type-invariant.
        // - The resulting reference borrows `storage` immutably, 
        //   guaranteeing it won't be invalidted by moving or destroying storage.
        ptr.as_ref()
    }

    /// Resolved the handle to a reference
    /// 
    /// # Safety
    /// 
    /// - `self` must have been allocated by `storage`, or a shared storage.
    /// - `self` must still be valid.
    /// - `self` must be associated to a block of memory containing a valid instance of `T`.
    /// - No access through a mutable reference to this of `T` must overlap with accesses through the result.
    /// - The reference is only guaranteed to be valid as long as `self` is valid.
    /// - The reference is only guaranteed to be valid as long as pointers resovled from `self` are not invalidated.
    ///   Most notably, unless `storage` implements `StoreStable`, any method call on `store` including other `resolve` calls, may invalidate the reference.
    pub const unsafe fn resolve_mut<'a, S>(&self, storage: &'a mut S) -> &'a mut T where
        S: ~const StorageSingle<Handle = H>
    {
        // Safety
        // - `self.handle` was allocated by `storage` or a shared storage, as per pre-conditions.
        // - `self.handle` is still valid, as per pre-conditions
        let mut ptr = self.resolve_raw_mut(storage);

        // Safety
        // - `ptr` points to a live instance of `T`, as per type-invariant.
        // - The resulting reference borrows `storage` immutably, 
        //   guaranteeing it won't be invalidted by moving or destroying storage.
        ptr.as_mut()
    }

    /// Resolves the handle to a non-null pointer
    /// 
    /// # Safety
    /// 
    /// - `self` must have been allocated by `storage`, or a shared storage.
    /// - `self` must still be valid
    /// - The pointer is only guaranteed to be valid as long as `self` is valid.
    /// - The pointer is only guaranteed to be valid as long as pointer s resovled from `self` are not invalidated.
    ///   Most notably, unless `storage` implements `StorageStable`, any method call on `storage`, including other `resolve` calls, may invalidate the pointer.
    /// - The pointer must not be access mutably.
    pub const unsafe fn resolve_raw<S>(&self, storage: &S) -> NonNull<T> where
        S: ~const StorageSingle<Handle = H>
    {
        // Safety
        // - `self.handle` was alloced by `storage`, or a shared storage, as per pre-conditions
        // - `self.handle` is still valid, as per pre-conditions
        let ptr = unsafe { storage.resolve(self.handle) };

        // Safety
        // - `self.handle` is still valid, so therefore the resulting pointer is valid.
        // - The pointer should not be access mutably, as per pre-conditions
        let ptr = NonNull::new_unchecked(ptr.as_ptr() as *mut ());

        NonNull::from_raw_parts(ptr.cast(), self.metadata.get())
    }

    /// Resolves the handle to a non-null pointer
    /// 
    /// # Safety
    /// 
    /// - `self` must have been allocated by `storage`, or a shared storage.
    /// - `self` must still be valid
    /// - The pointer is only guaranteed to be valid as long as `self` is valid.
    /// - The pointer is only guaranteed to be valid as long as pointer s resovled from `self` are not invalidated.
    ///   Most notably, unless `storage` implements `StorageStable`, any method call on `storage`, including other `resolve` calls, may invalidate the pointer.
    pub const unsafe fn resolve_raw_mut<S>(&self, storage: &mut S) -> NonNull<T> where
        S: ~const StorageSingle<Handle = H>
    {
        // Safety
        // - `self.handle` was alloced by `storage`, or a shared storage, as per pre-conditions
        // - `self.handle` is still valid, as per pre-conditions
        let ptr = unsafe { storage.resolve_mut(self.handle) };

        NonNull::from_raw_parts(ptr.cast(), self.metadata.get())
    }

    /// Coerces the handle into another.
    /// 
    /// If `self` is valid, the resulting typed handle is valid; otherwise it is invalid.
    pub unsafe fn coerce<U: ?Sized>(&self) -> TypedSingleHandle<U, H> where
        T: Unsize<U>
    {
        let metadata = self.metadata.coerce::<U>();
        TypedSingleHandle { handle: self.handle, metadata }
    }
}

impl<T, H: Copy> TypedSingleHandle<[T], H> {
    /// Create a dangling handle.
    /// 
    /// Calls `handle_alloc_error` if the creation of the handle fails.
    pub const fn dangling_slice<S>(storage: &S) -> Self where
        S: ~const StorageBase<Handle = H>
    {
        let Ok(this) = Self::try_dangling_slice(storage) else {
            alloc::handle_alloc_error(Layout::new::<T>());
        };
        this
    }

    /// try to create a dangling handle.
    /// 
    /// Returns `AllocError` on failure
    pub const fn try_dangling_slice<S>(storage: &S) -> Result<Self, AllocError> where
        S: ~const StorageBase<Handle = H>
    {
        let handle = storage.dangling(Alignment::of::<T>())?;

        Ok(Self {
            handle,
            metadata: TypedMetadata::from_metadata(0),
        }) 
    }

    /// Allocate a new handle, with enough space for `size` elements of `T`.
    /// 
    /// Calls `handle_alloc_error` if the creation of the handle fails.
    /// 
    /// The allocted memory is left uninitialized.
    pub const fn allocate_sliced<S>(size: usize, storage: &mut S) -> Self where
        S: ~const StorageBase<Handle = H> + ~const StorageSingle<Handle = H>
    {
        let Ok(this) = Self::try_allocate_sliced(size, storage) else {
            alloc::handle_alloc_error(Layout::new::<T>())
        };
        this
    }

    /// Try to allocate a new handle, with enough space for `size` elements of `T`.
    /// 
    /// The allocted memory is left uninitialized.
    pub const fn try_allocate_sliced<S>(size:usize, storage: &mut S) -> Result<Self, AllocError> where
        S: ~const StorageBase<Handle = H> + ~const StorageSingle<Handle = H>
    {
        if core::mem::size_of::<T>() == 0 {
            let Ok(mut this) = Self::try_dangling_slice(storage) else {
                alloc::handle_alloc_error(Layout::new::<T>())
            };
            
            this.metadata = TypedMetadata::from_metadata(usize::MAX);
            return Ok(this);
        }

        let layout = Self::layout(size)?;
        let (handle, bytes) = storage.allocate(layout)?;

        debug_assert!(bytes >= layout.size());

        let metadata = TypedMetadata::from_metadata(bytes / core::mem::size_of::<T>());
        Ok(Self { handle, metadata })
    }

    /// Allocate a new handle, with enough space for `size` elements of `T`.
    /// 
    /// Calls `handle_alloc_error` if the creation of the handle fails.
    /// 
    /// The allocted memory is zeroed out.
    pub const fn allocate_zeroed_sliced<S>(size:usize, storage: &mut S) -> Self where
        S: ~const StorageBase<Handle = H> + ~const StorageSingle<Handle = H>
    {
        let Ok(this) = Self::try_allocate_zeroed_sliced(size, storage) else {
            alloc::handle_alloc_error(Layout::new::<T>())
        };
        this
    }

    /// Try to allocate a new handle, with enough space for `size` elements of `T`.
    /// 
    /// The allocted memory is zeroed out.
    pub const fn try_allocate_zeroed_sliced<S>(size:usize, storage: &mut S) -> Result<Self, AllocError> where
        S:  ~const StorageBase<Handle = H> + ~const StorageSingle<Handle = H>
    {
        if core::mem::size_of::<T>() == 0 {
            let Ok(mut this) = Self::try_dangling_slice(storage) else {
                alloc::handle_alloc_error(Layout::new::<T>())
            };
            
            this.metadata = TypedMetadata::from_metadata(usize::MAX);
            return Ok(this);
        }

        let layout = Self::layout(size)?;
        let (handle, bytes) = storage.allocate_zeroed(layout)?;

        debug_assert!(bytes >= layout.size());

        let metadata = TypedMetadata::from_metadata(bytes / core::mem::size_of::<T>());
        Ok(Self { handle, metadata })
    }

    /// Returns whether the memory area associated to `self` may not contain any element.
    pub const fn is_empty(&self) -> bool {
        self.metadata.get() == 0
    }

    /// Return the number of elements the memory area associated to `self` may contain.
    pub const fn len(&self) -> usize {
        self.metadata.get()
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
        S: ~const StorageSingle<Handle = H>
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
        S: ~const StorageSingle<Handle = H>
    {
        debug_assert!(new_size >= self.len());

        let old_layout = Self::layout(self.len())?;
        let new_layout = Self::layout(new_size)?;

        // Safety:
        // - `self.handle` was allocated by `storage` or a shared storage, as per pre-conditons
        // - `self.handle` is still valid, as per pre-conditions
        // - `old_layout` fits the block of memory associated to `self.handle`, by construction.
        // - `new_layout`'s size is greater than or equal to the size of `old_layout`, as per pre-conditions.
        let (handle, bytes) = unsafe { storage.grow(self.handle, old_layout, new_layout) }?;

        debug_assert!(bytes >= new_layout.size());

        self.handle = handle;
        self.metadata = TypedMetadata::from_metadata(bytes / core::mem::size_of::<T>());

        Ok(())
    }

    /// Grows the block of memory associated with the handle.
    /// 
    /// On success, all the copies of the handle are invalidated, and the extra memory is zeroed out.
    /// OCalls `handle_alloc_error` on failure.
    /// 
    /// # Safety
    /// 
    /// - `self` must have been allocated by `storage`, or a shared storage.
    /// - `self` must still be valid.
    /// - `new_size` must be greater than or euqal to `self.len()`.
    pub const unsafe fn grow_zeroed<S>(&mut self, new_size: usize, storage: &mut S) where
        S: ~const StorageSingle<Handle = H>
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
        S: ~const StorageSingle<Handle = H>
    {
        debug_assert!(new_size >= self.len());

        let old_layout = Self::layout(self.len())?;
        let new_layout = Self::layout(new_size)?;

        // Safety:
        // - `self.handle` was allocated by `storage` or a shared storage, as per pre-conditons
        // - `self.handle` is still valid, as per pre-conditions
        // - `old_layout` fits the block of memory associated to `self.handle`, by construction.
        // - `new_layout`'s size is greater than or equal to the size of `old_layout`, as per pre-conditions.
        let (handle, bytes) = unsafe { storage.grow_zeroed(self.handle, old_layout, new_layout) }?;

        debug_assert!(bytes >= new_layout.size());

        self.handle = handle;
        self.metadata = TypedMetadata::from_metadata(bytes / core::mem::size_of::<T>());

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
        S: ~const StorageSingle<Handle = H>
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
        S: ~const StorageSingle<Handle = H>
    {
        debug_assert!(new_size >= self.len());
        debug_assert!(region.src_offset < self.len());
        debug_assert!(region.dst_offset < new_size);
        debug_assert!(region.size <= self.len());
        debug_assert!(region.src_offset + region.size <= self.len());
        debug_assert!(region.dst_offset + region.size <= new_size);

        let old_layout = Self::layout(self.len())?;
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
        self.metadata = TypedMetadata::from_metadata(bytes / core::mem::size_of::<T>());

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
        S: ~const StorageSingle<Handle = H>
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
        S: ~const StorageSingle<Handle = H>
    {
        debug_assert!(new_size >= self.len());

        let old_layout = Self::layout(self.len())?;
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
        self.metadata = TypedMetadata::from_metadata(bytes / core::mem::size_of::<T>());

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
        S: ~const StorageSingle<Handle = H>
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
        S: ~const StorageSingle<Handle = H>
    {
        debug_assert!(new_size <= self.len());

        if mem::size_of::<T>() == 0 {
            return Ok(());
        }

        let Ok(old_layout) = Self::layout(self.len()) else { return Err(AllocError) };
        let Ok(new_layout) = Self::layout(new_size) else { return Err(AllocError) };

        let result = unsafe { storage.shrink(self.handle, old_layout, new_layout) };

        let Ok((handle, bytes)) = result else { return Err(AllocError); };

        debug_assert!(bytes >= new_layout.size());

        self.handle = handle;
        self.metadata = TypedMetadata::from_metadata(bytes / core::mem::size_of::<T>());

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
        S: ~const StorageSingle<Handle = H>
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
        S: ~const StorageSingle<Handle = H>
    {
        debug_assert!(new_size <= self.len());
        debug_assert!(region.src_offset < self.len());
        debug_assert!(region.dst_offset < new_size);
        debug_assert!(region.size <= self.len());
        debug_assert!(region.src_offset + region.size <= self.len());
        debug_assert!(region.dst_offset + region.size <= new_size);

        if mem::size_of::<T>() == 0 {
            return Ok(());
        }

        let Ok(old_layout) = Self::layout(self.len()) else { return Err(AllocError) };
        let Ok(new_layout) = Self::layout(new_size) else { return Err(AllocError) };

        // Safety:
        // - `region` is defined in terms of element, as per pre-conditions.
        // - `region`'s members can never overflow `isize`, as per pre-conditions
        let region = region.to_typed::<T>();

        let result = unsafe { storage.shrink_region(self.handle, old_layout, new_layout, region) };

        let Ok((handle, bytes)) = result else { return Err(AllocError); };

        debug_assert!(bytes >= new_layout.size());

        self.handle = handle;
        self.metadata = TypedMetadata::from_metadata(bytes / core::mem::size_of::<T>());

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
        S: ~const StorageSingle<Handle = H>
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
        S: ~const StorageSingle<Handle = H>
    {
        debug_assert!(new_size <= self.len());
        debug_assert!(region.src_offset < self.len());
        debug_assert!(region.dst_offset < new_size);
        debug_assert!(region.size <= self.len());
        debug_assert!(region.src_offset + region.size <= self.len());
        debug_assert!(region.dst_offset + region.size <= new_size);

        if mem::size_of::<T>() == 0 {
            return Ok(());
        }

        let Ok(old_layout) = Self::layout(self.len()) else { return Err(AllocError) };
        let Ok(new_layout) = Self::layout(new_size) else { return Err(AllocError) };

        // Safety:
        // - `region` is defined in terms of element, as per pre-conditions.
        // - `region`'s members can never overflow `isize`, as per pre-conditions
        let region = region.to_typed::<T>();

        let result = unsafe { storage.shrink_region_zeroed(self.handle, old_layout, new_layout, region) };

        let Ok((handle, bytes)) = result else { return Err(AllocError); };

        debug_assert!(bytes >= new_layout.size());

        self.handle = handle;
        self.metadata = TypedMetadata::from_metadata(bytes / core::mem::size_of::<T>());

        Ok(())
    }
}

impl<T: ?Sized, H: Copy> Clone for TypedSingleHandle<T, H> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: ?Sized, H: Copy> Copy for TypedSingleHandle<T, H> {}

impl<T, H> TypedSingleHandle<[T], H> {
    const fn layout(size: usize) -> Result<Layout, AllocError> {
        let Some(size) = core::mem::size_of::<T>().checked_mul(size) else {
            return Err(AllocError);
        };

        let align = core::mem::align_of::<T>();

        let Ok(layout) = Layout::from_size_align(size, align) else { return Err(AllocError); };
        Ok(layout)
    }
}