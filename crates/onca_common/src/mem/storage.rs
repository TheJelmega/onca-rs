//! This defines a storage interface inspired by and based on the storage proposal for rust by matthieu-m, to allow the use of custom data for the underlying memory of containers
//! 
//! The 3rd iteration of the proposal can be found here: https://github.com/matthieu-m/storage/

use std::{alloc::{AllocError, Layout}, num::NonZeroUsize, process::Output, ptr::{Alignment, NonNull}};

mod wrappers;

mod typed_metadata;
mod typed_handle;
mod typed_single_handle;

mod unique_handle;
mod unique_single_handle;

mod sliced_handle;
mod sliced_single_handle;

pub use wrappers::*;

pub use typed_metadata::*;
pub use typed_handle::*;
pub use typed_single_handle::*;

pub use unique_handle::*;
pub use unique_single_handle::*;

pub use sliced_handle::*;
pub use sliced_single_handle::*;

/// A representation of a copy region used by storages' `..._region...` variants
/// 
/// # Safety
/// 
/// When using a copy region, the following guarantees need to be honored:
/// - `src_offset` points to a valid location in the source memory, meaning that it is less than the size of the source memory region.
/// - `dst_offset` points to a valid location in the destination memory, meaning that it is less than the size of the destination memory region.
/// - `size` must not exceed the size of either the source or destination memory sizes.
/// - `src_offset + copy_size` must not overflow the memory region to copy from, meaning that it is less than or equal to the size of the source memory region.
/// - `dst_offset + copy_size` must not overflow the memory region to copy from, meaning that it is less than or equal to the size of the destination memory region.
/// - As `src_offset` and `dst_offset` must point to valid memory, and the size of allocated cannot overflow 'isize', as per 'Layout`'s guarantees,
///   neither `src_offset`, nor `dst_offset` will overflow `isize`
/// - As `size` must represent a range in valid memory, and the size of allocated cannot overflow 'isize', as per 'Layout`'s guarantees, `size` will overflow `isize`.
#[derive(Clone, Copy, Debug)]
pub struct CopyRegion {
    /// Offset in the memory to copy from.
    pub src_offset: usize,
    /// Offset in the memory to copy to.
    pub dst_offset: usize,
    /// Size of the memory region to copy
    pub size:       usize,
}

impl CopyRegion {
    /// Create a memory region for a typed region.
    /// 
    /// `src_offset`, `dst_offset`, and `num_elements` are all defined in terms of elements of type `T`.
    /// 
    /// # Safety
    /// 
    /// The resulting region must adhere to the guarantees of `CopyRegion`, requiring the inputs to have the following guarantees:
    /// - `src_offset` points to a valid element in the source memory, meaning that it is less than the number of elements in the source memory region.
    /// - `dst_offset` points to a valid element in the destination memory, meaning that it is less than the number of elements of the destination memory region.
    /// - `copy_size` must not exceed the size of either the number of elements in the source or destination memory.
    /// - `src_offset + copy_size` must not overflow the memory region to copy from, meaning that it is less than or equal to the number of elements in the source memory.
    /// - `dst_offset + copy_size` must not overflow the memory region to copy from, meaning that it is less than or equal to the number of elements in the destination memory.
    pub const fn new_typed<T>(src_offset: usize, dst_offset: usize, num_elements: usize) -> Self {
        let src_offset = src_offset * core::mem::size_of::<T>();
        let dst_offset = dst_offset * core::mem::size_of::<T>();
        let size = num_elements * core::mem::size_of::<T>();

        Self {
            src_offset,
            dst_offset,
            size: todo!(),
        }
    }

    /// Convert a copy region defined in elements to a copy region defined in bytes
    /// 
    /// # Safety
    /// 
    /// - `self` is guaranteed to be currently defined in elements of type `T`.
    pub const unsafe fn to_typed<T>(self) -> Self {
        let src_offset = self.src_offset * core::mem::size_of::<T>();
        let dst_offset = self.dst_offset * core::mem::size_of::<T>();
        let size = self.size * core::mem::size_of::<T>();

        Self {
            src_offset,
            dst_offset,
            size: todo!(),
        }
    }
}

/// A base trait for differnt storage types, introcducing the handle type, and the ability to allocate dangling handles.
/// 
/// THis trzit is separate from the main `Storage` trait to allow `const StorageBase` implementation even when the `Storage` implementation themselves cannot be `const`.
#[const_trait]
pub unsafe trait StorageBase {
    /// A handle to the underlying memory managed by the storage.
    type Handle: Copy;

    /// Does the storage use minimum size optimization, this means if something like a container container can assume thatp
    /// they can allocate a minimum number of elements to more optimally allocate an initial block of memory?
    const USE_MIN_SIZE_OPTIMIZE: bool;

    /// Creates a dangling handle.
    /// 
    /// The only methods of a storage which may be called with a dangling handle are the `resolve` and `resolve_mut` methods.
    /// The poiner so obtained is guaranteed to be at least algined according to `alignment`, though it remains invalid and cannot be derefereced.
    /// 
    /// For all other purposes, a dangling handle is never valid, and thus cannot be deallocated, grown, nor shrunk.
    /// Furthermore there is no explicit way to distinguish whether a handle is dnalging or not.
    /// It is up to the user to remember whether a given handle is dangling, valid, or used to be valid but was invalidated
    fn dangling(&self, alignment: Alignment) -> Result<Self::Handle, AllocError>;
}

// TODO: Can this be coherced with Storage by requiring something other than a `NonNull<u8>`
/// A trait abstracting a generic memory storage, where a handle can represent a disjoint memory space split over multiple regions.
/// 
/// This trait return handles to allcoated memory, which can be freely copied and stored, then resolved into actual pointers at later time.
/// 
/// This trait requires a type that define the granularity of disjoint memory spaces, meaning that each distinct disjoint space is required to fit a multiple of `size_of::<T>()` elements.
/// 
/// # Safety
/// 
/// Only valid handles may be safely resolved.
/// When a handle is invalidated, all its copies are also invalidated at the same time, and all pointers resolved from the handle or any of its copies are invalidated as well.
/// 
/// Handle invalidation:
/// 
/// - A handle is immediately invalidated when used as an argument to the `Storage::deallocate` method.
/// - A handle is invalidated when used as an argument to the `Self::grow...` and `Self::shrink...` family of methods and these methods succeed.
/// 
/// Handle conversion:
/// 
/// - A handle can be converted into a `Self::DisjointPtr`, for example via `Into` or `TryInto`, and the handle is valid, possibly dangling,
///   then the resulting pointer must be equal to the result of the `Storage::resolve`and obtaining this pointer must NOT invalid any other handle.
/// - If a handle can be created from a `Self::DisjointPt`, for example via `From`, or `TryFrom`,
///   then the resulting handle is a copy of the handle which resolved into the `Self::DisjointPt` in the first place.
/// 
/// Pointer invalidation:
/// 
/// All pointers resolved by an instance of `Storage`:
/// - may be invalidated when dropping this instance of `Storage`.
/// - may be invalidated when moving this instance of `Storage`.
/// - may be invalidated when calling `Storage::allocate`, `Storage::deallocate`, `Storage::grow`, and `Storage::shrink`, or their variants.
///   Pointers are only guaranteed to remain valid acrosss those calls for instance also implement `StorageStable`.
/// - from a _different_ handle may be invalidiated when calling `Store::resolve`.
///   Pointers from different handles are only guaranteed to remain valid across those calls for instances also implementing `StorageStable`.
/// 
/// Memory guarantees:
/// - A handle will always refer to a disjoint memory space where each allocation is a multiple of `size_of::<T>()`
///
/// A specific implementation of `Storage` may provide extended validity guarantees, and should implement the extend guarantees traits when it does so.
#[const_trait]
pub unsafe trait Storage: StorageBase {
    /// Resolves a `handle` into a `Self::DisjointPtr` to a disjoint region of memory.
    /// 
    /// Unless `self` implements `StorageStable`, all previously resolved ponters from different handles may be invalidated.
    /// 
    /// # Safety
    /// 
    /// - `handle` must be allocated by self, or a shared storage (see `StorageShared`).
    /// - `handle` must still be valid.
    /// - The resulting `DisjointPtr` is only valid for as long as the handle is valid itself, and may be invalidated sooner, see [Pointer Invalidation].
    unsafe fn resolve(&self, handle: Self::Handle) -> NonNull<u8>;

    /// Attempts to allocate a block of memory.
    /// 
    /// On success, returns a `Handle` to a block of memory meeting the size and alignment guarantees of `Layout` and actual size of the block of memory.
    /// 
    /// # Errors
    /// 
    /// Returning `Err` indicates that either the memory is exhausted, or the storage cannot satisfy `layout` constraints.
    fn allocate(&self, layout: Layout) -> Result<(Self::Handle, usize), AllocError>;

    /// Attempts to allocate a block of memory, but unlike `allocate`, it also ensures that the associated block of memory is zero-initialized.
    /// 
    /// On success, returns a `Handle` to a block of memory meeting the size and alignment guarantees of `Layout` and actual size of the block of memory.
    /// 
    /// Returning `Err` indicates that either the memory is exhausted, or the storage cannot satisfy `layout` constraints.
    fn allocate_zeroed(&self, layout: Layout) -> Result<(Self::Handle, usize), AllocError> {
        let Ok((handle, size)) = self.allocate(layout) else { return Err(AllocError); };

        // Safety:
        // - `handle` has been allocated by `self`.
        // - `handle` is still valid, since no operation was performed on `self`.
        let pointer = unsafe { self.resolve(handle) };

        // Safety:
        // - `pointer` is valid, since `handle`` is valid.
        // - `pointer` points to an area of at least `size` bytes.
        // - Access to the next `size` bytes is exclusive.
        unsafe { pointer.write_bytes(0, size) };

        Ok((handle, size))
    }

    /// Deallocates the memory referenced by `handle`.
    /// 
    /// This invalidates `handle` and all its copies, as well as all pointers resolved from `handle` or any of its copies.
    /// 
    /// Unless `self` implements `StorageStable`, all previously resolved pointers may be invalidated.
    /// 
    /// # Safety
    /// 
    /// - `handle` must have been allocated by `self`, or a shared storage (see `StorageShared`).
    /// - `handle` mus still be valid.
    /// - `layout` must fit the associated block of memory.
    unsafe fn deallocate(&self, handle: Self::Handle, layout: Layout);

    /// Attempts to extend the block of memory associated with `handle`.
    /// 
    /// On success, returns a new `Self::Handle` associated with the extended block of memory,
    /// and may invalidate `handle` and all its copies, as well as all pointers resolved from `handle` or any of its copies.
    /// 
    /// On failure, 'handle' and all its copies are still valid, though any pointer resolved from `handle` or any of its copies may have been invalidated.
    /// 
    /// Unless `self` implements `StorageStable`, all previously resolved pinters may be invalidated.
    /// 
    /// # Safety
    /// 
    /// - `handle` must have been allocated by `self`, or a shared storage, see `StorageShared`.
    /// - `handle` must still be valid.
    /// - `old_layout` must fit the associated block of memory.
    /// - `new_layout.size()` must be greater than or equal to `old_layout.size()`.
    /// 
    /// # Errors
    /// 
    /// Returning `Err` indicated that either the memory is exhausted, or the store cannot satisfy `new_layout` constraints.
    unsafe fn grow(&self, handle: Self::Handle, old_layout: Layout, new_layout: Layout) -> Result<(Self::Handle, usize), AllocError> {
        // FIXME(const): Add message when available in const contexts
        debug_assert!(new_layout.size() > old_layout.size());
        
        // FIXME(const): Use `?` when available in const contexts
        let Ok((new_handle, new_size)) = self.allocate(new_layout) else { return Err(AllocError) };

        // Safety:
        // - `handle` has been allocated by `self` or a shared storage, according to the pre-conditions of `grow_region`
        // - `handle` is valid, as it was valid at the beginning of this function as per the pre-conditions of `grow_region`,
        //   and has not been invalidated by `self.allocate` since `self` can store multiple allocations
        let current_ptr = self.resolve(new_handle);

        // Safety:
        // - `new_handle` has been allocated by `self`
        // - `new_handle` is still valid, since only `self.resolve` was called which doesn't invalidate handles.
        let new_ptr = self.resolve(handle);

        // Safety:
        // - `current_ptr` is valid for reads, as `handle` is valid.
        // - `new_ptr` is valid for writes, as `handle` is valid _and_ exclusive access  is guaranteed.
        // - `current_ptr` and `new_ptr` are valid for `old_layout.size()` bytes, as per the pre-conditons of `grow`.
        current_ptr.copy_to_nonoverlapping(new_ptr, old_layout.size());

        // Safety:
        // - `handle` has been allocated by `self` or a shared `Storage`, as per the pre-conditions of `grow_region`
        // - `handle` is valid, as it was valid at beginning of this function as per the pre-conditions of `grow_region`
        // - `old_layout` gits `handle`, as per the pre-conditions of `grow_region`
        self.deallocate(handle, old_layout);

        Ok((new_handle, new_size))
    }

    /// Behaves like `grow`, but also ensure that the associated block of memory past the `copy_size` is zero-initialized
    /// 
    /// # Safety
    /// 
    /// See `grow`.
    /// 
    /// # Errors
    ///  
    /// Returning `Err` indicated that either the memory is exhausted, or the store cannot satisfy `new_layout` constraints.
    unsafe fn grow_zeroed(&self, handle: Self::Handle, old_layout: Layout, new_layout: Layout) -> Result<(Self::Handle, usize), AllocError> {
        // Safety:
        // - All preconditions of `grow` are preconditions of `grow_zeroed`.
        let Ok((handle, new_size)) = self.grow(handle, old_layout, new_layout) else { return Err(AllocError); };

        // Safety:
        // - `handle` has been allocated by `self`
        // - `handle` is still valid, since no operation was performed on `self`.
        let pointer = self.resolve(handle);

        let zero_offset = old_layout.size();

        // Safety:
        // - Both starting and resulting pointers are in bounds of the same allocated objects as `old_layout` fits `pointer`, as per the pre-conditions of `grow_copy_zeroed`
        // - The offset does not overflow `isize` as `old_layout.size()` does not.
        let pointer = pointer.add(zero_offset);

        // Safety
        // - `pointer` is valid, since `handle` is valid.
        // - `pointer` points to an area of at least `new_size - copy_size`.
        // - Access to the next `new_size - dst_offset` bytes is exclusive.
        pointer.write_bytes(0, new_size - zero_offset);

        Ok((handle, new_size))
    }

    /// Behaves like `grow`, but also allows the region of memory that needs to be copied, and the destination offset for it to be copied to to be passed,
    /// allowing the amount of data needing to be copied to be minimized, and thus avoiding unnecessary copies.
    /// 
    /// # Safety
    /// 
    /// See `grow`, with the additional contraint of:
    /// 
    /// - `region` must adhere to the containts specified by `CopyRegion` for source (current), and desination (extended) memory regions.
    /// 
    /// # Errors
    /// 
    /// Returning `Err` indicated that either the memory is exhausted, or the store cannot satisfy `new_layout` constraints.
    unsafe fn grow_region(&self, handle: Self::Handle, old_layout: Layout, new_layout: Layout, region: CopyRegion) -> Result<(Self::Handle, usize), AllocError> {
        // FIXME(const): Add message when available in const contexts
        debug_assert!(new_layout.size() > old_layout.size());
        debug_assert!(region.src_offset + region.size <= old_layout.size());
        debug_assert!(region.dst_offset + region.size <= new_layout.size());
        
        // FIXME(const): Use `?` when available in const contexts
        let Ok((new_handle, new_size)) = self.allocate(new_layout) else { return Err(AllocError) };

        // Safety:
        // - `handle` has been allocated by `self` or a shared storage, according to the pre-conditions of `grow_region`
        // - `handle` is valid, as it was valid at the beginning of this function as per the pre-conditions of `grow_region`,
        //   and has not been invalidated by `self.allocate` since `self` can store multiple allocations
        let current_ptr = self.resolve(new_handle);

        // Safety:
        // - `new_handle` has been allocated by `self`
        // - `new_handle` is still valid, since only `self.resolve` was called which doesn't invalidate handles.
        let new_ptr = self.resolve(handle);

        // Safety:
        // - Both starting and resulting pointers are in bounds of the same allocated objects as `region.offset` fits `current_ptr`, as per the pre-conditions of `grow_region`.
        // - The offset does not overflow `isize` as `region.src_offset` does not.
        let current_ptr = current_ptr.add(region.src_offset);

        // Safety:
        // - Both starting and resulting pointers are in bounds of the same allocated objects as `dst_offset` fits `new_ptr`, as per the pre-conditions of `grow_region`.
        // - The offset does not overflow `isize` as `region.dst_offset` does not.
        let new_ptr = new_ptr.add(region.dst_offset);

        // Safety:
        // - `current_ptr` is valid for reads, as `handle` is valid.
        // - `new_ptr` is valid for writes, as `handle` is valid _and_ exclusive access  is guaranteed.
        // - `current_ptr` and `new_ptr` are valid for `region.size` bytes, as per the pre-conditons of `grow_region`.
        current_ptr.copy_to_nonoverlapping(new_ptr, region.size);

        // Safety:
        // - `handle` has been allocated by `self` or a shared `Storage`, as per the pre-conditions of `grow_region`
        // - `handle` is valid, as it was valid at beginning of this function as per the pre-conditions of `grow_region`
        // - `old_layout` gits `handle`, as per the pre-conditions of `grow_region`
        self.deallocate(handle, old_layout);

        Ok((new_handle, new_size))
    }

    /// Behaves like `grow_region`, but also ensure that the associated block of memory outside the copied `region` is zero-initialized
    /// 
    /// # Safety
    /// 
    /// See `grow_region`.
    /// 
    /// # Errors
    ///  
    /// Returning `Err` indicated that either the memory is exhausted, or the store cannot satisfy `new_layout` constraints.
    unsafe fn grow_region_zeroed(&self, handle: Self::Handle, old_layout: Layout, new_layout: Layout, region: CopyRegion) -> Result<(Self::Handle, usize), AllocError> {
        // Safety:
        // - All preconditions of `grow_region` are preconditions of `grow_region_zeroed`.
        let Ok((handle, new_size)) = self.grow_region(handle, old_layout, new_layout, region) else { return Err(AllocError); };

        // Safety:
        // - `handle` has been allocated by `self`
        // - `handle` is still valid, since no operation was performed on `self`.
        let pointer = self.resolve(handle);

        // Clear 'head' bytes

        // Safety
        // - `pointer` is valid, since `handle` is valid.
        // - `pointer` points to an area of at least `dst_offset`.
        // - Access to the next `dst_offset` bytes is exclusive.
        // - `write_bytes` can safely handle an count of 0
        pointer.write_bytes(0, region.dst_offset);
        
        // Clear 'tail' bytes

        // Safety:
        // - Both starting and resulting pointers are in bounds of the same allocated objects as defined by `region`, and therefore fit `pointer`, as per the pre-conditions of `grow_copy_zeroed`
        // - The offset does not overflow `isize` as `region.dst_offset` does not.
        let pointer = pointer.add(region.dst_offset);

        // Safety
        // - `pointer` is valid, since `handle` is valid.
        // - `pointer` points to an area of at least `new_size - copy_size`.
        // - Access to the next `new_size - dst_offset` bytes is exclusive.
        pointer.write_bytes(0, new_size - region.size - region.dst_offset);

        Ok((handle, new_size))
    }

    /// Attempts to shrink the block of memory associated with `handle`.
    /// 
    /// On success, returns a new `Self::Handle` assocaited whti the shrunked block of memory,
    /// and may invalidate `handle` and all its copies, as well as all pointers resolved from `handle` or any of its copies.
    /// 
    /// On failure, `handle` and all tis copies are still valid, though any pointer resolved from `handle` or any of its copies may have been invalidated.
    /// 
    /// Unless `self` implements `StoreStable`, all previuosly resolved pointes may be invalidated.
    /// 
    /// # Safety
    /// 
    /// - `handle` must have been allocated by `self` or a shared `Storage`.
    /// - `handle` must still be valid.
    /// - `old_layout` must fit the associated block of memory.
    /// - `new_layout.size()` must be smalle than or equal to `old_layout.size()`.
    /// 
    /// # Errors
    /// 
    /// Returning `Err` indicated that either the memory is exhausted, or the store cannot satisfy `new_layout` constraints.
    unsafe fn shrink(&self, handle: Self::Handle, old_layout: Layout, new_layout: Layout) -> Result<(Self::Handle, usize), AllocError> {
        // FIXME(const): Add message when available in const contexts
        debug_assert!(new_layout.size() < old_layout.size());

        // FIXME(const): Use `?` when available in const contexts
        let Ok((new_handle, new_size)) = self.allocate(new_layout) else { return Err(AllocError); };

        // Safety
        // - `handle` has been allocated by `self` or a shared `Storage`, as per the pre-conditions of `shrink_copy`.
        // - `handle` is valid, as it was valid at the beginning of this function as the pre-conditons of `shink_copy`
        //   and has not been invalidated by `self.allocate` since self can store mulitple allocations.
        let current_ptr = self.resolve(handle);

        // Safety:
        // - `new_handle` has been allocated by `self`.
        // - `new_handle` is still valid, since only `self.resolve` was called which doesn't invalidate handles.
        let new_ptr = self.resolve(new_handle);

        // Safety:
        // - `current_ptr` is valid for reads, as `handle` is valid.
        // - `new_ptr` is valid for writes, as `handle` is valid _and_ exclusive access  is guaranteed.
        // - `current_ptr` and `new_ptr` are valid for `region.size` bytes, as per the pre-conditons of `grow_region`.
        current_ptr.copy_to_nonoverlapping(new_ptr, new_size);

        // Safety:
        // - `handle` has been allocated by `self` or a shared `Storage`, as per the pre-conditions of `shink_copy`
        // - `handle` is valid, as it was valid at beginning of this function as per the pre-conditions of `shink_copy`
        // - `old_layout` gits `handle`, as per the pre-conditions of `shink_copy`
        self.deallocate(handle, old_layout);

        Ok((new_handle, new_size))
    }

    /// Behaves like `shrink`, but also allows the region of memory that needs to be copied, and the destination offset for it to be copied to to be passed,
    /// allowing the amount of data needing to be copied to be minimized, and thus avoiding unnecessary copies.
    /// 
    /// # Safety
    /// 
    /// See `shrink`, with the additional contraint of:
    /// 
    /// - `copy_size` must be larger than 0, as otherwise, this should be handled by deallocating the current handle and allocating a new one.
    /// - `src_offset` must be a valid location within the original memory block.
    /// - `src_offset + copy_size` must be smaller or equal to `old_layout.size()`.
    /// - `dst_offset` must be a valid location within the extended memory block.
    /// - `dst_offset + copy_size` must be samller or equal to `new_layout.size()`.
    /// 
    /// # Errors
    /// 
    /// Returning `Err` indicated that either the memory is exhausted, or the store cannot satisfy `new_layout` constraints.
    unsafe fn shrink_region(&self, handle: Self::Handle, old_layout: Layout, new_layout: Layout, region: CopyRegion) -> Result<(Self::Handle, usize), AllocError> {
        // FIXME(const): Add message when available in const contexts
        debug_assert!(new_layout.size() < old_layout.size());
        debug_assert!(region.size > 0);
        debug_assert!(region.src_offset + region.size <= old_layout.size());
        debug_assert!(region.dst_offset + region.size <= new_layout.size());

        // FIXME(const): Use `?` when available in const contexts
        let Ok((new_handle, new_size)) = self.allocate(new_layout) else { return Err(AllocError); };

        // Safety
        // - `handle` has been allocated by `self` or a shared `Storage`, as per the pre-conditions of `shrink_copy`.
        // - `handle` is valid, as it was valid at the beginning of this function as the pre-conditons of `shink_copy`
        //   and has not been invalidated by `self.allocate` since self can store mulitple allocations.
        let current_ptr = self.resolve(handle);

        // Safety:
        // - `new_handle` has been allocated by `self`.
        // - `new_handle` is still valid, since only `self.resolve` was called which doesn't invalidate handles.
        let new_ptr = self.resolve(new_handle);

        // Safety:
        // - Both starting and resulting pointers are in bounds of the same allocated objects as `region.offset` fits `current_ptr`, as per the pre-conditions of `grow_region`.
        // - The offset does not overflow `isize` as `region.src_offset` does not.
        let current_ptr = current_ptr.add(region.src_offset);

        // Safety:
        // - Both starting and resulting pointers are in bounds of the same allocated objects as `dst_offset` fits `new_ptr`, as per the pre-conditions of `grow_region`.
        // - The offset does not overflow `isize` as `region.dst_offset` does not.
        let new_ptr = new_ptr.add(region.dst_offset);

        // Safety:
        // - `current_ptr` is valid for reads, as `handle` is valid.
        // - `new_ptr` is valid for writes, as `handle` is valid _and_ exclusive access  is guaranteed.
        // - `current_ptr` and `new_ptr` are valid for `region.size` bytes, as per the pre-conditons of `grow_region`.
        current_ptr.copy_to_nonoverlapping(new_ptr, region.size);

        // Safety:
        // - `handle` has been allocated by `self` or a shared `Storage`, as per the pre-conditions of `shink_copy`
        // - `handle` is valid, as it was valid at beginning of this function as per the pre-conditions of `shink_copy`
        // - `old_layout` gits `handle`, as per the pre-conditions of `shink_copy`
        self.deallocate(handle, old_layout);

        Ok((new_handle, new_size))
    }

    
    /// Behaves like `shrink_region`, but also ensure that the associated block of memory outside the copied `region` is zero-initialized
    /// 
    /// # Safety
    /// 
    /// See `shrink_region`.
    /// 
    /// # Errors
    ///  
    /// Returning `Err` indicated that either the memory is exhausted, or the store cannot satisfy `new_layout` constraints.
    unsafe fn shrink_region_zeroed(&self, handle: Self::Handle, old_layout: Layout, new_layout: Layout, region: CopyRegion) -> Result<(Self::Handle, usize), AllocError> {
        // Safety:
        // - All preconditions of `grow_copy` are preconditions of `grow_copy_zeroed`.
        let Ok((handle, new_size)) = self.grow_region(handle, old_layout, new_layout, region) else { return Err(AllocError); };

        // Safety:
        // - `handle` has been allocated by `self`
        // - `handle` is still valid, since no operation was performed on `self`.
        let pointer = self.resolve(handle);

        // Clear 'head' bytes

        // Safety
        // - `pointer` is valid, since `handle` is valid.
        // - `pointer` points to an area of at least `dst_offset`.
        // - Access to the next `dst_offset` bytes is exclusive.
        // - `write_bytes` can safely handle an count of 0
        pointer.write_bytes(0, region.dst_offset);
        
        // Clear 'tail' bytes

        // Safety:
        // - Both starting and resulting pointers are in bounds of the same allocated objects as defined by `region`, and therefore fit `pointer`, as per the pre-conditions of `grow_copy_zeroed`
        // - The offset does not overflow `isize` as `region.dst_offset` does not.
        let pointer = pointer.add(region.dst_offset);

        // Safety
        // - `pointer` is valid, since `handle` is valid.
        // - `pointer` points to an area of at least `new_size - copy_size`.
        // - Access to the next `new_size - dst_offset` bytes is exclusive.
        pointer.write_bytes(0, new_size - region.size - region.dst_offset);

        Ok((handle, new_size))
    }
}

/// A refinement of `Storage` specialized for storing a slice, with the size of the slice encoded within the handle,
/// which can reduce the space required to store a slice by not having to store a `usize` next to it.
/// 
/// # Safety
/// 
/// Size invalidation
/// 
/// All sizes resolved by an instance of `StorageSized`:
/// - may be invalidated when dropping this instance of `StorageSized`.
/// - may be invalidated when moving this instance of `StorageSized`
/// - may be invalidated when calling `Storage::allocate`, `Storage::deallocate`, `Storage::grow`, and `Storage::shrink`, or their variants.
///   Sizes are only guaranteed to remain valid acrosss those calls for instance also implement `StorageStable`.
#[const_trait]
pub unsafe trait StorageSliced: Storage {
    /// Resolves the `handle` to the size of the underlying memory.
    /// 
    /// This does not invalidate any previously resolved pointers, even when `StorageStable` is not implemented.
    /// 
    /// Any invalid handle is required to return a size of 0.
    /// 
    /// # Safety
    /// 
    /// - `handle` must have been allcoated by self, or a shared storage.
    /// - `handle` must still be valid.
    /// - The resulting size is only valid for as long as the `handle` is valid itself, and may be invalidated sooner
    unsafe fn resolve_size(&self, handle: Self::Handle) -> usize;

    /// Resolves the `handle` into a pointer to the first byte of the associated block of memory, and its size.
    /// 
    /// Unless `self` implements `StorageStable`, all previously resolved pointers from different handles may be invalidated.
    /// 
    /// _Note: see `resolve_mut` for mutable dereferenceable pointers._
    /// 
    /// # Safety
    /// 
    /// - `handle` must have been allocated by `self`.
    /// - `handle` must still be valid.
    /// - The resulting pointer is only valid for as long as the `handle` is valid itself, and may be invalidated sooner, see [Pointer Invalidation].
    unsafe fn resolve_sliced(&self, handle: Self::Handle) -> (NonNull<u8>, usize);
}

// TODO: Can this be coherced with Storage by requiring something other than a `NonNull<u8>`
/// A trait abstracting a generic memory storage, where a handle can represent a disjoint memory space split over multiple regions.
/// 
/// This trait return handles to allcoated memory, which can be freely copied and stored, then resolved into actual pointers at later time.
/// 
/// This trait requires a type that define the granularity of disjoint memory spaces, meaning that each distinct disjoint space is required to fit a multiple of `size_of::<T>()` elements.
/// 
/// # Safety
/// 
/// Only valid handles may be safely resolved.
/// When a handle is invalidated, all its copies are also invalidated at the same time, and all pointers resolved from the handle or any of its copies are invalidated as well.
/// 
/// Handle invalidation:
/// 
/// - A handle is immediately invalidated when used as an argument to the `Storage::deallocate` method.
/// - A handle is invalidated when used as an argument to the `Self::grow...` and `Self::shrink...` family of methods and these methods succeed.
/// 
/// Handle conversion:
/// 
/// - A handle can be converted into a `Self::DisjointPtr`, for example via `Into` or `TryInto`, and the handle is valid, possibly dangling,
///   then the resulting pointer must be equal to the result of the `Storage::resolve`and obtaining this pointer must NOT invalid any other handle.
/// - If a handle can be created from a `Self::DisjointPt`, for example via `From`, or `TryFrom`,
///   then the resulting handle is a copy of the handle which resolved into the `Self::DisjointPt` in the first place.
/// 
/// Pointer invalidation:
/// 
/// All pointers resolved by an instance of `Storage`:
/// - may be invalidated when dropping this instance of `Storage`.
/// - may be invalidated when moving this instance of `Storage`.
/// - may be invalidated when calling `Storage::allocate`, `Storage::deallocate`, `Storage::grow`, and `Storage::shrink`, or their variants.
///   Pointers are only guaranteed to remain valid acrosss those calls for instance also implement `StorageStable`.
/// - from a _different_ handle may be invalidiated when calling `Store::resolve`.
///   Pointers from different handles are only guaranteed to remain valid across those calls for instances also implementing `StorageStable`.
/// 
/// Memory guarantees:
/// - A handle will always refer to a disjoint memory space where each allocation is a multiple of `size_of::<T>()`
///
/// A specific implementation of `Storage` may provide extended validity guarantees, and should implement the extend guarantees traits when it does so.
#[const_trait]
pub unsafe trait StorageSingle: StorageBase {
    /// Resolves a `handle` into a `Self::DisjointPtr` to a disjoint region of memory.
    /// 
    /// Unless `self` implements `StorageStable`, all previously resolved ponters from different handles may be invalidated.
    /// 
    /// _Note: see `resolve_mut` for mutable dereferenceable pointers._
    /// 
    /// # Safety
    /// 
    /// - `handle` must be allocated by self, or a shared storage (see `StorageShared`).
    /// - `handle` must still be valid.
    /// - The resulting `DisjointPtr` is only valid for as long as the `handle`` is valid itself, and may be invalidated sooner, see [Pointer Invalidation].
    unsafe fn resolve(&self, handle: Self::Handle) -> NonNull<u8>;

    /// Resolved the `handle` into a `Self::DisjointPtr` to the first byte of the associated memory block.
    /// 
    /// # Safety
    /// 
    /// - `handle` must have been allocated by `self`.
    /// - `handle` must still be valid.
    /// - The resulting `DisjointPtr` is only valid for as long as the `handle` is valid itself, and may be invalidated sooner, see [Pointer Invalidation].
    unsafe fn resolve_mut(&mut self, handle: Self::Handle) -> NonNull<u8>;

    /// Attempts to allocate a block of memory.
    /// 
    /// On success, returns a `Handle` to a block of memory meeting the size and alignment guarantees of `Layout` and actual size of the block of memory.
    /// 
    /// # Errors
    /// 
    /// Returning `Err` indicates that either the memory is exhausted, or the storage cannot satisfy `layout` constraints.
    fn allocate(&mut self, layout: Layout) -> Result<(Self::Handle, usize), AllocError>;

    /// Attempts to allocate a block of memory, but unlike `allocate`, it also ensures that the associated block of memory is zero-initialized.
    /// 
    /// On success, returns a `Handle` to a block of memory meeting the size and alignment guarantees of `Layout` and actual size of the block of memory.
    /// 
    /// Returning `Err` indicates that either the memory is exhausted, or the storage cannot satisfy `layout` constraints.
    fn allocate_zeroed(&mut self, layout: Layout) -> Result<(Self::Handle, usize), AllocError> {
        let Ok((handle, size)) = self.allocate(layout) else { return Err(AllocError); };

        // Safety:
        // - `handle` has been allocated by `self`.
        // - `handle` is still valid, since no operation was performed on `self`.
        let pointer = unsafe { self.resolve_mut(handle) };

        // Safety:
        // - `pointer` is valid, since `handle`` is valid.
        // - `pointer` points to an area of at least `size` bytes.
        // - Access to the next `size` bytes is exclusive.
        unsafe { pointer.write_bytes(0, size) };

        Ok((handle, size))
    }

    /// Deallocates the memory referenced by `handle`.
    /// 
    /// This invalidates `handle` and all its copies, as well as all pointers resolved from `handle` or any of its copies.
    /// 
    /// Unless `self` implements `StorageStable`, all previously resolved pointers may be invalidated.
    /// 
    /// # Safety
    /// 
    /// - `handle` must have been allocated by `self`, or a shared storage (see `StorageShared`).
    /// - `handle` mus still be valid.
    /// - `layout` must fit the associated block of memory.
    unsafe fn deallocate(&mut self, handle: Self::Handle, layout: Layout);

    /// Attempts to extend the block of memory associated with `handle`.
    /// 
    /// On success, returns a new `Self::Handle` associated with the extended block of memory,
    /// and may invalidate `handle` and all its copies, as well as all pointers resolved from `handle` or any of its copies.
    /// 
    /// On failure, 'handle' and all its copies are still valid, though any pointer resolved from `handle` or any of its copies may have been invalidated.
    /// 
    /// Unless `self` implements `StorageStable`, all previously resolved pinters may be invalidated.
    /// 
    /// # Safety
    /// 
    /// - `handle` must have been allocated by `self`, or a shared storage, see `StorageShared`.
    /// - `handle` must still be valid.
    /// - `old_layout` must fit the associated block of memory.
    /// - `new_layout.size()` must be greater than or equal to `old_layout.size()`.
    /// 
    /// # Errors
    /// 
    /// Returning `Err` indicated that either the memory is exhausted, or the store cannot satisfy `new_layout` constraints.
    unsafe fn grow(&mut self, handle: Self::Handle, old_layout: Layout, new_layout: Layout) -> Result<(Self::Handle, usize), AllocError>;

    /// Behaves like `grow`, but also ensure that the associated block of memory past the `copy_size` is zero-initialized
    /// 
    /// # Safety
    /// 
    /// See `grow`.
    /// 
    /// # Errors
    ///  
    /// Returning `Err` indicated that either the memory is exhausted, or the store cannot satisfy `new_layout` constraints.
    unsafe fn grow_zeroed(&mut self, handle: Self::Handle, old_layout: Layout, new_layout: Layout) -> Result<(Self::Handle, usize), AllocError> {
        // Safety:
        // - All preconditions of `grow` are preconditions of `grow_zeroed`.
        let Ok((handle, new_size)) = self.grow(handle, old_layout, new_layout) else { return Err(AllocError); };

        // Safety:
        // - `handle` has been allocated by `self`
        // - `handle` is still valid, since no operation was performed on `self`.
        let pointer = self.resolve_mut(handle);

        let zero_offset = old_layout.size();

        // Safety:
        // - Both starting and resulting pointers are in bounds of the same allocated objects as `old_layout` fits `pointer`, as per the pre-conditions of `grow_copy_zeroed`
        // - The offset does not overflow `isize` as `old_layout.size()` does not.
        let pointer = pointer.add(zero_offset);

        // Safety
        // - `pointer` is valid, since `handle` is valid.
        // - `pointer` points to an area of at least `new_size - copy_size`.
        // - Access to the next `new_size - dst_offset` bytes is exclusive.
        pointer.write_bytes(0, new_size - zero_offset);

        Ok((handle, new_size))
    }

    /// Behaves like `grow`, but also allows the region of memory that needs to be copied, and the destination offset for it to be copied to to be passed,
    /// allowing the amount of data needing to be copied to be minimized, and thus avoiding unnecessary copies.
    /// 
    /// # Safety
    /// 
    /// See `grow`, with the additional contraint of:
    /// 
    /// - `region` must adhere to the containts specified by `CopyRegion` for source (current), and desination (extended) memory regions.
    /// 
    /// # Errors
    /// 
    /// Returning `Err` indicated that either the memory is exhausted, or the store cannot satisfy `new_layout` constraints.
    unsafe fn grow_region(&mut self, handle: Self::Handle, old_layout: Layout, new_layout: Layout, region: CopyRegion) -> Result<(Self::Handle, usize), AllocError>;

    /// Behaves like `grow_region`, but also ensure that the associated block of memory outside the copied `region` is zero-initialized
    /// 
    /// # Safety
    /// 
    /// See `grow_region`.
    /// 
    /// # Errors
    ///  
    /// Returning `Err` indicated that either the memory is exhausted, or the store cannot satisfy `new_layout` constraints.
    unsafe fn grow_region_zeroed(&mut self, handle: Self::Handle, old_layout: Layout, new_layout: Layout, region: CopyRegion) -> Result<(Self::Handle, usize), AllocError> {
        // Safety:
        // - All preconditions of `grow_region` are preconditions of `grow_region_zeroed`.
        let Ok((handle, new_size)) = self.grow_region(handle, old_layout, new_layout, region) else { return Err(AllocError); };

        // Safety:
        // - `handle` has been allocated by `self`
        // - `handle` is still valid, since no operation was performed on `self`.
        let pointer = self.resolve_mut(handle);

        // Clear 'head' bytes

        // Safety
        // - `pointer` is valid, since `handle` is valid.
        // - `pointer` points to an area of at least `dst_offset`.
        // - Access to the next `dst_offset` bytes is exclusive.
        // - `write_bytes` can safely handle an count of 0
        pointer.write_bytes(0, region.dst_offset);
        
        // Clear 'tail' bytes

        // Safety:
        // - Both starting and resulting pointers are in bounds of the same allocated objects as defined by `region`, and therefore fit `pointer`, as per the pre-conditions of `grow_copy_zeroed`
        // - The offset does not overflow `isize` as `region.dst_offset` does not.
        let pointer = pointer.add(region.dst_offset);

        // Safety
        // - `pointer` is valid, since `handle` is valid.
        // - `pointer` points to an area of at least `new_size - copy_size`.
        // - Access to the next `new_size - dst_offset` bytes is exclusive.
        pointer.write_bytes(0, new_size - region.size - region.dst_offset);

        Ok((handle, new_size))
    }

    /// Attempts to shrink the block of memory associated with `handle`.
    /// 
    /// On success, returns a new `Self::Handle` assocaited whti the shrunked block of memory,
    /// and may invalidate `handle` and all its copies, as well as all pointers resolved from `handle` or any of its copies.
    /// 
    /// On failure, `handle` and all tis copies are still valid, though any pointer resolved from `handle` or any of its copies may have been invalidated.
    /// 
    /// Unless `self` implements `StoreStable`, all previuosly resolved pointes may be invalidated.
    /// 
    /// # Safety
    /// 
    /// - `handle` must have been allocated by `self` or a shared `Storage`.
    /// - `handle` must still be valid.
    /// - `old_layout` must fit the associated block of memory.
    /// - `new_layout.size()` must be smalle than or equal to `old_layout.size()`.
    /// 
    /// # Errors
    /// 
    /// Returning `Err` indicated that either the memory is exhausted, or the store cannot satisfy `new_layout` constraints.
    unsafe fn shrink(&mut self, handle: Self::Handle, old_layout: Layout, new_layout: Layout) -> Result<(Self::Handle, usize), AllocError>;

    /// Behaves like `shrink`, but also allows the region of memory that needs to be copied, and the destination offset for it to be copied to to be passed,
    /// allowing the amount of data needing to be copied to be minimized, and thus avoiding unnecessary copies.
    /// 
    /// # Safety
    /// 
    /// See `shrink`, with the additional contraint of:
    /// 
    /// - `copy_size` must be larger than 0, as otherwise, this should be handled by deallocating the current handle and allocating a new one.
    /// - `src_offset` must be a valid location within the original memory block.
    /// - `src_offset + copy_size` must be smaller or equal to `old_layout.size()`.
    /// - `dst_offset` must be a valid location within the extended memory block.
    /// - `dst_offset + copy_size` must be samller or equal to `new_layout.size()`.
    /// 
    /// # Errors
    /// 
    /// Returning `Err` indicated that either the memory is exhausted, or the store cannot satisfy `new_layout` constraints.
    unsafe fn shrink_region(&mut self, handle: Self::Handle, old_layout: Layout, new_layout: Layout, region: CopyRegion) -> Result<(Self::Handle, usize), AllocError>;

    
    /// Behaves like `shrink_region`, but also ensure that the associated block of memory outside the copied `region` is zero-initialized
    /// 
    /// # Safety
    /// 
    /// See `shrink_region`.
    /// 
    /// # Errors
    ///  
    /// Returning `Err` indicated that either the memory is exhausted, or the store cannot satisfy `new_layout` constraints.
    unsafe fn shrink_region_zeroed(&mut self, handle: Self::Handle, old_layout: Layout, new_layout: Layout, region: CopyRegion) -> Result<(Self::Handle, usize), AllocError> {
        // Safety:
        // - All preconditions of `grow_copy` are preconditions of `grow_copy_zeroed`.
        let Ok((handle, new_size)) = self.grow_region(handle, old_layout, new_layout, region) else { return Err(AllocError); };

        // Safety:
        // - `handle` has been allocated by `self`
        // - `handle` is still valid, since no operation was performed on `self`.
        let pointer = self.resolve_mut(handle);

        // Clear 'head' bytes

        // Safety
        // - `pointer` is valid, since `handle` is valid.
        // - `pointer` points to an area of at least `dst_offset`.
        // - Access to the next `dst_offset` bytes is exclusive.
        // - `write_bytes` can safely handle an count of 0
        pointer.write_bytes(0, region.dst_offset);
        
        // Clear 'tail' bytes

        // Safety:
        // - Both starting and resulting pointers are in bounds of the same allocated objects as defined by `region`, and therefore fit `pointer`, as per the pre-conditions of `grow_copy_zeroed`
        // - The offset does not overflow `isize` as `region.dst_offset` does not.
        let pointer = pointer.add(region.dst_offset);

        // Safety
        // - `pointer` is valid, since `handle` is valid.
        // - `pointer` points to an area of at least `new_size - copy_size`.
        // - Access to the next `new_size - dst_offset` bytes is exclusive.
        pointer.write_bytes(0, new_size - region.size - region.dst_offset);

        Ok((handle, new_size))
    }
}

/// A trait specializing `StorageSingle` to also include the size of the allocation within the handle, which could allow types using storage to reduce space requirements.
/// 
/// # Safety
/// 
/// Size invalidation
/// 
/// All sizes resolved by an instance of `StorageSized`:
/// - may be invalidated when dropping this instance of `StorageSized`.
/// - may be invalidated when moving this instance of `StorageSized`
/// - may be invalidated when calling `Storage::allocate`, `Storage::deallocate`, `Storage::grow`, and `Storage::shrink`, or their variants.
///   Sizes are only guaranteed to remain valid acrosss those calls for instance also implement `StorageStable`.
#[const_trait]
pub unsafe trait StorageSingleSliced: StorageSingle {
    /// Resolves the `handle` to the size of the underlying memory.
    /// 
    /// This does not invalidate any previously resolved pointers, even when not implementing `StorageStable`.
    /// 
    /// Any invalid handle is required to return a size of 0.
    /// 
    /// # Safety
    /// 
    /// - `handle` must have been allcoated by self, or a shared storage.
    /// - `handle` must still be valid.
    /// - The resulting size is only valid for as long as the `handle` is valid itself, and may be invalidated sooner.
    unsafe fn resolve_size(&self, handle: Self::Handle) -> usize;

    /// Resolves the `handle` into a pointer to the first byte of the associated block of memory, and its size.
    /// 
    /// Unless `self` implements `StorageStable`, all previously resolved pointers from different handles may be invalidated.
    /// 
    /// _Note: see `resolve_mut` for mutable dereferenceable pointers._
    /// 
    /// # Safety
    /// 
    /// - `handle` must have been allocated by `self`.
    /// - `handle` must still be valid.
    /// - The resulting pointer is only valid for as long as the `handle` is valid itself, and may be invalidated sooner, see [Pointer Invalidation].
    unsafe fn resolve_sliced(&self, handle: Self::Handle) -> (NonNull<u8>, usize);

    /// Resolved the `handle` into a pointer to the first byte of the associated memory block and its size.
    /// 
    /// Unless `self` implements `StorageStable`, all previously resolved pointers from different handles may be invalidated.
    /// 
    /// # Safety
    /// 
    /// - `handle` must have been allocated by `self`.
    /// - `handle` must still be valid.
    /// - The resulting pointer is only valid for as long as the `handle` is valid itself, and may be invalidated sooner, see [Pointer Invalidation].
    unsafe fn resolve_mut_sliced(&mut self, handle: Self::Handle) -> (NonNull<u8>, usize);
}

/// A refinement of a `Storage` which guaranteees which guarantees that the block of memory are stable in memory across method calls, but not necessarily across moves.
/// 
/// If the blocks of memory should be stable in memory across moves as well, then `StorePinning` is required.
/// 
/// # Safety
/// 
/// Implementers of this trait must guarantee that a handle always resolves to the same block of memory for as long as it is valid and the instance of the store is not moved.
pub unsafe trait StorageStable {}

/// A refinement of a store which guuarantees that the blocks of memory are pinned in memory
/// 
/// # Safety
/// 
/// Implementers of this must guarantee that a handle alwyas resolve to the same block of memory for as long as it is valid, in particular even after the instance of the store was moved.
/// 
/// As a corralary, forgetting the instance of a storage -- which is moving without dropping -- means that he resolved pointers will remain pinned
/// until either the instance of the store is recovered (from scratch) and dropped,
/// or until the lifetime bound of the `Storage` concrete type (if not `static`) expires, whichever comes first.
pub unsafe trait StoragePinning: StorageStable {}
 
/// A refinement of `StoragePinning` which allows multiple instances to share the handles and their associated blocks of memory.
/// 
/// Normally, a handle created by one instance of `Storage` cannot be used in  any way with another, different, instance of `Storage`.
/// This trait lifts this restriction _partly_ by created sets of sharing stores.
/// In essence, all storages belonging to the same set of sharing stores can be considrered "parts" of a single storage:
/// all handles created by one "part" can be used with any other "part", and the store is not dropped until all its "parts" are dropped.
/// 
/// A set of sharing storages is effectively the equivalient of a `Rc<Storage>` or `Arc<Storage>`
/// 
/// # Safety
/// 
/// Implementers of this trait must guarantee that a handle created by one part of a sharing set may be used with any other part: resolved, deallocated, grown, or shrunk.
pub unsafe trait StorageSharing: StoragePinning {
    /// Error returned if sharing is not currently possible
    type SharingError;

    /// Returns whether two instances belong to the same sharing set.
    /// 
    /// The implementation is permitted to return `false` even if the two instances do, belong to the same sharing set.
    /// This method is only meant to allow users who lost track of whether the implementation are sharing to possibly recover this piece of information.
    fn is_sharing_with(&self, other: &Self) -> bool;

    /// Creates a new instance of `Storage` belonging to the same sharing set as `self`.
    fn share(&self) -> Result<Self, Self::SharingError> where
        Self: Sized;
}
