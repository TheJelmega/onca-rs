use core::{
    mem::size_of,
    ptr::{null, null_mut},
};
use std::{ptr::NonNull, alloc::Layout};
use crate::{
    alloc::Allocator,
    mem::get_memory_manager
};

/// Linear/Bump allocator
/// 
/// An allocator that can freely allocate when there is enough space left in it, but it cannot deallocate,
/// deallocation only takes place for all allocations at once in `reset()`
pub struct StackAllocator {
    max_align:     u16,
    buffer:        NonNull<u8>,
    buffer_layout: Layout,
    head:          *mut u8,
    end:           *mut u8,
    id:            u16
}

impl StackAllocator {
    /// Create a new stack allocator from a buffer and a maximum alignment for allocations
    pub fn new(mut buffer: NonNull<u8>, buffer_layout: Layout, max_align: u16) -> Self {
        debug_assert!(max_align.is_power_of_two());

        let head = buffer.as_ptr();
        let end = unsafe {
            buffer.as_ptr().add(buffer_layout.size())
        };

        Self { max_align, buffer, buffer_layout, head, end, id: 0 }
    }

    /// Reset the linear allocator to its empty state
    pub fn reset(&mut self) {
        self.head = self.buffer.as_ptr();
    }
}

impl Allocator for StackAllocator {
    unsafe fn alloc(&mut self, mut layout: Layout) -> Option<NonNull<u8>> {
        if layout.align() > self.max_align as usize {
            // Layout exceeds allocator's maximum alignment
            return None;
        }

        let ptr = self.head;
        let back_padding = layout.padding_needed_for(self.max_align as usize);
        let new_head = ptr.add(layout.size() + back_padding);

        if new_head.offset_from(self.end) >= 0 {
            None
        } else {
            self.head = new_head;
            NonNull::new(ptr)
        }
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, mut layout: Layout) {
        assert!(self.owns(ptr, layout), "Cannot deallocate an allocation that isn't owned by the allocator");

        let ptr_mut = ptr.as_ptr();
        let back_padding = layout.padding_needed_for(self.max_align as usize);
        let expected_head = ptr_mut.add(layout.size() + back_padding);

        assert!(expected_head == self.head, "Invalid deallocation order");

        if expected_head != self.head {
            // TODO: Warning
            return;
        }

        self.head = ptr_mut;
    }

    fn owns(&self, ptr: NonNull<u8>, _layout: Layout) -> bool {
        ptr >= self.buffer && ptr.as_ptr() <= self.end
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

            let buffer = unsafe { base_alloc.alloc(buffer_layout).unwrap() };
            let mut alloc = StackAllocator::new(buffer, buffer_layout, 8);

            let layout = Layout::new::<u64>();
            let ptr = alloc.alloc(layout).unwrap();
            alloc.dealloc(ptr, layout);
        }
    }

    #[test]
    fn align_too_large() {
        unsafe {
            let mut base_alloc = Mallocator;
            let buffer_layout = Layout::from_size_align_unchecked(256, 8);

            let buffer = unsafe { base_alloc.alloc(buffer_layout).unwrap() };
            let mut alloc = StackAllocator::new(buffer, buffer_layout, 8);

            let ptr = alloc.alloc(Layout::from_size_align_unchecked(8, 16));
            match ptr {
                None => {},
                Some(_) => panic!()
            }
        }
    }

    #[test]
    fn multi_allocs() {
        unsafe {
            let mut base_alloc = Mallocator;
            let buffer_layout = Layout::from_size_align_unchecked(256, 8);

            let buffer = unsafe { base_alloc.alloc(buffer_layout).unwrap() };
            let mut alloc = StackAllocator::new(buffer, buffer_layout, 8);

            let layout0 = Layout::new::<u16>();
            let layout1 = Layout::new::<u64>();
            let layout2 = Layout::new::<u32>();

            let ptr0 = alloc.alloc(layout0).unwrap();
            let ptr1 = alloc.alloc(layout1).unwrap();
            let ptr2 = alloc.alloc(layout2).unwrap();

            alloc.dealloc(ptr2, layout2);
            alloc.dealloc(ptr1, layout1);
            alloc.dealloc(ptr0, layout0);
        }
    }

    #[test]
    #[should_panic]
    fn invalid_dealloc_order() {
        unsafe {
            let mut base_alloc = Mallocator;
            let buffer_layout = Layout::from_size_align_unchecked(256, 8);

            let buffer = unsafe { base_alloc.alloc(buffer_layout).unwrap() };
            let mut alloc = StackAllocator::new(buffer, buffer_layout, 8);

            let layout0 = Layout::new::<u16>();
            let layout1 = Layout::new::<u64>();
            let layout2 = Layout::new::<u32>();

            let ptr0 = alloc.alloc(layout0).unwrap();
            let ptr1 = alloc.alloc(layout1).unwrap();
            let ptr2 = alloc.alloc(layout2).unwrap();

            alloc.dealloc(ptr1, layout1);
            alloc.dealloc(ptr0, layout0);
            alloc.dealloc(ptr2, layout2);
        }
    }
}