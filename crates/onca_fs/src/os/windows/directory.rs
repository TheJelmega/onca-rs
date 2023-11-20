use std::io::Error;
use onca_common::{
    prelude::*,
    io,
};
use windows::{
    Win32::{
        Storage::FileSystem::{CreateDirectoryA, RemoveDirectoryA},
        Foundation::ERROR_ALREADY_EXISTS,
    },
    core::PCSTR,
};
use crate::Path;

pub(crate) fn create(path: &Path) -> io::Result<()> {
    scoped_alloc!(AllocId::TlsTemp);

    let path = path.to_path_buf();
    create_dir(PCSTR(path.as_ptr()))
}

pub(crate) fn create_recursive(path: &Path) -> io::Result<()> {
    scoped_alloc!(AllocId::TlsTemp);

    let mut parent_paths = Vec::new();
    for ancestor in path.ancestors() {
        parent_paths.push(ancestor);
    }

    for cur_dir in parent_paths.into_iter().rev() {
        let path = cur_dir.to_path_buf();
        create_dir(PCSTR(path.as_ptr()))?;
    }
    Ok(())
}

fn create_dir(pcstr: PCSTR) -> io::Result<()> {
    match unsafe { CreateDirectoryA(pcstr, None) } {
        Err(err) if err.code().0 as u32 != ERROR_ALREADY_EXISTS.0 =>
            Err(std::io::Error::from_raw_os_error(err.code().0)),
        _ => Ok(())
    }
}

pub(crate) fn remove(path: &Path) -> io::Result<()> {
    scoped_alloc!(AllocId::TlsTemp);
    
    let path = path.to_path_buf();
    unsafe { RemoveDirectoryA(PCSTR(path.as_ptr())) }
        .map_err(|err| Error::from_raw_os_error(err.code().0))
    
}