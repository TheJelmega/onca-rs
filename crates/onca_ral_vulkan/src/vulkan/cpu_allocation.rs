use core::{ptr::null_mut, ffi::c_void, mem::ManuallyDrop};

use onca_core::{
    prelude::*,
    alloc::{Layout, Allocation},
    collections::HashMap,
    mem::{AllocInitState, get_memory_manager},
    sync::Mutex
};
use ash::vk;

pub struct AllocationUserdata {
    alloc          : UseAlloc,
    // TODO: If layouts are stored differently, this could be removed
    layout_mapping : HashMap<*const u8, Layout>
}

#[derive(Clone)]
pub struct AllocationCallbacks {
    callbacks : vk::AllocationCallbacks,
    user_data : Arc<Mutex<AllocationUserdata>>
}

type UserDataType = Mutex<AllocationUserdata>;

impl AllocationCallbacks {

    pub fn new(alloc: UseAlloc) -> AllocationCallbacks {
        let mut this = Self {
            callbacks: vk::AllocationCallbacks::builder()
                .pfn_allocation(Some(Self::alloc))
                .pfn_reallocation(Some(Self::realloc))
                .pfn_free(Some(Self::free))
                .pfn_internal_allocation(Some(Self::notify_alloc))
                .pfn_internal_free(Some(Self::notify_free))
            .build(),
            user_data: Arc::new(Mutex::new(AllocationUserdata {
                alloc,
                layout_mapping: HashMap::new(),
            })),
        };

        this.callbacks.p_user_data = unsafe { Arc::data_ptr(&this.user_data) as *mut c_void };
        this
    }

    //pub fn set_alloc(&mut self, alloc: UseAlloc) {
    //    self.user_data.lock().alloc = alloc;
    //}

    pub fn get_some_vk_callbacks(&self) -> Option<&vk::AllocationCallbacks> {
        Some(&self.callbacks)
    }

    pub fn get_vk_callbacks(&self) -> &vk::AllocationCallbacks {
        &self.callbacks
    }

    extern "system" fn alloc(userdata: *mut c_void, size: usize, align: usize, _alloc_scope: vk::SystemAllocationScope) -> *mut c_void {
        // TODO: alloc_scope to mem tag when mem tags are reimplemented
        let this_mutex = unsafe { &mut *(userdata as *mut UserDataType) };
        let mut this = this_mutex.lock();
        let layout = Layout::new_size_align(size, align);
        let _scope_alloc = ScopedAlloc::new(this.alloc);

        let alloc = match get_memory_manager().alloc_raw(AllocInitState::Uninitialized, layout) {
            Some(alloc) => alloc,
            None => return null_mut()
        };
        let alloc = ManuallyDrop::new(alloc);

        this.layout_mapping.insert(alloc.ptr(), alloc.layout());
        alloc.ptr_mut() as *mut c_void
    }

    extern "system" fn realloc(userdata: *mut c_void, original: *mut c_void, size: usize, align: usize, _alloc_scope: vk::SystemAllocationScope) -> *mut c_void {
        // TODO: alloc_scope to mem tag when mem tags are reimplemented
        let this_mutex = unsafe { &mut *(userdata as *mut UserDataType) };
        let mut this = this_mutex.lock();
        let layout = Layout::new_size_align(size, align);
        let _scope_alloc = ScopedAlloc::new(this.alloc);

        // Directly access, as vulkan should not be able to give a pointer that wasn't allocated via Self::alloc
        let old_ptr = original as *const u8;
        let old_layout = this.layout_mapping[&old_ptr];
        let old = Allocation::from_raw(original, old_layout);

        let new = match get_memory_manager().realloc(old, layout, AllocInitState::Uninitialized) {
            Ok(new) => new,
            Err(old) => {
                // According to vulkan spec, we must not free the old allocation
                _ = ManuallyDrop::new(old);
                return null_mut();
            }
        };

        let new_ptr = new.ptr() as *const u8;
        if old_ptr != new_ptr {
            this.layout_mapping.remove(&old_ptr);
        }
        this.layout_mapping.insert(new_ptr, layout);
        
        let new = ManuallyDrop::new(new);
        new.ptr_mut()
    }

    extern "system" fn free(userdata: *mut c_void, memory: *mut c_void) {
        if memory == null_mut() {
            return;
        }

        let this_mutex = unsafe { &mut *(userdata as *mut UserDataType) };
        let mut this = this_mutex.lock();

        // Directly access, as vulkan should not be able to give a pointer that wasn't allocated via Self::alloc
        let ptr = memory as *const u8;
        let layout = this.layout_mapping[&ptr];
        this.layout_mapping.remove(&ptr);

        get_memory_manager().dealloc(Allocation::new(memory, layout));
    }

    extern "system" fn notify_alloc(_userdata: *mut c_void, _size: usize, _alloc_type: vk::InternalAllocationType, _scope: vk::SystemAllocationScope) {
        // TODO
    }

    extern "system" fn notify_free(_userdata: *mut c_void, _size: usize, _alloc_type: vk::InternalAllocationType, _scope: vk::SystemAllocationScope) {
        // TODO
    }
}