use core::{
    future::Future,
    task::Poll,
};

use crate::{alloc::UseAlloc, prelude::DynArray};

use super::Result;


/// An asynchronous read result
pub trait AsyncReadResult : Future {
    /// Wait until the asynchronous read has been completed or the timeout was reached
    /// 
    /// If the timeout was reached, the function will return `Poll::Pending`
    fn wait(&mut self, timeout: u32) -> Poll<Result<DynArray<u8>>>;

    /// Cancel the current asynchronous read
    fn cancel(&mut self) -> Result<()>;
}

/// An asynchronous write result
pub trait AsyncWriteResult : Future {

    fn wait(&mut self, timeout: u32) -> Poll<Result<u64>>;

    /// Cancel the current asynchronous write
    /// 
    /// Cancelling an asynchrounous read will leave the contents of the file in an unknown state
    fn cancel(&mut self) -> Result<()>;
}

/// The `AsyncRead` trait allows for asynchornously reading bytes from a source.
/// 
/// Implementors of the `AsyncRead` trait are called `readers`.
/// 
/// Readers are defined by one required method [`read_async()`].
pub trait AsyncRead {
    /// Type representing the `Future` returned by an asynchronous read operation
    type AsyncResult : AsyncReadResult;

    /// Pull some bytes from this source into the specified buffer, returning a future that will fill in the buffer
    /// 
    /// This function does not block waiting for data.
    /// 
    /// If the return of the `AsyncResult` is [`Ok(buf)`], the implementations must guarantee that `0 <= buf.len() <= bytes_to_read`.
    /// If 'buf.len()` is '0', then it can indicate on of two scenarios:
    /// 
    /// 1. The reader has reached its "end of file" and will likely no longer be able to produce bytes from any location after it.
    ///    Note that this does not mean tha the reader will *alwyas* no longer be able to produce bytes starting from this location.
    ///    As an example, for a file, it is possible to reach  the end of file and get a zero as result, but if more data is appended to the file, future calls to `read_async` at this location will return more data
    /// 2. The `bytes_to_read` specified was 0
    /// 
    /// It is not an error if the returned `buf.len()` is smaller than `bytes_to_read`, even when the reader is not at the end of the stream yet.
    /// This may happen for example because fewer bytes are actually available right now (e.g. being close to end-of-file) or because read_async() was interrupted or cancelled.
    /// 
    /// # Errors
    /// 
    /// If an error is returned, then it must be guaranteed that no bytes were read.
    /// 
    /// If the function is unable to create a future, an error is returned.
    fn read_async(&mut self, bytes_to_read: u64, alloc: UseAlloc) -> Result<Self::AsyncResult>;
}

pub trait AsyncWrite {
    /// Type representing the `Future` returned by an asynchronous write operation
    type AsyncResult : AsyncWriteResult;

    /// Write a buffe into this writer, returning a future that will write out the buffer.
    /// 
    /// This function does not block waiting for data to be written.
    /// 
    /// The `AsyncResult` returned will attempt to write the entire contents of `bug`, bu the entire write might not succeed, or the write may also generate an error.
    /// A call to `write_async` represents *at most one* attempt to wirte to any wrapped object.
    /// 
    /// If the return of the `AsyncResult` is [`Ok(n)`], then it must be guaranteed that `n < buf.len()`.
    /// A return value of `0` typically means that the underlying object is no longer able to accept bytes and will likely not be able to in the future as well, or that the buffer provided is emtpy.
    /// 
    /// # Errors
    /// 
    /// It is **not** considered an error if the entire buffer could not be written to this writer.
    /// 
    /// An error of the [`ErrorKind::Interrupted`] kind is non-fatal and the write opeartion should be retired if there is nothing else to do.
    /// 
    /// If the function is unable to create a future, an error is returned.
    fn write_async(&mut self, buf: DynArray<u8>, alloc: UseAlloc) -> Result<Self::AsyncResult>;
}
