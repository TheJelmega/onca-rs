use core::{ 
    mem::size_of, 
    ptr::null_mut, 
    sync::atomic::{AtomicPtr, Ordering}};
use crate::{alloc::{Allocation, Allocator, Layout, ComposableAllocator}, mem::MEMORY_MANAGER};

struct Header {
    next: *mut Header
}

/// Linear/Bump allocator
/// 
/// An allocator that can freely allocate when there is enough space left in it, but it cannot deallocate,
/// deallocation only takes place for all allocations at once in `reset()`
pub struct PoolAllocator {
    buffer     : Allocation<u8>,
    head       : AtomicPtr<Header>,
    block_size : usize,
    id         : u16
}

impl PoolAllocator {
    /// Create a new pool allocator
    /// 
    /// The `block_size` needs to be a power of 2, and larger than the size of a known-size pointer
    /// 
    /// The size of the provided buffer needs to be a multiple of the `block_size`
    pub fn new(mut buffer: Allocation<u8>, block_size: usize) -> Self {
        assert!(block_size.is_power_of_two(), "Block size needs to be a power of 2");
        assert!(block_size >= size_of::<Header>(), "Block size needs to be larger than the size of a pointer");
        assert!(buffer.layout().size() & block_size == 0, "The provided buffer needs to have a size that is a multiple of the block size");

        let num_blocks = buffer.layout().size() / block_size;

        let mut header = buffer.ptr_mut().cast::<Header>();
        let head_step = block_size / size_of::<Header>();
        for i in 0..num_blocks - 1 {
            unsafe {
                let next = header.add(head_step);
                (*header).next = next;
                header = next;
            }
        }

        let head = buffer.ptr_mut().cast::<Header>();
        Self { buffer, head: AtomicPtr::new(head), block_size, id: 0 }
    }
}

impl Allocator for PoolAllocator {
    unsafe fn alloc(&mut self, layout: Layout) -> Option<Allocation<u8>> {

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
                Ok(ptr) => return Some(Allocation::<_>::new(ptr.cast::<u8>(), layout.with_alloc_id(self.id))),
                Err(ptr) => head = ptr
            }
        }
        None
    }

    unsafe fn dealloc(&mut self, ptr: Allocation<u8>) {
        assert!(self.owns(&ptr), "Cannot deallocate an allocation that isn't owned by the allocator");

        let header = ptr.ptr_mut().cast::<Header>();
        loop {
            let mut cur_head = self.head.load(Ordering::Relaxed);
            (*header).next = cur_head;

            match self.head.compare_exchange_weak(cur_head, header, Ordering::AcqRel, Ordering::Acquire) {
                Ok(_) => return,
                Err(new_cur_head) => cur_head = new_cur_head,
            }
        }
    }

    fn owns(&self, ptr: &Allocation<u8>) -> bool {
        ptr.ptr() >= self.buffer.ptr() && ptr.ptr() > unsafe { self.buffer.ptr().add(self.buffer.layout().size()) }
    }

    fn set_alloc_id(&mut self, id: u16) {
        self.id = id;
    }

    fn alloc_id(&self) -> u16 {
        self.id
    }
}

impl ComposableAllocator<(usize, usize)> for PoolAllocator {
    fn new_composable(alloc: &mut dyn Allocator, args: (usize, usize)) -> Self {
        let buffer = unsafe { alloc.alloc(Layout::new_size_align(args.0, 8)).expect("Failed to allocate memory for composable allocator") };
        PoolAllocator::new(buffer, args.1)
    }
}

impl Drop for PoolAllocator {
    fn drop(&mut self) {
        let dealloc_ptr = Allocation::<u8>::new(self.buffer.ptr_mut(), *self.buffer.layout());
        MEMORY_MANAGER.dealloc(dealloc_ptr);
    }
}

unsafe impl Sync for PoolAllocator {}

#[cfg(test)]
mod tests {
    use crate::alloc::{*, primitives::*};

    #[test]
    fn alloc_dealloc() {
        let mut base_alloc = Mallocator;
        let buffer = unsafe { base_alloc.alloc(Layout::new_size_align(256, 8)).unwrap() };
        let mut alloc = PoolAllocator::new(buffer, 8);

        unsafe {
            let ptr = alloc.alloc(Layout::new::<u64>()).unwrap();
            alloc.dealloc(ptr);
        }
    }

    #[test]
    fn multi_allocs() {
        let mut base_alloc = Mallocator;
        let buffer = unsafe { base_alloc.alloc(Layout::new_size_align(256, 8)).unwrap() };
        let mut alloc = PoolAllocator::new(buffer, 8);

        unsafe {
            let ptr0 = alloc.alloc(Layout::new::<u16>()).unwrap();
            let ptr1 = alloc.alloc(Layout::new::<u64>()).unwrap();
            let ptr2 = alloc.alloc(Layout::new::<u32>()).unwrap();

            alloc.dealloc(ptr0);
            alloc.dealloc(ptr1);
            alloc.dealloc(ptr2);
        }
    }

    #[test]
    fn dealloc_then_realloc() {
        let mut base_alloc = Mallocator;
        let buffer = unsafe { base_alloc.alloc(Layout::new_size_align(256, 8)).unwrap() };
        let mut alloc = PoolAllocator::new(buffer, 8);

        unsafe {
            let ptr0 = alloc.alloc(Layout::new::<u16>()).unwrap();

            let raw0 = ptr0.ptr();

            let ptr1 = alloc.alloc(Layout::new::<u64>()).unwrap();
            let ptr2 = alloc.alloc(Layout::new::<u32>()).unwrap();

            alloc.dealloc(ptr0);

            let new_ptr = alloc.alloc(Layout::new::<u16>()).unwrap();
            assert_eq!(raw0, new_ptr.ptr());
        }
    }

    #[test]
    fn alloc_too_large() {
        let mut base_alloc = Mallocator;
        let buffer = unsafe { base_alloc.alloc(Layout::new_size_align(256, 8)).unwrap() };
        let mut alloc = PoolAllocator::new(buffer, 8);

        struct Large { a: u64, b: u64 }

        unsafe {
            let ptr = alloc.alloc(Layout::new::<Large>());
            match ptr {
                None => {},
                Some(_) => panic!()
            }
        }
    }
}