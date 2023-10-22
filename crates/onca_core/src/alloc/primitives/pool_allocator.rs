use core::{ 
    mem::size_of, 
    ptr::null_mut, 
    sync::atomic::{AtomicPtr, Ordering}
};
use std::{ptr::NonNull, alloc::Layout};

use crate::{
    alloc::{Allocator, ComposableAllocator, AllocId},
    mem::{self, AllocInitState, get_memory_manager},
};

struct Header {
    next: *mut Header
}

/// Linear/Bump allocator
/// 
/// An allocator that can freely allocate when there is enough space left in it, but it cannot deallocate,
/// deallocation only takes place for all allocations at once in `reset()`
pub struct PoolAllocator {
    buffer:        NonNull<u8>,
    buffer_layout: Layout,
    head:          AtomicPtr<Header>,
    block_size:    usize,
    id:            u16
}

impl PoolAllocator {
    /// Create a new pool allocator
    /// 
    /// The `block_size` needs to be a power of 2, and larger than the size of a known-size pointer
    /// 
    /// The size of the provided buffer needs to be a multiple of the `block_size`
    pub fn new(mut buffer: NonNull<u8>, buffer_layout: Layout, block_size: usize) -> Self {
        assert!(block_size.is_power_of_two(), "Block size needs to be a power of 2");
        assert!(block_size >= size_of::<Header>(), "Block size needs to be larger than the size of a pointer");
        assert!(buffer_layout.size() & block_size == 0, "The provided buffer needs to have a size that is a multiple of the block size");

        let num_blocks = buffer_layout.size() / block_size;

        let mut header = buffer.as_ptr().cast::<Header>();
        let head_step = block_size / size_of::<Header>();
        for i in 0..num_blocks - 1 {
            unsafe {
                let next = header.add(head_step);
                (*header).next = next;
                header = next;
            }
        }

        let head = buffer.as_ptr().cast::<Header>();
        Self { buffer, buffer_layout, head: AtomicPtr::new(head), block_size, id: 0 }
    }
}

impl Allocator for PoolAllocator {
    unsafe fn alloc(&mut self, layout: Layout) -> Option<NonNull<u8>> {

        if layout.align() > self.block_size {
            return None;
        }

        if self.head.load(Ordering::Relaxed) == null_mut() {
            return None; 
        }

        let mut head = self.head.load(Ordering::Relaxed);
        while head != null_mut() {
            
            let next = (*head).next;

            match self.head.compare_exchange_weak(head, next, Ordering::AcqRel, Ordering::Acquire) {
                Ok(ptr) => return NonNull::new(ptr.cast()),
                Err(ptr) => head = ptr
            }
        }
        None
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        assert!(self.owns(ptr, layout), "Cannot deallocate an allocation that isn't owned by the allocator");

        let header = ptr.as_ptr().cast::<Header>();
        loop {
            let mut cur_head = self.head.load(Ordering::Relaxed);
            (*header).next = cur_head;

            match self.head.compare_exchange_weak(cur_head, header, Ordering::AcqRel, Ordering::Acquire) {
                Ok(_) => return,
                Err(new_cur_head) => cur_head = new_cur_head,
            }
        }
    }

    fn owns(&self, ptr: NonNull<u8>, _ayout: Layout) -> bool {
        let end = unsafe { self.buffer.as_ptr().add(self.buffer_layout.size()) };
        ptr >= self.buffer && ptr.as_ptr() <= end
    }

    fn set_alloc_id(&mut self, id: u16) {
        self.id = id;
    }

    fn alloc_id(&self) -> u16 {
        self.id
    }
}

impl ComposableAllocator<(usize, usize)> for PoolAllocator {
    fn new_composable(args: (usize, usize)) -> Self {
        let buffer_layout = Layout::from_size_align(args.0, 8).expect("Invalid `PoolAllocator::new_composable` parameters");
        let buffer = unsafe { get_memory_manager().alloc_raw(AllocInitState::Uninitialized, buffer_layout, None).expect("Failed to allocate memory for composable allocator") };
        PoolAllocator::new(buffer, buffer_layout, args.1)
    }
}

impl Drop for PoolAllocator {
    fn drop(&mut self) {
        unsafe { get_memory_manager().dealloc(self.buffer, self.buffer_layout) };
    }
}

unsafe impl Sync for PoolAllocator {}

#[cfg(test)]
mod tests {
    use std::alloc::Layout;

    use crate::alloc::{*, primitives::*};

    #[test]
    fn alloc_dealloc() {
        unsafe {
            let mut base_alloc = Mallocator;
            let buffer_layout = Layout::from_size_align_unchecked(258, 8);

            let buffer = unsafe { base_alloc.alloc(buffer_layout).unwrap() };
            let mut alloc = PoolAllocator::new(buffer, buffer_layout, 8);

            let layout = Layout::new::<u64>();
            let ptr = alloc.alloc(layout).unwrap();
            alloc.dealloc(ptr, layout);
        }
    }

    #[test]
    fn multi_allocs() {
        unsafe {
            let mut base_alloc = Mallocator;
            let buffer_layout = Layout::from_size_align_unchecked(258, 8);

            let buffer = unsafe { base_alloc.alloc(buffer_layout).unwrap() };
            let mut alloc = PoolAllocator::new(buffer, buffer_layout, 8);

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

    #[test]
    fn dealloc_then_realloc() {
        unsafe {
            let mut base_alloc = Mallocator;
            let buffer_layout = Layout::from_size_align_unchecked(258, 8);

            let buffer = unsafe { base_alloc.alloc(buffer_layout).unwrap() };
            let mut alloc = PoolAllocator::new(buffer, buffer_layout, 8);

            let layout0 = Layout::new::<u16>();
            let layout1 = Layout::new::<u64>();
            let layout2 = Layout::new::<u32>();

            let ptr0 = alloc.alloc(layout0).unwrap();

            let ptr1 = alloc.alloc(layout1).unwrap();
            let ptr2 = alloc.alloc(layout2).unwrap();

            alloc.dealloc(ptr0, layout0);

            let new_ptr = alloc.alloc(layout0).unwrap();
            assert_eq!(ptr0, new_ptr);
        }
    }

    #[test]
    fn alloc_too_large() {
        struct Large { a: u64, b: u64 }

        unsafe {
            let mut base_alloc = Mallocator;
            let buffer_layout = Layout::from_size_align_unchecked(258, 8);

            let buffer = unsafe { base_alloc.alloc(buffer_layout).unwrap() };
            let mut alloc = PoolAllocator::new(buffer, buffer_layout, 8);

            let ptr = alloc.alloc(Layout::new::<Large>());
            match ptr {
                None => {},
                Some(_) => panic!()
            }
        }
    }
}