use core::{cell::UnsafeCell, ptr::{write_bytes, copy_nonoverlapping}};
use std::{borrow::BorrowMut, io};
use crate::{
    alloc::{Allocator, Allocation, Layout, primitives::Mallocator, UseAlloc},
    sync::Mutex,
    lock
};
use once_cell::sync::Lazy;

pub static MEMORY_MANAGER : MemoryManager = MemoryManager::new();

struct State
{
    malloc : Mallocator,
    allocs : [Option<*mut dyn Allocator>; Layout::MAX_ALLOC_ID as usize],
    mutex  : Mutex
}

impl State {
    
    fn new() -> Self {
        Self{ malloc: Mallocator, allocs: [None; Layout::MAX_ALLOC_ID as usize], mutex: Mutex::new() }
    }

}

/// Memory manager
// TODO: Extended tags
pub struct MemoryManager
{
    state : Lazy<UnsafeCell<State>>
}

impl MemoryManager {
    
    /// Create a new memory manager
    pub const fn new() -> Self {
        Self { state: Lazy::new(|| {
            let mut state = UnsafeCell::new(State::new());
            state.get_mut().mutex = Mutex::new();
            state
        }) }
    }

    /// Register an allocator to the manager and set its allocator id
    pub fn register_allocator(&self, alloc: *mut dyn Allocator) {
        let state = unsafe { &mut *self.state.get() };
        let mut id : usize = 0;

        lock!(state.mutex);
        for (i, alloc) in state.allocs.into_iter().enumerate() {
            if let None = alloc {
                id = i;
                break;
            }
        }

        unsafe { (*alloc).set_alloc_id(id as u16) };
        state.allocs[id] = Some(alloc);
    }

    /// Get an allocator
    pub fn get_allocator(&self, alloc: UseAlloc) -> Option<&mut dyn Allocator> {
        let state = unsafe { &mut *self.state.get() };
        match alloc {
            UseAlloc::Default => Some(&mut state.malloc),
            UseAlloc::Id(id) => {
                if id >= Layout::MAX_ALLOC_ID {
                    Some(&mut state.malloc)
                } else {
                    lock!(state.mutex);
                    match state.allocs[id as usize] {
                        None => None,
                        Some(alloc) => Some(unsafe{ &mut *alloc })
                    }
                }
            }
        }
    }

    /// Get the default allocator
    pub fn get_default_allocator(&self) -> &mut dyn Allocator {
        let state = unsafe { &mut *self.state.get() };
        lock!(state.mutex);
        &mut state.malloc
    }

    /// Allocate a raw allocation with the given allocator and layout
    pub fn alloc_raw(&self, alloc: UseAlloc, layout: Layout) -> Option<Allocation<u8>> {
        let alloc = self.get_allocator(alloc);
        match alloc {
            None => None,
            Some(alloc) => unsafe{ alloc.alloc(layout) }
        }
    }

    pub fn alloc_raw_zeroed(&self, alloc: UseAlloc, layout: Layout) -> Option<Allocation<u8>> {
        let allocation = self.alloc_raw(alloc, layout);
        match allocation {
            None => None,
            Some(ptr) => unsafe {
                write_bytes(ptr.ptr_mut(), 0, ptr.layout().size());
                Some(ptr)
            }
        }
    }

    /// Allocate memory with the given allocator
    pub fn alloc<T>(&self, alloc: UseAlloc) -> Option<Allocation<T>> {
        match self.alloc_raw(alloc, Layout::new::<T>()) {
            None => None,
            Some(ptr) => Some(ptr.cast())
        }
    }

    /// Allocate memory with the given allocator and zero it
    pub fn alloc_zeroed<T>(&self, alloc: UseAlloc) -> Option<Allocation<T>> {
        match self.alloc_raw_zeroed(alloc, Layout::new::<T>()) {
            None => None,
            Some(ptr) => Some(ptr.cast())
        }
    }

    /// Deallocate memory
    pub fn dealloc<T: ?Sized>(&self, ptr: Allocation<T>) {
        if let Some(alloc) = self.get_allocator(UseAlloc::Id(ptr.layout().alloc_id())) {
            unsafe {
                (*alloc).dealloc(ptr.cast())
            }
        } else {
            panic!("Failed to retrieve allocator to deallocate memory");
        }
    }

    /// Grow a given allocation to a newly provided size
    /// 
    /// Alignment of the new layout needs to match that of the old
    /// 
    /// If new memory was unable to be allocated, the result will contain an `Err(...)` with the original allocator
    pub fn grow<T>(&self, ptr: Allocation<T>, new_layout: Layout) -> Result<Allocation<T>, Allocation<T>> {
        // TODO(jel): should these be asserts or just return an Err
        assert!(new_layout.size() > ptr.layout().size(), "new size needs to be larger that the current size");
        assert!(ptr.ptr() != core::ptr::null(), "Cannot grow from null");

        if ptr.ptr() == core::ptr::null() {
            return match self.alloc_raw(UseAlloc::Id(ptr.layout().alloc_id()), new_layout) {
                Some(mem) => Ok(mem.cast()),
                None => Err(ptr)
            };
        }
        
        let copy_count = ptr.layout().size();
        match self.alloc_raw(UseAlloc::Id(ptr.layout().alloc_id()), new_layout) {
            Some(mem) => unsafe {
                copy_nonoverlapping(ptr.ptr() as *const u8, mem.ptr_mut(), copy_count);
                self.dealloc(ptr);
                Ok(mem.cast())
            },
            None => Err(ptr)
        }
    }

    /// Grow a given allocation to a newly provided size and zero the new memory
    /// 
    /// Alignment of the new layout needs to match that of the old
    /// 
    /// If new memory was unable to be allocated, the result will contain an `Err(...)` with the original allocator
    pub fn grow_zeroed<T>(&self, ptr: Allocation<T>, new_layout: Layout) -> Result<Allocation<T>, Allocation<T>> {
        let old_size = ptr.layout().size();
        match self.grow(ptr, new_layout) {
            Ok(mem) => unsafe {
                let new_size = mem.layout().size();
                let count = new_size - old_size;
                let write_ptr = (mem.ptr_mut() as *mut u8).add(old_size);
                core::ptr::write_bytes(write_ptr, 0, count);
                Ok(mem)
            },
            Err(mem) => Err(mem)
        }
    }

    /// Shrink a given allocator to a newly provided size
    pub fn shrink<T>(&self, ptr: Allocation<T>, new_layout: Layout) -> Result<Allocation<T>, Allocation<T>> {
        if new_layout.size() == 0 {
            self.dealloc(ptr);
            return Ok(unsafe{ Allocation::<T>::null() });
        }

        // TODO(jel): should these be asserts or just return an Err
        assert!(new_layout.size() < ptr.layout().size(), "new size needs to be larger that the current size");

        match self.alloc_raw(UseAlloc::Id(ptr.layout().alloc_id()), new_layout) {
            Some(mem) => unsafe {
                let count = mem.layout().size();
                copy_nonoverlapping(ptr.ptr() as *const u8, mem.ptr_mut(), count);
                self.dealloc(ptr);
                Ok(mem.cast())
            },
            None => Err(ptr)
        }
    }
}


unsafe impl Sync for MemoryManager {}
unsafe impl Send for MemoryManager {}