use std::{ptr::NonNull, alloc::Layout};

use crate::{
    mem::{AllocInitState, get_memory_manager},
    sync::Mutex,
    alloc::*, 
};

/// Bitmap allocator
/// 
/// A bitmap allocator stores a set of blocks with a fixed size, whenever memory is requested, the allocator will find a spot with enough contiguous blocks to store the allocation,
/// it well then mark these in a bitmap to keep track of which blocks are in use and which are available
pub struct BitmapAllocator
{
    buffer:        NonNull<u8>,
    buffer_layout: Layout,
    block_size:    usize,
    num_blocks:    usize,
    num_manage:    usize,
    mutex:         Mutex<()>,
    id:            u16
}

impl BitmapAllocator {
    const SEARCH_MASKS: [u8; 9] = [0x00, 0x80, 0xC0, 0xE0, 0xF0, 0xF8, 0xFC, 0xFE, 0xFF];
    
    /// Calculate the memory needed to store the required data and metadata for an allocator with `num_blocks` of size `block_size`
    pub fn calc_needed_memory_size(block_size: usize, num_blocks : usize) -> usize {
        let num_management_blocks = Self::calc_num_management_blocks(block_size, num_blocks);
        (num_management_blocks + num_blocks) * block_size
    }

    /// Create a bitmap allocator
    /// 
    /// `alloc` denotes the allocator used to allocate the arena
    /// `block_size` denotes the size of the bitmapped blocks
    /// `num_blocks` denotes the number of block in the allocator
    /// 
    /// # Panics
    /// 
    /// The function will panic if it the provided memory is not large enough, since the size should be calculated using `calc_needed_memory_size`
    pub fn new(buffer: NonNull<u8>, buffer_layout: Layout, block_size: usize, num_blocks : usize) -> Self {
        assert!(buffer_layout.size() >= Self::calc_needed_memory_size(block_size, num_blocks), "Provided buffer is not large enough, use `calc_needed_memory_size` to ge the needed size");

        let num_manage = Self::calc_num_management_blocks(block_size, num_blocks);
        /// Clear management blocks
        unsafe {
            core::ptr::write_bytes(buffer.as_ptr(), 0, num_manage * block_size)
        }

        Self { 
            buffer,
            buffer_layout,
            block_size,
            num_blocks,
            num_manage,
            mutex: Mutex::new(()),
            id: 0
        }
    }


    fn calc_num_management_blocks(block_size: usize, num_blocks: usize) -> usize {
        let num_bytes = (num_blocks + 7) / 8;
        (num_bytes + block_size - 1) / block_size
    }

    unsafe fn mark_bits(buffer: NonNull<u8>, first_bit: usize, mut num_bits: usize, set: bool) {
        assert!(num_bits != 0, "Can't mark 0 bits");

        let first_byte_offset = first_bit / 8;
        let mut byte_ptr = buffer.as_ptr().add(first_byte_offset);

        let mut offset_in_first_byte = first_bit & 0x7;
        while num_bits != 0 
        {
            let num_to_mark = num_bits.min(8);
            let mask = Self::SEARCH_MASKS[num_to_mark] >> offset_in_first_byte;

            if set {
                *byte_ptr |= mask;
            } else {
                *byte_ptr &= !mask;
            }

            offset_in_first_byte = 0;
            byte_ptr = byte_ptr.add(1);
            num_bits = num_bits.saturating_sub(8);
        }
    }
}

impl Allocator for BitmapAllocator {
    unsafe fn alloc(&mut self, layout: Layout) -> Option<NonNull<u8>> {
        let blocks_needed = (layout.size() + self.block_size - 1) / self.block_size;
        if blocks_needed >= self.num_blocks {
            return None;
        }

        let mut first_search_byte = self.buffer.as_ptr();

        let _guard = self.mutex.lock();
        'outer: for i in 0..self.num_blocks {
            if i & 0x7 == 0 && i != 0 {
                first_search_byte = first_search_byte.add(1);
            }

            let mut bit_idx = i & 0x7;
            let mut blocks_read = 0;

            while blocks_read < blocks_needed {
                let max_block_this_iter = 8 - bit_idx;
                let blocks_this_iter = blocks_needed.min(max_block_this_iter);
                let search_mask = Self::SEARCH_MASKS[blocks_this_iter] >> bit_idx;
                if *first_search_byte & search_mask != 0 {
                    continue 'outer; 
                }

                blocks_read += blocks_this_iter;
                bit_idx = 0;
            }

            // If we get here, we found a space
            Self::mark_bits(self.buffer, i, blocks_needed, true);

            let offset = (self.num_manage + i) * self.block_size;
            let ptr = self.buffer.as_ptr().add(offset);
            return NonNull::new(ptr);
        }
        None
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        assert!(self.owns(ptr, layout), "Cannot deallocate an allocation that isn't owned by the allocator");

        let mut block_idx = unsafe { ptr.as_ptr().offset_from(self.buffer.as_ptr()) } as usize;
        block_idx = block_idx / self.block_size - self.num_manage;

        let num_blocks = (layout.size() + self.block_size - 1) / self.block_size;

        let _guard = self.mutex.lock();
        unsafe { Self::mark_bits(self.buffer, block_idx, num_blocks, false) };
    }

    fn owns(&self, ptr: NonNull<u8>, _layout: Layout) -> bool {
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

impl ComposableAllocator<(usize, usize)> for BitmapAllocator {
    fn new_composable(args: (usize, usize)) -> Self {
        let buffer_size = BitmapAllocator::calc_needed_memory_size(args.0, args.1);
        let buffer_layout = Layout::from_size_align(buffer_size, 8).expect("Invalid `BitmapAllocator::new_composable` parameters");
        let buffer = unsafe { get_memory_manager().alloc_raw(AllocInitState::Uninitialized, buffer_layout, None).expect("Failed to allocate memory for composable allocator") };
        BitmapAllocator::new(buffer, buffer_layout, args.0, args.1)
    }
}

impl Drop for BitmapAllocator {
    fn drop(&mut self) {
        unsafe { get_memory_manager().dealloc(self.buffer, self.buffer_layout) };
    }
}

unsafe impl Sync for BitmapAllocator {}

#[cfg(test)]
mod tests {
    use std::alloc::Layout;

    use crate::alloc::{*, primitives::*};

    #[test]
    fn alloc_dealloc() {
        
        let block_size = 8;
        let num_blocks = 16;
        let buffer_size = BitmapAllocator::calc_needed_memory_size(block_size, num_blocks);
        
        unsafe {
            let mut base_alloc = Mallocator;
            let buffer_layout = Layout::from_size_align_unchecked(buffer_size, 8);

            let buffer = base_alloc.alloc(buffer_layout).unwrap();
            let mut alloc = BitmapAllocator::new(buffer, buffer_layout, block_size, num_blocks);

            let layout = Layout::new::<u64>();
            let ptr = alloc.alloc(layout).unwrap();
            alloc.dealloc(ptr, layout);
        }
    }

    #[test]
    fn multi_allocs() {
        let block_size = 8;
        let num_blocks = 16;
        let buffer_size = BitmapAllocator::calc_needed_memory_size(block_size, num_blocks);

        unsafe {
            let mut base_alloc = Mallocator;
            let buffer_layout = Layout::from_size_align_unchecked(buffer_size, 8);

            let buffer = base_alloc.alloc(buffer_layout).unwrap();
            let mut alloc = BitmapAllocator::new(buffer, buffer_layout, block_size, num_blocks);

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
        let block_size = 8;
        let num_blocks = 16;
        let buffer_size = BitmapAllocator::calc_needed_memory_size(block_size, num_blocks);

        unsafe {
            let mut base_alloc = Mallocator;
            let buffer_layout = Layout::from_size_align_unchecked(buffer_size, 8);

            let buffer = base_alloc.alloc(buffer_layout).unwrap();
            let mut alloc = BitmapAllocator::new(buffer, buffer_layout, block_size, num_blocks);

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
        let block_size = 8;
        let num_blocks = 16;
        let buffer_size = BitmapAllocator::calc_needed_memory_size(block_size, num_blocks);

        unsafe {
            let mut base_alloc = Mallocator;
            let buffer_layout = Layout::from_size_align_unchecked(buffer_size, 8);

            let buffer = base_alloc.alloc(buffer_layout).unwrap();
            let mut alloc = BitmapAllocator::new(buffer, buffer_layout, block_size, num_blocks);

            let layout = Layout::from_size_align_unchecked(buffer_size, 8);
            let ptr = alloc.alloc(layout);
            match ptr {
                None => {},
                Some(_) => panic!()
            }
        }
    }

    #[test]
    fn multiple_with_mixed_deallocs() {
        let block_size = 8;
        let num_blocks = 16;
        let buffer_size = BitmapAllocator::calc_needed_memory_size(block_size, num_blocks);

        struct Large { a: u64, b: u64 }

        unsafe {
            let mut base_alloc = Mallocator;
            let buffer_layout = Layout::from_size_align_unchecked(buffer_size, 8);

            let buffer = base_alloc.alloc(buffer_layout).unwrap();
            let mut alloc = BitmapAllocator::new(buffer, buffer_layout, block_size, num_blocks);

            let layout0 = Layout::new::<Large>();
            let layout1 = Layout::new::<u64>();
            let layout2 = Layout::new::<u64>();

            let ptr0 = alloc.alloc(layout0).unwrap();
            let ptr1 = alloc.alloc(layout1).unwrap();
            let ptr2 = alloc.alloc(layout2).unwrap();

            alloc.dealloc(ptr0, layout0);
            alloc.dealloc(ptr1, layout1);

            let ptr4 = alloc.alloc(Layout::new::<u64>()).unwrap();
            let ptr5 = alloc.alloc(Layout::new::<Large>()).unwrap();

            assert!(ptr4 == ptr0);
            assert!(ptr5.as_ptr() == ptr0.as_ptr().add(8));
        }
    }
}