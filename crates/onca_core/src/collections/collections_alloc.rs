use core::{ptr::NonNull, cell::UnsafeCell};

use crate::{alloc::{Layout, Allocator, UseAlloc, Allocation}, mem::MEMORY_MANAGER};

extern crate alloc;

pub struct Alloc {
    layout : UnsafeCell<Layout>
}

impl Alloc {
    
    pub fn new(alloc: UseAlloc) -> Self {
        match alloc {
            UseAlloc::Default => Self::with_id(Layout::MAX_ALLOC_ID),
            UseAlloc::Alloc(alloc) => Self::with_id(alloc.alloc_id()),
            UseAlloc::Id(id) => Self::with_id(id),
        }
    }

    pub fn layout(&self) -> &Layout {
        unsafe{ &*self.layout.get() }
    } 

    fn with_id(id: u16) -> Self {
        Alloc { layout: UnsafeCell::new(Layout::null().with_alloc_id(id)) }
    }
}

unsafe impl std::alloc::Allocator for Alloc {
    fn allocate(&self, layout: std::alloc::Layout) -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        let layout = Layout::new_size_align(layout.size(), layout.align());

        unsafe {
            let self_layout = &mut *self.layout.get();
            let alloc = MEMORY_MANAGER.alloc_raw(UseAlloc::Id(self_layout.alloc_id()), layout);

            match alloc {
                Some(ptr) => {
                    let slice_ptr = core::ptr::slice_from_raw_parts_mut(ptr.ptr_mut(), ptr.layout().size());
                    *self_layout = layout;
                    Ok(NonNull::new_unchecked(slice_ptr))
                },
                None => { Err(alloc::alloc::AllocError) }
            }
        }
    }

    unsafe fn deallocate(&self, ptr: std::ptr::NonNull<u8>, layout: std::alloc::Layout) {
        let layout = unsafe { *self.layout.get() };
        let alloc = Allocation::new(ptr.as_ptr(), layout);
        MEMORY_MANAGER.dealloc(alloc);
    }
}

impl Clone for Alloc {
    fn clone(&self) -> Self {
        Self::with_id(self.layout().alloc_id())
    }   
}