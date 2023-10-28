use core::{mem::{size_of, align_of}, ptr::null_mut};
use std::{ptr::NonNull, alloc::Layout};

use crate::{
    mem::{AllocInitState, get_memory_manager},
    sync::Mutex,
    alloc::*,
};


struct FreeBlock {
    next: Option<NonNull<FreeBlock>>,
    size: usize
}

struct Header {
    front: u16,
    back:  u16,
}

// TODO: Add multiple search implementations, currently only fit-first (which could be atomic)

/// Free-list allocator
/// 
/// An allocator that manages a block of memory using a list to track all free space in the allocator
pub struct FreelistAllocator {
    buffer:        NonNull<u8>,
    buffer_layout: Layout,
    head:          Mutex<Option<NonNull<FreeBlock>>>,
    id:            u16
}

impl FreelistAllocator {
    /// Create a free-list allocator with the given memory as its arena
    pub fn new(buffer: NonNull<u8>, buffer_layout: Layout) -> Self {
        let mut head = buffer.cast::<FreeBlock>();
        unsafe {
            head.as_mut().next = None;
            head.as_mut().size = buffer_layout.size();
        }

        Self { buffer, buffer_layout, head: Mutex::new(Some(head)), id: 0 }
    }

    // Allocate from the first fitting element (not optimal for fragmentation)
    unsafe fn alloc_first(&mut self, layout: Layout) -> Option<NonNull<u8>> {
        let size = layout.size();

        // Guaranteed to be at least 8
        let align = layout.align();

        let alloc_block_size = size_of::<Header>();

        let mut prev_block: Option<NonNull<FreeBlock>> = None;
        
        let mut head = self.head.lock();
        let mut cur_block_opt = *head;
        while let Some(mut cur_block) = cur_block_opt {
            let mut next_block = cur_block.as_mut().next;

            let ptr = cur_block.as_ptr() as *mut u8;
            let padding = alloc_block_size + ptr.add(alloc_block_size).align_offset(align);

            // The allocation needs to have at minimum enough space to put a free block, as it's otherwise unable to deallocate the memory without overwriting other memory
            let padded_size = core::cmp::max(padding + size, size_of::<FreeBlock>());
            // Make sure the pointer will be aligned correctly
            let padded_size = padded_size + ptr.add(padded_size).align_offset(align_of::<FreeBlock>()); //padded_size.next_multiple_of(align_of::<FreeBlock>());

            if padded_size <= cur_block.as_ref().size {
                let remaining_size = cur_block.as_ref().size - padded_size;
                let aligned_ptr = ptr.add(padding);
                
                let back = if remaining_size < size_of::<FreeBlock>() {
                    remaining_size
                } else {
                    next_block = Some(Self::write_free_block(NonNull::new_unchecked(ptr.add(padded_size)), next_block, remaining_size));
                    padded_size - padding - size
                };

                match &mut prev_block {
                    Some(block) => block.as_mut().next = next_block,
                    None => *head = next_block,
                }

                Self::write_alloc_block(NonNull::new_unchecked(aligned_ptr), padding as u16, back as u16);

                return NonNull::new(aligned_ptr);
            }

            prev_block = cur_block_opt;
            cur_block_opt = cur_block.as_ref().next;
        }

        None
    }

    unsafe fn write_alloc_block(ptr: NonNull<u8>, front: u16, back: u16) {
        let alloc_block = (ptr.as_ptr() as *mut Header).sub(1);
        *alloc_block = Header { front, back };
    }

    unsafe fn write_free_block(ptr: NonNull<u8>, next: Option<NonNull<FreeBlock>>, size: usize) -> NonNull<FreeBlock> {
        let mut free_block = ptr.cast::<FreeBlock>();
        *free_block.as_mut() = FreeBlock { next, size };
        free_block
    }

    unsafe fn get_orig_ptr_and_padding(ptr: NonNull<u8>) -> (NonNull<u8>, usize, usize) {
        let alloc_block = (ptr.as_ptr() as *mut Header).sub(1);
        let header = &*alloc_block;
        let orig = NonNull::new_unchecked(ptr.as_ptr().sub(header.front as usize));
        (orig, header.front as usize, header.back as usize)
    }

    unsafe fn coalesce(mut first: NonNull<FreeBlock>, second: NonNull<FreeBlock>) {
        let first_u8 = first.as_ptr() as *mut u8;
        let first_size = first.as_ref().size;
        let second_u8 = second.as_ptr() as *mut u8;

        if first_u8.add(first_size) == second_u8 {
            first.as_mut().size += second.as_ref().size;
            first.as_mut().next = second.as_ref().next;
        }
    }
}

impl Allocator for FreelistAllocator {
    unsafe fn alloc(&mut self, layout: Layout) -> Option<NonNull<u8>> {
        self.alloc_first(layout)
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        assert!(self.owns(ptr, layout), "Cannot deallocate an allocation that isn't owned by the allocator");

        let (orig_ptr, front, back) = Self::get_orig_ptr_and_padding(ptr);
        let mut prev_block = None;

        let mut head = self.head.lock();
        let mut next_block = *head;

        while let Some(next) = next_block {
            if next.as_ptr() > orig_ptr.as_ptr().cast() {
                break;
            }

            prev_block = next_block;
            next_block = next.as_ref().next;
        }

        let cur_block = Self::write_free_block(orig_ptr, next_block, layout.size() + front + back);

        if let Some(next) = next_block {
            Self::coalesce(cur_block, next);
        }

        if let Some(mut prev_block) = prev_block {
            prev_block.as_mut().next = Some(cur_block);
            Self::coalesce(prev_block, cur_block);
        } else {
            *head = Some(cur_block);
        }
    }

    fn owns(&self, ptr: NonNull<u8>, _layout: Layout) -> bool {
        let end = unsafe { self.buffer.as_ptr().add(self.buffer_layout.size()) };
        ptr >= self.buffer && ptr.as_ptr() < end
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
        let buffer_layout = Layout::from_size_align(args, 8).expect("Invalid `FreelistAllocator::new_composable` parameters");
        let buffer = unsafe { get_memory_manager().alloc_raw(AllocInitState::Uninitialized, buffer_layout, None).expect("Failed to allocate memory for composable allocator") };
        FreelistAllocator::new(buffer, buffer_layout)
    }
}

impl Drop for FreelistAllocator {
    fn drop(&mut self) {
        unsafe { get_memory_manager().dealloc(self.buffer, self.buffer_layout) };
    }
}

unsafe impl Sync for FreelistAllocator {}

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
            let mut alloc = FreelistAllocator::new(buffer, buffer_layout);

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
            let mut alloc = FreelistAllocator::new(buffer, buffer_layout);
            
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
            let buffer_layout = Layout::from_size_align_unchecked(256, 8);

            let buffer = base_alloc.alloc(buffer_layout).unwrap();
            let mut alloc = FreelistAllocator::new(buffer, buffer_layout);

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
        unsafe {
            let mut base_alloc = Mallocator;
            let buffer_layout = Layout::from_size_align_unchecked(256, 8);

            let buffer = base_alloc.alloc(buffer_layout).unwrap();
            let mut alloc = FreelistAllocator::new(buffer, buffer_layout);

            let layout = Layout::from_size_align_unchecked(300, 8);
            let ptr = alloc.alloc(layout);
            match ptr {
                None => {},
                Some(_) => panic!()
            }
        }
    }
}