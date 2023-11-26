use std::{
    ffi::c_void,
    mem::{size_of, self},
    num::NonZeroU64,
    sync::atomic::{AtomicU32, Ordering}, task::Poll, io::SeekFrom,
};
use onca_common::{
    prelude::*,
    io, sync::Mutex,
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
            SetFileTime,  GetTempFileNameA, FILE_GENERIC_READ, FILE_GENERIC_WRITE, FILE_GENERIC_EXECUTE, FILE_ACCESS_RIGHTS, FILE_ATTRIBUTE_ENCRYPTED, FILE_FLAG_DELETE_ON_CLOSE, FILE_FLAG_NO_BUFFERING, FILE_FLAG_OVERLAPPED, FILE_FLAG_RANDOM_ACCESS, FILE_FLAG_SEQUENTIAL_SCAN, FILE_FLAG_WRITE_THROUGH, FILE_ATTRIBUTE_ARCHIVE, ReadFileEx, WriteFileEx
        }, 
        Foundation::{GetLastError, HANDLE, CloseHandle, FILETIME, ERROR_SUCCESS, ERROR_TIMEOUT, WAIT_EVENT, BOOL}, System::{IO::{OVERLAPPED, CancelIoEx, CancelIo}, Threading::WaitForSingleObjectEx}, 
    }, 
    core::PCSTR,
};

use crate::{Path, Permission, OpenMode, FileCreateFlags, PathBuf, os::windows::MAX_PATH, FileAsyncWriteResult, FileAsyncReadResult, FileAccessFlags};

use super::{INVALID_FILE_SIZE, high_low_to_u64};

/// Pathbuf must be null terminated
pub(crate) fn get_compressed_size_pathbuf(path: &PathBuf) -> io::Result<Option<NonZeroU64>> {
    scoped_alloc!(AllocId::TlsTemp);
    
    let path = path.to_path_buf();

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

    let path = path.to_path_buf();
    unsafe { DeleteFileA(PCSTR(path.as_ptr())) }.map_err(|err| io::Error::from_raw_os_error(err.code().0))
}

pub struct FileHandle(pub(crate) HANDLE);

impl crate::file::FileHandle for FileHandle {
    fn flush_data(&mut self) -> io::Result<()> {
        self.flush()
    }

    fn flush_all(&mut self) -> io::Result<()> {
        self.flush()
    }

    fn cancel_all_thread_async_io(&mut self) -> io::Result<()> {
        unsafe { CancelIoEx(self.0, None) }.map_err(|err| io::Error::from_raw_os_error(err.code().0))
    }

    fn cancel_all_async_io(&mut self) -> io::Result<()> {
        unsafe { CancelIo(self.0) }.map_err(|err| io::Error::from_raw_os_error(err.code().0))
    }

    fn set_len(&mut self, len: u64) -> io::Result<()> {
        let mut file_end_info = FILE_END_OF_FILE_INFO::default();
        file_end_info.EndOfFile = len as i64;

        unsafe { SetFileInformationByHandle(self.0, FileEndOfFileInfo, &file_end_info as *const _ as *const c_void , size_of::<FILE_END_OF_FILE_INFO>() as u32) }
            .map_err(|err| io::Error::from_raw_os_error(err.code().0))
    }

    fn set_modified(&mut self, time: u64) -> io::Result<()> {
        let mut file_time = FILETIME::default();
        file_time.dwLowDateTime = time as u32;
        file_time.dwHighDateTime = (time >> 32) as u32;

        unsafe { SetFileTime(self.0, None, None, Some(&file_time)) }.map_err(|err| io::Error::from_raw_os_error(err.code().0))
    }

    fn set_permissions(&mut self, permissions: Permission) -> io::Result<()> {
        self.set_attrib(FILE_ATTRIBUTE_READONLY, !permissions.contains(Permission::Write))
    }

    fn set_hidden(&mut self, hidden: bool) -> io::Result<()> {
        self.set_attrib(FILE_ATTRIBUTE_HIDDEN, hidden)
    }

    fn set_content_indexed(&mut self, content_indexed: bool) -> io::Result<()> {
        self.set_attrib(FILE_ATTRIBUTE_NOT_CONTENT_INDEXED, !content_indexed)
    }

    fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
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

    fn write(&mut self, mut buf: &[u8]) -> io::Result<usize> {
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

    fn flush(&mut self) -> io::Result<()> {
        unsafe { FlushFileBuffers(self.0) }.map_err(|err| io::Error::from_raw_os_error(err.code().0))
    }

    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        let (dist, method) = match pos {
            io::SeekFrom::Start(pos)   => (pos as i64, FILE_BEGIN),
            io::SeekFrom::End(pos)     => (pos, FILE_END),
            io::SeekFrom::Current(pos) => (pos, FILE_CURRENT),
        };
        self.win32_seek(dist, method)
    }

    fn read_async(&mut self, bytes_to_read: u64) -> io::Result<FileAsyncReadResult> {
        let cursor_pos = self.seek(SeekFrom::Current(0))?;
        
        let mut overlapped = Box::new(OVERLAPPED::default());
        overlapped.Anonymous.Anonymous.Offset = cursor_pos as u32;
        overlapped.Anonymous.Anonymous.OffsetHigh = (cursor_pos >> 32) as u32;
        
        let completion_data = Box::new(AsyncIOCompletionData::new());
        overlapped.hEvent = unsafe { mem::transmute(&*completion_data) };
        
        
        let mut buffer = Vec::with_capacity(bytes_to_read as usize);
        unsafe { buffer.set_len(bytes_to_read as usize) };
        unsafe { ReadFileEx(
            self.0,
            Some(&mut buffer),
            &mut *overlapped,
            Some(io_completion_callback)
        ) }.map_err(|err| io::Error::from_raw_os_error(err.code().0))?;

        Ok(Box::new(AsyncReadResult{
            file_handle: self.0,
            buffer,
            overlapped,
            completion_data
        }))
    }

    fn write_async(&mut self, buf: Vec<u8>) -> io::Result<FileAsyncWriteResult> {
        let cursor_pos = self.seek(SeekFrom::Current(0))?;

        let mut overlapped = Box::new(OVERLAPPED::default());
        overlapped.Anonymous.Anonymous.Offset = cursor_pos as u32;
        overlapped.Anonymous.Anonymous.OffsetHigh = (cursor_pos >> 32) as u32;

        let completion_data = Box::new(AsyncIOCompletionData::new());
        overlapped.hEvent = unsafe { mem::transmute(&*completion_data) };

        unsafe { WriteFileEx(
            self.0,
            Some(&buf),
            &mut *overlapped,
            Some(io_completion_callback)
        ) }.map_err(|err| io::Error::from_raw_os_error(err.code().0))?;

        Ok(Box::new(AsyncWriteResult{
            file_handle: self.0,
            _buffer: buf,
            overlapped,
            completion_data
        }))
    }
}

impl FileHandle {
    pub(crate) fn create(
        path: &Path,
        open_mode: OpenMode,
        access_perms: Permission,
        shared_access_perms: Permission,
        create_flags: FileCreateFlags,
        access_flags: FileAccessFlags,
        open_link: bool,
        temporary: bool
    ) -> io::Result<(Box<dyn crate::FileHandle>, PathBuf)> {
        let mut path_buf = path.to_path_buf();

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
                // SAFETY: file_name will always be valid
                let file_name = unsafe { Path::new_unchecked(std::str::from_utf8_unchecked(&file_name[..temp_end])) };
                path_buf.push(file_name)
            }
        }
        
        let mut win32_access = FILE_ACCESS_RIGHTS(0);
        if access_perms.contains(Permission::Read) {
            win32_access |= FILE_GENERIC_READ;
        }
        if access_perms.contains(Permission::Write) {
            win32_access |= FILE_GENERIC_WRITE;
        } else if access_perms.contains(Permission::Append) {
            win32_access |= FILE_APPEND_DATA;
        }
        if access_perms.contains(Permission::Execute) {
            win32_access |= FILE_GENERIC_EXECUTE;
        }
        if access_perms.contains(Permission::Delete) {
            win32_access |= DELETE;
        }
    
        let mut win32_access_share = 0;
        if shared_access_perms.contains(Permission::Read) {
            win32_access_share |= FILE_SHARE_READ.0;
        }
        if shared_access_perms.contains(Permission::Write) || shared_access_perms.contains(Permission::Append) {
            win32_access_share |= FILE_SHARE_WRITE.0;
        }
        // Do this is an assert, as the user should never pass Execture here
        assert!(shared_access_perms & Permission::Execute != Permission::Execute, "Cannot share file execute permission");

        let win32_create_disposition = match open_mode {
            OpenMode::OpenOrCreate      => OPEN_ALWAYS,
            OpenMode::OpenExisting      => OPEN_EXISTING,
            OpenMode::CreateNonExisting => CREATE_NEW,
            OpenMode::CreateAlways      => CREATE_ALWAYS,
            OpenMode::TruncateExisting  => TRUNCATE_EXISTING,
        };

        let mut win32_flags = 0;
        if create_flags.contains(FileCreateFlags::ReadOnly)         { win32_flags |= FILE_ATTRIBUTE_READONLY.0; }
        if create_flags.contains(FileCreateFlags::Hidden)           { win32_flags |= FILE_ATTRIBUTE_HIDDEN.0; }
        if create_flags.contains(FileCreateFlags::AllowBackup)      { win32_flags |= FILE_ATTRIBUTE_ARCHIVE.0; }
        if create_flags.contains(FileCreateFlags::Encrypted)        { win32_flags |= FILE_ATTRIBUTE_ENCRYPTED.0; }
        if create_flags.contains(FileCreateFlags::DeleteOnClose)    { win32_flags |= FILE_FLAG_DELETE_ON_CLOSE.0; }
        if access_flags.contains(FileAccessFlags::NoBuffering)      { win32_flags |= FILE_FLAG_NO_BUFFERING.0; }
        if access_flags.contains(FileAccessFlags::SupportAsync)     { win32_flags |= FILE_FLAG_OVERLAPPED.0; }
        if access_flags.contains(FileAccessFlags::RandomAccess)     { win32_flags |= FILE_FLAG_RANDOM_ACCESS.0; }
        if access_flags.contains(FileAccessFlags::SequentialAccess) { win32_flags |= FILE_FLAG_SEQUENTIAL_SCAN.0; }
        if access_flags.contains(FileAccessFlags::WriteThrough)     { win32_flags |= FILE_FLAG_WRITE_THROUGH.0; }
        if create_flags.contains(FileCreateFlags::AllowBackup)      { win32_flags |= FILE_FLAG_BACKUP_SEMANTICS.0; }

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
            Ok(handle) => Ok((Box::new(FileHandle(handle)), path_buf)),
            Err(err) => Err(io::Error::from_raw_os_error(err.code().0))
        }
    }

    fn win32_seek(&mut self, dist: i64, method: SET_FILE_POINTER_MOVE_METHOD) -> io::Result<u64> {
        let mut cursor_pos = 0;
        unsafe { SetFilePointerEx(self.0, dist, Some(&mut cursor_pos), method) }
            .map_or_else(|err| Err(io::Error::from_raw_os_error(err.code().0)), |_| Ok(cursor_pos as u64))
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

//--------------------------------------------------------------

/// Windoows async IO completion state
#[derive(Clone, Copy)]
enum AsyncIOCompletionState {
    /// Async operation is still in flight
    InFlight,
    /// Async operation has completed successfully
    Completed(u64),
    /// Async operation has completed with an error
    Unsuccessful(u32),
    /// Async opeartion has completed successfully, but the buffer is already returned
    Exhausted,
}

unsafe extern "system" fn io_completion_callback(error_code: u32, bytes_transfered: u32, overlapped: *mut OVERLAPPED) {
    let completion_data : &mut AsyncIOCompletionData = mem::transmute((*overlapped).hEvent);
    let mut state = completion_data.state.lock();
    if error_code == ERROR_SUCCESS.0 {
        *state = AsyncIOCompletionState::Completed(bytes_transfered as u64);
    } else {
        *state = AsyncIOCompletionState::Unsuccessful(error_code);
    }
}

struct AsyncIOCompletionData {
    state : Mutex<AsyncIOCompletionState>,
}

impl AsyncIOCompletionData {
    fn new() -> AsyncIOCompletionData {
        AsyncIOCompletionData { state: Mutex::new(AsyncIOCompletionState::InFlight) }
    }
}

pub(crate) struct AsyncReadResult {
    file_handle     : HANDLE, 
    buffer          : Vec<u8>,
    overlapped      : Box<OVERLAPPED>,
    completion_data : Box<AsyncIOCompletionData>,
}

impl io::AsyncIOResult for AsyncReadResult {
    type Output = io::Result<Vec<u8>>;

    fn poll(&mut self) -> core::task::Poll<Self::Output> {
        let state = *self.completion_data.state.lock();
        match state {
            AsyncIOCompletionState::InFlight              => Poll::Pending,
            AsyncIOCompletionState::Completed(bytes_read) => Poll::Ready(Ok(self.take_buffer_and_exhaust(bytes_read))),
            AsyncIOCompletionState::Unsuccessful(err)     => Poll::Ready(Err(io::Error::from_raw_os_error(err as i32))),
            AsyncIOCompletionState::Exhausted             => Poll::Ready(Err(io::Error::new(io::ErrorKind::Other, "Data was already taken from this result")))
        }
    }

    fn wait(&mut self, timeout: u32) -> Poll<Self::Output> {
        const SUCCESS: WAIT_EVENT = WAIT_EVENT(ERROR_SUCCESS.0);
        const TIMEOUT: WAIT_EVENT = WAIT_EVENT(ERROR_TIMEOUT.0);

        match unsafe { WaitForSingleObjectEx(self.file_handle, timeout, BOOL(1)) } {
            SUCCESS |
            TIMEOUT => {
                let state = *self.completion_data.state.lock();
                match state {
                    AsyncIOCompletionState::InFlight              => Poll::Pending,
                    AsyncIOCompletionState::Completed(bytes_read) => Poll::Ready(Ok(self.take_buffer_and_exhaust(bytes_read))),
                    AsyncIOCompletionState::Unsuccessful(err)     => Poll::Ready(Err(io::Error::from_raw_os_error(err as i32))),
                    AsyncIOCompletionState::Exhausted             => Poll::Ready(Err(io::Error::new(io::ErrorKind::Other, "Data was already taken from this result")))
                }
            },
            res => Poll::Ready(Err(io::Error::from_raw_os_error(res.0 as i32))),
        }
    }

    fn cancel(&mut self) -> io::Result<()> {
        unsafe { CancelIoEx(self.file_handle, Some(&*self.overlapped)) }
            .map_err(|err| io::Error::from_raw_os_error(err.code().0))
    }
}

impl AsyncReadResult {
    fn take_buffer_and_exhaust(&mut self, bytes_read: u64) -> Vec<u8> {
        *self.completion_data.state.lock() = AsyncIOCompletionState::Exhausted;
        let mut buffer = mem::take(&mut self.buffer);
        unsafe { buffer.set_len(bytes_read as usize) };
        buffer
    }
}

pub(crate) struct AsyncWriteResult {
    file_handle : HANDLE,
    _buffer     : Vec<u8>,
    overlapped  : Box<OVERLAPPED>,
    completion_data : Box<AsyncIOCompletionData>,
}

impl io::AsyncIOResult for AsyncWriteResult {
    type Output = io::Result<u64>;

    fn poll(&mut self) -> core::task::Poll<Self::Output> {
        match *self.completion_data.state.lock() {
            AsyncIOCompletionState::InFlight              => Poll::Pending, 
            AsyncIOCompletionState::Completed(bytes_read) => Poll::Ready(Ok(bytes_read)),
            AsyncIOCompletionState::Unsuccessful(err)     => Poll::Ready(Err(io::Error::from_raw_os_error(err as i32))),
            AsyncIOCompletionState::Exhausted             => Poll::Ready(Err(io::Error::new(io::ErrorKind::Other, "Data was already taken from this result")))
        }
    }

    fn wait(&mut self, timeout: u32) -> Poll<Self::Output> {
        const SUCCESS: WAIT_EVENT = WAIT_EVENT(ERROR_SUCCESS.0);
        const TIMEOUT: WAIT_EVENT = WAIT_EVENT(ERROR_TIMEOUT.0);

        match unsafe { WaitForSingleObjectEx(self.file_handle, timeout, BOOL(1)) } {
            SUCCESS |
            TIMEOUT => {
                match *self.completion_data.state.lock() {
                    AsyncIOCompletionState::InFlight              => Poll::Pending,
                    AsyncIOCompletionState::Completed(bytes_read) => Poll::Ready(Ok(bytes_read)),
                    AsyncIOCompletionState::Unsuccessful(err)     => Poll::Ready(Err(io::Error::from_raw_os_error(err as i32))),
                    AsyncIOCompletionState::Exhausted             => Poll::Ready(Err(io::Error::new(io::ErrorKind::Other, "Data was already taken from this result")))
                }
            },
            res => Poll::Ready(Err(io::Error::from_raw_os_error(res.0 as i32))),
        }
    }

    fn cancel(&mut self) -> io::Result<()> {
        unsafe { CancelIoEx(self.file_handle, Some(&*self.overlapped)) }
            .map_err(|err| io::Error::from_raw_os_error(err.code().0))
    }
}

