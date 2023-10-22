use onca_core::{
    prelude::*,
    io,
};
use windows::{
    Win32::{
        Storage::FileSystem::{CreateDirectoryA, RemoveDirectoryA},
        Foundation::{GetLastError, ERROR_ALREADY_EXISTS},
    },
    core::PCSTR,
};
use crate::Path;

pub(crate) fn create(path: &Path) -> io::Result<()> {
    unsafe {
        let _scope_alloc = ScopedAlloc::new(AllocId::TlsTemp);

        let path = path.to_null_terminated_path_buf();
        create_dir(PCSTR(path.as_ptr()))
    }
}

pub(crate) fn create_recursive(path: &Path) -> io::Result<()> {
    unsafe {
        let _scope_alloc = ScopedAlloc::new(AllocId::TlsTemp);

        let mut parent_paths = Vec::new();
        for component in path.ancestors() {
            parent_paths.push(component);
        }

        for cur_dir in parent_paths.into_iter().rev() {
            let path = cur_dir.to_null_terminated_path_buf();
            create_dir(PCSTR(path.as_ptr()))?;
        }
        Ok(())
    }
}

unsafe fn create_dir(pcstr: PCSTR) -> io::Result<()> {
    let res = CreateDirectoryA(pcstr, None).as_bool();
    if res {
        Ok(())
    } else {
        let err = GetLastError();
        if err == ERROR_ALREADY_EXISTS {
            Ok(())
        } else {
            Err(io::Error::from_raw_os_error(err.0 as i32))
        }
    }
}

pub(crate) fn remove(path: &Path) -> io::Result<()> {
    unsafe {
        let _scope_alloc = ScopedAlloc::new(AllocId::TlsTemp);

        let path = path.to_null_terminated_path_buf();
        let res = RemoveDirectoryA(PCSTR(path.as_ptr())).as_bool();
        if res {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }
}