use std::{
    cell::{UnsafeCell, Cell},
    ptr::{copy_nonoverlapping, write_bytes, null},
    borrow::BorrowMut,
    sync::atomic::AtomicUsize,
    alloc::GlobalAlloc, num::NonZeroU16,
};

use std::{alloc::Layout, ptr::NonNull};

use once_cell::sync::OnceCell;

use crate::{
    alloc::{
        Allocator, AllocId, NUM_RESERVED_ALLOC_IDS, get_active_alloc, ScopedAlloc,
        primitives::{Mallocator, FreelistAllocator}, AllocHeader, onca_free, onca_malloc,
    },
    sync::{RwLock, Mutex}, MiB, scoped_alloc
};

thread_local! {
    pub static TLS_ALLOC: OnceCell<UnsafeCell<FreelistAllocator>> = OnceCell::new();
}

struct MemoryManagerPtr(*const MemoryManager);

unsafe impl Send for MemoryManagerPtr {}
unsafe impl Sync for MemoryManagerPtr {}

static MEMORY_MANAGER : RwLock<MemoryManagerPtr> = RwLock::new(MemoryManagerPtr(null()));

pub fn set_memory_manager(manager: &MemoryManager) {
    *MEMORY_MANAGER.write() = MemoryManagerPtr(manager as *const _);
}

pub fn get_memory_manager() -> &'static MemoryManager {
    let ptr = MEMORY_MANAGER.read().0;
    assert!(ptr != null(), "Memory manager was not set");
    unsafe { &*ptr }
}

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
    malloc          : UnsafeCell<Mallocator>,
    allocs          : [Option<*mut dyn Allocator>; MemoryManager::MAX_REGISTERABLE_ALLOCS as usize + 1],
    // Cannot be 0, as 0 is the untracked allocator and is a special case
    default         : NonZeroU16,
}

impl State {
    const fn new() -> Self {
        Self{ 
            malloc: UnsafeCell::new(Mallocator),
            allocs: [None; MemoryManager::MAX_REGISTERABLE_ALLOCS as usize + 1],
            default: unsafe { NonZeroU16::new_unchecked(1) },
        }
    }
}

/// Memory manager
pub struct MemoryManager
{
    state : RwLock<State>,
}

impl MemoryManager {
    /// Maximum number of registerable allocators
    pub const MAX_REGISTERABLE_ALLOCS: u16 = Self::MAX_ALLOC_ID - NUM_RESERVED_ALLOC_IDS - 1; // - 1 for default alloc ID

    /// Maximum valid allocator id
    pub const MAX_ALLOC_ID: u16 = 4095;

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
            Self::MAX_ALLOC_ID
        }
    }

    pub fn set_default_allocator(&self, alloc: AllocId) {
        let default = match NonZeroU16::new(alloc.get_id()) {
            Some(id) => id,
            // Cannot set the default allocator to untracked
            None => return,
        };

        assert!(alloc.get_id() != Self::MAX_ALLOC_ID);
        self.state.write().default = default;
    }

    /// Get an allocator
    pub fn get_allocator(&self, mut alloc: AllocId) -> Option<&mut dyn Allocator> {
        /// Handle any default id
        if alloc.get_id() == Self::MAX_ALLOC_ID {
            let default_id = self.state.read().default;
            alloc = AllocId::Id(default_id.get());
        }

        let mut state = self.state.read();
        let alloc_ref : Option<&mut dyn Allocator> = match alloc.get_id() {
            0 => unreachable!("Cannot get the untracked allocator directly"),
            1 => Some(unsafe { &mut *state.malloc.get()}),
            2 => Self::get_tls_alloc(),
            id => match state.allocs[(id - NUM_RESERVED_ALLOC_IDS) as usize] {
                None => None,
                Some(alloc) => Some(unsafe{ &mut *alloc })
            }
        };
        alloc_ref
    }

    /// Allocate a raw allocation with the given layout, using the active allocator and memory tag.
    pub unsafe fn alloc_raw(&self, init_state: AllocInitState, layout: Layout, alloc_id_override: Option<AllocId>) -> Option<NonNull<u8>> {
        Self::handle_alloc(layout, init_state, true, alloc_id_override, |alloc_id, layout| {
            if alloc_id == AllocId::Untracked {
                (onca_malloc(layout), true)
            } else {
               match self.get_allocator(alloc_id) {
                   Some(alloc) => (alloc.alloc(layout), alloc.supports_free()),
                   None => (None, false),
               }
            }

        })
    }

    /// Deallocate memory
    pub unsafe fn dealloc(&self, ptr: NonNull<u8>, layout: Layout) {
        Self::handle_dealloc(ptr, layout, |alloc_id, ptr, layout| {
            if alloc_id == AllocId::Untracked {
                onca_free(ptr, layout);
            } else {
               if let Some(alloc) = self.get_allocator(alloc_id) {
                   alloc.dealloc(ptr.cast(), layout)
               } else {
                   panic!("Failed to retrieve allocator to deallocate memory");
               }
            }
            
        })
    }

    pub unsafe fn alloc_untracked(init_state: AllocInitState, layout: Layout) -> NonNull<u8> {
        // Ensure the correct alloc id is used
        scoped_alloc!(AllocId::Untracked);
        Self::handle_alloc(layout, init_state, false, Some(AllocId::Untracked), |_, layout| {
            (onca_malloc(layout), true)
        }).unwrap()
    }

    pub unsafe fn dealloc_untracked(ptr: NonNull<u8>, layout: Layout) {
        Self::handle_dealloc(ptr, layout, |alloc_id, ptr, layout| {
            assert!(alloc_id == AllocId::Untracked, "Trying to deallocate tracked memory as untracked memory");
            onca_free(ptr, layout);
        })
    }

    unsafe fn handle_alloc<F>(layout: Layout, init_state: AllocInitState, tracked: bool, alloc_id_override: Option<AllocId>, f: F) -> Option<NonNull<u8>> where
        F: Fn(AllocId, Layout) -> (Option<NonNull<u8>>, bool)
    {
        let alloc_id = alloc_id_override.unwrap_or_else(|| get_active_alloc());
        let (layout, offset) = Self::calc_layout_and_offset(layout)?;

        let (ptr, freeable) = f(alloc_id, layout);
        let ptr = ptr?;
        if init_state == AllocInitState::Zeroed {
            write_bytes(ptr.as_ptr(), 0, layout.size());
        }

        let ptr = NonNull::new_unchecked(ptr.as_ptr().add(offset));
        let alloc_header = AllocHeader::from_non_null(ptr);
        // TODO: tracking
        *alloc_header = AllocHeader::new(alloc_id.get_id(), tracked, freeable);

        Some(ptr)
    }

    unsafe fn handle_dealloc<F>(ptr: NonNull<u8>, layout: Layout, f: F) where
        F: Fn(AllocId, NonNull<u8>, Layout)
    {
        let alloc_header = AllocHeader::from_non_null(ptr);
        if !alloc_header.is_freeable() {
            return;
        }

        let (layout, offset) = Self::calc_layout_and_offset(layout).expect("Failed to recreate layout which allocated memory");
        let alloc_id = AllocId::Id(alloc_header.alloc_id());

        let dealloc_ptr = NonNull::new_unchecked(ptr.as_ptr().sub(offset));
        f(alloc_id, dealloc_ptr, layout);
    }

    fn calc_layout_and_offset(layout: Layout) -> Option<(Layout, usize)> {
        const HEADER_LAYOUT: Layout = Layout::new::<AllocHeader>();
        match HEADER_LAYOUT.extend(layout) {
            Ok(tuple) => Some(tuple),
            Err(_) => None,
        }
    }

    fn get_tls_alloc<'a>() -> Option<&'a mut dyn Allocator> {
        // .with() could allocate memory and if we keep the tls stack for that, we'll get into infinite recursion
        scoped_alloc!(AllocId::Malloc);

        Some(TLS_ALLOC.with(|tls| {
            let alloc = tls.get_or_init(|| unsafe {
                let layout = Layout::from_size_align(MiB(1), 8).unwrap();
                let buffer = get_memory_manager().alloc_raw(AllocInitState::Uninitialized, layout, Some(AllocId::Malloc)).expect("Failed to allocate TLS allocator memory buffer");
                UnsafeCell::new(FreelistAllocator::new(buffer, layout))
            });
            

            // SAFETY: This will always be valid when called
            unsafe { &mut *alloc.get() }
        }))
    }
}


unsafe impl Sync for MemoryManager {}
unsafe impl Send for MemoryManager {}