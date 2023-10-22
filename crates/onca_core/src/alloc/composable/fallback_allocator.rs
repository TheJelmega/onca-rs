use std::{alloc::Layout, ptr::NonNull};

use crate::alloc::*;

/// Fallback allocator
/// 
/// An allocator that will first try to allocate memory using its main allocator, if that fails, the allocator will fallback on its secondary allocator
pub struct FallbackAllocator<A: Allocator, F: Allocator> {
    main:     A,
    fallback: F,
    id:      u16
}

impl <A: Allocator, F: Allocator> FallbackAllocator<A, F> {
    /// Create a new fallback allocator
    /// 
    /// `alloc` denotes the main allocator
    /// 
    /// `fallback` denotes the secondary allocator to use when the main allocator fails to allocate the memory
    pub fn new(main: A, fallback: F) -> Self {
        Self{ main, fallback, id: 0 }
    }
}

impl <A: Allocator, F: Allocator> Allocator for FallbackAllocator<A, F> {
    unsafe fn alloc(&mut self, layout: Layout) -> Option<NonNull<u8>> {
        self.main.alloc(layout).map_or_else(|| self.fallback.alloc(layout), |ptr| Some(ptr))
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        if self.main.owns(ptr, layout) {
            self.main.dealloc(ptr, layout);
        } else if self.fallback.owns(ptr, layout) {
            self.fallback.dealloc(ptr, layout);
        } else {
            panic!("Cannot deallocate an allocation that isn't owned by the allocator");
        }
    }

    fn owns(&self, ptr: NonNull<u8>, layout: Layout) -> bool {
        self.main.owns(ptr, layout) || self.fallback.owns(ptr, layout)
    }

    fn set_alloc_id(&mut self, id: u16) {
        self.id = id;
    }

    fn alloc_id(&self) -> u16 {
        self.id
    }
}

unsafe impl<A: Allocator + Sync, F: Allocator + Sync> Sync for FallbackAllocator<A, F> {}