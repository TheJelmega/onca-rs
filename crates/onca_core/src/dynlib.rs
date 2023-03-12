use core::mem::ManuallyDrop;

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
        os::load(&path.to_onca_string()).map(|handle| DynLib { handle })
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
    pub fn get_proc_address(&self, proc_name: &str) -> Option<fn()> {
        let _scope_alloc = ScopedAlloc::new(crate::prelude::UseAlloc::TlsTemp);
        // Go via a `String` to make sure it is null-terminated
        os::get_proc_address(self.handle, &proc_name.to_onca_string())
    }
}

impl Drop for DynLib {
    fn drop(&mut self) {
        os::close(self.handle);
    }
}