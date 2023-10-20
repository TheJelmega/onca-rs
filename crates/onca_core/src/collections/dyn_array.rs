use core::{
    fmt,
    slice::{self, SliceIndex},
    iter,
    iter::FusedIterator,
    mem::{self, MaybeUninit, ManuallyDrop},
    ops::{RangeBounds, Range, Deref, DerefMut, Index, IndexMut},
    ptr::{self, NonNull},
    hash::{Hash, Hasher},
    array,
    cmp,
};
use crate::{
    alloc::{UseAlloc, Allocation, Layout, self, ScopedAlloc},
    mem::{AllocInitState, get_memory_manager},
    KiB,
};

use super::{ExtendFunc, ExtendElement, impl_slice_partial_eq_generic, imp::dyn_array::SliceToImpDynArray};
use super::imp::dyn_array as imp;
use imp::DynArrayBuffer;

extern crate alloc as rs_alloc;

// Even if we wanted to, we can't exactly wrap alloc::vec::RawVec as it isn't public
pub(super) struct DynamicBuffer<T> {
    ptr : Allocation<T>,
    cap : usize
}

impl<T> DynamicBuffer<T> {
    // Tiny Buffers are dumb, skip to:
    // - 8 if the element size is 1, because any heap allocator is likely to round up a request of less than 8 bytes to at least 8 bytes.
    // - 4 if elements are moderate-size (<= `KiB).
    // - 1 otherwise, to acoid wastin too much space for very short dynarrs
    const MIN_NON_ZERO_CAP : usize = if mem::size_of::<T>() == 1 {
        8
    } else if mem::size_of::<T>() <= KiB(1) {
        4
    } else {
        1
    };

    fn allocate(capacity: usize, init_state: AllocInitState) -> Self {
        if mem::size_of::<T>() == 0 || capacity == 0 {
            Self::new()
        } else {
            let layout = Layout::array::<T>(capacity);
            let res = get_memory_manager().alloc_raw(init_state, layout);
            let ptr = match res {
                Some(ptr) => ptr,
                None => panic!("Failed to allocate memory")
            }.cast();

            Self { ptr, cap:capacity }
        }
    }

    fn needs_to_grow(&self, len: usize, additional: usize) -> bool {
        additional > self.cap.wrapping_sub(len)
    }

    fn grow_amortized(&mut self, len: usize, additional: usize) -> Result<usize, std::collections::TryReserveError> {
        // This is ensured by the calling contexts.
        debug_assert!(additional > 0);

        if mem::size_of::<T>() == 0 {
            // Since we return a capacity of `usize::MAX` when `elem_size` is 0, getting to here necessarily means that `DynamicBuffer` is overfull
            return Err(std::collections::TryReserveErrorKind::CapacityOverflow.into());
        }

        // Nothing we can really do about these checks, sadly
        let required_cap = len.checked_add(additional).ok_or(std::collections::TryReserveErrorKind::CapacityOverflow)?;

        // This guarantees exponential growth. The doubling cannot overflow because `cap <= isize::MAX` and the type of `cap` is usize.
        // While rust increases the capacity by 2x, we will increase it by 1.5x, so we don't get to a run-away capacity as fast
        // PERF(jel): What impact does 1.5x have compared to 2x?
        let cap = cmp::max(self.cap + self.cap / 2, required_cap);
        let cap = cmp::max(Self::MIN_NON_ZERO_CAP, cap);

        let new_layout = Layout::array::<T>(cap);

        self.finish_grow(new_layout, cap)
    }

    fn grow_exact(&mut self, len: usize, additional: usize) -> Result<usize, std::collections::TryReserveError> {
        if mem::size_of::<T>() == 0 {
            // Since we return a capacity of `usize::MAX` when `elem_size` is 0, getting to here necessarily means that `DynamicBuffer` is overfull
            return Err(std::collections::TryReserveErrorKind::CapacityOverflow.into());
        }

        let cap = len.checked_add(additional).ok_or(std::collections::TryReserveErrorKind::CapacityOverflow)?;

        let new_layout = Layout::array::<T>(cap);


        self.finish_grow(new_layout, cap)
    }

    pub fn finish_grow(&mut self, new_layout: Layout, new_cap: usize) -> Result<usize, std::collections::TryReserveError> {
        if self.cap == 0 {
            let _scope_alloc = ScopedAlloc::new(UseAlloc::Id(self.allocator_id()));

            let res = get_memory_manager().alloc_raw(AllocInitState::Uninitialized, new_layout);
            self.ptr = match res {
                Some(ptr) => ptr.cast(),
                None => {
                    let rs_layout = unsafe { std::alloc::Layout::from_size_align_unchecked(new_layout.size(), new_layout.align()) };
                    let err_kind = std::collections::TryReserveErrorKind::AllocError { layout: rs_layout, non_exhaustive: () };
                    return Err(err_kind.into());
                }
            };
        } else {
            self.ptr = match get_memory_manager().grow(mem::replace(&mut self.ptr, unsafe { Allocation::const_null() }), new_layout) {
                Ok(ptr) => ptr,
                Err(_) => {
                    let rs_layout = unsafe { std::alloc::Layout::from_size_align_unchecked(new_layout.size(), new_layout.align()) };
                    let err_kind = std::collections::TryReserveErrorKind::AllocError { layout: rs_layout, non_exhaustive: () };
                    return Err(err_kind.into());
                }
            };
        }
        self.cap = new_cap;
        Ok(new_cap)
    }

}

impl<T> imp::DynArrayBuffer<T> for DynamicBuffer<T> {
    fn new() -> Self {
        Self { ptr: unsafe { Allocation::null() }, cap: 0 }
    }

    fn with_capacity(capacity: usize) -> Self {
        Self::allocate(capacity, AllocInitState::Uninitialized)
    }

    fn with_capacity_zeroed(capacity: usize) -> Self {
        Self::allocate(capacity, AllocInitState::Zeroed)
    }

    fn reserve(&mut self, len: usize, additional: usize) -> usize {
        if self.needs_to_grow(len, additional) {
            self.grow_amortized(len, additional).expect("Failed to allocate memory");
        }
        self.cap
    }

    fn try_reserve(&mut self, len: usize, additional: usize) -> Result<usize, std::collections::TryReserveError> {
        if self.needs_to_grow(len, additional) {
            self.grow_amortized(len, additional)
        } else {
            Ok(self.cap)
        }       
    }

    fn reserve_exact(&mut self, len: usize, additional: usize) -> usize {
        self.try_reserve_exact(len, additional).expect("Failed to allocate memory")
    }

    fn try_reserve_exact(&mut self, len: usize, additional: usize) -> Result<usize, std::collections::TryReserveError> {
        if self.needs_to_grow(len, additional) {
            self.grow_exact(len, additional)
        } else {
            Ok(self.cap)
        }
    }

    fn shrink_to_fit(&mut self, cap: usize) {
        assert!(cap < self.cap, "Tried to shrink to a larger capacity");

        if self.cap == 0 {
            return;
        }

        let new_layout = Layout::array::<T>(cap);
        self.ptr = match get_memory_manager().shrink(mem::replace(&mut self.ptr, unsafe { Allocation::const_null() }), new_layout) {
            Ok(ptr) => ptr,
            Err(_) => {
                //let rs_layout = unsafe { std::alloc::Layout::from_size_align_unchecked(new_layout.size(), new_layout.align()) };
                //let err_kind = std::collections::TryReserveErrorKind::AllocError { layout: rs_layout, non_exhaustive: () };
                //return Err(err_kind.into());
                panic!("Could not shrink buffer")
            }
        };
        self.cap = cap;
    }

    fn capacity(&self) -> usize {
        if mem::size_of::<T>() == 0 {
            usize::MAX
        } else {
            self.cap
        }
    }

    fn as_ptr(&self) -> *const T {
        self.ptr.ptr()
    }

    fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr.ptr_mut()
    }

    fn layout(&self) -> Layout {
        self.ptr.layout()
    }

    fn allocator_id(&self) -> u16 {
        self.ptr.layout().alloc_id()
    }
}

impl<T> Drop for DynamicBuffer<T> {
    fn drop(&mut self) {
        if self.cap > 0 {
            get_memory_manager().dealloc(mem::replace(&mut self.ptr, unsafe { Allocation::const_null() }))
        }
    }
}
