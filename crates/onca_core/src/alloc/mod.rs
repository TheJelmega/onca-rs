mod alloc;

pub mod primitives;
pub mod composable;

use std::{alloc::GlobalAlloc, ptr::NonNull};

pub use alloc::*;

use crate::mem::{get_memory_manager, AllocInitState, MemoryManager};


pub struct OncaGlobalAlloc;

unsafe impl GlobalAlloc for OncaGlobalAlloc {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        if get_active_alloc() == AllocId::Untracked {
            MemoryManager::alloc_untracked(AllocInitState::Uninitialized, layout).as_ptr()
        } else {
            get_memory_manager().alloc_raw(AllocInitState::Uninitialized, layout).expect("Failed to allocate").as_ptr()
        }

    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        let header = AllocHeader::from_non_null(NonNull::new_unchecked(ptr));
        if header.alloc_id() == AllocId::Untracked.get_id() {
            MemoryManager::dealloc_untracked(NonNull::new_unchecked(ptr), layout);
        } else {
            get_memory_manager().dealloc(NonNull::new_unchecked(ptr), layout)
        }
    }
}