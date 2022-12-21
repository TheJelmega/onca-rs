use onca_core::io;
use windows::Win32::{Storage::FileSystem::{CreateHardLinkW, CreateSymbolicLinkW, SYMBOLIC_LINK_FLAGS, SYMBOLIC_LINK_FLAG_DIRECTORY}};

use crate::Path;

use super::path_to_null_terminated_utf16;

pub fn hard_link(source: &Path, dest: &Path) -> io::Result<()> {
    unsafe {
        let (_s_buf, s_pcwstr) = path_to_null_terminated_utf16(source);
        let (_d_buf, d_pcwstr) = path_to_null_terminated_utf16(dest);

        let res = CreateHardLinkW(d_pcwstr, s_pcwstr, None).as_bool();
        if res {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }
}

pub fn symlink_file(source: &Path, dest: &Path) -> io::Result<()> {
    unsafe {
        let (_s_buf, s_pcwstr) = path_to_null_terminated_utf16(source);
        let (_d_buf, d_pcwstr) = path_to_null_terminated_utf16(dest);

        let res = CreateSymbolicLinkW(d_pcwstr, s_pcwstr, SYMBOLIC_LINK_FLAGS(0)).as_bool();
        if res {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }
}

pub fn symlink_dir(source: &Path, dest: &Path) -> io::Result<()> {
    unsafe {
        let (_s_buf, s_pcwstr) = path_to_null_terminated_utf16(source);
        let (_d_buf, d_pcwstr) = path_to_null_terminated_utf16(dest);

        let res = CreateSymbolicLinkW(d_pcwstr, s_pcwstr, SYMBOLIC_LINK_FLAG_DIRECTORY).as_bool();
        if res {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }
}