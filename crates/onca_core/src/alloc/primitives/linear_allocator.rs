use core::mem::size_of;
use crate::{alloc::{Allocation, Allocator, Layout, MemTag}, mem::MEMORY_MANAGER};

/// Linear/Bump allocator
/// 
/// An allocator that can freely allocate when there is enough space left in it, but it cannot deallocate,
/// deallocation only takes place for all allocations at once in `reset()`
pub struct LinearAllocator {
    buffer : Allocation<u8>,
    head   : *mut u8,
    end    : *mut u8,
    id     : u16
}

impl LinearAllocator {
    /// Create a new stack allocator from a buffer
    pub fn new(mut buffer: Allocation<u8>) -> Self
    {
        let head = buffer.ptr_mut();
        let end = unsafe {
            buffer.ptr_mut().add(Allocation::<u8>::layout(&buffer).size())
        };

        Self { buffer, head, end, id: 0 }
    }

    /// Reset the linear allocator to its empty state
    pub fn reset(&mut self) {
        self.head = Allocation::<u8>::ptr_mut(&mut self.buffer);
    }
}

impl Allocator for LinearAllocator {
    unsafe fn alloc(&mut self, layout: Layout, mem_tag: MemTag) -> Option<Allocation<u8>> {
        let align = layout.align();
        let padding = self.head.align_offset(align);
        let aligned_ptr = self.head.add(padding);
        let new_head = aligned_ptr.add(layout.size());

        if new_head.offset_from(self.end) >= 0 {
            None
        } else {
            self.head = new_head;
            Some(Allocation::<_>::new(aligned_ptr, layout.with_alloc_id(self.id), mem_tag))
        }
    }

    unsafe fn dealloc(&mut self, ptr: Allocation<u8>) {
        assert!(self.owns(&ptr), "Cannot deallocate an allocation that isn't owned by the allocator");
        // No-op
    }

    fn set_alloc_id(&mut self, id: u16) {
        self.id = id;
    }

    fn alloc_id(&self) -> u16 {
        self.id
    }
}

impl Drop for LinearAllocator {
    fn drop(&mut self) {
        MEMORY_MANAGER.dealloc(core::mem::replace(&mut self.buffer, unsafe { Allocation::null() }));
    }
}

#[cfg(test)]
mod tests {
    use crate::alloc::{*, primitives::*};

    #[test]
    fn alloc_dealloc() {
        let mut base_alloc = Mallocator;
        let buffer = unsafe { base_alloc.alloc(Layout::new_size_align(256, 8), CoreMemTag::Test.to_mem_tag()).unwrap() };
        let mut alloc = LinearAllocator::new(buffer);

        unsafe {
            let ptr = alloc.alloc(Layout::new::<u64>(), CoreMemTag::Test.to_mem_tag()).unwrap();
            alloc.dealloc(ptr);
        }
    }

    #[test]
    fn multi_allocs() {
        let mut base_alloc = Mallocator;
        let buffer = unsafe { base_alloc.alloc(Layout::new_size_align(256, 8), CoreMemTag::Test.to_mem_tag()).unwrap() };
        let mut alloc = LinearAllocator::new(buffer);

        unsafe {
            let ptr0 = alloc.alloc(Layout::new::<u16>(), CoreMemTag::Test.to_mem_tag()).unwrap();
            let ptr1 = alloc.alloc(Layout::new::<u64>(), CoreMemTag::Test.to_mem_tag()).unwrap();
            let ptr2 = alloc.alloc(Layout::new::<u32>(), CoreMemTag::Test.to_mem_tag()).unwrap();

            alloc.dealloc(ptr0);
            alloc.dealloc(ptr1);
            alloc.dealloc(ptr2);
        }
    }
}