use core::{
    mem,
    ffi::c_void,
    sync::atomic::{AtomicU32, Ordering},
};
use std::mem::size_of;
use onca_core::{
    io,
    alloc::{UseAlloc},
};
use windows::{
    Win32::{
        Storage::FileSystem::{
            GetCompressedFileSizeW, 
            CreateFileW, 
            FILE_APPEND_DATA, FILE_ACCESS_FLAGS, FILE_SHARE_READ, FILE_SHARE_WRITE, FILE_SHARE_MODE, DELETE,
            OPEN_ALWAYS, OPEN_EXISTING, CREATE_NEW, CREATE_ALWAYS, TRUNCATE_EXISTING,
            DeleteFileW, 
            ReadFile,  WriteFile, FlushFileBuffers,
            SetFilePointerEx, SET_FILE_POINTER_MOVE_METHOD, FILE_BEGIN, FILE_CURRENT, FILE_END,
            FILE_FLAGS_AND_ATTRIBUTES, FILE_ATTRIBUTE_TEMPORARY, FILE_FLAG_OPEN_REPARSE_POINT, FILE_ATTRIBUTE_READONLY, FILE_ATTRIBUTE_HIDDEN, FILE_ATTRIBUTE_NOT_CONTENT_INDEXED, FILE_FLAG_BACKUP_SEMANTICS,
            SetFileInformationByHandle, GetFileInformationByHandleEx, FileBasicInfo,  FILE_BASIC_INFO, FileEndOfFileInfo, FILE_END_OF_FILE_INFO,
            SetFileTime,  GetTempFileNameW
        }, 
        Foundation::{GetLastError, NO_ERROR, HANDLE, CloseHandle, FILETIME}, 
        System::SystemServices::{GENERIC_READ, GENERIC_EXECUTE, GENERIC_WRITE}
    }, 
    core::PCWSTR,
};

use crate::{Path, Permission, OpenMode, FileCreateFlags, PathBuf, os::windows::MAX_PATH};

use super::{path_to_null_terminated_utf16, INVALID_FILE_SIZE, high_low_to_u64};

pub(crate) fn get_compressed_size(path: &Path) -> io::Result<u64> {
    unsafe {
        let (_buf, pcwstr) = path_to_null_terminated_utf16(path);

        let mut high = 0;
        let low = GetCompressedFileSizeW(pcwstr, Some(&mut high));
        if low == INVALID_FILE_SIZE {
            let error = GetLastError();
            if error == NO_ERROR {
                Ok(high_low_to_u64(high, low))
            } else {
                Err(io::Error::from_raw_os_error(error.0 as i32))
            }
        } else {
            Ok(high_low_to_u64(high, low))
        }
    }
} 

pub(crate) fn delete(path: &Path) -> io::Result<()> {
    unsafe {
        let (_buf, pcwstr) = path_to_null_terminated_utf16(path);
        let res = DeleteFileW(pcwstr).as_bool();
        if res {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }
}

pub struct FileHandle(HANDLE);

impl FileHandle {
    pub(crate) fn create(path: &Path, open_mode: OpenMode, access: Permission, shared_access: Permission, flags: FileCreateFlags, alloc: UseAlloc, open_link: bool, temporary: bool) -> io::Result<(FileHandle, PathBuf)> {
        unsafe {
            let (mut buf, pcwstr) = path_to_null_terminated_utf16(path);
            let mut path_buf = path.to_path_buf(alloc);

            if temporary {
                let mut file_name = [0u16; MAX_PATH];
                let path_name = ['.' as u16, 0];
                let prefix_string = ['O' as u16, 'N' as u16, 'C' as u16, 'A' as u16, 0];
                static UUNIQUE : AtomicU32 = AtomicU32::new(1);
                let unique = UUNIQUE.fetch_add(1, Ordering::AcqRel);
                let res = GetTempFileNameW(PCWSTR(path_name.as_ptr() as *const _), PCWSTR(prefix_string.as_ptr() as *const _), unique, &mut file_name);
                assert!(res != 0);

                let temp_end = file_name.iter().position(|&c| c == 0).unwrap_or_default();
                if temp_end > 0 {
                    buf.extend_from_slice(&file_name[..temp_end]);
                    path_buf.push(PathBuf::from_utf16_lossy(&file_name[..temp_end], alloc))
                }
            }
            
            let mut win32_access = 0;
            if access.is_set(Permission::Read) {
                win32_access |= GENERIC_READ;
            }
            if access.is_set(Permission::Write) {
                win32_access |= GENERIC_WRITE;
            } else if access.is_set(Permission::Append) {
                win32_access |= FILE_APPEND_DATA.0;
            }
            if access.is_set(Permission::Execute) {
                win32_access |= GENERIC_EXECUTE;
            }
            if access.is_set(Permission::Delete) {
                win32_access |= DELETE.0;
            }
        
            let mut win32_access_share = 0;
            if shared_access.is_set(Permission::Read) {
                win32_access_share |= FILE_SHARE_READ.0;
            }
            if shared_access.is_set(Permission::Write) || shared_access.is_set(Permission::Append) {
                win32_access_share |= FILE_SHARE_WRITE.0;
            }
            // Do this is an assert, as the user should never pass Execture here
            assert!(shared_access & Permission::Execute != Permission::Execute, "Cannot share file execute permission");

            let win32_create_disposition = match open_mode {
                OpenMode::OpenOrCreate => OPEN_ALWAYS,
                OpenMode::OpenExisting => OPEN_EXISTING,
                OpenMode::CreateNonExisting => CREATE_NEW,
                OpenMode::CreateAlways => CREATE_ALWAYS,
                OpenMode::TruncateExisting => TRUNCATE_EXISTING,
            };

            let mut win32_flags = mem::transmute_copy::<_, u32>(&flags);
            if flags.is_set(FileCreateFlags::AllowBackup) {
                win32_flags |= FILE_FLAG_BACKUP_SEMANTICS.0;
            }

            if open_link {
                win32_flags |= FILE_FLAG_OPEN_REPARSE_POINT.0;
            }
            if temporary {
                win32_flags |= FILE_ATTRIBUTE_TEMPORARY.0;
            }
            
            let handle = CreateFileW(
                pcwstr,
                FILE_ACCESS_FLAGS(win32_access),
                FILE_SHARE_MODE(win32_access_share),
                None,
                win32_create_disposition,
                FILE_FLAGS_AND_ATTRIBUTES(win32_flags),
                HANDLE::default()
            );
            match handle {
                Ok(handle) => Ok((FileHandle(handle), path_buf)),
                Err(err) => Err(io::Error::from_raw_os_error(err.code().0))
            }
        }
    }

    pub(crate) fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        unsafe {
            unsafe fn read_impl(handle: HANDLE, ptr: *mut u8, len: u32) -> io::Result<usize> {
                let mut bytes_read = 0;
                let res = ReadFile(handle, Some(ptr as *mut c_void), len, Some(&mut bytes_read), None);
                if res.as_bool() {
                    Ok(bytes_read as usize)
                } else {
                    Err(io::Error::last_os_error())
                }
            }

            let mut buf_len = buf.len();
            let mut buf_ptr = buf.as_mut_ptr();
            if buf_len <= u32::MAX as usize {
                read_impl(self.0, buf_ptr, buf_len as u32)

            // While it's extremely unlikely someone will read >4GiB into memory, we still need to be able to do it
            } else {
                let mut total_read = 0;
                // Initialize to 1 to start the first read cycle, Win32 will overwrite the value, so we just care that the value is > 0
                let mut bytes_read = 1;

                while bytes_read > 0 {
                    let to_read = if buf_len > u32::MAX as usize { u32::MAX } else { buf_len as u32 };
                    bytes_read = read_impl(self.0, buf_ptr, to_read)?;

                    buf_len -= bytes_read;
                    total_read += bytes_read;
                    buf_ptr = buf_ptr.add(bytes_read);
                }
                
                Ok(total_read)
            }
        }
    }
    
    pub(crate) fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        unsafe {
            unsafe fn write_impl(handle: &mut FileHandle, ptr: *const u8, len: u32) -> io::Result<usize> {
                let mut bytes_written = 0;
                let res = WriteFile(handle.0, Some(ptr as *const c_void), len, Some(&mut bytes_written), None);
                if res.as_bool() {
                    Ok(bytes_written as usize)
                } else {
                    Err(io::Error::last_os_error())
                }
            }

            let mut buf_ptr = buf.as_ptr();
            let mut buf_len = buf.len();
            if buf_len <= i32::MAX as usize {
                write_impl(self, buf_ptr, buf_len as u32)

            // While it's extremely unlikely someone will write >4GiB from memory, we still need to be able to do it
            } else {
                let mut total_written = 0;
                // Initialize to 1 to start the first read cycle, Win32 will overwrite the value, so we just care that the value is > 0
                let mut bytes_written = 1;

                while bytes_written > 0 {
                    let to_write = if buf_len > u32::MAX as usize { u32::MAX } else { buf_len as u32 };
                    bytes_written = write_impl(self, buf_ptr, to_write)?;

                    buf_len -= bytes_written;
                    total_written += bytes_written;
                    buf_ptr = buf_ptr.add(bytes_written);
                }
                Ok(total_written)
            }
        }
    }

    pub(crate) fn flush(&mut self) -> io::Result<()> {
        unsafe {
            let res = FlushFileBuffers(self.0);
            if res.as_bool() {    
                Ok(())
            } else {
                Err(io::Error::last_os_error())
            }
        }
    }

    pub(crate) fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        unsafe { 
            let (dist, method) = match pos {
                io::SeekFrom::Start(pos) => (pos as i64, FILE_BEGIN),
                io::SeekFrom::End(pos) => (pos, FILE_END),
                io::SeekFrom::Current(pos) => (pos, FILE_CURRENT),
            };

            self.win32_seek(dist, method)
        }
    }

    unsafe fn win32_seek(&mut self, dist: i64, method: SET_FILE_POINTER_MOVE_METHOD) -> io::Result<u64> {
        let mut cursor_pos = 0;
        let res = SetFilePointerEx(self.0, dist, Some(&mut cursor_pos), method);
        if res.as_bool() {
            Ok(cursor_pos as u64)
        } else {
            Err(io::Error::last_os_error())
        }
    }

    pub(crate) fn sync_data(&mut self) -> io::Result<()> {
        self.sync()
    }

    pub(crate) fn sync_all(&mut self) -> io::Result<()> {
        self.sync()
    }

    fn sync(&mut self) -> io::Result<()> {
        unsafe {
            let res = FlushFileBuffers(self.0);
            if res.as_bool() {
                Ok(())
            } else {
                Err(io::Error::last_os_error())
            }
        }
    }

    pub(crate) fn set_len(&mut self, size: u64) -> io::Result<()> {
        unsafe {
            
            let mut file_end_info = FILE_END_OF_FILE_INFO::default();
            file_end_info.EndOfFile = size as i64;

            let res = SetFileInformationByHandle(self.0, FileEndOfFileInfo, &file_end_info as *const _ as *const c_void , size_of::<FILE_END_OF_FILE_INFO>() as u32);
            if res.as_bool() {
                Ok(())
            } else {
                Err(io::Error::last_os_error())
            }
        }
    }

    pub fn set_modified(&mut self, time: u64) -> io::Result<()> {
        unsafe {
            let mut file_time = FILETIME::default();
            file_time.dwLowDateTime = time as u32;
            file_time.dwHighDateTime = (time >> 32) as u32;

            let res = SetFileTime(self.0, None, None, Some(&file_time));
            if res.as_bool() {
                Ok(())
            } else {
                Err(io::Error::last_os_error())
            }
        }
    }

    /// Does not set actual user permissions, but sets the general file flags, currently this is just setting the 'write flag'
    pub fn set_permissions(&mut self, permissions: Permission) -> io::Result<()> {
        self.set_attrib(FILE_ATTRIBUTE_READONLY, !permissions.is_set(Permission::Write))
    }

    pub fn set_hidden(&mut self, hidden: bool) -> io::Result<()> {
        self.set_attrib(FILE_ATTRIBUTE_HIDDEN, hidden)
    }

    pub fn set_content_indexed(&mut self, content_indexed: bool) -> io::Result<()> {
        self.set_attrib(FILE_ATTRIBUTE_NOT_CONTENT_INDEXED, !content_indexed)
    }

    fn set_attrib(&mut self, attrib: FILE_FLAGS_AND_ATTRIBUTES, set: bool) -> io::Result<()> {
        unsafe {
            let mut file_info = FILE_BASIC_INFO::default();
            let res = GetFileInformationByHandleEx(self.0, FileBasicInfo, &mut file_info as *mut _ as *mut c_void, size_of::<FILE_BASIC_INFO>() as u32);
            if !res.as_bool() {
                return Err(io::Error::last_os_error());
            }

            if set {
                file_info.FileAttributes &= !attrib.0;
            } else {
                file_info.FileAttributes |= attrib.0;
            }

            let res = SetFileInformationByHandle(self.0, FileBasicInfo, &file_info as *const _ as *const c_void, size_of::<FILE_BASIC_INFO>() as u32);
            if res.as_bool() {
                Ok(())
            } else {
                Err(io::Error::last_os_error())
            }
        }
    }
}

impl Drop for FileHandle {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            unsafe { CloseHandle(self.0); }
        }
    }
}





