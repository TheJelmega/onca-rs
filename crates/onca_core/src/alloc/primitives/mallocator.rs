use crate::alloc::{Allocator, Layout, Allocation};

extern crate alloc;

/// Allocator calling directly to the system allocator
/// 
/// Mallocator uses rust's global allocator to retrieve memory
/// 
/// This allocator has a special allocator id, which will always refer to the Mallocator: 0xFFFF
pub struct Mallocator;

impl Allocator for Mallocator {
    unsafe fn alloc(&mut self, layout: Layout) -> Option<Allocation<u8>> {
        let rs_layout = core::alloc::Layout::from_size_align_unchecked(layout.size(), layout.align());
        let ptr = unsafe { alloc::alloc::alloc(rs_layout) };
        if ptr == core::ptr::null_mut() {
            None
        } else {
            Some(Allocation::<u8>::new(ptr, layout.with_alloc_id(Layout::MAX_ALLOC_ID)))
        }
    }

    unsafe fn dealloc(&mut self, ptr: Allocation<u8>) {
        assert!(self.owns(&ptr), "Cannot deallocate an allocation that isn't owned by the allocator");

        let rs_layout = core::alloc::Layout::from_size_align_unchecked(ptr.layout().size(), ptr.layout().align());
        unsafe { alloc::alloc::dealloc(ptr.ptr_mut(), rs_layout) }
    }

    fn owns(&self, ptr: &Allocation<u8>) -> bool {
        return ptr.layout().alloc_id() == Layout::MAX_ALLOC_ID
    } 

    fn set_alloc_id(&mut self, _id: u16) {
        // Do nothing
    }

    fn alloc_id(&self) -> u16 {
        Layout::MAX_ALLOC_ID
    }
}

#[cfg(test)]
mod test {
    use crate::alloc::*;
    use super::Mallocator;

    #[test]
    fn alloc_dealloc() {
        let mut alloc = Mallocator;

        unsafe {
            let ptr = alloc.alloc(Layout::new::<u64>()).unwrap();
            alloc.dealloc(ptr);
        }
    }
}