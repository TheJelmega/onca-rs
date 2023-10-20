use core::mem::{ManuallyDrop, size_of};

use crate::{os::dynlib as os, prelude::ScopedAlloc, strings::ToString};

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
        let _scope_alloc = ScopedAlloc::new(crate::prelude::UseAlloc::TlsTemp);
        // Go via a `String` to make sure it is null-terminated
        os::load(&path.to_string()).map(|handle| DynLib { handle })
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

        let _scope_alloc = ScopedAlloc::new(crate::prelude::UseAlloc::TlsTemp);
        // Go via a `String` to make sure it is null-terminated
        let addr = os::get_proc_address(self.handle, &proc_name.to_string());
        addr.map(|addr| unsafe { *(core::mem::transmute::<_, *const T>(&addr)) })
    }
}

impl Drop for DynLib {
    fn drop(&mut self) {
        os::close(self.handle);
    }
}