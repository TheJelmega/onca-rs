mod alloc;

pub mod primitives;
pub mod composable;

use std::{alloc::{GlobalAlloc, Layout}, ptr::NonNull};

pub use alloc::*;

use crate::{mem::{get_memory_manager, AllocInitState, MemoryManager}, scoped_alloc};

pub struct OncaGlobalAlloc;

unsafe impl GlobalAlloc for OncaGlobalAlloc {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        Self::alloc_raw(&self, AllocInitState::Uninitialized, layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        let header = AllocHeader::from_non_null(NonNull::new_unchecked(ptr));
        if header.alloc_id() == AllocId::Untracked.get_id() {
            MemoryManager::dealloc_untracked(NonNull::new_unchecked(ptr), layout);
        } else {
           get_memory_manager().dealloc(NonNull::new_unchecked(ptr), layout)
        }
    }

    unsafe fn alloc_zeroed(&self, layout: std::alloc::Layout) -> *mut u8 {
        Self::alloc_raw(&self, AllocInitState::Zeroed, layout)
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: std::alloc::Layout, new_size: usize) -> *mut u8 {
        let alloc_header = AllocHeader::from_non_null(NonNull::new_unchecked(ptr));
        scoped_alloc!(AllocId::Id(alloc_header.alloc_id()));

        let new_layout = Layout::from_size_align(new_size, layout.align()).unwrap();
        let new = self.alloc(new_layout);

        let copy_size = layout.size().max(new_size);
        std::ptr::copy_nonoverlapping(ptr, new, copy_size);

        self.dealloc(ptr, layout);

        return new;
    }
}

impl OncaGlobalAlloc {
    unsafe fn alloc_raw(&self, init_state: AllocInitState, layout: Layout) -> *mut u8 {
        if get_active_alloc() == AllocId::Untracked {
            MemoryManager::alloc_untracked(init_state, layout).as_ptr()
        } else {
           get_memory_manager().alloc_raw(init_state, layout, None).expect("Failed to allocate").as_ptr()
        }
    }
}