use core::{
    mem::size_of,
    ptr::{null, null_mut},
};
use crate::{alloc::{Allocation, Allocator, Layout, MemTag}, mem::MEMORY_MANAGER};

/// Linear/Bump allocator
/// 
/// An allocator that can freely allocate when there is enough space left in it, but it cannot deallocate,
/// deallocation only takes place for all allocations at once in `reset()`
pub struct StackAllocator {
    max_align : u16,
    buffer    : Allocation<u8>,
    head      : *mut u8,
    end       : *mut u8,
    id        : u16
}

impl StackAllocator {
    /// Create a uninitialized stack allocator
    pub const fn new_uninit(max_align: u16) -> Self {
        Self { max_align, buffer: unsafe { Allocation::const_null() }, head: null_mut(), end: null_mut(), id: 0 }
    }

    /// Create a new stack allocator from a buffer and a maximum alignment for allocations
    pub fn new(mut buffer: Allocation<u8>, max_align: u16) -> Self
    {
        let head = buffer.ptr_mut();
        let end = unsafe {
            buffer.ptr_mut().add(Allocation::<u8>::layout(&buffer).size())
        };

        Self { max_align, buffer, head, end, id: 0 }
    }

    /// Initialized an unitialized stack allocator
    pub fn init(&mut self, buffer: Allocation<u8>) {
        self.buffer = buffer;
        self.head = self.buffer.ptr_mut();
        self.end = unsafe {
            self.buffer.ptr_mut().add(Allocation::<u8>::layout(&self.buffer).size())
        };
    }

    /// Reset the linear allocator to its empty state
    pub fn reset(&mut self) {
        self.head = Allocation::<u8>::ptr_mut(&mut self.buffer);
    }

    /// Check if the allocator is initialized
    pub fn is_initialized(&self) -> bool {
        self.buffer.ptr() != null()
    }
}

impl Allocator for StackAllocator {
    unsafe fn alloc(&mut self, mut layout: Layout, mem_tag: MemTag) -> Option<Allocation<u8>> {
        assert!(self.is_initialized(), "Trying to allocate memory using an uninitialized stack allocator");

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
            Some(Allocation::<_>::from_raw(ptr, layout.with_alloc_id(self.id), mem_tag))
        }
    }

    unsafe fn dealloc(&mut self, ptr: Allocation<u8>) {
        assert!(self.owns(&ptr), "Cannot deallocate an allocation ({}) that isn't owned by the allocator ({})", ptr.layout().alloc_id(), self.id);

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
        MEMORY_MANAGER.dealloc(core::mem::replace(&mut self.buffer, unsafe { Allocation::const_null() }));
    }
}

#[cfg(test)]
mod tests {
    use crate::alloc::{*, primitives::*};

    #[test]
    fn alloc_dealloc() {
        let mut base_alloc = Mallocator;
        let buffer = unsafe { base_alloc.alloc(Layout::new_size_align(256, 8), CoreMemTag::test()).unwrap() };
        let mut alloc = StackAllocator::new(buffer, 8);

        unsafe {
            let ptr = alloc.alloc(Layout::new::<u64>(), CoreMemTag::test()).unwrap();
            alloc.dealloc(ptr);
        }
    }

    #[test]
    fn align_too_large() {
        let mut base_alloc = Mallocator;
        let buffer = unsafe { base_alloc.alloc(Layout::new_size_align(256, 8), CoreMemTag::test()).unwrap() };
        let mut alloc = StackAllocator::new(buffer, 8);

        unsafe {
            let ptr = alloc.alloc(Layout::new_size_align(8, 16), CoreMemTag::test());
            match ptr {
                None => {},
                Some(_) => panic!()
            }
        }
    }

    #[test]
    fn multi_allocs() {
        let mut base_alloc = Mallocator;
        let buffer = unsafe { base_alloc.alloc(Layout::new_size_align(256, 8), CoreMemTag::test()).unwrap() };
        let mut alloc = StackAllocator::new(buffer, 8);

        unsafe {
            let ptr0 = alloc.alloc(Layout::new::<u16>(), CoreMemTag::test()).unwrap();
            let ptr1 = alloc.alloc(Layout::new::<u64>(), CoreMemTag::test()).unwrap();
            let ptr2 = alloc.alloc(Layout::new::<u32>(), CoreMemTag::test()).unwrap();

            alloc.dealloc(ptr2);
            alloc.dealloc(ptr1);
            alloc.dealloc(ptr0);
        }
    }

    #[test]
    #[should_panic]
    fn invalid_dealloc_order() {
        let mut base_alloc = Mallocator;
        let buffer = unsafe { base_alloc.alloc(Layout::new_size_align(256, 8), CoreMemTag::test()).unwrap() };
        let mut alloc = StackAllocator::new(buffer, 8);

        unsafe {
            let ptr0 = alloc.alloc(Layout::new::<u16>(), CoreMemTag::test()).unwrap();
            let ptr1 = alloc.alloc(Layout::new::<u64>(), CoreMemTag::test()).unwrap();
            let ptr2 = alloc.alloc(Layout::new::<u32>(), CoreMemTag::test()).unwrap();

            alloc.dealloc(ptr1);
            alloc.dealloc(ptr0);
            alloc.dealloc(ptr2);
        }
    }
}