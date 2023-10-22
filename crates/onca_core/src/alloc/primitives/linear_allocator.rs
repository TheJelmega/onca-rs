use core::mem::size_of;
use std::{alloc::Layout, ptr::NonNull};
use crate::{alloc::{Allocator, AllocHeader}, mem::get_memory_manager};

/// Linear/Bump allocator
/// 
/// An allocator that can freely allocate when there is enough space left in it, but it cannot deallocate,
/// deallocation only takes place for all allocations at once in `reset()`
pub struct LinearAllocator {
    buffer:        NonNull<u8>,
    buffer_layout: Layout,
    head:          *mut u8,
    end:           *mut u8,
    id:            u16
}

impl LinearAllocator {
    /// Create a new stack allocator from a buffer
    pub fn new(mut buffer: NonNull<u8>, buffer_layout: Layout) -> Self
    {
        let head = buffer.as_ptr();
        let end = unsafe {
            buffer.as_ptr().add(buffer_layout.size())
        };

        Self { buffer, buffer_layout, head, end, id: 0 }
    }

    /// Reset the linear allocator to its empty state
    pub fn reset(&mut self) {
        self.head = self.buffer.as_ptr();
    }
}

impl Allocator for LinearAllocator {
    unsafe fn alloc(&mut self, layout: Layout) -> Option<NonNull<u8>> {
        let align = layout.align();
        let padding = self.head.align_offset(align);
        let aligned_ptr = self.head.add(padding);
        let new_head = aligned_ptr.add(layout.size());

        if new_head.offset_from(self.end) >= 0 {
            None
        } else {
            self.head = new_head;
            NonNull::new(aligned_ptr)
        }
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        assert!(self.owns(ptr, layout), "Cannot deallocate an allocation that isn't owned by the allocator");
        // No-op
    }

    fn owns(&self, ptr: NonNull<u8>, _layout: Layout) -> bool {
        ptr >= self.buffer && ptr.as_ptr() < self.end
    }

    fn set_alloc_id(&mut self, id: u16) {
        self.id = id;
    }

    fn alloc_id(&self) -> u16 {
        self.id
    }

    fn supports_free(&self) -> bool {
        false
    }
}

impl Drop for LinearAllocator {
    fn drop(&mut self) {
        unsafe { get_memory_manager().dealloc(self.buffer, self.buffer_layout) };
    }
}

#[cfg(test)]
mod tests {
    use std::alloc::Layout;

    use crate::alloc::{*, primitives::*};

    #[test]
    fn alloc_dealloc() {
        unsafe {
            let mut base_alloc = Mallocator;
            let buffer_layout = Layout::from_size_align_unchecked(256, 8);
            
            let buffer = base_alloc.alloc(buffer_layout).unwrap();
            let mut alloc = LinearAllocator::new(buffer, buffer_layout);

            let layout = Layout::new::<u64>();
            let ptr = alloc.alloc(layout).unwrap();
            alloc.dealloc(ptr, layout);
        }
    }

    #[test]
    fn multi_allocs() {
        unsafe {
            let mut base_alloc = Mallocator;
            let buffer_layout = Layout::from_size_align_unchecked(256, 8);

            let buffer = base_alloc.alloc(buffer_layout).unwrap();
            let mut alloc = LinearAllocator::new(buffer, buffer_layout);

            let layout0 = Layout::new::<u16>();
            let layout1 = Layout::new::<u64>();
            let layout2 = Layout::new::<u32>();

            let ptr0 = alloc.alloc(layout0).unwrap();
            let ptr1 = alloc.alloc(layout1).unwrap();
            let ptr2 = alloc.alloc(layout2).unwrap();

            alloc.dealloc(ptr0, layout0);
            alloc.dealloc(ptr1, layout1);
            alloc.dealloc(ptr2, layout2);
        }
    }
}