use onca_core::{
    prelude::*,   
    io, alloc::ScopedAlloc
};
use windows::{
    Win32::Storage::FileSystem::{CreateHardLinkA, CreateSymbolicLinkA, SYMBOLIC_LINK_FLAGS, SYMBOLIC_LINK_FLAG_DIRECTORY},
    core::PCSTR
};

use crate::Path;

pub fn hard_link(source: &Path, dest: &Path) -> io::Result<()> {
    unsafe {
        let _scope_alloc = ScopedAlloc::new(AllocId::TlsTemp);

        let source = source.to_null_terminated_path_buf();
        let dest = dest.to_null_terminated_path_buf();

        let res = CreateHardLinkA(PCSTR(source.as_ptr()), PCSTR(dest.as_ptr()), None).as_bool();
        if res {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }
}

pub fn symlink_file(source: &Path, dest: &Path) -> io::Result<()> {
    unsafe {
        let source = source.to_null_terminated_path_buf();
        let dest = dest.to_null_terminated_path_buf();

        let res = CreateSymbolicLinkA(PCSTR(source.as_ptr()), PCSTR(dest.as_ptr()), SYMBOLIC_LINK_FLAGS(0)).as_bool();
        if res {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }
}

pub fn symlink_dir(source: &Path, dest: &Path) -> io::Result<()> {
    unsafe {
        let source = source.to_null_terminated_path_buf();
        let dest = dest.to_null_terminated_path_buf();

        let res = CreateSymbolicLinkA(PCSTR(source.as_ptr()), PCSTR(dest.as_ptr()), SYMBOLIC_LINK_FLAG_DIRECTORY).as_bool();
        if res {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }
}