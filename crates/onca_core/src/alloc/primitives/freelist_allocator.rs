use core::{mem::size_of, ptr::null_mut};

use crate::{
    mem::MEMORY_MANAGER,
    sync::Mutex,
    alloc::*,
};


struct FreeBlock {
    next : *mut FreeBlock,
    size : usize
}

struct Header {
    padding : u16
}

// TODO: Add multiple search implementations, currently only fit-first (which could be atomic)

/// Free-list allocator
/// 
/// An allocator that manages a block of memory using a list to track all free space in the allocator
pub struct FreelistAllocator {
    buffer : Allocation<u8>,
    head   : *mut FreeBlock,
    mutex  : Mutex<()>,
    id     : u16
}

impl FreelistAllocator {
    /// Create a stack allocator with the given memory as its arena
    pub fn new(buffer: Allocation<u8>) -> Self {
        let head = buffer.ptr_mut().cast::<FreeBlock>();
        unsafe {
            (*head).next = null_mut();
            (*head).size = buffer.layout().size();
        }

        Self { buffer, head, mutex: Mutex::new(()), id: 0 }
    }

    // Allocate from the first fitting element (not optimal for fragmentation)
    unsafe fn alloc_first(&mut self, layout: &mut Layout) -> *mut u8
    {
        let size = layout.size();
        let align = layout.align();
        let alloc_block_size = size_of::<Header>();

        let mut cur_block = self.head;
        let mut prev_block : *mut FreeBlock = null_mut();

        let _guard = self.mutex.lock();
        while cur_block != null_mut() {
            let mut next_block = (*cur_block).next;

            let ptr = cur_block as *mut u8;
            let padding = alloc_block_size + ptr.add(alloc_block_size).align_offset(align);

            // The allocation needs to have at minimum enough space to put a free block, as it's otherwise unable to deallocate the memory without overwriting other memory
            let padded_size = core::cmp::max(padding + size, size_of::<FreeBlock>());
            if padded_size <= (*cur_block).size {
                let aligned_ptr = ptr.add(padding);
                let remaining_size = (*cur_block).size - padded_size;
                
                Self::write_alloc_block(aligned_ptr, padding as u16);

                if remaining_size < size_of::<FreeBlock>() {
                    if remaining_size > 0 {
                        layout.expand_packed(Layout::new_size_align(remaining_size, 1));
                    }
                } else {
                    layout.expand_packed(Layout::new_size_align(padded_size - size, 1));
                    next_block = Self::write_free_block(aligned_ptr.add(size), next_block, remaining_size);
                }

                if prev_block == null_mut() {
                    self.head = next_block;
                } else { 
                    (*prev_block).next = next_block;
                }

                return ptr.add(padding);
            }

            cur_block = (*cur_block).next;
        }

        null_mut()
    }

    unsafe fn write_alloc_block(ptr: *mut u8, front: u16)
    {
        let alloc_block = (ptr as *mut Header).sub(size_of::<Header>());
        (*alloc_block).padding = front;
    }

    unsafe fn write_free_block(ptr: *mut u8, next: *mut FreeBlock, size: usize) -> *mut FreeBlock
    {
        let free_block = ptr as *mut FreeBlock;
        (*free_block).next = next;
        (*free_block).size = size;
        free_block
    }

    unsafe fn get_orig_ptr_and_front_padding(ptr: *mut u8) -> (*mut u8, usize)
    {
        let alloc_block = (ptr as *mut Header).sub(size_of::<Header>());
        let front = unsafe { (*alloc_block).padding };
        (ptr.sub(front as usize), front.into())
    }

    unsafe fn coalesce(prev: *mut FreeBlock, mut cur: *mut FreeBlock, next: *mut FreeBlock)
    {
        let prev_u8 = prev as *mut u8;
        let cur_u8 = cur as *mut u8;
        let next_u8 = next as *mut u8;

        let prev_size = if prev != null_mut() { (*prev).size } else { 0 };
        let cur_size = (*cur).size;

        if (cur_u8).add(cur_size) == next_u8 {
            (*cur).size += (*next).size;
            (*cur).next = (*next).next;
        }
        if prev_u8.add(prev_size) == cur_u8 {
            (*prev).size += cur_size;
            (*prev).next = (*cur).next;
        }
    }
}

impl Allocator for FreelistAllocator {
    unsafe fn alloc(&mut self, layout: Layout, mem_tag: MemTag) -> Option<Allocation<u8>> {

        let mut layout = layout;
        let ptr = {
            self.alloc_first(&mut layout)
        };

        if ptr == null_mut() {
            None
        } else {
            Some(Allocation::<_>::new(ptr, layout.with_alloc_id(self.id), mem_tag))
        }
    }

    unsafe fn dealloc(&mut self, ptr: Allocation<u8>) {
        assert!(self.owns(&ptr), "Cannot deallocate an allocation that isn't owned by the allocator");

        let (orig_ptr, front_pad) = Self::get_orig_ptr_and_front_padding(ptr.ptr_mut());
        let mut prev_block = null_mut();
        let _guard = self.mutex.lock();
        let mut next_block = self.head;

        while next_block != null_mut() && (next_block as *mut u8) < orig_ptr
        {
            prev_block = next_block;
            next_block = unsafe { (*next_block).next };
        }

        let cur_block = Self::write_free_block(orig_ptr, next_block, ptr.layout().size() + front_pad);

        unsafe { Self::coalesce(prev_block, cur_block, next_block) };

        if prev_block == null_mut()
            { self.head = cur_block }
    }

    fn owns(&self, ptr: &Allocation<u8>) -> bool {
        ptr.ptr() >= self.buffer.ptr() && ptr.ptr() > unsafe { self.buffer.ptr().add(self.buffer.layout().size()) }
    }

    fn set_alloc_id(&mut self, id: u16) {
        todo!()
    }

    fn alloc_id(&self) -> u16 {
        todo!()
    }
}

impl ComposableAllocator<usize> for FreelistAllocator {
    fn new_composable(alloc: UseAlloc, args: usize) -> Self {
        let buffer = unsafe { MEMORY_MANAGER.alloc_raw(alloc, Layout::new_size_align(args, 8), CoreMemTag::Allocators.to_mem_tag()).expect("Failed to allocate memory for composable allocator") };
        FreelistAllocator::new(buffer)
    }

    fn owns_composable(&self, allocation: &Allocation<u8>) -> bool {
        let addr = allocation.ptr();
        let start = self.buffer.ptr();
        let end = unsafe { start.add(self.buffer.layout().size()) };

        addr >= start && addr <= end
    }
}

impl Drop for FreelistAllocator {
    fn drop(&mut self) {
        MEMORY_MANAGER.dealloc(core::mem::replace(&mut self.buffer, unsafe { Allocation::null() }));
    }
}

unsafe impl Sync for FreelistAllocator {}

#[cfg(test)]
mod tests {
    use crate::alloc::{*, primitives::*};

    #[test]
    fn alloc_dealloc() {
        let mut base_alloc = Mallocator;
        let buffer = unsafe { base_alloc.alloc(Layout::new_size_align(256, 8), CoreMemTag::Test.to_mem_tag()).unwrap() };
        let mut alloc = FreelistAllocator::new(buffer);

        unsafe {
            let ptr = alloc.alloc(Layout::new::<u64>(), CoreMemTag::Test.to_mem_tag()).unwrap();
            alloc.dealloc(ptr);
        }
    }

    #[test]
    fn multi_allocs() {
        let mut base_alloc = Mallocator;
        let buffer = unsafe { base_alloc.alloc(Layout::new_size_align(256, 8), CoreMemTag::Test.to_mem_tag()).unwrap() };
        let mut alloc = FreelistAllocator::new(buffer);

        unsafe {
            let ptr0 = alloc.alloc(Layout::new::<u16>(), CoreMemTag::Test.to_mem_tag()).unwrap();
            let ptr1 = alloc.alloc(Layout::new::<u64>(), CoreMemTag::Test.to_mem_tag()).unwrap();
            let ptr2 = alloc.alloc(Layout::new::<u32>(), CoreMemTag::Test.to_mem_tag()).unwrap();

            alloc.dealloc(ptr0);
            alloc.dealloc(ptr1);
            alloc.dealloc(ptr2);
        }
    }

    #[test]
    fn dealloc_then_realloc() {
        let mut base_alloc = Mallocator;
        let buffer = unsafe { base_alloc.alloc(Layout::new_size_align(256, 8), CoreMemTag::Test.to_mem_tag()).unwrap() };
        let mut alloc = FreelistAllocator::new(buffer);

        unsafe {
            let ptr0 = alloc.alloc(Layout::new::<u16>(), CoreMemTag::Test.to_mem_tag()).unwrap();

            let raw0 = ptr0.ptr();

            let ptr1 = alloc.alloc(Layout::new::<u64>(), CoreMemTag::Test.to_mem_tag()).unwrap();
            let ptr2 = alloc.alloc(Layout::new::<u32>(), CoreMemTag::Test.to_mem_tag()).unwrap();

            alloc.dealloc(ptr0);

            let new_ptr = alloc.alloc(Layout::new::<u16>(), CoreMemTag::Test.to_mem_tag()).unwrap();
            assert_eq!(raw0, new_ptr.ptr());
        }
    }

    #[test]
    fn alloc_too_large() {
        let mut base_alloc = Mallocator;
        let buffer = unsafe { base_alloc.alloc(Layout::new_size_align(256, 8), CoreMemTag::Test.to_mem_tag()).unwrap() };
        let mut alloc = FreelistAllocator::new(buffer);

        unsafe {
            let ptr = alloc.alloc(Layout::new_size_align(300, 8), CoreMemTag::Test.to_mem_tag());
            match ptr {
                None => {},
                Some(_) => panic!()
            }
        }
    }
}