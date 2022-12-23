use crate::alloc::*;

/// Fallback allocator
/// 
/// An allocator that will first try to allocate memory using its main allocator, if that fails, the allocator will fallback on its secondary allocator
pub struct FallbackAllocator<A: Allocator, F: Allocator> {
    main     : A,
    fallback : F
}

impl <A: Allocator, F: Allocator> FallbackAllocator<A, F> {
    /// Create a new fallback allocator
    /// 
    /// `alloc` denotes the main allocator
    /// 
    /// `fallback` denotes the secondary allocator to use when the main allocator fails to allocate the memory
    pub fn new(main: A, fallback: F) -> Self {
        Self{ main, fallback }
    }
}

impl <A: Allocator, F: Allocator> Allocator for FallbackAllocator<A, F> {
    unsafe fn alloc(&mut self, layout: Layout, mem_tag: MemTag) -> Option<Allocation<u8>> {
        let opt = self.main.alloc(layout, mem_tag);
        if let Some(ptr) = opt {
            Some(ptr)
        } else {
            self.fallback.alloc(layout, mem_tag)
        }
    }

    unsafe fn dealloc(&mut self, ptr: Allocation<u8>) {
        if self.main.owns(&ptr) {
            self.main.dealloc(ptr);
        } else if self.fallback.owns(&ptr) {
            self.fallback.dealloc(ptr)
        } else {
            panic!("Cannot deallocate an allocation ({}) that isn't owned by the allocator ({} or {})", ptr.layout().alloc_id(), self.main.alloc_id(), self.fallback.alloc_id());
        }
    }

    fn set_alloc_id(&mut self, id: u16) {
        // no-op
    }

    fn alloc_id(&self) -> u16 {
        0
    }
}

unsafe impl<A: Allocator + Sync, F: Allocator + Sync> Sync for FallbackAllocator<A, F> {}