use super::{mem_tag::MemTag, Allocation, Layout};
use core::cell::Cell;

/// Allocator/Arena that can provide access to heap memory for the program
pub trait Allocator {
    /// Allocate memory from an allocator/arena
    ///
    /// If no memory could be allocated, a null `Allocation<u8>` should be returned
    unsafe fn alloc(&mut self, layout: Layout, mem_tag: MemTag) -> Option<Allocation<u8>>;

    /// Deallocate an allocation
    ///
    /// #Panicks
    ///
    /// Deallocation may panic, since an incorrect allocation should not happen
    unsafe fn dealloc(&mut self, ptr: Allocation<u8>);

    /// Check if the allocator owns the allocation
    fn owns(&self, ptr: &Allocation<u8>) -> bool {
        ptr.layout().alloc_id() == self.alloc_id()
    }

    /// Sets the allocator id for the allocator
    fn set_alloc_id(&mut self, id: u16);

    /// Gets the allocator id for the allocator
    fn alloc_id(&self) -> u16;
}

/// An allocator that can be used to compose other allocators
pub trait ComposableAllocator<Args>: Allocator {
    /// Create a new allocator
    fn new_composable(args: Args) -> Self;

    /// Check if the composable sub-allocator owns the allocation
    fn owns_composable(&self, allocation: &Allocation<u8>) -> bool;
}

/// Enum telling what allocator to use for any structure that allocates memory
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UseAlloc {
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
/// - 0: Malloc
/// - 1: Tls Temporary
pub const NUM_RESERVED_ALLOC_IDS: u16 = 2;

impl UseAlloc {
    pub const fn get_id(&self) -> u16 {
        match self {
            UseAlloc::Default => Layout::MAX_ALLOC_ID,
            UseAlloc::Malloc => 0,
            Self::TlsTemp => 1,
            UseAlloc::Id(id) => *id,
        }
    }
}

impl From<&dyn Allocator> for UseAlloc {
    fn from(alloc: &dyn Allocator) -> Self {
        UseAlloc::Id(alloc.alloc_id())
    }
}

thread_local! {
    static TLS_ACTIVE_ALLOC_ID : Cell<u16> = Cell::new(Layout::MAX_ALLOC_ID);
}

/// Get the active allocator used on this thread
pub fn get_active_alloc() -> UseAlloc {
    UseAlloc::Id(TLS_ACTIVE_ALLOC_ID.get())
}

/// Set the active allocator used on this thread
pub fn set_active_alloc(alloc: UseAlloc) {
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
    pub fn new(alloc: UseAlloc) -> ScopedAlloc {
        let old_id = get_active_alloc().get_id();
        set_active_alloc(alloc);
        ScopedAlloc { old_id }
    }

    /// Set the current scoped allocator
    pub fn set(&self, alloc: UseAlloc) {
        set_active_alloc(alloc);
    }
}

impl Drop for ScopedAlloc {
    fn drop(&mut self) {
        set_active_alloc(UseAlloc::Id(self.old_id));
    }
}
