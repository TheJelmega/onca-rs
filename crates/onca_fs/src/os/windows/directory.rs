use onca_core::{
    alloc::{UseAlloc},
    collections::SmallDynArray,
    io,
};
use windows::{
    Win32::{
        Storage::FileSystem::{CreateDirectoryW, RemoveDirectoryW},
        Foundation::{GetLastError, ERROR_ALREADY_EXISTS},
    },
    core::PCWSTR,
};
use crate::{Path, FsMemTag};
use super::path_to_null_terminated_utf16;

pub(crate) fn create(path: &Path, temp_alloc: UseAlloc) -> io::Result<()> {
    unsafe {
        let (_buf, pcwstr) = path_to_null_terminated_utf16(path, temp_alloc);
        create_dir(pcwstr)
    }
}

pub(crate) fn create_recursive(path: &Path, temp_alloc: UseAlloc) -> io::Result<()> {
    unsafe {
        let mut parent_paths = SmallDynArray::<_, 8>::new(temp_alloc, FsMemTag::Temporary.to_mem_tag());
        for component in path.ancestors() {
            parent_paths.push(component);
        }

        for cur_dir in parent_paths.into_iter().rev() {
            let (_buf, pcwstr) = path_to_null_terminated_utf16(cur_dir, temp_alloc);
            create_dir(pcwstr)?;
        }
        Ok(())
    }
}

unsafe fn create_dir(pcwstr: PCWSTR) -> io::Result<()> {
    let res = CreateDirectoryW(pcwstr, None).as_bool();
    if res {
        Ok(())
    } else {
        let err = GetLastError();
        if err == ERROR_ALREADY_EXISTS {
            Ok(())
        } else {
            Err(io::Error::from_raw_os_error(GetLastError().0 as i32))
        }
    }
}

pub(crate) fn remove(path: &Path, temp_alloc: UseAlloc) -> io::Result<()> {
    unsafe {
        let (_but, pcwstr) = path_to_null_terminated_utf16(path, temp_alloc);
        let res = RemoveDirectoryW(pcwstr).as_bool();
        if res {
            Ok(())
        } else {
            Err(io::Error::from_raw_os_error(GetLastError().0 as i32))
        }
    }
}