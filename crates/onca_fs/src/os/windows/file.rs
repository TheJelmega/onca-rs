use std::{
    ffi::c_void,
    mem::size_of,
    num::NonZeroU64,
    sync::atomic::{AtomicU32, Ordering},
};
use onca_core::{
    prelude::*,
    io,
};
use windows::{
    Win32::{
        Storage::FileSystem::{
            GetCompressedFileSizeA, 
            CreateFileA, 
            FILE_APPEND_DATA, FILE_SHARE_READ, FILE_SHARE_WRITE, FILE_SHARE_MODE, DELETE,
            OPEN_ALWAYS, OPEN_EXISTING, CREATE_NEW, CREATE_ALWAYS, TRUNCATE_EXISTING,
            DeleteFileA, 
            ReadFile,  WriteFile, FlushFileBuffers,
            SetFilePointerEx, SET_FILE_POINTER_MOVE_METHOD, FILE_BEGIN, FILE_CURRENT, FILE_END,
            FILE_FLAGS_AND_ATTRIBUTES, FILE_ATTRIBUTE_TEMPORARY, FILE_FLAG_OPEN_REPARSE_POINT, FILE_ATTRIBUTE_READONLY, FILE_ATTRIBUTE_HIDDEN, FILE_ATTRIBUTE_NOT_CONTENT_INDEXED, FILE_FLAG_BACKUP_SEMANTICS,
            SetFileInformationByHandle, GetFileInformationByHandleEx, FileBasicInfo,  FILE_BASIC_INFO, FileEndOfFileInfo, FILE_END_OF_FILE_INFO,
            SetFileTime,  GetTempFileNameA, FILE_GENERIC_READ, FILE_GENERIC_WRITE, FILE_GENERIC_EXECUTE, FILE_ACCESS_RIGHTS, FILE_ATTRIBUTE_ENCRYPTED, FILE_FLAG_DELETE_ON_CLOSE, FILE_FLAG_NO_BUFFERING, FILE_FLAG_OVERLAPPED, FILE_FLAG_RANDOM_ACCESS, FILE_FLAG_SEQUENTIAL_SCAN, FILE_FLAG_WRITE_THROUGH, FILE_ATTRIBUTE_ARCHIVE
        }, 
        Foundation::{GetLastError, HANDLE, CloseHandle, FILETIME}, 
    }, 
    core::PCSTR,
};

use crate::{Path, Permission, OpenMode, FileCreateFlags, PathBuf, os::windows::MAX_PATH};

use super::{INVALID_FILE_SIZE, high_low_to_u64};

/// Pathbuf must be null terminated
pub(crate) fn get_compressed_size_pathbuf(path: &PathBuf) -> io::Result<Option<NonZeroU64>> {
    scoped_alloc!(AllocId::TlsTemp);
    
    let path = path.to_null_terminated_path_buf();

    let mut high = 0;
    let low = unsafe { GetCompressedFileSizeA(PCSTR(path.as_ptr()), Some(&mut high)) };
    if low == INVALID_FILE_SIZE {
        match unsafe { GetLastError() } {
            Ok(_)    => Ok(NonZeroU64::new(high_low_to_u64(high, low))),
            Err(err) => Err(io::Error::from_raw_os_error(err.code().0)),
        }
    } else {
        Ok(NonZeroU64::new(high_low_to_u64(high, low)))
    }
} 

pub(crate) fn delete(path: &Path) -> io::Result<()> {
    scoped_alloc!(AllocId::TlsTemp);
    let _scope_alloc = ScopedAlloc::new(AllocId::TlsTemp);

    let path = path.to_null_terminated_path_buf();
    unsafe { DeleteFileA(PCSTR(path.as_ptr())) }.map_err(|err| io::Error::from_raw_os_error(err.code().0))
}

pub struct FileHandle(pub(crate) HANDLE);

impl FileHandle {
    pub(crate) fn create(path: &Path, open_mode: OpenMode, access: Permission, shared_access: Permission, flags: FileCreateFlags, open_link: bool, temporary: bool) -> io::Result<(FileHandle, PathBuf)> {
        let mut path_buf = path.to_null_terminated_path_buf();

        if temporary {
            let mut file_name = [0u8; MAX_PATH];
            let path_name = ['.' as u16, 0];
            let prefix_string = ['O' as u16, 'N' as u16, 'C' as u16, 'A' as u16, 0];
            static UUNIQUE : AtomicU32 = AtomicU32::new(1);
            let unique = UUNIQUE.fetch_add(1, Ordering::AcqRel);
            let res = unsafe { GetTempFileNameA(PCSTR(path_name.as_ptr() as *const _), PCSTR(prefix_string.as_ptr() as *const _), unique, &mut file_name) };
            if res != 0 {
                unsafe { GetLastError() }.map_err(|err| io::Error::from_raw_os_error(err.code().0))?;
            }

            let temp_end = file_name.iter().position(|&c| c == 0).unwrap_or_default();
            if temp_end > 0 {
                let _scope_alloc = ScopedAlloc::new(AllocId::TlsTemp);
                path_buf.push(PathBuf::from_utf8_lossy(&file_name[..temp_end]))
            }
        }
        
        let mut win32_access = FILE_ACCESS_RIGHTS(0);
        if access.contains(Permission::Read) {
            win32_access |= FILE_GENERIC_READ;
        }
        if access.contains(Permission::Write) {
            win32_access |= FILE_GENERIC_WRITE;
        } else if access.contains(Permission::Append) {
            win32_access |= FILE_APPEND_DATA;
        }
        if access.contains(Permission::Execute) {
            win32_access |= FILE_GENERIC_EXECUTE;
        }
        if access.contains(Permission::Delete) {
            win32_access |= DELETE;
        }
    
        let mut win32_access_share = 0;
        if shared_access.contains(Permission::Read) {
            win32_access_share |= FILE_SHARE_READ.0;
        }
        if shared_access.contains(Permission::Write) || shared_access.contains(Permission::Append) {
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

        let mut win32_flags = 0;
        if flags.contains(FileCreateFlags::ReadOnly) {
            win32_flags = FILE_ATTRIBUTE_READONLY.0;
        }
        if flags.contains(FileCreateFlags::Hidden) {
            win32_flags |= FILE_ATTRIBUTE_HIDDEN.0;
        }
        if flags.contains(FileCreateFlags::AllowBackup) {
            win32_flags |= FILE_ATTRIBUTE_ARCHIVE.0;
        }
        if flags.contains(FileCreateFlags::Encrypted) {
            win32_flags |= FILE_ATTRIBUTE_ENCRYPTED.0;
        }
        if flags.contains(FileCreateFlags::DeleteOnClose) {
            win32_flags |= FILE_FLAG_DELETE_ON_CLOSE.0;
        }
        if flags.contains(FileCreateFlags::NoBuffering) {
            win32_flags |= FILE_FLAG_NO_BUFFERING.0;
        }
        if flags.contains(FileCreateFlags::SupportAsync) {
            win32_flags |= FILE_FLAG_OVERLAPPED.0;
        }
        if flags.contains(FileCreateFlags::RandomAccess) {
            win32_flags |= FILE_FLAG_RANDOM_ACCESS.0;
        }
        if flags.contains(FileCreateFlags::SequentialAccess) {
            win32_flags |= FILE_FLAG_SEQUENTIAL_SCAN.0;
        }
        if flags.contains(FileCreateFlags::WriteThrough) {
            win32_flags |= FILE_FLAG_WRITE_THROUGH.0;
        }

        if flags.contains(FileCreateFlags::AllowBackup) {
            win32_flags |= FILE_FLAG_BACKUP_SEMANTICS.0;
        }

        if open_link {
            win32_flags |= FILE_FLAG_OPEN_REPARSE_POINT.0;
        }
        if temporary {
            win32_flags |= FILE_ATTRIBUTE_TEMPORARY.0;
        }
        
        let handle = unsafe { CreateFileA(
            PCSTR(path_buf.as_ptr()),
            win32_access.0,
            FILE_SHARE_MODE(win32_access_share),
            None,
            win32_create_disposition,
            FILE_FLAGS_AND_ATTRIBUTES(win32_flags),
            HANDLE::default()
        ) };
        match handle {
            Ok(handle) => Ok((FileHandle(handle), path_buf)),
            Err(err) => Err(io::Error::from_raw_os_error(err.code().0))
        }
    }

    pub(crate) fn read(&self, mut buf: &mut [u8]) -> io::Result<usize> {
        fn read_impl(handle: HANDLE, arr: &mut [u8]) -> io::Result<usize> {
            let mut bytes_read = 0;
            unsafe { ReadFile(handle, Some(arr), Some(&mut bytes_read), None) }
                .map_or_else(|err| Err(io::Error::from_raw_os_error(err.code().0)), |_| Ok(bytes_read as usize))
        }

        if buf.len() <= u32::MAX as usize {
            read_impl(self.0, &mut buf)

        // While it's extremely unlikely someone will read >4GiB into memory, we still need to be able to do it
        } else {
            let mut total_read = 0;
            // Initialize to 1 to start the first read cycle, Win32 will overwrite the value, so we just care that the value is > 0
            let mut bytes_read = 1;

            while bytes_read > 0 {
                let to_read = buf.len().max(u32::MAX as usize);
                bytes_read = read_impl(self.0, &mut buf[..to_read])?;
                total_read += bytes_read;
                buf = &mut buf[bytes_read..];
            }
            
            Ok(total_read)
        }
    }
    
    pub(crate) fn write(&mut self, mut buf: &[u8]) -> io::Result<usize> {
        fn write_impl(handle: &mut FileHandle, buf: &[u8]) -> io::Result<usize> {
            let mut bytes_written = 0;
            unsafe { WriteFile(handle.0, Some(buf), Some(&mut bytes_written), None) }
            .map_or_else(|err| Err(io::Error::from_raw_os_error(err.code().0)), |_| Ok(bytes_written as usize))
        }

        if buf.len() <= i32::MAX as usize {
            write_impl(self, buf)

        // While it's extremely unlikely someone will write >4GiB from memory, we still need to be able to do it
        } else {
            let mut total_written = 0;
            // Initialize to 1 to start the first read cycle, Win32 will overwrite the value, so we just care that the value is > 0
            let mut bytes_written = 1;

            while bytes_written > 0 {
                let to_write = buf.len().max(u32::MAX as usize);
                bytes_written = write_impl(self, &buf[..to_write])?;

                total_written += bytes_written;
                buf = &buf[bytes_written..];
            }
            Ok(total_written)
        }
    }

    pub(crate) fn flush(&mut self) -> io::Result<()> {
        unsafe { FlushFileBuffers(self.0) }.map_err(|err| io::Error::from_raw_os_error(err.code().0))
    }

    pub(crate) fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        let (dist, method) = match pos {
            io::SeekFrom::Start(pos) => (pos as i64, FILE_BEGIN),
            io::SeekFrom::End(pos) => (pos, FILE_END),
            io::SeekFrom::Current(pos) => (pos, FILE_CURRENT),
        };
        self.win32_seek(dist, method)
    }

    fn win32_seek(&mut self, dist: i64, method: SET_FILE_POINTER_MOVE_METHOD) -> io::Result<u64> {
        let mut cursor_pos = 0;
        unsafe { SetFilePointerEx(self.0, dist, Some(&mut cursor_pos), method) }
            .map_or_else(|err| Err(io::Error::from_raw_os_error(err.code().0)), |_| Ok(cursor_pos as u64))
    }

    pub(crate) fn flush_data(&mut self) -> io::Result<()> {
        self.flush()
    }

    pub(crate) fn flush_all(&mut self) -> io::Result<()> {
        self.flush()
    }

    pub(crate) fn set_len(&mut self, size: u64) -> io::Result<()> {
        let mut file_end_info = FILE_END_OF_FILE_INFO::default();
        file_end_info.EndOfFile = size as i64;

        unsafe { SetFileInformationByHandle(self.0, FileEndOfFileInfo, &file_end_info as *const _ as *const c_void , size_of::<FILE_END_OF_FILE_INFO>() as u32) }
            .map_err(|err| io::Error::from_raw_os_error(err.code().0))
    }

    pub fn set_modified(&mut self, time: u64) -> io::Result<()> {
        let mut file_time = FILETIME::default();
        file_time.dwLowDateTime = time as u32;
        file_time.dwHighDateTime = (time >> 32) as u32;

        unsafe { SetFileTime(self.0, None, None, Some(&file_time)) }.map_err(|err| io::Error::from_raw_os_error(err.code().0))
    }

    /// Does not set actual user permissions, but sets the general file flags, currently this is just setting the 'write flag'
    pub fn set_permissions(&mut self, permissions: Permission) -> io::Result<()> {
        self.set_attrib(FILE_ATTRIBUTE_READONLY, !permissions.contains(Permission::Write))
    }

    pub fn set_hidden(&mut self, hidden: bool) -> io::Result<()> {
        self.set_attrib(FILE_ATTRIBUTE_HIDDEN, hidden)
    }

    pub fn set_content_indexed(&mut self, content_indexed: bool) -> io::Result<()> {
        self.set_attrib(FILE_ATTRIBUTE_NOT_CONTENT_INDEXED, !content_indexed)
    }

    fn set_attrib(&mut self, attrib: FILE_FLAGS_AND_ATTRIBUTES, set: bool) -> io::Result<()> {
            let mut file_info = FILE_BASIC_INFO::default();
            unsafe { GetFileInformationByHandleEx(self.0, FileBasicInfo, &mut file_info as *mut _ as *mut c_void, size_of::<FILE_BASIC_INFO>() as u32) }
                .map_err(|err| io::Error::from_raw_os_error(err.code().0))?;

            if set {
                file_info.FileAttributes &= !attrib.0;
            } else {
                file_info.FileAttributes |= attrib.0;
            }
            unsafe { SetFileInformationByHandle(self.0, FileBasicInfo, &file_info as *const _ as *const c_void, size_of::<FILE_BASIC_INFO>() as u32) }
                .map_err(|err| io::Error::from_raw_os_error(err.code().0))
    }
}

impl Drop for FileHandle {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            unsafe { _ = CloseHandle(self.0); }
        }
    }
}





