use core::cell::Cell;
use std::{ptr::NonNull, alloc::Layout};

use crate::mem::{MemTag, MemoryManager};

//------------------------------------------------------------------------------------------------------------------------------

/// Header pefixed to allocation with info + magic value
/// ```
/// check_code              tracked    unused
///     |     memory tag      | freeable |  alloc_idalloc_id
/// +------+-----....------+----+----+--------+--------+
/// | AC40 |               |    |    |        |        |
/// +------+-----....------+----+----+--------+--------+
/// 0     16               48   49   50       52       64
/// ````
pub struct AllocHeader {
    data: u16,
    memtag: MemTag,
    magic: [u8; 2],
}

impl AllocHeader { 
    const MAGIC: [u8; 2] = [ 0x04, 0xCA ];
    const ALLOC_ID_MASK: u16 = 0x0FFF;
    const TRACKED_BIT:   u16 = 0x8000;
    const FREEABLE_BIT:   u16 = 0x4000;

    /// Create a new alloc header
    pub fn new(alloc_id: u16, tracked: bool, freeable: bool) -> Self {
        let alloc_id = alloc_id & Self::ALLOC_ID_MASK;
        let tracked = if tracked { Self::TRACKED_BIT } else { 0 };
        let freeable = if freeable { Self::FREEABLE_BIT } else { 0 };
        Self {
            data: alloc_id | tracked | freeable,
            memtag: MemTag::default(),
            magic: Self::MAGIC,
        }
    }


    /// Get the allocator_id
    pub fn alloc_id(&self) -> u16 {
        self.data & Self::ALLOC_ID_MASK
    }

    /// Check if the allocation is tracked
    pub fn is_tracked(&self) -> bool {
        self.data & Self::TRACKED_BIT == Self::TRACKED_BIT
    }

    /// Check if the allocation is tracked
    pub fn is_freeable(&self) -> bool {
        self.data & Self::FREEABLE_BIT == Self::FREEABLE_BIT
    }

    /// Check fi the allocation is valid
    pub fn is_valid(&self) -> bool {
        self.magic == Self::MAGIC
    }

    // Get the header that prepends the allocation
    /// 
    /// SAFETY
    /// 
    /// Returned static reference is only valid as long as the allocation is valid
    pub unsafe fn from_non_null<T: ?Sized>(ptr: NonNull<T>) -> &'static mut Self {
        let ptr = (ptr.as_ptr() as *mut AllocHeader).offset(-1);
        &mut *ptr
    }
}

static_assertions::const_assert!(AllocHeader::ALLOC_ID_MASK >= MemoryManager::MAX_ALLOC_ID);

//------------------------------------------------------------------------------------------------------------------------------

//------------------------------------------------------------------------------------------------------------------------------

/// Allocator/Arena that can provide access to heap memory for the program
pub trait Allocator {
    /// Allocate memory from an allocator/arena
    ///
    ///  # Return
    /// 
    /// If no memory could be allocated, `None` should be returned.
    /// 
    /// If memory could be allocated, the memory needs to have at least 8 bytes of space before the returned pointer to store metadata
    /// 
    /// # Guarantees
    /// 
    /// It is guaranteed that the layout will have:
    /// - an alignment of at least 8 bytes.
    /// - Included overhead to store metadata (this will be equal to the minimum of the `min(alignment, 8)`)
    unsafe fn alloc(&mut self, layout: Layout) -> Option<NonNull<u8>>;

    /// Deallocate an allocation
    ///
    /// #Panicks
    ///
    /// Deallocation may panic, since an incorrect allocation should not happen
    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout);

    /// Check if the allocator own the allocation
    /// 
    /// This function is mainly meant for sub-allocators, as the main allocator the memory comes from will be defined in the allocation's
    fn owns(&self, ptr: NonNull<u8>, layout: Layout) -> bool;

    /// Set the allocator's alloc id
    fn set_alloc_id(&mut self, id: u16);
    /// Get the allocator's alloc id
    fn alloc_id(&self) -> u16;

    /// Does the allocator support freeing of memory, if not, it means freeing of memory will only happen when the full allocator has been reset
    /// 
    /// Default implementation assumes the allocator can free
    fn supports_free(&self) -> bool { true }
}

/// An allocator that can be used to compose other allocators
pub trait ComposableAllocator<Args>: Allocator {
    /// Create a new allocator
    fn new_composable(args: Args) -> Self;
}

//------------------------------------------------------------------------------------------------------------------------------

/// Enum telling what allocator to use for any structure that allocates memory
#[derive(Clone, Copy, Eq, Debug)]
pub enum AllocId {
    /// Use the untracked allocator (direct call to mallocator, ignoring memory manager)
    Untracked,
    /// Use the default allocator
    Default,
    /// Use the system allocator
    Malloc,
    /// Use a thead local temporary allocator (1MiB per thread, maximum alignment of 16-bytes).
    ///
    /// Note: Uses a stack allocator, so allocations need to be deallocated in the reverse order of allocation.
    TlsTemp,
    /// Use the allocator associated with the given id
    Id(u16),
}

/// Reserved allocs IDs
/// - 0: Untracked
/// - 1: Malloc
/// - 2: Tls Temporary
pub const NUM_RESERVED_ALLOC_IDS: u16 = 3;

impl AllocId {
    pub const fn get_id(&self) -> u16 {
        match self {
            Self::Default => MemoryManager::MAX_ALLOC_ID,
            Self::Untracked => 0,
            Self::Malloc => 1,
            Self::TlsTemp => 2,
            Self::Id(id) => *id,
        }
    }
}

impl PartialEq for AllocId {
    fn eq(&self, other: &Self) -> bool {
        self.get_id() == other.get_id()
    }
}

impl From<&dyn Allocator> for AllocId {
    fn from(alloc: &dyn Allocator) -> Self {
        AllocId::Id(alloc.alloc_id())
    }
}

thread_local! {
    // Default to untracked allocations
    static TLS_ACTIVE_ALLOC_ID : Cell<u16> = Cell::new(0);
}

/// Get the active allocator used on this thread
pub fn get_active_alloc() -> AllocId {
    AllocId::Id(TLS_ACTIVE_ALLOC_ID.get())
}

/// Set the active allocator used on this thread
pub fn set_active_alloc(alloc: AllocId) {
    TLS_ACTIVE_ALLOC_ID.set(alloc.get_id());
}

/// Scoped allocator.
///
/// Sets the active alloc on this thread for the current scope, and resets it to the previous allocator once it exits the scope
pub struct ScopedAlloc {
    old_id: u16,
}

impl ScopedAlloc {
    /// Create a new scoped allocator
    pub fn new(alloc: AllocId) -> ScopedAlloc {
        let old_id = get_active_alloc().get_id();
        set_active_alloc(alloc);
        ScopedAlloc { old_id }
    }

    /// Set the current scoped allocator
    pub fn set(&self, alloc: AllocId) {
        set_active_alloc(alloc);
    }
}

impl Drop for ScopedAlloc {
    fn drop(&mut self) {
        set_active_alloc(AllocId::Id(self.old_id));
    }
}

#[macro_export]
macro_rules! scoped_alloc {
    ($alloc:expr) => {
        let _scope_alloc = ScopedAlloc::new($alloc);
    };
}