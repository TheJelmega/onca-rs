use core::{ptr::NonNull, cell::UnsafeCell};

use crate::{
    alloc::{Layout, UseAlloc, Allocation, get_active_alloc, ScopedAlloc},
    mem::{MEMORY_MANAGER, AllocInitState}
};

extern crate alloc;

pub struct Alloc {
    layout  : UnsafeCell<Layout>,
}

impl Alloc {
    pub fn new() -> Self {
        Self::with_id(get_active_alloc().get_id())
    }

    pub fn layout(&self) -> Layout {
        unsafe{ *self.layout.get() }
    } 

    fn with_id(id: u16) -> Self {
        Alloc { layout: UnsafeCell::new(Layout::null().with_alloc_id(id)) }
    }
}

unsafe impl alloc::alloc::Allocator for Alloc {
    fn allocate(&self, layout: std::alloc::Layout) -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        let layout = Layout::new_size_align(layout.size(), layout.align());

        unsafe {
            let self_layout = &mut *self.layout.get();
            let _scope_alloc = ScopedAlloc::new(UseAlloc::Id(self_layout.alloc_id()));

            let alloc = MEMORY_MANAGER.alloc_raw(AllocInitState::Uninitialized, layout);
            match alloc {
                Some(ptr) => {
                    let slice_ptr = core::ptr::slice_from_raw_parts_mut(ptr.ptr_mut(), ptr.layout().size());
                    *self_layout = ptr.layout();
                    Ok(NonNull::new_unchecked(slice_ptr))
                },
                None => { Err(alloc::alloc::AllocError) }
            }
        }
    }

    unsafe fn deallocate(&self, ptr: std::ptr::NonNull<u8>, layout: std::alloc::Layout) {
        let layout = unsafe { *self.layout.get() };
        let alloc = Allocation::from_raw(ptr.as_ptr(), layout);
        MEMORY_MANAGER.dealloc(alloc);
    }
}

impl Clone for Alloc {
    fn clone(&self) -> Self {
        Self::with_id(self.layout().alloc_id())
    }   
}