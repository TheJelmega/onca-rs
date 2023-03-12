use core::{
    cell::{UnsafeCell, Cell},
    ptr::{copy_nonoverlapping, write_bytes},
    borrow::BorrowMut
};
use crate::{
    alloc::{
        Allocator, Allocation, Layout, UseAlloc, NUM_RESERVED_ALLOC_IDS, get_active_alloc, ScopedAlloc,
        primitives::{Mallocator, FreelistAllocator},
    },
    sync::{RwLock, Mutex}, MiB, collections::HashMap
};

thread_local! {
    pub static TLS_TEMP_ALLOC : UnsafeCell<FreelistAllocator> = UnsafeCell::new(FreelistAllocator::new_uninit());
}

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
    malloc          : Mallocator,
    allocs          : [Option<*mut dyn Allocator>; MemoryManager::MAX_REGISTERABLE_ALLOCS as usize],
    default         : u16,
}

impl State {
    const fn new() -> Self {
        Self{ 
            malloc: Mallocator,
            allocs: [None; MemoryManager::MAX_REGISTERABLE_ALLOCS as usize],
            default: 0,
        }
    }
}

/// Memory manager
pub struct MemoryManager
{
    state : RwLock<State>,
    //layout_and_tags : Mutex<Option<LayoutAndMemtagStorage>>,
}

impl MemoryManager {
    /// Maximum number of registerable allocators
    pub const MAX_REGISTERABLE_ALLOCS : u16 = Layout::MAX_ALLOC_ID - NUM_RESERVED_ALLOC_IDS - 1; // - 1 for default alloc ID

    /// Create a new memory manager
    pub const fn new() -> Self {
        Self { state: RwLock::new(State::new()), /*layout_and_tags: Mutex::new(None)*/ }
    }

    /// Initialize the memory manager (needs to be called before doing any allocation)
    pub fn init(&self) {
        //let mut locked = self.layout_and_tags.lock();
        //if locked.is_none() {
        //    *locked = Some(LayoutAndMemtagStorage::new())
        //}
    }

    /// Register an allocator to the manager and set its allocator id
    pub fn register_allocator(&self, alloc: *mut dyn Allocator) -> u16 {
        let mut state = self.state.write();
        let idx = state.allocs.iter().position(|alloc| alloc.is_none());
        if let Some(idx) = idx {
            let id = idx as u16 + NUM_RESERVED_ALLOC_IDS;
            unsafe { (*alloc).set_alloc_id(id) };
            state.allocs[idx as usize] = Some(alloc);

            id
        } else {
            Layout::MAX_ALLOC_ID
        }
    }

    pub fn set_default_allocator(&self, alloc: UseAlloc) {
        self.state.write().default = alloc.get_id();
    }

    /// Get an allocator
    pub fn get_allocator(&self, alloc: UseAlloc) -> Option<&mut dyn Allocator> {
        /// Handle any default id
        match alloc {
            UseAlloc::Default => {
                let default_id = self.state.read().default;
                return self.get_allocator(UseAlloc::Id(default_id));
            },
            UseAlloc::Id(id) if id >= Layout::MAX_ALLOC_ID => {
                let default_id = self.state.read().default;
                return self.get_allocator(UseAlloc::Id(default_id));
            }
            _ => (),
        }

        let mut state = self.state.read();
        let alloc_ref : Option<&dyn Allocator> = match alloc {
            UseAlloc::Default => unreachable!(),
            UseAlloc::Malloc => Some(&state.malloc),
            UseAlloc::TlsTemp => Self::get_tls_alloc(),
            UseAlloc::Id(id) => {
                match id {
                    0 => Some(&state.malloc),
                    1 => Self::get_tls_alloc(),
                    id => match state.allocs[(id - NUM_RESERVED_ALLOC_IDS) as usize] {
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

    /// Allocate a raw allocation with the given layout, using the active allocator and memory tag.
    pub fn alloc_raw(&self, init_state: AllocInitState, layout: Layout) -> Option<Allocation<u8>> {
        let alloc = self.get_allocator(get_active_alloc())?;
        let mut ptr = unsafe{ alloc.alloc(layout) }?;
        if init_state == AllocInitState::Zeroed {
            unsafe { write_bytes(ptr.ptr_mut()  , 0, layout.size()) };
        }

        Some(ptr)
    }

    /// Allocate memory with the given layout, using the active allocator and memory tag.
    pub fn alloc<T>(&self, init_state: AllocInitState) -> Option<Allocation<T>> {
        self.alloc_raw(init_state, Layout::new::<T>()).map(|ptr| ptr.cast())
    }

    /// Deallocate memory
    pub fn dealloc<T: ?Sized>(&self, ptr: Allocation<T>) {
        if ptr.layout().size() == 0 {
            return;
        }

        if let Some(alloc) = self.get_allocator(UseAlloc::Id(ptr.layout().alloc_id())) {
            unsafe { (*alloc).dealloc(ptr.cast()) }
        } else {
            panic!("Failed to retrieve allocator to deallocate memory");
        }
    }

    /// Grow a given allocation to a newly provided size
    /// 
    /// Alignment of the new layout needs to match that of the old
    /// 
    /// If new memory was unable to be allocated, the result will contain an `Err(...)` with the original allocation
    pub fn grow<T>(&self, ptr: Allocation<T>, new_layout: Layout) -> Result<Allocation<T>, Allocation<T>> {
        /// Old layout could be larger, as allocators are free to allocate more memory that needed, and do report it in the returned layout
        if new_layout.size() <= ptr.layout().size() {
            return Ok(ptr);
        }

        let _scope_alloc = ScopedAlloc::new(UseAlloc::Id(ptr.layout().alloc_id()));

        if ptr.ptr() == core::ptr::null() {
            return match self.alloc_raw(AllocInitState::Uninitialized, new_layout) {
                Some(mem) => Ok(mem.cast()),
                None => Err(ptr)
            };
        }
        
        let copy_count = ptr.layout().size();
        match self.alloc_raw(AllocInitState::Uninitialized, new_layout) {
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
            return Ok(unsafe{ Allocation::<T>::const_null() });
        }

        // TODO(jel): should these be asserts or just return an Err
        assert!(new_layout.size() < ptr.layout().size(), "new size needs to be smaller that the current size");

        let _scope_alloc = ScopedAlloc::new(UseAlloc::Id(ptr.layout().alloc_id()));

        match self.alloc_raw(AllocInitState::Uninitialized, new_layout) {
            Some(mem) => unsafe {
                let count = mem.layout().size();
                copy_nonoverlapping(ptr.ptr() as *const u8, mem.ptr_mut(), count);
                self.dealloc(ptr);
                Ok(mem.cast())
            },
            None => Err(ptr)
        }
    }

    fn get_tls_alloc() -> Option<&'static dyn Allocator> {
        unsafe {
            let is_init = TLS_TEMP_ALLOC.with(|tls| (*tls.get()).is_initialized());
            if !is_init {
                let _scope_alloc = ScopedAlloc::new(UseAlloc::Malloc);

                let layout = Layout::new_size_align(MiB(1), 8);
                let buffer = MEMORY_MANAGER.alloc_raw(AllocInitState::Uninitialized, layout);
                let buffer = match buffer {
                    None => return None,
                    Some(buf) => buf
                };
                TLS_TEMP_ALLOC.with(|tls| {
                    (*tls.get()).init(buffer);
                    (*tls.get()).set_alloc_id(1)
                });
            }
            Some(&mut *TLS_TEMP_ALLOC.with(|tls| tls.get()))
        }
    }
}


unsafe impl Sync for MemoryManager {}
unsafe impl Send for MemoryManager {}