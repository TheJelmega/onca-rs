use core::cell::UnsafeCell;
use std::borrow::BorrowMut;
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

    /// Get an allocator from its id
    pub fn get_allocator(&self, id: u16) -> Option<&mut dyn Allocator> {
        let state = unsafe { &mut *self.state.get() };
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

    /// Get the default allocator
    pub fn get_default_allocator(&self) -> &mut dyn Allocator {
        let state = unsafe { &mut *self.state.get() };
        lock!(state.mutex);
        &mut state.malloc
    }

    /// Allocate a raw allocation with the given allocator and layout
    pub fn alloc_raw(&self, alloc: UseAlloc, layout: Layout) -> Option<Allocation<u8>> {
        let alloc = match alloc {
            UseAlloc::Default => self.get_default_allocator(),
            UseAlloc::Alloc(alloc) => alloc,
            UseAlloc::Id(id) => {
                match self.get_allocator(id) {
                    None => return None,
                    Some(alloc) => alloc
                }
            }
        };

        unsafe {
            match alloc.alloc(layout) {
                None => None,
                Some(ptr) => Some(ptr)
            }
        }
    }

    /// Allocate memory
    pub fn alloc<T>(&self, alloc: UseAlloc) -> Option<Allocation<T>> {
        match self.alloc_raw(alloc, Layout::new::<T>()) {
            None => None,
            Some(ptr) => Some(ptr.cast())
        }
    }

    /// Deallocate memory
    pub fn dealloc<T: ?Sized>(&self, ptr: Allocation<T>) {
        if let Some(alloc) = self.get_allocator(ptr.layout().alloc_id()) {
            unsafe {
                (*alloc).dealloc(ptr.cast())
            }
        } else {
            panic!("Failed to retrieve allocator to deallocate memory");
        }
    }
}


unsafe impl Sync for MemoryManager {}
unsafe impl Send for MemoryManager {}