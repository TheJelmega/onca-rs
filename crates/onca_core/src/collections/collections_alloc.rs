use core::{ptr::NonNull, cell::UnsafeCell};

use crate::{
    alloc::{Layout, UseAlloc, Allocation, CoreMemTag, MemTag, get_active_alloc, get_active_mem_tag, ScopedAlloc, ScopedMemTag},
    mem::{MEMORY_MANAGER, AllocInitState}
};

extern crate alloc;

pub struct Alloc {
    layout  : UnsafeCell<Layout>,
    mem_tag : UnsafeCell<MemTag>
}

impl Alloc {
    pub fn new() -> Self {
        Self::with_id(get_active_alloc().get_id(), get_active_mem_tag())
    }

    pub fn layout(&self) -> Layout {
        unsafe{ *self.layout.get() }
    } 

    pub fn mem_tag(&self) -> MemTag {
        unsafe{ *self.mem_tag.get() }
    } 

    fn with_id(id: u16, mem_tag: MemTag) -> Self {
        Alloc { layout: UnsafeCell::new(Layout::null().with_alloc_id(id)), mem_tag: UnsafeCell::new(mem_tag) }
    }
}

unsafe impl alloc::alloc::Allocator for Alloc {
    fn allocate(&self, layout: std::alloc::Layout) -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        let layout = Layout::new_size_align(layout.size(), layout.align());

        unsafe {
            let self_layout = &mut *self.layout.get();
            let self_mem_tag = &mut *self.mem_tag.get();
            
            let _scope_alloc = ScopedAlloc::new(UseAlloc::Id(self_layout.alloc_id()));
            let _scope_mem_tag = ScopedMemTag::new(*self_mem_tag);

            let alloc = MEMORY_MANAGER.alloc_raw(AllocInitState::Uninitialized, layout);

            match alloc {
                Some(ptr) => {
                    let slice_ptr = core::ptr::slice_from_raw_parts_mut(ptr.ptr_mut(), ptr.layout().size());
                    *self_layout = ptr.layout();
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
        let alloc = Allocation::new_tagged(ptr.as_ptr(), layout, mem_tag);
        MEMORY_MANAGER.dealloc(alloc);
    }
}

impl Clone for Alloc {
    fn clone(&self) -> Self {
        Self::with_id(self.layout().alloc_id(), self.mem_tag())
    }   
}