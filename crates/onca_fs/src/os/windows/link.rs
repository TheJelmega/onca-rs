use onca_common::{
    prelude::*,   
    io, alloc::ScopedAlloc
};
use windows::{
    Win32::Storage::FileSystem::{CreateHardLinkA, CreateSymbolicLinkA, SYMBOLIC_LINK_FLAGS, SYMBOLIC_LINK_FLAG_DIRECTORY},
    core::PCSTR
};

use crate::Path;

pub fn hard_link(source: &Path, dest: &Path) -> io::Result<()> {
    scoped_alloc!(AllocId::TlsTemp);
    let source = source.to_path_buf();
    let dest = dest.to_path_buf();

    unsafe { CreateHardLinkA(PCSTR(source.as_ptr()), PCSTR(dest.as_ptr()), None) }
        .map_err(|err| io::Error::from_raw_os_error(err.code().0))
}

pub fn symlink_file(source: &Path, dest: &Path) -> io::Result<()> {
    scoped_alloc!(AllocId::TlsTemp);
    let source = source.to_path_buf();
    let dest = dest.to_path_buf();

    let res = unsafe { CreateSymbolicLinkA(PCSTR(source.as_ptr()), PCSTR(dest.as_ptr()), SYMBOLIC_LINK_FLAGS(0)) }.as_bool();
    if res {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

pub fn symlink_dir(source: &Path, dest: &Path) -> io::Result<()> {
    scoped_alloc!(AllocId::TlsTemp);
    let source = source.to_path_buf();
    let dest = dest.to_path_buf();

    let res = unsafe { CreateSymbolicLinkA(PCSTR(source.as_ptr()), PCSTR(dest.as_ptr()), SYMBOLIC_LINK_FLAG_DIRECTORY).as_bool() };
    if res {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}