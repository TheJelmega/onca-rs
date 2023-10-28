use std::{ptr::NonNull, alloc::Layout};

use crate::alloc::*;

/// Segregator arena
/// 
/// An allocator arena that decides which allocator to used based on boundary value, which is inclusive for the first allocator.
pub struct SegregatorArena<A: Allocator, B: Allocator> {
    boundary: usize,
    le_alloc: A,
    gt_alloc: B,
    id:       u16
}

impl<A: Allocator, B: Allocator> SegregatorArena<A, B> {
    /// Create a new segregator arena
    /// 
    /// The `boundary` defines the maximum inclusive size to be allocated on the `le_alloc`, otherwise allocation will be done on the `gt_alloc`.
    pub fn new(boundary: usize, le_alloc: A, gt_alloc: B) -> SegregatorArena<A, B> {
        Self { boundary, le_alloc, gt_alloc, id: 0 }
    }
}

impl<A: Allocator, B: Allocator> Allocator for SegregatorArena<A, B> {
    unsafe fn alloc(&mut self, layout: Layout) -> Option<NonNull<u8>> {
        if layout.size() <= self.boundary {
            self.le_alloc.alloc(layout)
        } else {
            self.gt_alloc.alloc(layout)
        }
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        if layout.size() <= self.boundary {
            self.le_alloc.dealloc(ptr, layout);
        } else {
            self.gt_alloc.dealloc(ptr, layout);
        }
    }

    fn owns(&self, ptr: NonNull<u8>, layout: Layout) -> bool {
        if layout.size() <= self.boundary {
            self.le_alloc.owns(ptr, layout)
        } else {
            self.gt_alloc.owns(ptr, layout)
        }
    }

    fn set_alloc_id(&mut self, id: u16) {
        self.id = id;
        self.le_alloc.set_alloc_id(id);
        self.gt_alloc.set_alloc_id(id);
    }

    fn alloc_id(&self) -> u16 {
        self.id
    }
}

unsafe impl<A: Allocator + Sync, B: Allocator + Sync> Sync for SegregatorArena<A, B> {}