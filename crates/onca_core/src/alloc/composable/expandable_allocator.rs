use crate::{
    sync::Mutex,
    collections::DynArray,
    alloc::*,
};
use super::*;

// TODO(jel): Use atomic container and get rid of mutex
/// Expandable allocator
pub struct ExpandableArena<A, Args>
    where A    : Allocator + ComposableAllocator<Args>,
          Args : Copy
{
    allocs         : Mutex<DynArray<A>>,
    args           : Args,
    arena_alloc_id : u16,
    id             : u16
}

impl<A, Args> ExpandableArena<A, Args>
where A    : Allocator + ComposableAllocator<Args>,
      Args : Copy  
{
    pub fn new(args: Args, arena_alloc: UseAlloc) -> Self {
        Self {
            allocs: Mutex::new(DynArray::new(arena_alloc, CoreMemTag::Allocators.to_mem_tag())),
            args,
            arena_alloc_id: arena_alloc.get_id(),
            id: 0
        }
    }
}

impl<A, Args> Allocator for ExpandableArena<A, Args> 
where
    A    : Allocator + ComposableAllocator<Args>,
    Args : Copy   
{
    unsafe fn alloc(&mut self, layout: Layout, mem_tag: MemTag) -> Option<Allocation<u8>> {
        {
            let mut allocs = self.allocs.lock();
            for alloc in allocs.iter_mut() {
                match alloc.alloc(layout, mem_tag) {
                    Some(allocation) => return Some(allocation),
                    None => {}
                }
            }
        }

        let mut new_alloc = A::new_composable(UseAlloc::Id(self.arena_alloc_id), self.args);
        new_alloc.set_alloc_id(self.id);
        let allocation = new_alloc.alloc(layout, mem_tag);
        if let None = allocation {
            return None;
        }

        let mut allocs = self.allocs.lock();
        allocs.push(new_alloc);

        allocation
    }

    unsafe fn dealloc(&mut self, ptr: Allocation<u8>) {
        let mut allocs = self.allocs.lock();
        for alloc in allocs.iter_mut() {
            if alloc.owns_composable(&ptr) {
                alloc.dealloc(ptr);
                return;
            }
        }
    }

    fn set_alloc_id(&mut self, id: u16) {
        self.id = id;
    }

    fn alloc_id(&self) -> u16 {
        self.id
    }
}
