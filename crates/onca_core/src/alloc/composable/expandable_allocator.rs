use crate::{
    sync::Mutex,
    alloc::*,
};
use super::*;

/// Expandable allocator
pub struct ExpandableArena<A, Args>
    where A    : Allocator + ComposableAllocator<Args>,
          Args : Copy
{
    allocs         : Mutex<Vec<A>>,
    args           : Args,
    arena_alloc_id : u16,
    id             : u16
}

impl<A, Args> ExpandableArena<A, Args>
where A    : Allocator + ComposableAllocator<Args>,
      Args : Copy  
{
    pub fn new(args: Args, arena_alloc: UseAlloc) -> Self {
        let _scope_alloc = ScopedAlloc::new(arena_alloc);
        Self {
            allocs: Mutex::new(Vec::new()),
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
    unsafe fn alloc(&mut self, layout: Layout) -> Option<Allocation<u8>> {
        {
            let mut allocs = self.allocs.lock();
            for alloc in allocs.iter_mut() {
                match alloc.alloc(layout) {
                    Some(allocation) => return Some(allocation),
                    None => {}
                }
            }
        }
        let _scope_alloc = ScopedAlloc::new(UseAlloc::Id(self.arena_alloc_id));

        let mut new_alloc = A::new_composable(self.args);
        new_alloc.set_alloc_id(self.id);
        let allocation = new_alloc.alloc(layout);
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
