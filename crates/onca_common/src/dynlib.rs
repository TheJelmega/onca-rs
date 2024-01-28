use core::mem::{ManuallyDrop, size_of};

use crate::{os::dynlib as os, prelude::{ScopedAlloc, AllocId}, strings::{ToString, StringExtensions}, scoped_alloc};

/// Dynamic library
pub struct DynLib {
    handle: os::DynLibHandle,
}

impl DynLib {
    /// Load a dynamic library
    /// 
    /// # Errors
    /// 
    /// If a dynamic library could not be loaded, an error with an OS error will be returned
    pub fn load(path: &str) -> Result<DynLib, i32> {
        scoped_alloc!(AllocId::TlsTemp);
        // Go via a `String` to make sure it is null-terminated
        let mut path = path.to_string();
        path.null_terminate();
        os::load(&path).map(|handle| DynLib { handle })
    }

    /// Close a dynamic library, this has the same result as dropping the dynamic library, except that it has a return value
    /// 
    /// # Error
    /// 
    /// If the dynamic library could not be closed, an error with an OS error will be returned
    pub fn close(dynlib: DynLib) -> Result<(), i32> {
        let me = ManuallyDrop::new(dynlib);
        os::close(me.handle)
    }

    /// Get a function pointer to the given function
    pub fn get<T: Copy>(&self, proc_name: &str) -> Option<T> {
        // This is probably the best we can do to insure that `T` is a function pointer
        if size_of::<T>() != size_of::<fn()>() {
            return None;
        }

        scoped_alloc!(AllocId::TlsTemp);
        // Go via a `String` to make sure it is null-terminated
        let mut proc_name = proc_name.to_string();
        proc_name.null_terminate();
        let addr = os::get_proc_address(self.handle, &proc_name);
        addr.map(|addr| unsafe { *(core::mem::transmute::<_, *const T>(&addr)) })
    }

    pub fn get_indexed<T: Copy>(&self, idx: usize) -> Option<T> {
        // This is probably the best we can do to insure that `T` is a function pointer
        if size_of::<T>() != size_of::<fn()>() {
            return None;
        }

        let addr = os::get_proc_address_indexed(self.handle, idx);
        addr.map(|addr| unsafe { *(core::mem::transmute::<_, *const T>(&addr)) })
    }
}

impl Drop for DynLib {
    fn drop(&mut self) {
        os::close(self.handle);
    }
}