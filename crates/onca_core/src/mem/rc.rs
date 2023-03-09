use core::{
    any::Any,
    borrow::Borrow,
    cell::Cell,
    cmp::*,
    marker::{PhantomData, Unsize},
    mem::{MaybeUninit, ManuallyDrop, forget},
    ops::{Deref, CoerceUnsized},
    ptr::{drop_in_place, read, null}
};
use crate::alloc::{Allocation, Layout, UseAlloc, MemTag, ScopedAlloc, ScopedMemTag};
use crate::mem::MEMORY_MANAGER;

use super::AllocInitState;

struct RcData<T: ?Sized> {
    strong : Cell<usize>,
    weak   : Cell<usize>,
    value  : T
}

pub struct Rc<T: ?Sized> {
    ptr     : Allocation<RcData<T>>,
    phantom : PhantomData<T>
}

pub struct Weak<T: ?Sized> {
    ptr : Allocation<RcData<T>>
}

impl<T: ?Sized> RcData<T> {
    fn inc_strong(&self) {
        let strong = self.strong.get();
        self.strong.set(strong + 1);
    }

    fn dec_strong(&self) {
        let strong = self.strong.get();
        self.strong.set(strong - 1);
    }

    #[inline]
    fn strong_count(&self) -> usize {
        self.strong.get()
    }

    fn inc_weak(&self) {
        let weak = self.weak.get();
        self.weak.set(weak + 1);
    }

    fn dec_weak(&self) {
        let weak = self.weak.get();
        self.weak.set(weak - 1);
    }

    #[inline]
    fn weak_count(&self) -> usize {
        self.weak.get()
    }
}

impl<T: ?Sized> Rc<T> {
    pub fn downgrade(this: &Self) -> Weak<T> {
        this.inner().inc_weak();
        Weak { ptr: unsafe { this.ptr.duplicate() } }
    }

    /// Get the strong count for the `Rc`
    #[inline]
    pub fn strong_count(this: &Self) -> usize {
        this.inner().strong_count()
    }

    /// Get the weak count for the `Rc`
    #[inline]
    pub fn weak_count(this: &Self) -> usize {
        this.inner().weak_count()
    }

    /// Get the layout
    #[inline]
    pub fn layout(this: &Self) -> Layout {
        this.ptr.layout()
    }

    /// Get the allocator id of the allocation
    #[inline]
    pub fn allocator_id(this: &Self) -> u16 {
        this.ptr.layout().alloc_id()
    }

    /// Get the memory tag
    #[inline]
    pub fn mem_tag(this: &Self) -> MemTag {
        this.ptr.mem_tag()
    }

    /// Get a mutable reference to the value, when the `Rc` is unique (only 1 strong and no weak)
    pub fn get_mut(this: &mut Self) -> Option<&mut T> {
        if Self::is_unique(this) {
            Some(unsafe { Self::get_mut_unchecked(this) })
        } else {
            None
        }
    }

    /// Get a mutable reference to the value, works when `Rc` isn't unique
    #[inline]
    pub unsafe fn get_mut_unchecked(this: &mut Self) -> &mut T {
        &mut this.ptr.get_mut().value
    }

    /// Makes a mutable reference of the given `Rc`
    /// 
    /// If the `Rc` is not unique, the inner data will be copied and the `Rc` will contain a new allocation
    pub fn make_mut(this: &mut Rc<T>) -> &mut T
        where T: Clone
    {
        if !Self::is_unique(this) {
            let _scope_alloc = ScopedAlloc::new(UseAlloc::Id(Self::allocator_id(this)));
            let _scope_mem_tag = ScopedMemTag::new(Self::mem_tag(this));

            let mut new = Self::new_uninit();
            unsafe {
                let data = Rc::get_mut_unchecked(this);
                Rc::get_mut_unchecked(&mut new).as_mut_ptr().write(data.clone());
                *this = new.assume_init();
            }
        }

        unsafe { Self::get_mut_unchecked(this) }
    }

    /// Check if 2 `Rc`s contain the same pointer
    #[inline]
    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        this.ptr.ptr() == other.ptr.ptr()
    }

    
    /// Check if an `Rc` and a `Weak` contain the same pointer
    pub fn weak_ptr_eq(this: &Self, weak: &Weak<T>) -> bool {
        this.ptr.ptr() == weak.ptr.ptr()
    }


    #[inline]
    fn is_unique(this: &Self) -> bool {
        Self::strong_count(this) == 1 && Self::weak_count(this) == 0
    }

    #[inline]
    fn inner(&self) -> &RcData<T> {
        self.ptr.get_ref()
    }
}

impl<T> Rc<T> {

    /// Create a new `Rc` using the given allocator
    #[inline]
    pub fn new(x: T) -> Self {
        Self::try_new(x).expect("Failed to allocate memory")
    }

    /// Create a new `Rc` using the value produced by the given closure, which itself has access to a weak pointer to the data
    pub fn new_cyclic<F: FnOnce(Weak<T>) -> T>(fun: F) -> Option<Self> {
        let mut uninit = Self::try_new_uninit().expect("Failed to allocate memory");
        uninit.inner().dec_strong();
        uninit.inner().inc_weak();

        let weak_ptr = unsafe{ uninit.ptr.duplicate_cast::<RcData<T>>() };
        let weak = Weak::<T>{ ptr: weak_ptr };
        uninit.ptr.get_mut().value.write(fun(weak));

        uninit.inner().inc_strong();
        Some(uninit.assume_init())
    }

    /// Try to create a new `Rc` using the default allocator
    pub fn try_new(x: T) -> Option<Self> {
        let mut uninit = Self::try_new_uninit();
        match uninit {
            None => None,
            Some(mut uninit) => {
                uninit.ptr.get_mut().value.write(x);
                Some(uninit.assume_init())
            }
        }
    }

    /// Creates new `Rc` with an uninitialized value, using the default allocator
    #[inline]
    pub fn new_uninit() -> Rc<MaybeUninit<T>> {
        Self::try_new_uninit().expect("Failed to allocate memory")
    }

    /// Try to create a new `Rc` with an uninitialized value, using the default allocator
    pub fn try_new_uninit() -> Option<Rc<MaybeUninit<T>>> {
        let ptr = MEMORY_MANAGER.alloc::<RcData<MaybeUninit<T>>>(AllocInitState::Uninitialized);
        match ptr {
            None => None,
            Some(ptr) => Self::fill_uninit(ptr.cast())
        }
    }

    /// Return the inner value, if the `Rc` only has 1 strong reference, otherwise return the `Rc` that was passed in
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

    /// Unwrap the inner value, if the `Rc` only has 1 strong reference, otherwise clone the inner value
    pub fn unwrap_or_clone(this: Self) -> T 
        where T: Clone
    {
        Self::try_unwrap(this).unwrap_or_else(|rc| rc.as_ref().clone())
    }
    
    fn fill_uninit(ptr: Allocation<RcData<MaybeUninit<T>>>) -> Option<Rc<MaybeUninit<T>>> {
        let mut rc = Rc{ ptr: ptr.cast(), phantom: PhantomData };
        unsafe {
            let mut data = rc.ptr.get_mut();
            (&mut data.strong as *mut Cell<usize>).write(Cell::<_>::new(1));
            (&mut data.weak as *mut Cell<usize>).write(Cell::<_>::new(0));
        }
        Some(rc)
    }
}

impl<T> Rc<MaybeUninit<T>> {
    pub fn assume_init(self) -> Rc<T> {
        let this = ManuallyDrop::new(self);
        Rc { ptr: unsafe{ this.ptr.duplicate_cast::<RcData<T>>() }, phantom: PhantomData }
    }
}

impl<T> Rc<[MaybeUninit<T>]> {
    pub fn assume_init(self) -> Rc<T> {
        let this = ManuallyDrop::new(self);
        Rc { ptr: unsafe{ this.ptr.duplicate_cast::<RcData<T>>() }, phantom: PhantomData }
    }
}

impl Rc<dyn Any>
{
    /// Try to downcast to a concrete type, if the conversion failed, the original value will be found in the Err value
    pub fn downcast<T: Any>(self) -> Result<Rc<T>, Rc<dyn Any>>
    {
        if unsafe { self.ptr.get_ref().value.is::<T>() } {
            Ok(unsafe { self.downcast_unchecked() })
        } else {
            Err(self)
        }
    }

    /// Downcast to a concrete type, calling this on an invalid downcasted type will result in UB
    unsafe fn downcast_unchecked<T: Any>(self) -> Rc<T> {
        debug_assert!(unsafe { self.ptr.get_ref().value.is::<T>() });
        Rc::<T>{ ptr: unsafe{ self.ptr.duplicate_cast::<RcData<T>>() }, phantom: PhantomData }
    }
}

impl<T: ?Sized> AsRef<T> for Rc<T> {
    fn as_ref(&self) -> &T {
        &self.ptr.get_ref().value
    }
}

impl<T: ?Sized> Borrow<T> for Rc<T> {
    fn borrow(&self) -> &T {
        &self.ptr.get_ref().value
    }
}

impl<T: ?Sized> Deref for Rc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.ptr.get_ref().value
    }
}

impl<T: ?Sized> Clone for Rc<T> {
    /// Clone the pointer, increasing strong count
    fn clone(&self) -> Self {
        let mut ptr = unsafe{ self.ptr.duplicate() };
        *ptr.get_mut().strong.get_mut() += 1;
        Self{ ptr, phantom: PhantomData }
    }
}

impl<T: ?Sized> Drop for Rc<T> {
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

impl <T: PartialEq + ?Sized> PartialEq for Rc<T> {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref().eq(other.as_ref())
    }

    fn ne(&self, other: &Self) -> bool {
        self.as_ref().ne(other.as_ref())
    }
}

impl <T: Eq + ?Sized> Eq for Rc<T> {}

impl<T: PartialOrd + ?Sized> PartialOrd for Rc<T> {
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

impl<T: Ord + ?Sized> Ord for Rc<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}

impl<T, U> CoerceUnsized<Rc<U>> for Rc<T>
    where T : Unsize<U> + ?Sized,
          U : ?Sized
{}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

impl<T: ?Sized> Weak<T> {
    /// Get the strong count for the `Rc`
    pub fn strong_count(&self) -> usize {
        if self.is_valid() {
            self.inner().strong_count()
        } else {
            0
        }
    }

    /// Get the weak count for the `Rc`
    pub fn weak_count(&self) -> usize {
        if self.is_valid() {
            self.inner().weak_count()
        } else {
            0
        }
    }

    /// Upgrade to an 'Rc', `None` will be returned if the `Weak` points to an invalid pointer
    pub fn upgrade(&self) -> Option<Rc<T>> {
        if self.is_valid() && Self::strong_count(self) > 0 {
            let rc = Rc::<T>{ ptr: unsafe { self.ptr.duplicate() }, phantom: PhantomData };
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

    fn is_valid(&self) -> bool {
        self.ptr.ptr() as *const u8 != null()
    }
}

impl<T> Weak<T> {
    /// Create a new invalid weak pointer
    pub fn new() -> Self {
        Weak { ptr: unsafe { Allocation::<RcData<T>>::const_null() } }
    }
}

impl<T: ?Sized> Clone for Weak<T> {
    fn clone(&self) -> Self {
        if self.is_valid() {
            self.inner().inc_weak();
        }
        Weak { ptr: unsafe{ self.ptr.duplicate() } }
    }
}

impl<T> Default for Weak<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ?Sized> Drop for Weak<T> {
    fn drop(&mut self) {
        if self.is_valid() {
            self.inner().dec_weak();
            if self.strong_count() == 0 && self.weak_count() == 0 {
                MEMORY_MANAGER.dealloc(unsafe{ self.ptr.duplicate() });
            }
        }
    }
}

impl<T, U> CoerceUnsized<Weak<U>> for Weak<T>
    where T : Unsize<U> + ?Sized,
          U : ?Sized
{}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::{Rc, Weak};

    #[test]
    fn new() {
        let rc = Rc::<u64>::new(123);

        assert_eq!(Rc::strong_count(&rc), 1);
        assert_eq!(Rc::weak_count(&rc), 0);
        assert_eq!(*rc, 123);
    }

    #[test]
    fn clone_drop() {
        let rc = Rc::<u64>::new(123);

        {
            let rc2 = rc.clone();
            assert!(Rc::ptr_eq(&rc, &rc2));
            assert_eq!(Rc::strong_count(&rc), 2);
            assert_eq!(Rc::weak_count(&rc), 0);
        }

        assert_eq!(Rc::strong_count(&rc), 1);
        assert_eq!(Rc::weak_count(&rc), 0);
    }

    #[test]
    fn downgrade() {
        let rc = Rc::<u64>::new(123);

        {
            let weak = Rc::downgrade(&rc);
            
            assert_eq!(Rc::strong_count(&rc), 1);
            assert_eq!(Rc::weak_count(&rc), 1);
        }

        assert_eq!(Rc::strong_count(&rc), 1);
        assert_eq!(Rc::weak_count(&rc), 0);
    }

    #[test]
    fn upgrade_null() {
        let weak = Weak::<u64>::new();

        match weak.upgrade() {
            None => {},
            Some(_) => panic!()
        }
    }

    #[test]
    fn upgrade() {
        let rc = Rc::<u64>::new(123);
        let weak = Rc::downgrade(&rc);

        let upgraded = weak.upgrade();
        match &upgraded {
            None => panic!(),
            Some(ref tmp) => assert_eq!(**tmp, 123)
        }

        assert_eq!(Rc::strong_count(&rc), 2);
        assert_eq!(Rc::weak_count(&rc), 1);
    }

    #[test]
    fn cyclic() {
        let mut weak = Weak::<u64>::new();

        let rc = Rc::<u64>::new_cyclic(|wrc| { weak = wrc; 123 }).unwrap();

        assert_eq!(Rc::strong_count(&rc), 1);
        assert_eq!(Rc::weak_count(&rc), 1);
        assert_eq!(*rc, 123);

        assert_eq!(weak.strong_count(), 1);
        assert_eq!(weak.weak_count(), 1);
    }

    #[test]
    fn unique_unwrap() {
        let rc = Rc::<u64>::new(123);

        match Rc::try_unwrap(rc) {
            Ok(_) => {},
            Err(_) => panic!()
        }
    }

    #[test]
    fn non_unique_unwrap() {
        let rc = Rc::<u64>::new(123);
        let rc2 = rc.clone();

        match Rc::try_unwrap(rc) {
            Ok(_) => panic!(),
            Err(_) => {}
        }
    }

    #[test]
    fn unique_get_mut() {
        let mut rc = Rc::<u64>::new(123);

        match Rc::get_mut(&mut rc) {
            None => panic!(),
            Some(_) => {}
        }
    }

    #[test]
    fn non_unique_get_mut() {
        let mut rc = Rc::<u64>::new(123);
        let rc2 = rc.clone();

        match Rc::get_mut(&mut rc) {
            None => {},
            Some(_) => panic!()
        }
    }

    #[test]
    fn non_unique_make_mut() {
        let mut rc = Rc::<u64>::new(123);
        let rc2 = rc.clone();

        Rc::make_mut(&mut rc);

        assert!(!Rc::ptr_eq(&rc, &rc2));
    }
}