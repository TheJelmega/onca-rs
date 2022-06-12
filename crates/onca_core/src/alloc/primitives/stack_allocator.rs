use core::mem::size_of;
use crate::{alloc::{Allocation, Allocator, Layout}, mem::MEMORY_MANAGER};

/// Linear/Bump allocator
/// 
/// An allocator that can freely allocate when there is enough space left in it, but it cannot deallocate,
/// deallocation only takes place for all allocations at once in `reset()`
pub struct StackAllocator {
    max_align : u16,
    buffer : Allocation<u8>,
    head   : *mut u8,
    end    : *mut u8,
    id     : u16
}

impl StackAllocator {
    /// Create a new stack allocator from a buffer and a maximum alignment for allocations
    pub fn new(mut buffer: Allocation<u8>, max_align: u16) -> Self
    {
        let head = buffer.ptr_mut();
        let end = unsafe {
            buffer.ptr_mut().add(Allocation::<u8>::layout(&buffer).size())
        };

        Self { max_align, buffer, head, end, id: 0 }
    }
    /// Reset the linear allocator to its empty state
    pub fn reset(&mut self) {
        self.head = Allocation::<u8>::ptr_mut(&mut self.buffer);
    }
}

impl Allocator for StackAllocator {
    unsafe fn alloc(&mut self, mut layout: Layout) -> Option<Allocation<u8>> {
        if layout.align() > self.max_align as usize {
            // Layout exceeds allocator's maximum alignment
            return None;
        }

        let layout = layout.with_size_multiple_of(self.max_align as u64);
        let ptr = self.head;
        let new_head = ptr.add(layout.size());

        if new_head.offset_from(self.end) >= 0 {
            None
        } else {
            self.head = new_head;
            Some(Allocation::<_>::new(ptr, layout.with_alloc_id(self.id)))
        }
    }

    unsafe fn dealloc(&mut self, ptr: Allocation<u8>) {
        assert!(self.owns(&ptr), "Cannot deallocate an allocation that isn't owned by the allocator");

        let ptr_mut = ptr.ptr_mut();
        let expected_head = ptr_mut.add(ptr.layout().size());

        assert!(expected_head == self.head, "Invalid deallocation order");

        if expected_head != self.head {
            // TODO(jel): Warning
            return;
        }

        self.head = ptr_mut;
    }


    fn set_alloc_id(&mut self, id: u16) {
        self.id = id;
    }

    fn alloc_id(&self) -> u16 {
        self.id
    }
}

impl Drop for StackAllocator {
    fn drop(&mut self) {
        let dealloc_ptr = Allocation::<u8>::new(self.buffer.ptr_mut(), *self.buffer.layout());
        MEMORY_MANAGER.dealloc(dealloc_ptr);
    }
}

#[cfg(test)]
mod tests {
    use crate::alloc::{*, primitives::*};

    #[test]
    fn alloc_dealloc() {
        let mut base_alloc = Mallocator;
        let buffer = unsafe { base_alloc.alloc(Layout::new_size_align(256, 8)).unwrap() };
        let mut alloc = StackAllocator::new(buffer, 8);

        unsafe {
            let ptr = alloc.alloc(Layout::new::<u64>()).unwrap();
            alloc.dealloc(ptr);
        }
    }

    #[test]
    fn align_too_large() {
        let mut base_alloc = Mallocator;
        let buffer = unsafe { base_alloc.alloc(Layout::new_size_align(256, 8)).unwrap() };
        let mut alloc = StackAllocator::new(buffer, 8);

        unsafe {
            let ptr = alloc.alloc(Layout::new_size_align(8, 16));
            match ptr {
                None => {},
                Some(_) => panic!()
            }
        }
    }

    #[test]
    fn multi_allocs() {
        let mut base_alloc = Mallocator;
        let buffer = unsafe { base_alloc.alloc(Layout::new_size_align(256, 8)).unwrap() };
        let mut alloc = StackAllocator::new(buffer, 8);

        unsafe {
            let ptr0 = alloc.alloc(Layout::new::<u16>()).unwrap();
            let ptr1 = alloc.alloc(Layout::new::<u64>()).unwrap();
            let ptr2 = alloc.alloc(Layout::new::<u32>()).unwrap();

            alloc.dealloc(ptr2);
            alloc.dealloc(ptr1);
            alloc.dealloc(ptr0);
        }
    }

    #[test]
    #[should_panic]
    fn invalid_dealloc_order() {
        let mut base_alloc = Mallocator;
        let buffer = unsafe { base_alloc.alloc(Layout::new_size_align(256, 8)).unwrap() };
        let mut alloc = StackAllocator::new(buffer, 8);

        unsafe {
            let ptr0 = alloc.alloc(Layout::new::<u16>()).unwrap();
            let ptr1 = alloc.alloc(Layout::new::<u64>()).unwrap();
            let ptr2 = alloc.alloc(Layout::new::<u32>()).unwrap();

            alloc.dealloc(ptr1);
            alloc.dealloc(ptr0);
            alloc.dealloc(ptr2);
        }
    }
}