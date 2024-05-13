use core::alloc::{AllocError, Layout};
use std::ptr::NonNull;

use super::{CopyRegion, StorageBase, StoragePinning, StorageSingle, StorageStable};


pub struct StorageSingleSizedWrapper<T: StorageSingle> {
    storage: T,
    size:    usize,
}

impl<T: StorageSingle> StorageSingleSizedWrapper<T> {
    pub fn new(storage: T) -> Self {
        Self { storage, size: 0 }
    }
}

unsafe impl<T: StorageSingle> StorageBase for StorageSingleSizedWrapper<T> {
    type Handle = T::Handle;
    
    const USE_MIN_SIZE_OPTIMIZE: bool = T::USE_MIN_SIZE_OPTIMIZE;

    fn dangling(&self, alignment: std::ptr::Alignment) -> Result<Self::Handle, AllocError> {
        self.storage.dangling(alignment)
    }
}

unsafe impl<T: StorageSingle> StorageSingle for StorageSingleSizedWrapper<T> {
    unsafe fn resolve(&self, handle: Self::Handle) -> NonNull<u8> {
        self.storage.resolve(handle)
    }

    unsafe fn resolve_mut(&mut self, handle: Self::Handle) -> NonNull<u8> {
        self.storage.resolve_mut(handle)
    }

    fn allocate(&mut self, layout: Layout) -> Result<(Self::Handle, usize), AllocError> {
        let (handle, size) = self.storage.allocate(layout)?;
        self.size = size;
        Ok((handle, size))
    }

    fn allocate_zeroed(&mut self, layout: Layout) -> Result<(Self::Handle, usize), AllocError> {
        let (handle, size) = self.storage.allocate_zeroed(layout)?;
        self.size = size;
        Ok((handle, size))
    }

    unsafe fn deallocate(&mut self, handle: Self::Handle, layout: Layout) {
        self.storage.deallocate(handle, layout);
        self.size = 0;
    }

    unsafe fn grow(&mut self, handle: Self::Handle, old_layout: Layout, new_layout: Layout) -> Result<(Self::Handle, usize), AllocError> {
        let (handle, size) = self.storage.grow(handle, old_layout, new_layout)?;
        self.size = size;
        Ok((handle, size))
    }

    unsafe fn grow_zeroed(&mut self, handle: Self::Handle, old_layout: Layout, new_layout: Layout) -> Result<(Self::Handle, usize), AllocError> {
        let (handle, size) = self.storage.grow_zeroed(handle, old_layout, new_layout)?;
        self.size = size;
        Ok((handle, size))
    }

    unsafe fn grow_region(&mut self, handle: Self::Handle, old_layout: Layout, new_layout: Layout, region: CopyRegion) -> Result<(Self::Handle, usize), AllocError> {
        let (handle, size) = self.storage.grow_region(handle, old_layout, new_layout, region)?;
        self.size = size;
        Ok((handle, size))
    }

    unsafe fn grow_region_zeroed(&mut self, handle: Self::Handle, old_layout: Layout, new_layout: Layout, region: CopyRegion) -> Result<(Self::Handle, usize), AllocError> {
        let (handle, size) = self.storage.grow_region_zeroed(handle, old_layout, new_layout, region)?;
        self.size = size;
        Ok((handle, size))
    }

    unsafe fn shrink(&mut self, handle: Self::Handle, old_layout: Layout, new_layout: Layout) -> Result<(Self::Handle, usize), AllocError> {
        let (handle, size) = self.storage.shrink(handle, old_layout, new_layout)?;
        self.size = size;
        Ok((handle, size))
    }

    unsafe fn shrink_region(&mut self, handle: Self::Handle, old_layout: Layout, new_layout: Layout, region: CopyRegion) -> Result<(Self::Handle, usize), AllocError> {
        let (handle, size) = self.storage.shrink_region(handle, old_layout, new_layout, region)?;
        self.size = size;
        Ok((handle, size))
    }

    unsafe fn shrink_region_zeroed(&mut self, handle: Self::Handle, old_layout: Layout, new_layout: Layout, region: CopyRegion) -> Result<(Self::Handle, usize), AllocError> {
        let (handle, size) = self.storage.shrink_region_zeroed(handle, old_layout, new_layout, region)?;
        self.size = size;
        Ok((handle, size))
    }
}

unsafe impl<T: StorageSingle + StorageStable> StorageStable for StorageSingleSizedWrapper<T> {}

unsafe impl<T: StorageSingle + StoragePinning> StoragePinning for StorageSingleSizedWrapper<T> {}