use core::{ptr::NonNull, cell::UnsafeCell};

use crate::{alloc::{Layout, Allocator, UseAlloc, Allocation, CoreMemTag, MemTag}, mem::{MEMORY_MANAGER, AllocInitState}};

extern crate alloc;

pub struct Alloc {
    layout  : UnsafeCell<Layout>,
    mem_tag : UnsafeCell<MemTag>
}

impl Alloc {
    pub fn new(alloc: UseAlloc) -> Self {
        Self::with_id(alloc.get_id())
    }

    pub fn layout(&self) -> &Layout {
        unsafe{ &*self.layout.get() }
    } 

    fn with_id(id: u16) -> Self {
        Alloc { layout: UnsafeCell::new(Layout::null().with_alloc_id(id)), mem_tag: UnsafeCell::new(CoreMemTag::StdCollections.to_mem_tag()) }
    }
}

unsafe impl alloc::alloc::Allocator for Alloc {
    fn allocate(&self, layout: std::alloc::Layout) -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        let layout = Layout::new_size_align(layout.size(), layout.align());

        unsafe {
            let self_layout = &mut *self.layout.get();
            let self_mem_tag = &mut *self.mem_tag.get();
            let alloc = MEMORY_MANAGER.alloc_raw(AllocInitState::Uninitialized, UseAlloc::Id(self_layout.alloc_id()), layout, CoreMemTag::StdCollections.to_mem_tag());

            match alloc {
                Some(ptr) => {
                    let slice_ptr = core::ptr::slice_from_raw_parts_mut(ptr.ptr_mut(), ptr.layout().size());
                    *self_layout = layout;
                    *self_mem_tag = ptr.mem_tag();
                    Ok(NonNull::new_unchecked(slice_ptr))
                },
                None => { Err(alloc::alloc::AllocError) }
            }
        }
    }

    unsafe fn deallocate(&self, ptr: std::ptr::NonNull<u8>, layout: std::alloc::Layout) {
        let layout = unsafe { *self.layout.get() };
        let mem_tag = unsafe { *self.mem_tag.get() };
        let alloc = Allocation::new(ptr.as_ptr(), layout, mem_tag);
        MEMORY_MANAGER.dealloc(alloc);
    }
}

impl Clone for Alloc {
    fn clone(&self) -> Self {
        Self::with_id(self.layout().alloc_id())
    }   
}