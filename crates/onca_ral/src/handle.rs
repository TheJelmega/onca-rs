use core::{
    marker::Unsize, 
    borrow::Borrow,
    ops::Deref,
    fmt,
};

use onca_core::{
    prelude::*
};

/// RAL interface handle
/// 
/// # NOTE:
/// 
/// User should not access any functions on the handle directly and only call it via its wrappers
pub struct InterfaceHandle<T: ?Sized> {
    ptr : HeapPtr<T>
}

impl<T: ?Sized> InterfaceHandle<T> {
    pub fn new<U>(val: U) -> Self
    where
        U : Unsize<T>
    {
        InterfaceHandle { ptr: HeapPtr::<U>::new(val) }
    }

    /// Get the underlying type held by the handle
    /// 
    /// # Safety
    /// 
    /// It's up to the user to make sure that the type in the handle is the same as `U`
    pub unsafe fn as_concrete_type<U>(&self) -> &U {
        &*(self.ptr.as_ref() as *const T as *const U)
    }
}

impl<T: ?Sized> AsRef<T> for InterfaceHandle<T> {
    fn as_ref(&self) -> &T {
        &*self
    }
}

impl<T: ?Sized> Borrow<T> for InterfaceHandle<T> {
    fn borrow(&self) -> &T {
        &*self
    }
}

impl<T: ?Sized> Deref for InterfaceHandle<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.ptr
    }
}

impl<T: ?Sized> fmt::Debug for InterfaceHandle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InterfaceHandle").field("ptr", &self.ptr.ptr()).finish()
    }
}

//==============================================================================================================================

pub trait HandleImpl {
    type InterfaceHandle;

    unsafe fn interface(&self) -> &Self::InterfaceHandle;
}

/// RAL handle
pub struct Handle<T: HandleImpl> {
    arc : Arc<T>
}

impl<T: HandleImpl> Handle<T> {
    pub fn new(val: T) -> Self {
        Handle { arc: Arc::new(val) }
    }

    /// Create a new `Arc` using the value produced by the given closure, which itself has access to a weak pointer to the data
    pub fn new_cyclic<F: FnOnce(WeakHandle<T>) -> T>(fun: F) -> Self {
        Handle { arc: Arc::new_cyclic(|weak| fun(WeakHandle::from_weak(weak))) }
    }

    pub(crate) fn from_arc(handle: Arc<T>) -> Self {
        Self { arc: handle }
    }

    pub fn downgrade(handle: &Handle<T>) -> WeakHandle<T> {
        WeakHandle::from_weak(Arc::downgrade(&handle.arc))
    }

    /// Check if 2 `Arc`s contain the same pointer
    #[inline]
    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        Arc::ptr_eq(&this.arc, &other.arc)
    }

    /// Check if an `Arc` and a `AWeak` contain the same pointer
    pub fn weak_ptr_eq(this: &Self, weak: &WeakHandle<T>) -> bool {
        Arc::weak_ptr_eq(&this.arc, &weak.weak)
    }
}

impl<T: HandleImpl> AsRef<T> for Handle<T> {
    fn as_ref(&self) -> &T {
        &*self
    }
}

impl<T: HandleImpl> Borrow<T> for Handle<T> {
    fn borrow(&self) -> &T {
        &*self
    }
}

impl<T: HandleImpl> Deref for Handle<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.arc
    }
}

impl<T: HandleImpl> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self { arc: self.arc.clone() }
    }
}

impl<T: HandleImpl + fmt::Debug> fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Handle").field("value", self.arc.as_ref()).finish()
    }
}

impl<T: HandleImpl + fmt::Display> fmt::Display for Handle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_ref(), f)
    }
}

//==============================================================================================================================

/// RAL weak handle
pub struct WeakHandle<T: HandleImpl> {
    weak : AWeak<T>
}

impl<T: HandleImpl> WeakHandle<T> {
    pub(crate) fn from_weak(handle: AWeak<T>) -> Self {
        Self { weak: handle }
    }

    pub fn upgrade(this: &WeakHandle<T>) -> Option<Handle<T>> {
        Some(Handle::from_arc(AWeak::upgrade(&this.weak)?))
    }

    pub fn strong_count(this: &WeakHandle<T>) -> usize {
        AWeak::strong_count(&this.weak)
    }
}

// Derive does not seem to work correctly for Clone
impl<T: HandleImpl> Clone for WeakHandle<T> {
    fn clone(&self) -> Self {
        Self { weak: self.weak.clone() }
    }
}

// Derive does not seem to work correctly for Default
impl<T: HandleImpl> Default for WeakHandle<T> {
    fn default() -> Self {
        Self { weak: AWeak::default() }
    }
}


impl<T: HandleImpl + fmt::Debug> fmt::Debug for WeakHandle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match AWeak::upgrade(&self.weak) {
            Some(arc) => f.debug_struct("WeakHandle").field("value", arc.as_ref()).finish(),
            None => f.debug_struct("WeakHandle").field("value", &"<null>").finish(),
        }
    }
}