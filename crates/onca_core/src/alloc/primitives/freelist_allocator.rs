use core::{mem::{size_of, align_of}, ptr::null_mut};

use crate::{
    mem::{AllocInitState, get_memory_manager},
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
    /// Create a free-list allocator with the given memory as its arena
    pub fn new(buffer: Allocation<u8>) -> Self {
        let head = buffer.ptr_mut().cast::<FreeBlock>();
        unsafe {
            (*head).next = null_mut();
            (*head).size = buffer.layout().size();
        }

        Self { buffer, head, mutex: Mutex::new(()), id: 0 }
    }

    /// Create an uninitialized freelist allocator
    pub const fn new_uninit() -> Self {
        Self { buffer: unsafe { Allocation::const_null() }, head: null_mut(), mutex: Mutex::new(()), id: 0 }
    }

    /// Initialize an unitilialized freelist allocator
    pub fn init(&mut self, buffer: Allocation<u8>) {
        let _guard = self.mutex.lock();
        if self.buffer.ptr() != null_mut() {
            return;
        }

        self.buffer = buffer;
        self.head = self.buffer.ptr_mut().cast::<FreeBlock>();
        unsafe {
            (*self.head).next = null_mut();
            (*self.head).size = self.buffer.layout().size();
        }
    }

    /// Check if the allocator is initialized
    pub fn is_initialized(&self) -> bool {
        let _guard = self.mutex.lock();
        self.buffer.ptr() != null_mut()
    }

    // Allocate from the first fitting element (not optimal for fragmentation)
    unsafe fn alloc_first(&mut self, layout: &mut Layout) -> *mut u8 {
        assert!(self.is_initialized(), "Trying to allocate memory using an uninitialized free-list allocator");

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
            // Make sure the pointer will be aligned correctly
            let padded_size = padded_size.next_multiple_of(align_of::<FreeBlock>());

            if padded_size <= (*cur_block).size {
                let remaining_size = (*cur_block).size - padded_size;
                let aligned_ptr = ptr.add(padding);
                
                Self::write_alloc_block(aligned_ptr, padding as u16);

                if remaining_size < size_of::<FreeBlock>() {
                    if remaining_size > 0 {
                        *layout = layout.expand_packed(Layout::new_size_align(remaining_size, 1));
                    }
                } else {
                    let additional_size = padded_size - size - padding;
                    *layout = layout.expand_packed(Layout::new_raw(additional_size, 0, 1));
                    next_block = Self::write_free_block(ptr.add(padded_size), next_block, remaining_size);
                }

                if prev_block == null_mut() {
                    self.head = next_block;
                } else { 
                    (*prev_block).next = next_block;
                }

                return aligned_ptr;
            }

            prev_block = cur_block;
            cur_block = (*cur_block).next;
        }

        null_mut()
    }

    unsafe fn write_alloc_block(ptr: *mut u8, front: u16) {
        let alloc_block = (ptr as *mut Header).sub(1);
        (*alloc_block).padding = front;
    }

    unsafe fn write_free_block(ptr: *mut u8, next: *mut FreeBlock, size: usize) -> *mut FreeBlock {
        let free_block = ptr as *mut FreeBlock;
        (*free_block).next = next;
        (*free_block).size = size;
        free_block
    }

    unsafe fn get_orig_ptr_and_front_padding(ptr: *mut u8) -> (*mut u8, usize) {
        let alloc_block = (ptr as *mut Header).sub(1);
        let front = unsafe { (*alloc_block).padding } as usize;
        (ptr.sub(front), front)
    }

    unsafe fn coalesce(first: *mut FreeBlock, mut second: *mut FreeBlock) {
        let first_u8 = first as *mut u8;
        let first_size = (*first).size;
        let second_u8 = second as *mut u8;

        if first_u8.add(first_size) == second_u8 {
            (*first).size += (*second).size;
            (*first).next = (*second).next;
        }
    }
}

impl Allocator for FreelistAllocator {
    unsafe fn alloc(&mut self, layout: Layout) -> Option<Allocation<u8>> {
        let mut layout = layout;
        let ptr = {
            self.alloc_first(&mut layout)
        };

        if ptr == null_mut() {
            None
        } else {
            Some(Allocation::<_>::from_raw(ptr, layout.with_alloc_id(self.id)))
        }
    }

    unsafe fn dealloc(&mut self, ptr: Allocation<u8>) {
        assert!(self.owns(&ptr), "Cannot deallocate an allocation ({}) that isn't owned by the allocator ({})", ptr.layout().alloc_id(), self.id);

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

        if next_block != null_mut() {
            Self::coalesce(cur_block, next_block);
        }

        if prev_block == null_mut() {
            self.head = cur_block
        } else {
            (*prev_block).next = cur_block;
            Self::coalesce(prev_block, cur_block);
        }
    }

    fn set_alloc_id(&mut self, id: u16) {
        self.id = id;
    }

    fn alloc_id(&self) -> u16 {
        self.id
    }
}

impl ComposableAllocator<usize> for FreelistAllocator {
    fn new_composable(args: usize) -> Self {
        let buffer = unsafe { get_memory_manager().alloc_raw(AllocInitState::Uninitialized, Layout::new_size_align(args, 8)).expect("Failed to allocate memory for composable allocator") };
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
        get_memory_manager().dealloc(core::mem::replace(&mut self.buffer, unsafe { Allocation::const_null() }));
    }
}

unsafe impl Sync for FreelistAllocator {}

#[cfg(test)]
mod tests {
    use crate::alloc::{*, primitives::*};

    #[test]
    fn alloc_dealloc() {
        let mut base_alloc = Mallocator;
        let buffer = unsafe { base_alloc.alloc(Layout::new_size_align(256, 8)).unwrap() };
        let mut alloc = FreelistAllocator::new(buffer);

        unsafe {
            let ptr = alloc.alloc(Layout::new::<u64>()).unwrap();
            alloc.dealloc(ptr);
        }
    }

    #[test]
    fn multi_allocs() {
        let mut base_alloc = Mallocator;
        let buffer = unsafe { base_alloc.alloc(Layout::new_size_align(256, 8)).unwrap() };
        let mut alloc = FreelistAllocator::new(buffer);

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
        let mut alloc = FreelistAllocator::new(buffer);

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
        let mut alloc = FreelistAllocator::new(buffer);

        unsafe {
            let ptr = alloc.alloc(Layout::new_size_align(300, 8));
            match ptr {
                None => {},
                Some(_) => panic!()
            }
        }
    }
}