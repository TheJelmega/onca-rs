use onca_core::{io, alloc::UseAlloc};
use windows::Win32::{Storage::FileSystem::{CreateHardLinkW, CreateSymbolicLinkW, SYMBOLIC_LINK_FLAGS, SYMBOLIC_LINK_FLAG_DIRECTORY}, Foundation::GetLastError};

use crate::Path;

use super::path_to_null_terminated_utf16;

pub fn hard_link(source: &Path, dest: &Path, temp_alloc: UseAlloc) -> io::Result<()> {
    unsafe {
        let (_s_buf, s_pcwstr) = path_to_null_terminated_utf16(source, temp_alloc);
        let (_d_buf, d_pcwstr) = path_to_null_terminated_utf16(dest, temp_alloc);

        let res = CreateHardLinkW(d_pcwstr, s_pcwstr, None).as_bool();
        if res {
            Ok(())
        } else {
            Err(io::Error::from_raw_os_error(GetLastError().0 as i32))
        }
    }
}

pub fn symlink_file(source: &Path, dest: &Path, temp_alloc: UseAlloc) -> io::Result<()> {
    unsafe {
        let (_s_buf, s_pcwstr) = path_to_null_terminated_utf16(source, temp_alloc);
        let (_d_buf, d_pcwstr) = path_to_null_terminated_utf16(dest, temp_alloc);

        let res = CreateSymbolicLinkW(d_pcwstr, s_pcwstr, SYMBOLIC_LINK_FLAGS(0)).as_bool();
        if res {
            Ok(())
        } else {
            Err(io::Error::from_raw_os_error(GetLastError().0 as i32))
        }
    }
}

pub fn symlink_dir(source: &Path, dest: &Path, temp_alloc: UseAlloc) -> io::Result<()> {
    unsafe {
        let (_s_buf, s_pcwstr) = path_to_null_terminated_utf16(source, temp_alloc);
        let (_d_buf, d_pcwstr) = path_to_null_terminated_utf16(dest, temp_alloc);

        let res = CreateSymbolicLinkW(d_pcwstr, s_pcwstr, SYMBOLIC_LINK_FLAG_DIRECTORY).as_bool();
        if res {
            Ok(())
        } else {
            Err(io::Error::from_raw_os_error(GetLastError().0 as i32))
        }
    }
}