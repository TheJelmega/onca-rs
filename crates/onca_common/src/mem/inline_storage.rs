use std::{alloc::{AllocError, Layout}, mem::{size_of, ManuallyDrop}, ptr::NonNull};

use super::{CopyRegion, StorageBase, StorageSingle};




pub struct InlineStorage<T, const N: usize> {
    data: ManuallyDrop<[T; N]>,
}

unsafe impl<T, const N: usize> StorageBase for InlineStorage<T, N> {
    type Handle = ();

    const USE_MIN_SIZE_OPTIMIZE: bool = false;

    fn dangling(&self, alignment: std::ptr::Alignment) -> Result<Self::Handle, AllocError> {
        Ok(())
    }
}

unsafe impl<T, const N: usize> StorageSingle for InlineStorage<T, N> {
    unsafe fn resolve(&self, handle: Self::Handle) -> NonNull<u8> {
        let ptr = self.data.as_ptr();

        // Safety
        // - Inline memory is always valid.
        // - `resolve` guarantees that the memory will only be access immutably.
        NonNull::new_unchecked(ptr as *mut _)
    }

    unsafe fn resolve_mut(&mut self, handle: Self::Handle) -> NonNull<u8> {
        let ptr = self.data.as_mut_ptr();

        // Safety
        // - Inline memory is always valid.
        NonNull::new_unchecked(ptr.cast())
    }

    fn allocate(&mut self, layout: Layout) -> Result<(Self::Handle, usize), AllocError> {
        Ok(((), size_of::<T>() * N))
    }

    unsafe fn deallocate(&mut self, handle: Self::Handle, layout: Layout) {
        // Nothing to do here, as it's the users responsibility to make sure the data is dropped
    }

    unsafe fn grow(&mut self, handle: Self::Handle, old_layout: Layout, new_layout: Layout) -> Result<(Self::Handle, usize), AllocError> {
        Ok(((), size_of::<T>() * N))
    }

    unsafe fn grow_region(&mut self, handle: Self::Handle, old_layout: Layout, new_layout: Layout, region: CopyRegion) -> Result<(Self::Handle, usize), AllocError> {
        if new_layout.size() > size_of::<T>() * N {
            return Err(AllocError);
        }

        debug_assert!(new_layout.size() >= old_layout.size());

        if region.src_offset != 0 || region.dst_offset != 0 {
            // Debug checks to make sure all memory will remain within the bound of the elements
            debug_assert!((region.src_offset % size_of::<T>()) == 0);
            debug_assert!((region.dst_offset % size_of::<T>()) == 0);
            debug_assert!((region.size % size_of::<T>()) == 0);

            let src_ptr = self.data.as_ptr().byte_add(region.src_offset);
            let dst_ptr = self.data.as_mut_ptr().byte_add(region.dst_offset);
            core::ptr::copy(src_ptr, dst_ptr, region.size);
        }

        Ok(((), size_of::<T>() * N))
    }

    unsafe fn shrink(&mut self, handle: Self::Handle, old_layout: Layout, new_layout: Layout) -> Result<(Self::Handle, usize), AllocError> {
        Ok(((), size_of::<T>() * N))
    }

    unsafe fn shrink_region(&mut self, handle: Self::Handle, old_layout: Layout, new_layout: Layout, region: CopyRegion) -> Result<(Self::Handle, usize), AllocError> {
        debug_assert!(new_layout.size() <= old_layout.size());

        if region.src_offset != 0 || region.dst_offset != 0 {
            // Debug checks to make sure all memory will remain within the bound of the elements
            debug_assert!((region.src_offset % size_of::<T>()) == 0);
            debug_assert!((region.dst_offset % size_of::<T>()) == 0);
            debug_assert!((region.size % size_of::<T>()) == 0);

            let src_ptr = self.data.as_ptr().byte_add(region.src_offset);
            let dst_ptr = self.data.as_mut_ptr().byte_add(region.dst_offset);
            core::ptr::copy(src_ptr, dst_ptr, region.size);
        }

        Ok(((), size_of::<T>() * N))
    }
}