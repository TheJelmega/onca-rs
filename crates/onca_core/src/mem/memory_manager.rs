use core::{cell::UnsafeCell, ptr::{write_bytes, copy_nonoverlapping}};
use std::{borrow::BorrowMut, io};
use crate::{
    alloc::{Allocator, Allocation, Layout, primitives::Mallocator, UseAlloc, MemTag},
    sync::RwLock
};
use once_cell::sync::Lazy;

pub static MEMORY_MANAGER : MemoryManager = MemoryManager::new();

/// Defines how memory should be initialized, i.e. uninitialized or zeroed
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AllocInitState {
    /// The contents of the new memeory are uninitialized
    Uninitialized,
    /// The new memory is guaranteed to be zeroed
    Zeroed,
}

struct State
{
    malloc : Mallocator,
    allocs : [Option<*mut dyn Allocator>; Layout::MAX_ALLOC_ID as usize],
}

impl State {
    
    const fn new() -> Self {
        Self{ 
            malloc: Mallocator,
            allocs: [None; Layout::MAX_ALLOC_ID as usize]
        }
    }
}

/// Memory manager
// TODO: Extended tags
pub struct MemoryManager
{
    state : RwLock<State>
}

impl MemoryManager {
    /// Create a new memory manager
    pub const fn new() -> Self {
        Self { state: RwLock::new(State::new()) }
    }

    /// Register an allocator to the manager and set its allocator id
    pub fn register_allocator(&self, alloc: *mut dyn Allocator) {
        let mut id : u16 = 0;
        {
            let state = self.state.read();
            for (i, alloc) in state.allocs.into_iter().enumerate() {
                if let None = alloc {
                    id = i as u16;
                    break;
                }
            }
        }
        {
            let mut state = self.state.write();
            unsafe { (*alloc).set_alloc_id(id) };
            state.allocs[id as usize] = Some(alloc);
        }
    }

    /// Get an allocator
    pub fn get_allocator(&self, alloc: UseAlloc) -> Option<&mut dyn Allocator> {
        let mut state = self.state.read();
        let alloc_ref : Option<&dyn Allocator> = match alloc {
            // TODO(jel): default alloc is not always the mallocator
            UseAlloc::Default => Some(unsafe { &state.malloc }),
            UseAlloc::Malloc => Some(& state.malloc),
            UseAlloc::Id(id) => {
                if id == 0 {
                    // TODO(jel): default alloc is not always the mallocator
                    Some(&state.malloc)
                } else if id >= Layout::MAX_ALLOC_ID {
                    Some(&state.malloc)
                } else {
                    match state.allocs[id as usize] {
                        None => None,
                        Some(alloc) => Some(unsafe{ &*alloc })
                    }
                }
            }
        };
        
        alloc_ref.map(|val| unsafe {
            // SAFETY: Memory manager will never move, so pointer casting will always result in a correct result
            let mut_ptr = val as *const dyn Allocator as *mut dyn Allocator;
            &mut *mut_ptr
        })
    }

    /// Allocate a raw allocation with the given allocator and layout
    pub fn alloc_raw(&self, init_state: AllocInitState, alloc: UseAlloc, layout: Layout, mem_tag: MemTag) -> Option<Allocation<u8>> {
        let alloc = self.get_allocator(alloc);
        match alloc {
            None => None,
            Some(alloc) => {
                match unsafe{ alloc.alloc(layout, mem_tag) } {
                    None => None,
                    Some(ptr) => {
                        if init_state == AllocInitState::Zeroed {
                            unsafe { write_bytes(ptr.ptr_mut(), 0, layout.size()) };
                        }
                        Some(ptr)
                    }
                }
            }
        }
    }

    /// Allocate memory with the given allocator
    pub fn alloc<T>(&self, init_state: AllocInitState, alloc: UseAlloc, mem_tag: MemTag) -> Option<Allocation<T>> {
        match self.alloc_raw(init_state, alloc, Layout::new::<T>(), mem_tag) {
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
            return match self.alloc_raw(AllocInitState::Uninitialized, UseAlloc::Id(ptr.layout().alloc_id()), new_layout, ptr.mem_tag()) {
                Some(mem) => Ok(mem.cast()),
                None => Err(ptr)
            };
        }
        
        let copy_count = ptr.layout().size();
        match self.alloc_raw(AllocInitState::Uninitialized, UseAlloc::Id(ptr.layout().alloc_id()), new_layout, ptr.mem_tag()) {
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

        match self.alloc_raw(AllocInitState::Uninitialized, UseAlloc::Id(ptr.layout().alloc_id()), new_layout, ptr.mem_tag()) {
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