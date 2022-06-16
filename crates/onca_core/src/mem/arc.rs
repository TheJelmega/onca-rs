use core::{
    any::Any,
    sync::atomic::{AtomicUsize, Ordering},
    borrow::Borrow,
    cmp::*,
    marker::{PhantomData, Unsize},
    mem::{MaybeUninit, ManuallyDrop, forget},
    ops::{Deref, CoerceUnsized},
    ptr::{drop_in_place, read}
};
use crate::alloc::{Allocation, Allocator, Layout, UseAlloc};
use crate::mem::MEMORY_MANAGER;

struct RcData<T: ?Sized> {
    strong : AtomicUsize,
    weak   : AtomicUsize,
    value  : T
}

pub struct Arc<T: ?Sized> {
    ptr     : Allocation<RcData<T>>,
    phantom : PhantomData<T>
}

pub struct AWeak<T: ?Sized> {
    ptr : Allocation<RcData<T>>
}

impl<T: ?Sized> RcData<T> {

    fn inc_strong(&self) {
        self.strong.fetch_add(1, Ordering::AcqRel);
    }

    fn dec_strong(&self) {
        self.strong.fetch_sub(1, Ordering::AcqRel);
    }

    fn strong_count(&self) -> usize {
        self.strong.load(Ordering::Acquire)
    }

    fn inc_weak(&self) {
        self.weak.fetch_add(1, Ordering::AcqRel);
    }

    fn dec_weak(&self) {
        self.weak.fetch_sub(1, Ordering::AcqRel);
    }

    fn weak_count(&self) -> usize {
        self.weak.load(Ordering::Acquire)
    }

}

impl<T: ?Sized> Arc<T> {
    
    pub fn downgrade(this: &Self) -> AWeak<T> {
        this.inner().inc_weak();
        this.inner().dec_strong();
        AWeak { ptr: unsafe { this.ptr.duplicate() } }
    }

    /// Get the strong count for the `Arc`
    pub fn strong_count(this: &Self) -> usize {
        this.inner().strong_count()
    }

    /// Get the weak count for the `Arc`
    pub fn weak_count(this: &Self) -> usize {
        this.inner().weak_count()
    }

    /// Get the allocator id of the allocation
    pub fn allocator_id(this: &Self) -> u16 {
        this.ptr.layout().alloc_id()
    }

    pub fn allocator(this: &Self) -> &mut dyn Allocator {
        let id = Self::allocator_id(this);
        unsafe { &mut *MEMORY_MANAGER.get_allocator(id).unwrap() }
    }

    /// Get a mutable reference to the value, when the `Arc` is unique (only 1 strong and no weak)
    pub fn get_mut(this: &mut Self) -> Option<&mut T> {
        if Self::is_unique(this) {
            Some(unsafe { Self::get_mut_unchecked(this) })
        } else {
            None
        }
    }

    /// Get a mutable reference to the value, works when `Arc` isn't unique
    pub unsafe fn get_mut_unchecked(this: &mut Self) -> &mut T {
        &mut this.ptr.get_mut().value
    }

    /// Makes a mutable reference of the given `Arc`
    /// 
    /// If the `Arc` is not unique, the inner data will be copied and the `Arc` will contain a new allocation
    pub fn make_mut(this: &mut Arc<T>) -> &mut T
        where T: Clone
    {
        if !Self::is_unique(this) {
            let mut new = Self::new_uninit(UseAlloc::Id(Self::allocator_id(this)));
            unsafe {
                let data = Arc::get_mut_unchecked(this);
                Arc::get_mut_unchecked(&mut new).as_mut_ptr().write(data.clone());
                *this = new.assume_init();
            }
        }

        unsafe { Self::get_mut_unchecked(this) }
    }

    /// Check if 2 `Arc`s contain the same pointer
    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        this.ptr.ptr() == other.ptr.ptr()
    }

    fn is_unique(this: &Self) -> bool {
        Self::strong_count(this) == 1 && Self::weak_count(this) == 0
    }

    fn inner(&self) -> &RcData<T> {
        self.ptr.get_ref()
    }
}

impl<T> Arc<T> {

    /// Create a new `Arc` using the given allocator
    pub fn new(x: T, alloc: UseAlloc) -> Self {
        Self::try_new(x, alloc).expect("Failed to allocate memory")
    }

    /// Create a new `Arc` using the value produced by the given closure, which itself has access to a weak pointer to the data
    pub fn new_cyclic<F: FnOnce(&AWeak<T>) -> T>(fun: F, alloc: UseAlloc) -> Option<Self> {
        let mut uninit = Self::try_new_uninit(alloc).expect("Failed to allocate memory");
        uninit.inner().dec_strong();
        uninit.inner().inc_weak();

        let weak_ptr = unsafe{ uninit.ptr.duplicate_cast::<RcData<T>>() };
        let weak = AWeak::<T>{ ptr: weak_ptr };
        uninit.ptr.get_mut().value.write(fun(&weak));

        Some(uninit.assume_init())
    }

    /// Try to create a new `Arc` using the default allocator
    pub fn try_new(x: T, alloc: UseAlloc) -> Option<Self> {
        let mut uninit = Self::try_new_uninit(alloc);
        match uninit {
            None => None,
            Some(mut uninit) => {
                uninit.ptr.get_mut().value.write(x);
                Some(uninit.assume_init())
            }
        }
    }

    /// Creates new `Arc` with an uninitialized value, using the default allocator
    pub fn new_uninit(alloc: UseAlloc) -> Arc<MaybeUninit<T>> {
        Self::try_new_uninit(alloc).expect("Failed to allocate memory")
    }
    /// Try to create a new `Arc` with an uninitialized value, using the default allocator
    pub fn try_new_uninit(alloc: UseAlloc) -> Option<Arc<MaybeUninit<T>>> {
        let ptr = MEMORY_MANAGER.alloc::<RcData<MaybeUninit<T>>>(alloc);
        match ptr {
            None => None,
            Some(ptr) => Self::fill_uninit(ptr.cast())
        }
    }

    /// Return the inner value, if the `Arc` only has 1 strong reference, otherwise return the `Arc` that was passed in
    pub fn try_unwrap(this: Self) -> Result<T, Self> {
        if Self::strong_count(&this) == 1 {
            let val = unsafe { read(&*this) };
            this.inner().dec_strong();
            forget(this);
            Ok(val)
        } else {
            Err(this)
        }
    }

    /// Unwrap the inner value, if the `Arc` only has 1 strong reference, otherwise clone the inner value
    pub fn unwrap_or_clone(this: Self) -> T 
        where T: Clone
    {
        Self::try_unwrap(this).unwrap_or_else(|rc| rc.as_ref().clone())
    }
    
    fn fill_uninit(ptr: Allocation<RcData<MaybeUninit<T>>>) -> Option<Arc<MaybeUninit<T>>> {
        let mut rc = Arc{ ptr: ptr.cast(), phantom: PhantomData };
        unsafe {
            let mut data = rc.ptr.get_mut();
            (&mut data.strong as *mut AtomicUsize).write(AtomicUsize::new(1));
            (&mut data.weak as *mut AtomicUsize).write(AtomicUsize::new(0));
        }
        Some(rc)
    }
}

impl<T: ?Sized> AWeak<T> {
    /// Get the strong count for the `Arc`
    pub fn strong_count(&self) -> usize {
        self.inner().strong_count()
    }

    /// Get the weak count for the `Arc`
    pub fn weak_count(&self) -> usize {
        self.inner().weak_count()
    }

    /// Upgrade to an 'Arc', `None` will be returned if the `AWeak` points to an invalid pointer
    pub fn upgrade(&self) -> Option<Arc<T>> {
        if Self::strong_count(self) > 0 {
            let rc = Arc::<T>{ ptr: unsafe { self.ptr.duplicate() }, phantom: PhantomData };
            self.inner().inc_strong();
            Some(rc)
        } else {
            None
        }
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.ptr.ptr() == other.ptr.ptr()
    }

    fn inner(&self) -> &RcData<T> {
        self.ptr.get_ref()
    }
}

impl<T> AWeak<T> {
    /// Create a new invalid weak pointer
    pub fn new() -> Self {
        AWeak { ptr: unsafe { Allocation::<RcData<T>>::null() } }
    }
}

impl<T> Arc<MaybeUninit<T>> {
    pub fn assume_init(self) -> Arc<T> {
        let this = ManuallyDrop::new(self);
        Arc { ptr: unsafe{ this.ptr.duplicate_cast::<RcData<T>>() }, phantom: PhantomData }
    }
}

impl<T> Arc<[MaybeUninit<T>]> {
    pub fn assume_init(self) -> Arc<T> {
        let this = ManuallyDrop::new(self);
        Arc { ptr: unsafe{ this.ptr.duplicate_cast::<RcData<T>>() }, phantom: PhantomData }
    }
}

impl Arc<dyn Any>
{
    /// Try to downcast to a concrete type, if the conversion failed, the original value will be found in the Err value
    pub fn downcast<T: Any>(self) -> Result<Arc<T>, Arc<dyn Any>>
    {
        if unsafe { self.ptr.get_ref().value.is::<T>() } {
            Ok(unsafe { self.downcast_unchecked() })
        } else {
            Err(self)
        }
    }

    /// Downcast to a concrete type, calling this on an invalid downcasted type will result in UB
    unsafe fn downcast_unchecked<T: Any>(self) -> Arc<T> {
        debug_assert!(unsafe { self.ptr.get_ref().value.is::<T>() });
        Arc::<T>{ ptr: unsafe{ self.ptr.duplicate_cast::<RcData<T>>() }, phantom: PhantomData }
    }
}

impl<T: ?Sized> AsRef<T> for Arc<T> {
    fn as_ref(&self) -> &T {
        &self.ptr.get_ref().value
    }
}

impl<T: ?Sized> Borrow<T> for Arc<T> {
    fn borrow(&self) -> &T {
        &self.ptr.get_ref().value
    }
}

impl<T: ?Sized> Deref for Arc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.ptr.get_ref().value
    }
}

impl<T: ?Sized> Clone for Arc<T> {
    /// Clone the pointer, increasing strong count
    fn clone(&self) -> Self {
        let mut ptr = unsafe{ self.ptr.duplicate() };
        *ptr.get_mut().strong.get_mut() += 1;
        Self{ ptr, phantom: PhantomData }
    }
}

impl<T: ?Sized> Drop for Arc<T> {
    fn drop(&mut self) {
        self.ptr.get_mut().dec_strong();
        if Self::strong_count(self) == 0 {
            unsafe { drop_in_place(self.ptr.ptr_mut()) };
            if Self::weak_count(self) == 0 {
                MEMORY_MANAGER.dealloc(unsafe{ self.ptr.duplicate() });
            }
        }
    }
}

impl <T: PartialEq + ?Sized> PartialEq for Arc<T> {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref().eq(other.as_ref())
    }

    fn ne(&self, other: &Self) -> bool {
        self.as_ref().ne(other.as_ref())
    }
}

impl <T: Eq + ?Sized> Eq for Arc<T> {}

impl<T: PartialOrd + ?Sized> PartialOrd for Arc<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.as_ref().partial_cmp(other.as_ref())
    }

    fn lt(&self, other: &Self) -> bool {
        self.as_ref().lt(other.as_ref())
    }

    fn le(&self, other: &Self) -> bool {
        self.as_ref().le(other.as_ref())
    }

    fn gt(&self, other: &Self) -> bool {
        self.as_ref().gt(other.as_ref())
    }

    fn ge(&self, other: &Self) -> bool {
        self.as_ref().ge(other.as_ref())
    }
}

impl<T: Ord + ?Sized> Ord for Arc<T> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}

impl<T: ?Sized> Clone for AWeak<T> {
    fn clone(&self) -> Self {
        self.inner().inc_weak();
        AWeak { ptr: unsafe{ self.ptr.duplicate() } }
    }
}

impl<T> Default for AWeak<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ?Sized> Drop for AWeak<T> {
    fn drop(&mut self) {
        self.inner().dec_weak();
        if self.strong_count() == 0 && self.weak_count() == 0 {
            MEMORY_MANAGER.dealloc(unsafe{ self.ptr.duplicate() });
        }
    }
}

impl<T, U> CoerceUnsized<Arc<U>> for Arc<T>
    where T : Unsize<U> + ?Sized,
          U : ?Sized
{}

impl<T, U> CoerceUnsized<AWeak<U>> for AWeak<T>
    where T : Unsize<U> + ?Sized,
          U : ?Sized
{}

unsafe impl<T: ?Sized> Send for Arc<T> {}
unsafe impl<T: ?Sized> Sync for Arc<T> {}

unsafe impl<T: ?Sized> Send for AWeak<T> {}
unsafe impl<T: ?Sized> Sync for AWeak<T> {}