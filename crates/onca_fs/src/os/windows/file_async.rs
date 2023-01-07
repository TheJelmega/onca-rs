use core::{
    ffi::c_void,
    task::Poll,
    mem,
};

use onca_core::{
    prelude::*,
    io::{SeekFrom, self},
    mem::HeapPtr,
    alloc::ScopedMemTag,
};
use windows::Win32::{
    Storage::FileSystem::{ReadFileEx, WriteFileEx},
    System::{
        IO::{OVERLAPPED, CancelIo, CancelIoEx},
        Threading::WaitForSingleObjectEx
    },
    Foundation::{ERROR_SUCCESS, HANDLE, BOOL, ERROR_TIMEOUT}
};

use crate::FsMemTag;

use super::file::FileHandle;

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

unsafe extern "system" fn io_completion_callback(error_code: u32, bytes_transfered: u32, overlapped : *mut OVERLAPPED) {
    let completion_data : &mut AsyncIOCompletionData = mem::transmute((*overlapped).hEvent);
    if error_code == ERROR_SUCCESS.0 {
        completion_data.state = AsyncIOCompletionState::Completed(bytes_transfered as u64);
    } else {
        completion_data.state = AsyncIOCompletionState::Unsuccessful(error_code);
    }

    if let Some(waker) = completion_data.waker.take() {
        waker.wake();
    }
}

struct AsyncIOCompletionData {
    state : AsyncIOCompletionState,
    waker : Option<core::task::Waker>
}

impl AsyncIOCompletionData {
    fn new() -> AsyncIOCompletionData {
        AsyncIOCompletionData { state: AsyncIOCompletionState::InFlight, waker: None }
    }
}

pub(crate) struct AsyncReadResult {
    file_handle     : HANDLE, 
    buffer          : DynArray<u8>,
    overlapped      : HeapPtr<OVERLAPPED>,
    completion_data : HeapPtr<AsyncIOCompletionData>,
}

impl AsyncReadResult {
    pub fn poll(&mut self, cx: &mut core::task::Context<'_>) -> Poll<io::Result<DynArray<u8>>> {
        match self.completion_data.state {
            AsyncIOCompletionState::InFlight => {
                self.completion_data.waker = Some(cx.waker().clone());
                Poll::Pending
            } 
            AsyncIOCompletionState::Completed(bytes_read) => Poll::Ready(Ok(self.take_buffer_and_exhaust(bytes_read))),
            AsyncIOCompletionState::Unsuccessful(err) => Poll::Ready(Err(io::Error::from_raw_os_error(err as i32))),
            AsyncIOCompletionState::Exhausted => Poll::Ready(Err(io::const_io_error!(io::ErrorKind::Other, "Data was already taken from this result")))
        }
    }

    pub fn wait(&mut self, timeout: u32) -> Poll<io::Result<DynArray<u8>>> {
        unsafe {
            let res = WaitForSingleObjectEx(self.file_handle, timeout, BOOL(1));
            if res == ERROR_SUCCESS || res == ERROR_TIMEOUT {
                match self.completion_data.state {
                    AsyncIOCompletionState::InFlight => Poll::Pending,
                    AsyncIOCompletionState::Completed(bytes_read) => Poll::Ready(Ok(self.take_buffer_and_exhaust(bytes_read))),
                    AsyncIOCompletionState::Unsuccessful(err) => Poll::Ready(Err(io::Error::from_raw_os_error(err as i32))),
                    AsyncIOCompletionState::Exhausted => Poll::Ready(Err(io::const_io_error!(io::ErrorKind::Other, "Data was already taken from this result")))
                }
            } else {
                Poll::Ready(Err(io::Error::from_raw_os_error(res.0 as i32)))
            }
        }
    }

    pub fn cancel(&mut self) -> io::Result<()> {
        unsafe {
            let res = CancelIoEx(self.file_handle, Some(self.overlapped.ptr()));
            if res.as_bool() {
                Ok(())
            } else {
                Err(io::Error::last_os_error())
            }
        }
    }

    fn take_buffer_and_exhaust(&mut self, bytes_read: u64) -> DynArray<u8> {
        self.completion_data.state = AsyncIOCompletionState::Exhausted;
        let mut buffer = mem::take(&mut self.buffer);
        unsafe { buffer.set_len(bytes_read as usize) };
        buffer
    }
}

pub(crate) struct AsyncWriteResult {
    file_handle : HANDLE,
    #[allow(dead_code)] // we need to keep the buffer alive until this operation is finished, but the compiler complains that it's not read
    buffer      : DynArray<u8>,
    overlapped  : HeapPtr<OVERLAPPED>,
    completion_data : HeapPtr<AsyncIOCompletionData>,
}

impl AsyncWriteResult {
    pub fn poll(&mut self, cx: &mut core::task::Context<'_>) -> Poll<io::Result<u64>> {
        match self.completion_data.state {
            AsyncIOCompletionState::InFlight => {
                self.completion_data.waker = Some(cx.waker().clone());
                Poll::Pending
            } 
            AsyncIOCompletionState::Completed(bytes_read) => Poll::Ready(Ok(bytes_read)),
            AsyncIOCompletionState::Unsuccessful(err) => Poll::Ready(Err(io::Error::from_raw_os_error(err as i32))),
            AsyncIOCompletionState::Exhausted => Poll::Ready(Err(io::const_io_error!(io::ErrorKind::Other, "Data was already taken from this result")))
        }
    }

    pub fn wait(&mut self, timeout: u32) -> Poll<io::Result<u64>> {
        unsafe {
            let res = WaitForSingleObjectEx(self.file_handle, timeout, BOOL(1));
            if res == ERROR_SUCCESS || res == ERROR_TIMEOUT {
                match self.completion_data.state {
                    AsyncIOCompletionState::InFlight => Poll::Pending,
                    AsyncIOCompletionState::Completed(bytes_read) => Poll::Ready(Ok(bytes_read)),
                    AsyncIOCompletionState::Unsuccessful(err) => Poll::Ready(Err(io::Error::from_raw_os_error(err as i32))),
                    AsyncIOCompletionState::Exhausted => Poll::Ready(Err(io::const_io_error!(io::ErrorKind::Other, "Data was already taken from this result")))
                }
            } else {
                Poll::Ready(Err(io::Error::from_raw_os_error(res.0 as i32)))
            }
        }
    }

    pub fn cancel(&mut self) -> io::Result<()> {
        unsafe {
            let res = CancelIoEx(self.file_handle, Some(self.overlapped.ptr()));
            if res.as_bool() {
                Ok(())
            } else {
                Err(io::Error::last_os_error())
            }
        }
    }
}

impl FileHandle {
    pub(crate) fn read_async(&mut self, bytes_to_read: u64) -> io::Result<AsyncReadResult> {
        unsafe {
            let _scope_mem_tag = ScopedMemTag::new(FsMemTag::asynchronous());

            let cursor_pos = self.seek(SeekFrom::Current(0))?;

            let mut overlapped = HeapPtr::new(OVERLAPPED::default());
            overlapped.Anonymous.Anonymous.Offset = cursor_pos as u32;
            overlapped.Anonymous.Anonymous.OffsetHigh = (cursor_pos >> 32) as u32;

            let completion_data = HeapPtr::new(AsyncIOCompletionData::new());
            overlapped.hEvent = core::mem::transmute(completion_data.ptr());

            let mut buffer = DynArray::with_capacity(bytes_to_read as usize);
            buffer.set_len(bytes_to_read as usize);
            let res = ReadFileEx(
                self.0,
                Some(buffer.as_mut_ptr() as *mut c_void),
                bytes_to_read as u32,
                overlapped.ptr_mut(),
                Some(io_completion_callback)
            );
            if !res.as_bool() {
                return Err(io::Error::last_os_error());
            }

            Ok(AsyncReadResult{
                file_handle: self.0,
                buffer,
                overlapped,
                completion_data
            })
        }
    }

    pub(crate) fn write_async(&mut self, buffer: DynArray<u8>) -> io::Result<AsyncWriteResult> {
        unsafe {
            let _scope_mem_tag = ScopedMemTag::new(FsMemTag::temporary());

            let cursor_pos = self.seek(SeekFrom::Current(0))?;

            let mut overlapped = HeapPtr::new(OVERLAPPED::default());
            overlapped.Anonymous.Anonymous.Offset = cursor_pos as u32;
            overlapped.Anonymous.Anonymous.OffsetHigh = (cursor_pos >> 32) as u32;

            let completion_data = HeapPtr::new(AsyncIOCompletionData::new());
            overlapped.hEvent = core::mem::transmute(completion_data.ptr());

            let bytes_to_write = buffer.len() as u32;
            let res = WriteFileEx(
                self.0,
                Some(buffer.as_ptr() as *const c_void),
                bytes_to_write,
                overlapped.ptr_mut(),
                Some(io_completion_callback)
            );
            if !res.as_bool() {
                return Err(io::Error::last_os_error());
            }

            Ok(AsyncWriteResult{
                file_handle: self.0,
                buffer,
                overlapped,
                completion_data
            })
        }
    }


    pub(crate) fn cancel_all_async_io(&mut self) -> io::Result<()> {
        unsafe {
            let res = CancelIoEx(self.0, None);
            if res.as_bool() {
                Ok(())
            } else {
                Err(io::Error::last_os_error())
            }
        }
    }

    pub(crate) fn cancel_all_thread_async_io(&mut self) -> io::Result<()> {
        unsafe {
            let res = CancelIo(self.0);
            if res.as_bool() {
                Ok(())
            } else {
                Err(io::Error::last_os_error())
            }
        }
    }
}