use core::{
    //ptr,
    mem,
    ffi::c_void,
    sync::atomic::{AtomicU32, Ordering},
    borrow::{Borrow, BorrowMut}
};

use windows::core::*;

use crate::from_interface;

/// Com atomic reference count
pub struct ComRefCount {
    ref_count : AtomicU32
}

impl ComRefCount {
    /// Create a new Com atomic reference count
    pub fn new() -> ComRefCount {
        ComRefCount{ ref_count: AtomicU32::new(0) }
    }

    /// Increment the reference count
    pub fn add_ref(&mut self) -> u32 {
        self.ref_count.fetch_add(1, Ordering::Release) + 1
    }

    /// Decrement the reference cound and run the `clean_up` callback when the reference count reaches 0.
    pub fn release<F: FnMut()>(&mut self, mut clean_up: F) -> u32 {
        let count = self.ref_count.fetch_sub(1, Ordering::Relaxed) - 1;
        if count == 0 {
            clean_up();
        }
        count
    }
}

#[repr(C)]
pub struct ComInterface<I> {
    pub ptr: *mut I,
}

impl<I> ComInterface<I> {
    pub fn get_mut(&mut self) -> &mut I {
        from_interface(&mut self.ptr)
    }

    pub fn get(&self) -> &I {
        from_interface(&self.ptr as *const _ as *mut I)
    }

    pub fn to<T>(&self) -> &mut T {
        from_interface(&self.ptr as *const _ as *mut I)
    }
}

impl<I> AsRef<I> for ComInterface<I> {
    fn as_ref(&self) -> &I {
        self.get()
    }
}

impl<I> AsMut<I> for ComInterface<I> {
    fn as_mut(&mut self) -> &mut I {
        self.get_mut()
    }
}

impl<I> Borrow<I> for ComInterface<I> {
    fn borrow(&self) -> &I {
        self.get()
    }
}

impl<I> BorrowMut<I> for ComInterface<I> {
    fn borrow_mut(&mut self) -> &mut I {
        self.get_mut()
    }
}




pub const fn create_iunknown_vtable<I>(
    query_interface: unsafe extern "system" fn(&mut I, &GUID, *mut *const c_void) -> HRESULT,
    add_ref: unsafe extern "system" fn(&mut I) -> u32,
    release: unsafe extern "system" fn(&mut I) -> u32,
) -> IUnknown_Vtbl {
    unsafe {
        IUnknown_Vtbl {
            QueryInterface: mem::transmute(query_interface),
            AddRef: mem::transmute(add_ref),
            Release: mem::transmute(release)
        }
    }
}