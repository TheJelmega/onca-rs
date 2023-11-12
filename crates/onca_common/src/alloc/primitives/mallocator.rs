use std::{
    alloc::{Layout, GlobalAlloc},
    ptr::NonNull,
};

use crate::alloc::{Allocator, AllocHeader, AllocId, onca_malloc, onca_free};

extern crate alloc;

/// Allocator calling directly to the system allocator
/// 
/// Mallocator uses rust's global allocator to retrieve memory
/// 
/// This allocator has a special allocator id, which will always refer to the Mallocator: 0xFFFF
pub struct Mallocator;

impl Allocator for Mallocator {
    unsafe fn alloc(&mut self, layout: Layout) -> Option<NonNull<u8>> {
        onca_malloc(layout)
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        onca_free(ptr, layout)
    }

    fn owns(&self, _ptr: std::ptr::NonNull<u8>, _layout: Layout) -> bool {
        // We have no real way of knowing that we allocated this, so we'll just assume that we allocated it
        true
    }

    fn set_alloc_id(&mut self, _id: u16) {
        // Do nothing
    }

    fn alloc_id(&self) -> u16 {
        AllocId::Malloc.get_id()
    }
}

#[cfg(test)]
mod test {
    use std::alloc::Layout;

    use crate::alloc::*;
    use super::Mallocator;

    #[test]
    fn alloc_dealloc() {
        let mut alloc = Mallocator;
        let layout = Layout::new::<u64>();

        unsafe {
            let ptr = alloc.alloc(layout).unwrap();
            alloc.dealloc(ptr, layout);
        }
    }
}