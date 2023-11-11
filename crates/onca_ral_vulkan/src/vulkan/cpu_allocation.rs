use std::{
    ptr::null_mut,
    ffi::c_void,
    collections::HashMap,
    sync::Arc,
    alloc::Layout,
    ptr::NonNull
};

use onca_common::{
    prelude::*,
    mem::{AllocInitState, get_memory_manager},
    sync::Mutex
};
use ash::vk;

pub struct AllocationUserdata {
    alloc          : AllocId,
    layout_mapping : HashMap<NonNull<u8>, Layout>
}

#[derive(Clone)]
pub struct AllocationCallbacks {
    callbacks : vk::AllocationCallbacks,
    user_data : Arc<Mutex<AllocationUserdata>>
}

type UserDataType = Mutex<AllocationUserdata>;

impl AllocationCallbacks {

    pub fn new(alloc: AllocId) -> AllocationCallbacks {
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

        this.callbacks.p_user_data = Arc::as_ptr(&this.user_data) as *mut c_void;
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
        let layout = unsafe { Layout::from_size_align_unchecked(size, align) };
        let _scope_alloc = ScopedAlloc::new(this.alloc);

        let alloc = match unsafe { get_memory_manager().alloc_raw(AllocInitState::Uninitialized, layout, None) } {
            Some(alloc) => alloc,
            None => return null_mut()
        };

        this.layout_mapping.insert(alloc, layout);
        alloc.as_ptr().cast()
    }

    extern "system" fn realloc(userdata: *mut c_void, original: *mut c_void, size: usize, align: usize, _alloc_scope: vk::SystemAllocationScope) -> *mut c_void {
        // TODO: alloc_scope to mem tag when mem tags are reimplemented
        let this_mutex = unsafe { &mut *(userdata as *mut UserDataType) };
        let mut this = this_mutex.lock();
        let layout = unsafe { Layout::from_size_align_unchecked(size, align) };

        scoped_alloc!(this.alloc);
        if size == 0 {
            return match unsafe { get_memory_manager().alloc_raw(AllocInitState::Uninitialized, layout, None) } {
                Some(ptr) => {
                    this.layout_mapping.insert(ptr, layout);
                    ptr.as_ptr().cast()
                },
                None => null_mut(),
            };
        }

        // Directly access, as vulkan should not be able to give a pointer that wasn't allocated via Self::alloc
        let old_ptr = original as *mut u8;
        let old_layout = this.layout_mapping.remove(&unsafe { NonNull::new_unchecked(old_ptr) }).expect("Trying to get the layout for an allocation that was never allocated");

        let ptr = unsafe {
            let ptr = match get_memory_manager().alloc_raw(AllocInitState::Uninitialized, layout, None) {
                Some(new) => new,
                None => {
                    // According to vulkan spec, we must not free the old allocation
                    return null_mut();
                },
            };

            let copy_size = old_layout.size().min(layout.size());
            std::ptr::copy_nonoverlapping(old_ptr, ptr.as_ptr(), copy_size);

            get_memory_manager().dealloc(NonNull::new_unchecked(old_ptr), old_layout);

            ptr
        };

        this.layout_mapping.insert(ptr, layout);
        ptr.as_ptr().cast()
    }

    extern "system" fn free(userdata: *mut c_void, memory: *mut c_void) {
        if memory == null_mut() {
            return;
        }

        let this_mutex = unsafe { &mut *(userdata as *mut UserDataType) };
        let mut this = this_mutex.lock();

        // Directly access, as vulkan should not be able to give a pointer that wasn't allocated via Self::alloc
        let ptr = unsafe { NonNull::new_unchecked(memory) }.cast();
        let layout = this.layout_mapping.remove(&ptr).expect("Trying to get the layout for an allocation that was never allocated");

        unsafe { get_memory_manager().dealloc(ptr, layout) };
    }

    extern "system" fn notify_alloc(_userdata: *mut c_void, _size: usize, _alloc_type: vk::InternalAllocationType, _scope: vk::SystemAllocationScope) {
        // TODO
    }

    extern "system" fn notify_free(_userdata: *mut c_void, _size: usize, _alloc_type: vk::InternalAllocationType, _scope: vk::SystemAllocationScope) {
        // TODO
    }
}