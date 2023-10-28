use std::{
    task::Poll,
    mem,
};

use onca_core::io::{SeekFrom, self};
use windows::Win32::{
    Storage::FileSystem::{ReadFileEx, WriteFileEx},
    System::{
        IO::{OVERLAPPED, CancelIo, CancelIoEx},
        Threading::WaitForSingleObjectEx
    },
    Foundation::{ERROR_SUCCESS, HANDLE, BOOL, ERROR_TIMEOUT, WAIT_EVENT}
};

use super::file::FileHandle;

/// Windoows async IO completion state
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
    buffer          : Vec<u8>,
    overlapped      : Box<OVERLAPPED>,
    completion_data : Box<AsyncIOCompletionData>,
}

impl AsyncReadResult {
    pub fn poll(&mut self, cx: &mut core::task::Context<'_>) -> Poll<io::Result<Vec<u8>>> {
        match self.completion_data.state {
            AsyncIOCompletionState::InFlight => {
                self.completion_data.waker = Some(cx.waker().clone());
                Poll::Pending
            } 
            AsyncIOCompletionState::Completed(bytes_read) => Poll::Ready(Ok(self.take_buffer_and_exhaust(bytes_read))),
            AsyncIOCompletionState::Unsuccessful(err) => Poll::Ready(Err(io::Error::from_raw_os_error(err as i32))),
            AsyncIOCompletionState::Exhausted => Poll::Ready(Err(io::Error::new(io::ErrorKind::Other, "Data was already taken from this result")))
        }
    }

    

    pub fn wait(&mut self, timeout: u32) -> Poll<io::Result<Vec<u8>>> {
        const SUCCESS: WAIT_EVENT = WAIT_EVENT(ERROR_SUCCESS.0);
        const TIMEOUT: WAIT_EVENT = WAIT_EVENT(ERROR_TIMEOUT.0);

        match unsafe { WaitForSingleObjectEx(self.file_handle, timeout, BOOL(1)) } {
            SUCCESS |
            TIMEOUT => {
                match self.completion_data.state {
                    AsyncIOCompletionState::InFlight => Poll::Pending,
                    AsyncIOCompletionState::Completed(bytes_read) => Poll::Ready(Ok(self.take_buffer_and_exhaust(bytes_read))),
                    AsyncIOCompletionState::Unsuccessful(err) => Poll::Ready(Err(io::Error::from_raw_os_error(err as i32))),
                    AsyncIOCompletionState::Exhausted => Poll::Ready(Err(io::Error::new(io::ErrorKind::Other, "Data was already taken from this result")))
                }
            },
            res => Poll::Ready(Err(io::Error::from_raw_os_error(res.0 as i32))),
        }
    }

    pub fn cancel(&mut self) -> io::Result<()> {
        unsafe { CancelIoEx(self.file_handle, Some(&*self.overlapped)) }
            .map_err(|err| io::Error::from_raw_os_error(err.code().0))
    }

    fn take_buffer_and_exhaust(&mut self, bytes_read: u64) -> Vec<u8> {
        self.completion_data.state = AsyncIOCompletionState::Exhausted;
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

impl AsyncWriteResult {
    pub fn poll(&mut self, cx: &mut core::task::Context<'_>) -> Poll<io::Result<u64>> {
        match self.completion_data.state {
            AsyncIOCompletionState::InFlight => {
                self.completion_data.waker = Some(cx.waker().clone());
                Poll::Pending
            } 
            AsyncIOCompletionState::Completed(bytes_read) => Poll::Ready(Ok(bytes_read)),
            AsyncIOCompletionState::Unsuccessful(err) => Poll::Ready(Err(io::Error::from_raw_os_error(err as i32))),
            AsyncIOCompletionState::Exhausted => Poll::Ready(Err(io::Error::new(io::ErrorKind::Other, "Data was already taken from this result")))
        }
    }

    pub fn wait(&mut self, timeout: u32) -> Poll<io::Result<u64>> {
        const SUCCESS: WAIT_EVENT = WAIT_EVENT(ERROR_SUCCESS.0);
        const TIMEOUT: WAIT_EVENT = WAIT_EVENT(ERROR_TIMEOUT.0);

        match unsafe { WaitForSingleObjectEx(self.file_handle, timeout, BOOL(1)) } {
            SUCCESS |
            TIMEOUT => {
                match self.completion_data.state {
                    AsyncIOCompletionState::InFlight => Poll::Pending,
                    AsyncIOCompletionState::Completed(bytes_read) => Poll::Ready(Ok(bytes_read)),
                    AsyncIOCompletionState::Unsuccessful(err) => Poll::Ready(Err(io::Error::from_raw_os_error(err as i32))),
                    AsyncIOCompletionState::Exhausted => Poll::Ready(Err(io::Error::new(io::ErrorKind::Other, "Data was already taken from this result")))
                }
            },
            res => Poll::Ready(Err(io::Error::from_raw_os_error(res.0 as i32))),
        }
    }

    pub fn cancel(&mut self) -> io::Result<()> {
        unsafe { CancelIoEx(self.file_handle, Some(&*self.overlapped)) }
            .map_err(|err| io::Error::from_raw_os_error(err.code().0))
    }
}

impl FileHandle {
    pub(crate) fn read_async(&mut self, bytes_to_read: u64) -> io::Result<AsyncReadResult> {
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

        Ok(AsyncReadResult{
            file_handle: self.0,
            buffer,
            overlapped,
            completion_data
        })
    }

    pub(crate) fn write_async(&mut self, buffer: Vec<u8>) -> io::Result<AsyncWriteResult> {
        let cursor_pos = self.seek(SeekFrom::Current(0))?;

        let mut overlapped = Box::new(OVERLAPPED::default());
        overlapped.Anonymous.Anonymous.Offset = cursor_pos as u32;
        overlapped.Anonymous.Anonymous.OffsetHigh = (cursor_pos >> 32) as u32;

        let completion_data = Box::new(AsyncIOCompletionData::new());
        overlapped.hEvent = unsafe { mem::transmute(&*completion_data) };

        unsafe { WriteFileEx(
            self.0,
            Some(&buffer),
            &mut *overlapped,
            Some(io_completion_callback)
        ) }.map_err(|err| io::Error::from_raw_os_error(err.code().0))?;

        Ok(AsyncWriteResult{
            file_handle: self.0,
            _buffer: buffer,
            overlapped,
            completion_data
        })
    }


    pub(crate) fn cancel_all_async_io(&mut self) -> io::Result<()> {
        unsafe { CancelIoEx(self.0, None) }.map_err(|err| io::Error::from_raw_os_error(err.code().0))
    }

    pub(crate) fn cancel_all_thread_async_io(&mut self) -> io::Result<()> {
        unsafe { CancelIo(self.0) }.map_err(|err| io::Error::from_raw_os_error(err.code().0))
    }
}