use std::{
    alloc::Layout,
    marker::PhantomData,
    mem::{self, align_of, size_of, ManuallyDrop},
    ptr::NonNull
};
use std::alloc;

use crate::{
    collections::{ReserveStrategy, TryReserveError},
    mem::{CopyRegion, SlicedSingleHandle, StorageBase, StorageSingle, StorageSingleSliced}
};


/// Low level utility for more ergonomically allocating, reallocating, and deallocating
/// a buffer of memory in a storage without having to worry about all te corner cases involved.
/// In particular:
/// 
/// - Produces `SlicedSingleHandle::dangling` on zero-sized types
/// - Procudes `SlicedSingleHandle::dangling` on zero-lenght allocations.
/// - Avoid freeing `SlicedSingleHandle::dangling`
/// - Catches all overflows in capacity computations (promotes them to "capacity overflow" errors).
/// - Guards against overflowing your length
/// - Uses the excess returned from teh allocator to use the largest avialable allocation.
/// - Unlike `std::collection`'s implementation, we don't check for 32-bit, as we don't support it.
/// 
/// This type does not in anyway inspect the memory it manages. When dropped it *sill* free tis memeory, but it *won't* try to drop its contents.
/// It is up to the user of `RawArray` to handle the actual things *stored* inside of `RawArray`
/// 
/// Note that the excess of a zero-sized type is always infinitie, so `capacity()` always return `usize::MAX`.
/// This means that you need to be careful when round-tripping this type with a `Box<[T]>`, since `capacity()` won't yeild the lenght.
pub(crate) struct RawArray<T, S: StorageSingleSliced, R: ReserveStrategy> {
    handle:   SlicedSingleHandle<T, S::Handle>,
    storage:  S,
    _phantom: PhantomData<R>,
}

impl<T, S: StorageSingleSliced, R: ReserveStrategy> RawArray<T, S, R> {
    /// Tiny arrays are dumb, so like std's `RawVec`, skip to:
    /// - 8 if the element size is 1, because of how heap allocators are likely to round up a request of 8 bytes to at least 8 bytes (if not 16).
    /// - 4 if the element is moerate-sizec (<= 1KiB, which could fit nicely in a single OS memory page)
    /// - 1 otherwise, to avoid wasting too much space for very short arrays.
    pub const MIN_NON_ZERO_CAP: usize = if S::USE_MIN_SIZE_OPTIMIZE {
        if size_of::<T>() == 1 {
            8
        } else if size_of::<T>() <= 1024 {
            4
        } else {
            1
        }
    } else {
        0
    };

    /// Creates the biggest possible `RawArray` (in the storage) without allocating.
    /// If `T` has a non-zero size, the this makes a `RawArray` with a capacity of `0`.
    /// If `T` is zero-sized, the it makes a `RawArray` with a capacity of `usize::MAX`.
    /// Useful for implementing delayed allocation.
    #[must_use]
    pub const fn new_in(storage: S) -> Self where
        S: ~const StorageBase<Handle = <S as StorageBase>::Handle>
    {
        let handle =  SlicedSingleHandle::dangling(&storage);
        Self { handle, storage, _phantom: PhantomData }
    }
    
    /// Tries to create the biggest possible `RawArray` (in the storage) without allocating.
    /// If `T` has a non-zero size, the this makes a `RawArray` with a capacity of `0`.
    /// If `T` is zero-sized, the it makes a `RawArray` with a capacity of `usize::MAX`.
    /// Useful for implementing delayed allocation.
    /// 
    /// # Error
    /// 
    /// A `TryResultError` will be returned if a `RawArray` could not be created.
    #[must_use]
    pub fn try_new_in(storage: S) -> Result<Self, TryReserveError> {
        match SlicedSingleHandle::try_dangling(&storage) {
            Ok(handle) => Ok(Self { handle, storage, _phantom: PhantomData }),
            Err(_) => Err(TryReserveError::AllocError(unsafe { Layout::from_size_align_unchecked(0, align_of::<T>()) }))
        }   
    }

    /// Creates a `RawArray` (in the storage) with exactly the xapacity and alignment requirements for a `[T; capacity]`.
    /// This is equivalent to calling `RawArray::new` when `capactiy` is `0` or `T` is zero-sized.
    /// Note that if `T` is zero-sized this means you will *not* get a `RawArray` with the requested capacity.
    /// 
    /// Non-fallible version of `try_with_capacity_in`.
    /// 
    /// # Panics
    /// 
    /// Panics if the requested capacity exceed `isize::MAX` bytes.
    /// 
    /// # Aborts
    /// 
    /// Aborts on OOM
    #[must_use]
    pub const fn with_capacity_in(capacity: usize, mut storage: S) -> Self where
        S: ~const StorageBase<Handle = <S as StorageBase>::Handle> + ~const StorageSingle
    {
        let handle = SlicedSingleHandle::allocate(&mut storage, capacity);
        Self { handle, storage, _phantom: PhantomData }
    }

    /// Tries to create a `RawArray` (in the storage) with exactly the xapacity and alignment requirements for a `[T; capacity]`.
    /// This is equivalent to calling `RawArray::new` when `capactiy` is `0` or `T` is zero-sized.
    /// Note that if `T` is zero-sized this means you will *not* get a `RawArray` with the requested capacity.
    /// 
    /// # Panics
    /// 
    /// Panics if the requested capacity exceed `isize::MAX` bytes.
    /// 
    /// # Aborts
    /// 
    /// Aborts on OOM
    #[must_use]
    pub fn try_with_capacity_in(capacity: usize, mut storage: S) -> Result<Self, TryReserveError> {
        Self::try_allocate_in(capacity, false, storage)
    }

    /// Behaves like `with_capacity_zeroed_in`, but the underlying memory is zeroed out.
    #[must_use]
    pub const fn with_capacity_zeroed_in(capacity: usize, mut storage: S) -> Self where
        S: ~const StorageBase<Handle = <S as StorageBase>::Handle> + ~const StorageSingle
    {
        let handle = SlicedSingleHandle::allocate_zeroed(&mut storage, capacity);
        Self { handle, storage, _phantom: PhantomData }
    }

    /// Behaves like `try_with_capacity_zeroed_in`, but the underlying memory is zeroed out.
    #[must_use]
    pub fn try_with_capacity_zeroed_in(capacity: usize, mut storage: S) -> Result<Self, TryReserveError> {
        Self::try_allocate_in(capacity, true , storage)
    }

    fn try_allocate_in(capacity: usize, zero_out: bool, mut storage: S) -> Result<Self, TryReserveError> {
        // Don't allocate herer, because `drop`` will not deallocate when capacity is 0.
        if size_of::<T>() == 0 || capacity == 0 {
            Self::try_new_in(storage)
        } else {
            if capacity * size_of::<T>() > isize::MAX as usize {
                return Err(TryReserveError::CapacityOverflow);
            }
            
            let result = if zero_out {
                SlicedSingleHandle::try_allocate_zeroed(&mut storage, capacity)
            } else {
                SlicedSingleHandle::try_allocate_zeroed(&mut storage, capacity)
            };
            let handle = match result {
                Ok(handle) => handle,
                Err(_) => {
                    let layout = Layout::array::<T>(capacity).expect("If we got passed the size check, this shouldn't be able to panic");
                    return Err(TryReserveError::AllocError(layout));
                },
            };

            Ok(Self { handle, storage, _phantom: PhantomData })
        }
    }

    /// Get the capacity of the allocation.
    /// 
    /// This will always be `usize::MAX` if `T` is zero-sized.
    pub const fn capacity(&self) -> usize where
        S: ~const StorageSingleSliced
    {
        if size_of::<T>() == 0 {
            usize::MAX
        } else {
            self.handle.len(&self.storage)
        }
    }

    /// Get the storage used for the allocation.
    pub const fn storage(&self) -> &S {
        &self.storage
    }


    /// Get the handle used for the allocation
    pub const fn handle(&self) -> &SlicedSingleHandle<T, S::Handle> {
        &self.handle
    }

    pub const fn as_slice(&self) -> &[T] where
        S: ~const StorageSingleSliced
    {
        // Safety
        // - `self.handle` is valid or a dangling handle, both need to be resolved correctly according to `StorageSingleSliced`
        // - `self.handle` was obtained via `self.storage` in either case.
        unsafe { self.handle.resolve(&self.storage) }
    }

    pub const fn as_mut_slice(&mut self) -> &mut [T] where
        S: ~const StorageSingleSliced
    {
        // Safety
        // - `self.handle` is valid or a dangling handle, both need to be resolved correctly according to `StorageSingleSliced`
        // - `self.handle` was obtained via `self.storage` in either case.
        unsafe { self.handle.resolve_mut(&self.storage) }
    }

    /// Create a `RawArray` from a handle and a storage.
    /// 
    /// # Safety
    /// 
    /// - `handle` needs to have been allocated by `storage`.
    pub const unsafe fn from_raw_parts(handle: SlicedSingleHandle<T, S::Handle>, storage: S) -> Self {
        Self { handle, storage, _phantom: PhantomData }
    }

    /// Decompose a `RawArray` into its handle and allocator.
    pub const unsafe fn to_raw_parts(self) -> (SlicedSingleHandle<T, S::Handle>, S) {
        let this = ManuallyDrop::new(self);
        (this.handle, core::ptr::read(&this.storage))
    }

    /// Get a raw pointer to the start of the allocation.
    /// Note that this is a dangling pointer when either `capacity() == 0` or `T` is zero-sized.
    /// In the former case, you must be careful.
    pub const unsafe fn ptr(&self) -> *mut T where
        S: ~const StorageSingleSliced
    {
        let (ptr, _) = self.handle.resolve_raw(&self.storage);
        ptr.as_ptr().cast()
    }

    pub const unsafe fn non_null(&self) -> NonNull<T> where
        S: ~const StorageSingleSliced
    {
        let (ptr, _) = self.handle.resolve_raw(&self.storage);
        ptr.cast()
    }

    pub const fn current_memory(&self) -> Option<(NonNull<u8>, Layout)> where
        S: ~const StorageSingleSliced
    {
        if size_of::<T>() == 0 || self.capacity() == 0 {
            None
        } else {
            /// We could use Layout::array herer which ensures the absence  of isize and usize overflows and could hypothetically handle differences between stride and size,
            /// but this memory has already been allocated so we know it can't overflow and curretnly Rust does not support such types.
            /// So we can do better by skipping some checks and avoid an unwrap.
            debug_assert!(size_of::<T>() % align_of::<T>() == 0);
            unsafe {
                let align = align_of::<T>();
                let size = size_of::<T>().unchecked_mul(self.capacity());
                let layout = Layout::from_size_align_unchecked(size, align);
                Some((self.non_null().cast(), layout))
            }
        }
    }

    /// Ensured that hte buffer contains at least enough space to hold `len + additional` elements.
    /// If it doesn't already have enough capacity, will reallocate enough space plus comfortable slack space to get amortized. *O*(1) behavior.
    /// Will limit this behaviour if it would needlessly cause itself panics.
    /// 
    /// `len` may not exceed `self.capacity()`, as this can cause a panic.
    /// 
    /// This is ideal for implementing a bulk-push operation like `extend`.
    /// 
    /// # Panics
    /// 
    /// Panics if the new capacity exceeds `isize::MAX` bytes.
    /// 
    /// # Aborts
    /// 
    /// Aborts on OOM.
    pub fn reserve(&mut self, len: usize, additional: usize) {
        // Callers expect this function to be very cheap when there is already sufficient capacity.
        // Therefore, we move all the resizing and error-handling logic from grow_amortized and handle_reserve behinds a call,
        // while making sure that this function is likely to be inlines as just a comparison and a call if the comparison fails.
        #[cold]
        fn do_reserve_and_handle<T, S: StorageSingleSliced, R: ReserveStrategy>(
            slf: &mut RawArray<T, S, R>,
            len: usize,
            additional: usize,
        ) {
            debug_assert!(len < slf.capacity());
            if let Err(err) = slf.grow_amortized(len, additional) {
                handle_error(err);
            }
        }

        if self.needs_to_grow(len, additional) {
            do_reserve_and_handle(self, len, additional);
        }
    }

    /// A specialized version of `self.reserve(len, 1)`, which requires the caller to ensure `len == self.capacity()`.
    pub fn grow_one(&mut self) {
        if let Err(err) = self.grow_amortized(self.capacity(), 1) {
            handle_error(err);
        }
    }

    /// The same as `reserve`, but returns on errors instead of panicking or aborting.
    pub fn try_reserve(&mut self, len: usize, additional: usize) -> Result<(), TryReserveError> {
        if self.needs_to_grow(len, additional) {
            self.grow_amortized(len, additional);
        }
        unsafe {
            /// Inform the optimizer that the reservation has succeeded or wasn't needed
            core::hint::assert_unchecked(!self.needs_to_grow(len, additional));
        }
        Ok(())
    }

    /// Ensures that the buffer contains at least enough space to hold `len + additional` elements.
    /// If it doesn't already, will reallocate the minimum possible amougn of memory necessary.
    /// Generally this will be exactly the amount of memory neccessary, but in principle the allocator is free to give back more than we asked for.
    /// 
    /// `len` may not exceed `self.capacity()`, as this can cause a panic.
    /// 
    /// # Panics
    /// 
    /// Panics if the new capacity exceeds `isize::MAX` _bytes_.
    /// 
    /// # Aborts
    /// 
    /// Aborts on OOM.
    pub fn reserve_exact(&mut self, len: usize, additional: usize) {
        if let Err(err) = Self::try_reserve_exact(self, len, additional) {
            handle_error(err);
        }
    }
    
    pub fn try_reserve_exact(&mut self, len: usize, additional: usize) -> Result<(), TryReserveError> {
        if self.needs_to_grow(len, additional) {
            self.grow_exact(len, additional)?;
        }
        unsafe {
            // Inform the optimziat that the reservation has succeeded or wasn't needed
            core::hint::assert_unchecked(!self.needs_to_grow(len, additional));
        }
        Ok(())
    }

    /// Shrinks the buffer down to the sepcified capacity.
    /// If the given amount is 0, actually completely deallocates.
    /// 
    /// # Panics
    /// 
    /// Panics if the given amount is *larger* than the current capacity.
    /// 
    /// # Aborts
    /// 
    /// Aborts on OOM.
    pub fn shrink_to_fit(&mut self, cap: usize) {
        if let Err(err) = self.shrink(cap) {
            handle_error(err);
        }
    }

    //--------------------------------------------------------------

    /// Returns if the buffer needs to grow to fulfill the needed extra capacity.
    /// Mainly used to make inlining reserve-valls possible without inlining `grow`.
    fn needs_to_grow(&self, len: usize, additional: usize) -> bool {
        additional > self.capacity().wrapping_sub(len)
    }

    fn grow_amortized(&mut self, len: usize, additional: usize) -> Result<(), TryReserveError> {
        debug_assert!(additional > 0);

        if size_of::<T>() == 0 {
            /// Since we return a capacity of `usize::MAX` when `elem_size` is 0, getting to here necessarily means that `RawArray` is overfull.
            return Err(TryReserveError::CapacityOverflow);
        }

        let required_cap = len.checked_add(additional).ok_or(TryReserveError::CapacityOverflow)?;
        let cur_capacity = self.capacity();


        let new_cap = R::calculate(cur_capacity, required_cap);

        unsafe { self.finalize_grow(new_cap, len) }
    }

    fn grow_exact(&mut self, len: usize, additional: usize) -> Result<(), TryReserveError> {
        debug_assert!(additional > 0);

        if size_of::<T>() == 0 {
            /// Since we return a capacity of `usize::MAX` when `elem_size` is 0, getting to here necessarily means that `RawArray` is overfull.
            return Err(TryReserveError::CapacityOverflow);
        }

        let target_capacity = len.checked_add(additional).ok_or(());
        unsafe { self.finalize_grow(target_capacity, len) }
    }

    fn shrink(&mut self, cap: usize) -> Result<(), TryReserveError> {
        assert!(cap <= self.capacity(), "Tried to shrink to a larger capacity");

        if cap == 0 {
            unsafe { self.handle.deallocate(&mut self.storage); }

            // We need to manually set the handle
            self.handle = match SlicedSingleHandle::try_dangling(&self.storage) {
                Ok(handle) => handle,
                Err(_) => {
                    let layout = Layout::array::<T>(cap).unwrap();
                    return Err(TryReserveError::AllocError(layout));
                }
            };
        } else {
            match unsafe { self.handle.try_shrink(cap, &mut self.storage) } {
                Ok(_) => {},
                Err(_) => {
                    let layout = Layout::array::<T>(cap).unwrap();
                    return Err(TryReserveError::AllocError(layout));
                },
            }
        }


        Ok(())
    }

    unsafe fn finalize_grow(&mut self, new_cap: Result<usize, ()>, len: usize) -> Result<(), TryReserveError> {
        let Ok(new_cap) = new_cap else {
            return Err(TryReserveError::CapacityOverflow);
        };

        if self.handle.is_empty(&self.storage) {
            self.handle = match SlicedSingleHandle::try_allocate(&mut self.storage, new_cap) {
                Ok(handle) => handle,
                Err(_) => return Err(TryReserveError::AllocError(Layout::array::<T>(new_cap).unwrap())),
            }
        } else {
            let region = CopyRegion {
                src_offset: 0,
                dst_offset: 0,
                size: len,
            };

            if let Err(err) = self.handle.try_grow_region(new_cap, region, &mut self.storage) {
                return Err(TryReserveError::AllocError(Layout::array::<T>(new_cap).unwrap()));
            }
        }

        Ok(())
    }

}

unsafe impl<#[may_dangle] T, S: StorageSingleSliced, R: ReserveStrategy> Drop for RawArray<T, S, R> {
    fn drop(&mut self) {
        if self.handle.is_empty(&self.storage) {
            return;
        }

        unsafe { self.handle.deallocate(&mut self.storage) };
    }
}

/// Central function for reserve error handling
#[cold]
fn handle_error(e: TryReserveError) -> ! {
    match e {
        TryReserveError::CapacityOverflow => capacity_overflow(),
        TryReserveError::AllocError(layout) => alloc::handle_alloc_error(layout),
    }
}

fn capacity_overflow() -> ! {
    panic!("capacity_overflow");
}