use core::{
    alloc::{AllocError, Allocator, Layout},
    ptr::NonNull
};

use super::{CopyRegion, Storage, StorageBase, StoragePinning, StorageSingle, StorageStable, StoreSharing};

#[derive(Clone, Copy, Debug)]
pub struct AllocStorage<A: Allocator> {
    alloc: A
}

impl<A: Allocator> AllocStorage<A> {
    pub fn new(alloc: A) -> Self {
        Self { alloc }
    }
}

unsafe impl<A: Allocator> StorageBase for AllocStorage<A> {
    type Handle = NonNull<u8>;

    const USE_MIN_SIZE_OPTIMIZE: bool = true;
    
    fn dangling(&self, alignment: std::ptr::Alignment) -> Result<Self::Handle, AllocError> {
        unsafe { Ok(NonNull::new_unchecked(core::ptr::dangling_mut())) }
    }
}

unsafe impl<A: Allocator> Storage for AllocStorage<A> {
    unsafe fn resolve(&self, handle: Self::Handle) -> NonNull<u8> {
        handle
    }

    fn allocate(&self, layout: Layout) -> Result<(Self::Handle, usize), AllocError> {
        self.alloc.allocate(layout).map(|ptr| (ptr.cast(), ptr.len()))
    }

    fn allocate_zeroed(&self, layout: Layout) -> Result<(Self::Handle, usize), AllocError> {
        self.alloc.allocate_zeroed(layout).map(|ptr| (ptr.cast(), ptr.len()))
    }

    unsafe fn deallocate(&self, handle: Self::Handle, layout: Layout) {
        self.alloc.deallocate(handle, layout)
    }

    unsafe fn grow(&self, handle: Self::Handle, old_layout: Layout, new_layout: Layout) -> Result<(Self::Handle, usize), AllocError> {
        self.alloc.grow(handle, old_layout, new_layout).map(|ptr| (ptr.cast(), ptr.len()))
    }

    unsafe fn grow_zeroed(&self, handle: Self::Handle, old_layout: Layout, new_layout: Layout) -> Result<(Self::Handle, usize), AllocError> {
        self.alloc.grow_zeroed(handle, old_layout, new_layout).map(|ptr| (ptr.cast(), ptr.len()))
    }
}

unsafe impl<A: Allocator> StorageSingle for AllocStorage<A> {
    unsafe fn resolve(&self, handle: Self::Handle) -> NonNull<u8> {
        handle
    }

    unsafe fn resolve_mut(&mut self, handle: Self::Handle) -> NonNull<u8> {
        handle
    }

    fn allocate(&mut self, layout: Layout) -> Result<(Self::Handle, usize), AllocError> {
        self.alloc.allocate(layout).map(|ptr| (ptr.cast(), ptr.len()))
    }

    fn allocate_zeroed(&mut self, layout: Layout) -> Result<(Self::Handle, usize), AllocError> {
        self.alloc.allocate_zeroed(layout).map(|ptr| (ptr.cast(), ptr.len()))
    }

    unsafe fn deallocate(&mut self, handle: Self::Handle, layout: Layout) {
        self.alloc.deallocate(handle, layout)
    }

    unsafe fn grow(&mut self, handle: Self::Handle, old_layout: Layout, new_layout: Layout) -> Result<(Self::Handle, usize), AllocError> {
        <Self as Storage>::grow(&self, handle, old_layout, new_layout)
    }

    unsafe fn grow_region(&mut self, handle: Self::Handle, old_layout: Layout, new_layout: Layout, region: CopyRegion) -> Result<(Self::Handle, usize), AllocError> {
        <Self as Storage>::grow_region(self, handle, old_layout, new_layout, region)
    }

    unsafe fn shrink(&mut self, handle: Self::Handle, old_layout: Layout, new_layout: Layout) -> Result<(Self::Handle, usize), AllocError> {
        <Self as Storage>::shrink(&self, handle, old_layout, new_layout)
    }

    unsafe fn shrink_region(&mut self, handle: Self::Handle, old_layout: Layout, new_layout: Layout, region: CopyRegion) -> Result<(Self::Handle, usize), AllocError> {
        <Self as Storage>::shrink_region(self, handle, old_layout, new_layout, region)
    }
}

unsafe impl<A: Allocator> StorageStable for AllocStorage<A> {}

unsafe impl<A: Allocator> StoragePinning for AllocStorage<A> {}

unsafe impl<A: Allocator + Clone> StoreSharing for AllocStorage<A> {
    type SharingError = ();

    fn is_sharing_with(&self, other: &Self) -> bool {
        true
    }

    fn share(&self) -> Result<Self, Self::SharingError> where
        Self: Sized
    {
        Ok(self.clone())
    }
}