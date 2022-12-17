use super::{Layout, Allocation, mem_tag::MemTag};

/// Allocator/Arena that can provide access to heap memory for the program
pub trait Allocator
{
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
    fn owns(&self, ptr: &Allocation<u8>) -> bool { ptr.layout().alloc_id() == self.alloc_id() }

    /// Sets the allocator id for the allocator
    fn set_alloc_id(&mut self, id: u16);

    /// Gets the allocator id for the allocator
    fn alloc_id(&self) -> u16;
}

/// An allocator that can be used to compose other allocators
pub trait ComposableAllocator<Args> : Allocator
{
    /// Create a new allocator
    fn new_composable(alloc: &mut dyn Allocator, args: Args, mem_tag: MemTag) -> Self;
}

/// Enum telling what allocator to use for any structure that allocates memory
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UseAlloc {
    /// Use the default allocator
    Default,
    /// Use the system allocator
    Malloc,
    /// Use the allocator associated with the given id
    Id(u16)
}

impl UseAlloc {
    pub const fn get_id(&self) -> u16 {
        match self {
            UseAlloc::Default => Layout::MAX_ALLOC_ID,
            UseAlloc::Malloc => 0,
            UseAlloc::Id(id) => *id,
        }
    }
}

impl From<&dyn Allocator> for UseAlloc {
    fn from(alloc: &dyn Allocator) -> Self {
        UseAlloc::Id(alloc.alloc_id())
    }
}