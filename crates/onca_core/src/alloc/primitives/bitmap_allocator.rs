use crate::{
    mem::MEMORY_MANAGER,
    sync::Mutex,
    alloc::*, 
};

/// Bitmap allocator
/// 
/// A bitmap allocator stores a set of blocks with a fixed size, whenever memory is requested, the allocator will find a spot with enough contiguous blocks to store the allocation,
/// it well then mark these in a bitmap to keep track of which blocks are in use and which are available
pub struct BitmapAllocator
{
    buffer     : Allocation<u8>,
    block_size : usize,
    num_blocks : usize,
    num_manage : usize,
    mutex      : Mutex<()>,
    id         : u16
}

impl BitmapAllocator {
    
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
    pub fn new(buffer: Allocation<u8>, block_size: usize, num_blocks : usize) -> Self {
        assert!(buffer.layout().size() >= Self::calc_needed_memory_size(block_size, num_blocks), "Provided buffer is not large enough, use `calc_needed_memory_size` to ge the needed size");

        let num_manage = Self::calc_num_management_blocks(block_size, num_blocks);
        /// Clear management blocks
        unsafe {
            core::ptr::write_bytes(buffer.ptr_mut(), 0, num_manage * block_size)
        }

        Self { buffer,
               block_size,
               num_blocks,
               num_manage,
               mutex: Mutex::new(()),
               id: 0 }
    }


    fn calc_num_management_blocks(block_size: usize, num_blocks: usize) -> usize
    {
        let num_bytes = (num_blocks + 7) / 8;
        (num_bytes + block_size - 1) / block_size
    }

    unsafe fn mark_bits(buffer: *mut u8, first_bit: usize, mut num_bits: usize, set: bool)
    {
        assert!(num_bits != 0, "Can't mark 0 bits");

        let first_byte_offset = num_bits / 8;
        let mut byte_ptr = buffer.wrapping_add(first_byte_offset);

        let masks = [0x00, 0x80, 0xC0, 0xE0, 0xF0, 0xF8, 0xFC, 0xFE, 0xFF];

        let mut offset_in_first_byte = first_bit & 0x7;
        while num_bits != 0 
        {
            let num_to_mark = num_bits.min(8);
            let mask = masks[num_to_mark] >> offset_in_first_byte;
            let cur_byte = &mut *byte_ptr as &mut u8;
            *cur_byte = if set { *cur_byte | mask } else { *cur_byte &! mask };

            offset_in_first_byte = 0;
            byte_ptr = byte_ptr.wrapping_add(1);
            num_bits = if num_bits <= 8 { 0 } else { num_bits - 8 };
        }
    }
}

impl Allocator for BitmapAllocator {
    unsafe fn alloc(&mut self, layout: Layout, mem_tag: MemTag) -> Option<Allocation<u8>> {

        let blocks_needed = (layout.size() + self.block_size - 1) / self.block_size;
        if blocks_needed >= self.num_blocks {
            return None;
        }

        let masks = [0x00, 0x80, 0xC0, 0xE0, 0xF0, 0xF8, 0xFC, 0xFE, 0xFF];

        let mut first_search_byte = self.buffer.ptr_mut();
        let mut buffer = self.buffer.ptr_mut();

        let _guard = self.mutex.lock();
        'outer: for i in 0..self.num_blocks {
            if i & 0x7 == 0 && i > 0
                { first_search_byte = first_search_byte.add(1); }

            let mut bit_idx = i & 0x7;
            let mut blocks_read = 0;

            while blocks_read < blocks_needed {
                let max_block_this_iter = 8 - bit_idx;
                let blocks_this_iter = blocks_needed.min(max_block_this_iter);
                let search_mask = masks[blocks_this_iter] >> bit_idx;
                if (unsafe { *first_search_byte }) & search_mask != 0
                    { continue 'outer; }

                blocks_read += blocks_this_iter;
                bit_idx = 0;
            }

            // If we get here, we found a space
            Self::mark_bits(buffer, i, blocks_needed, true);

            let ptr = self.buffer.ptr_mut().add((self.num_manage + i) * self.block_size);
            let layout = layout.with_size_multiple_of(self.block_size as u64).with_alloc_id(self.id);
            return Some(Allocation::<_>::new(ptr, layout, mem_tag));
        }
        None
    }

    unsafe fn dealloc(&mut self, ptr: Allocation<u8>) {
        assert!(self.owns(&ptr), "Cannot deallocate an allocation that isn't owned by the allocator");

        let mut block_idx = unsafe { ptr.ptr().offset_from(self.buffer.ptr()) } as usize;
        block_idx = block_idx / self.block_size - self.num_manage;

        let num_blocks = (ptr.layout().size() + self.block_size - 1) / self.block_size;

        let _guard = self.mutex.lock();
        unsafe { Self::mark_bits(self.buffer.ptr_mut(), block_idx, num_blocks, false) };
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

impl ComposableAllocator<(usize, usize)> for BitmapAllocator {
    fn new_composable(alloc: &mut dyn Allocator, args: (usize, usize), mem_tag: MemTag) -> Self {
        let buffer_size = BitmapAllocator::calc_needed_memory_size(args.0, args.1);
        let buffer = unsafe { alloc.alloc(Layout::new_size_align(buffer_size, 8), mem_tag).expect("Failed to allocate memory for composable allocator") };
        BitmapAllocator::new(buffer, args.0, args.1)
    }
}

impl Drop for BitmapAllocator {
    fn drop(&mut self) {
        MEMORY_MANAGER.dealloc(core::mem::replace(&mut self.buffer, unsafe { Allocation::null() }));
    }
}

unsafe impl Sync for BitmapAllocator {}

#[cfg(test)]
mod tests {
    use crate::alloc::{*, primitives::*, mem_tag::MemTag};

    #[test]
    fn alloc_dealloc() {
        let mut base_alloc = Mallocator;

        let block_size = 8;
        let num_blocks = 16;
        let buffer_size = BitmapAllocator::calc_needed_memory_size(block_size, num_blocks);

        let buffer = unsafe { base_alloc.alloc(Layout::new_size_align(buffer_size, 8), CoreMemTag::Test.to_mem_tag()).unwrap() };
        let mut alloc = BitmapAllocator::new(buffer, block_size, num_blocks);

        unsafe {
            let ptr = alloc.alloc(Layout::new::<u64>(), CoreMemTag::Test.to_mem_tag()).unwrap();
            alloc.dealloc(ptr);
        }
    }

    #[test]
    fn multi_allocs() {
        let mut base_alloc = Mallocator;

        let block_size = 8;
        let num_blocks = 16;
        let buffer_size = BitmapAllocator::calc_needed_memory_size(block_size, num_blocks);

        let buffer = unsafe { base_alloc.alloc(Layout::new_size_align(buffer_size, 8), CoreMemTag::Test.to_mem_tag()).unwrap() };
        let mut alloc = BitmapAllocator::new(buffer, block_size, num_blocks);

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

        let block_size = 8;
        let num_blocks = 16;
        let buffer_size = BitmapAllocator::calc_needed_memory_size(block_size, num_blocks);

        let buffer = unsafe { base_alloc.alloc(Layout::new_size_align(buffer_size, 8), CoreMemTag::Test.to_mem_tag()).unwrap() };
        let mut alloc = BitmapAllocator::new(buffer, block_size, num_blocks);

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

        let block_size = 8;
        let num_blocks = 16;
        let buffer_size = BitmapAllocator::calc_needed_memory_size(block_size, num_blocks);

        let buffer = unsafe { base_alloc.alloc(Layout::new_size_align(buffer_size, 8), CoreMemTag::Test.to_mem_tag()).unwrap() };
        let mut alloc = BitmapAllocator::new(buffer, block_size, num_blocks);

        unsafe {
            let ptr = alloc.alloc(Layout::new_size_align(buffer_size, 8), CoreMemTag::Test.to_mem_tag());
            match ptr {
                None => {},
                Some(_) => panic!()
            }
        }
    }

    #[test]
    fn multiple_with_mixed_deallocs() {
        let mut base_alloc = Mallocator;

        let block_size = 8;
        let num_blocks = 16;
        let buffer_size = BitmapAllocator::calc_needed_memory_size(block_size, num_blocks);

        let buffer = unsafe { base_alloc.alloc(Layout::new_size_align(buffer_size, 8), CoreMemTag::Test.to_mem_tag()).unwrap() };
        let mut alloc = BitmapAllocator::new(buffer, block_size, num_blocks);

        struct Large { a: u64, b: u64 }

        unsafe {
            let ptr0 = alloc.alloc(Layout::new::<Large>(), CoreMemTag::Test.to_mem_tag()).unwrap();
            let ptr1 = alloc.alloc(Layout::new::<u64>(), CoreMemTag::Test.to_mem_tag()).unwrap();
            let ptr2 = alloc.alloc(Layout::new::<u64>(), CoreMemTag::Test.to_mem_tag()).unwrap();

            let raw_ptr0 = ptr0.ptr();

            alloc.dealloc(ptr0);
            alloc.dealloc(ptr1);

            let ptr4 = alloc.alloc(Layout::new::<u64>(), CoreMemTag::Test.to_mem_tag()).unwrap();
            let ptr5 = alloc.alloc(Layout::new::<Large>(), CoreMemTag::Test.to_mem_tag()).unwrap();

            assert!(ptr4.ptr() == raw_ptr0);
            assert!(ptr5.ptr() == raw_ptr0.add(8));
        }
    }
}