//! Traits, helpers, and type definition for core I/O functionality.
//! 
//! A lot of functionality is similar to `std::io`, but with support for Onca types.
//! 
//! The `onca_core::io` module contains a number of common things to use when doing input and output.
//! The most core part of this module would be the [`Read`] and [`Write`] taits, which provide a generic interface for reading input and writing output.
//! 
//! # Read and Write
//! 
//! Because they are traits, [`Read`] and [`Write`] are implemented by a number of other types, and can be implemented for other types too.
//! 
//! [`Read`] and [`Write`] are so important, implementors of the two traits have a nickname: readers and writers.
//! So sometimes `a reader` is mentioned instead of `a type that implements the [`Read`] trait`.
//! 
//! ## Seek and BufRead
//! 
//! Beyond that, tehre are two imponent traits that are provided: [`Seek`] and [`BufRead`].
//! Both of these build on top of a reader to control how the reading happens. 
//! [`Seek`] gives control where the next byte is coming from.
//! 
//! ## BufReader and BufWriter
//! 
//! Byte-based interfaces are unwieldy and can be inefficient, as it needs to be making near-constant calls to the operating system.
//! To help with this, `onca_core::io` comes with two structs, [`BufReader`] and [`BufWriter`], which wrap readers and writers.
//! The wrapper uses a buffer, reducing the number of call and providing nicer methods for accessing exactly what's needed.
//! 
//! [`BufWriter`] doesn't add any new ways of writing; it just buffers every call to [`write`][`Write::write`].
//! 
//! ## Iterator types
//! 
//! A large number of structures provided by `onca_core::io` are for varieous ways of iterating over I/O.
//! For example, [`Lines`] is used to split over lines
//! 
//! ## io::Result
//! 
//! This type is used as the return type of many `onca_core::io` functions that can cause an error, and can be returned from other functions as wel.
//! 
//! [`io::Result<()>`][`io::Result`] is a very comomn type for functions which don't have a 'real' return value, but do want to return erros if they happen.
//! 
//! ## Additional info
//! 
//! Unlike `std::io`, `onca_core` does not provide the facilities for reading from the standard input or writing to either the standard output or error.
//! For this functionality, see `onca_??::??`

use crate::{
    alloc::{UseAlloc, MemTag, ScopedAlloc, ScopedMemTag, get_active_mem_tag, get_active_alloc},
    collections::DynArray, 
    mem::HeapPtr,
    strings::String, 
    KiB, 
};

use core::{
    cmp,
    fmt,
    mem,
    slice,
    str,
    ops::{Deref, DerefMut},
    slice::memchr,
};
use std::convert::TryInto;

pub use self::{
    buffered::{BufReader, BufWriter, IntoInnerError},
    copy::copy,
    cursor::Cursor,
    error::{Error, ErrorKind, Result, SimpleMessage},
    readbuf::{BorrowedBuf, BorrowedCursor},
    utils::{emtpy, repeat, sink, Empty, Repeat, Sink},
    async_io::{AsyncReadResult, AsyncRead, AsyncWriteResult, AsyncWrite},
};

pub mod prelude;

pub(crate) mod copy;

mod cursor;
mod error;
mod impls;
mod readbuf;
mod buffered;
mod utils;
mod async_io;

pub use error::const_io_error;

const DEFAULT_BUF_SIZE : usize = KiB(8);

struct Guard<'a> {
    buf : &'a mut DynArray<u8>,
    len : usize
}

impl Drop for Guard<'_> {
    fn drop(&mut self) {
        unsafe { self.buf.set_len(self.len) }
    }
}

// Several `read_to_string` and `read_line` methods in the libarary will append data into a `String` buffer, but we need to pretty careful when doing this.
// The implementation will just call '.as_mut_dynarr()` and then delegate to a byte-oriented reading method, 
// but we must ensure that when returning we never leave `buf` in a state such that it contains invalid UTF-8 in its bounds
//
// To this end, we use an RAII guard (to protect against panics) which updates the lenghts of the string when it is dropped.
// This guead initially truncates the string to the prior length and only after we've validated that the new contents are valid UTF-8 do we alllow it to set a longer length.
//
// The unsafety in this function is twofold:
//
// 1. We're looking at the raw bytes of `bug`, so we take on the burden of UTF-8 checks.
// 2. We're passing a raw buffer to the function `f`, and it is expected that the function only *appends* bytes to the buffer.
//    We'll get undefined behavior if existing bytes are overwritten to have non-UTF-8 data
pub(crate) unsafe fn append_to_string<F>(buf: &mut String, f: F) -> Result<usize> 
where
    F : FnOnce(&mut DynArray<u8>) -> Result<usize>,
{
    let mut g = Guard{ len: buf.len(), buf: buf.as_mut_dynarr() };
    let ret = f(g.buf);
    if str::from_utf8(&g.buf[g.len..]).is_err() {
        ret.and_then(|_| {
            Err(const_io_error!(
                ErrorKind::InvalidData,
                "strim did not contain valid UTF-8"
            ))
        })
    } else {
        g.len = g.buf.len();
        ret
    }
}

// This uses an adaptive system to extend the vector when it fills. We want to avoid paying to allocate and zero a huge chunk of memory if the reader only has 4 bytes,
// while still making large reads if the reader does have a ton of data to return.
// Simple tacking on an extra DEFAULT_BUF_SIZE space every time is 4,500 times (!) slower than a default reservation size of 32 if the reader has a very small amount of data to return

pub(crate) fn default_read_to_end<R: Read + ?Sized>(r: &mut R, buf: &mut DynArray<u8>) -> Result<usize> {
    let start_len = buf.len();
    let start_cap = buf.capacity();

    let mut initialized = 0; // Extra initialized bytes from previous loop iteration
    loop {
        if buf.len() == buf.capacity() {
            buf.reserve(32); // buf is full, need more space
        }

        let mut read_buf : BorrowedBuf<'_> = buf.spare_capacity_mut().into();

        // SAFETY: THese bytes were initialized but not filled in the previous loop
        unsafe {
            read_buf.set_init(initialized);
        }

        let mut cursor = read_buf.unfilled();
        match r.read_buf(cursor.reborrow()) {
            Ok(()) => {},
            Err(e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => return Err(e)
        }

        if cursor.written() == 0 {
            return Ok(buf.len() - start_len);
        }

        // store how much was initialized but nor filled
        initialized = cursor.init_ref().len();

        // SAFETY: BorrowBuf's invariants mean this much memory is initilized
        unsafe {
            let new_len = read_buf.filled().len() + buf.len();
            buf.set_len(new_len);
        }

        if buf.len() == buf.capacity() && buf.capacity() == start_cap {
            // THe buffer might be an exact fit.
            // Let's read into a probe buffer and see if it returns`Ok(0)`
            // If so, we've avoided an unnecessary doubling of the capacity.
            // Buf if not, append the probe buffer to the primary buffer and let its capacity grow
            let mut probe = [0u8; 32];

            loop {
                match r.read(&mut probe) {
                    Ok(0) => return Ok(buf.len()),
                    Ok(n) =>  {
                        buf.extend_from_slice(&probe[..n]);
                        break;
                    },
                    Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                    Err(e) => return Err(e),
                }
            }
        }
    }
}

pub(crate) fn default_read_to_string<R: Read + ?Sized>(r: &mut R, buf: &mut String) -> Result<usize> {
    // Note taht we do *not* call `r.read_to_end()` here.
    // We are passing `&mut DynArray<u8>` (the raw contents of `buf`) into the `read_to_end` method to fill it up.
    // An arbitrary implementation could overwrite the entire contents of the dynarray, not just append to it (which is what we are expecting).
    //
    // To prevent extraneously checking the UTF-8-ness of the entire buffer we pass it  to our hardcoded `default_read_to_end` implementation which we know is guaranteed to only read data into the end of the buffer.
    unsafe{ append_to_string(buf, |b| default_read_to_end(r, b)) }
}

pub(crate) fn default_read_vectored<F>(read: F, bufs: &mut[IoSliceMut<'_>]) -> Result<usize> 
where
    F : FnOnce(&mut [u8]) -> Result<usize>
{
    let buf = bufs.iter_mut().find(|b| !b.is_empty()).map_or(&mut[][..], |b| &mut **b);
    read(buf)
}

pub(crate) fn default_write_vectored<F>(write: F, bufs: &[IoSlice<'_>]) -> Result<usize>
where
    F : FnOnce(&[u8]) -> Result<usize>
{
    let buf = bufs.iter().find(|b| !b.is_empty()).map_or(&[][..], |b| &**b);
    write(buf)
}

pub(crate) fn default_read_exact<R: Read + ?Sized>(this: &mut R, mut buf: &mut [u8]) -> Result<()> {
    while !buf.is_empty() {
        match this.read(buf) {
            Ok(0) => break,
            Ok(n) => {
                let tmp = buf;
                buf = &mut tmp[n..];
            },
            Err(ref e) if e.kind() == ErrorKind::Interrupted => {},
            Err(e) => return Err(e)
        }
    }
    if !buf.is_empty() {
        Err(error::const_io_error!(ErrorKind::UnexpectedEof, "failed to fill whole buffer"))
    } else {
        Ok(())
    }
}

pub(crate) fn default_read_buf<F>(read: F, mut cursor: BorrowedCursor<'_>) -> Result<()>
where
    F: FnOnce(&mut [u8]) -> Result<usize>
{
    let n = read(cursor.ensure_init().init_mut())?;
    unsafe {
        // SAFETY: we initialized using `ensure_init` so there is no uninit data to advance to
        cursor.advance(n);
    }
    Ok(())
}

/// The `Read` trait allows for reading bytes from a source.
/// 
/// Implementors of the `Read` trait are called 'readers'.
/// 
/// Readers are defined by one required method [`read()`].
/// Each call to [`read())`] will attempt to pull bytes from this source into a provided buffer.
/// A number of other methods are implemented in terms of [`read()`], giving implementors a number of ways to read bytes while only needing to implement a single method.
/// 
/// Readers are intended to be composable with one another. Many implementors throughout [`onca_core::io`] take and provide types which implemtn the `Read` trait.
/// 
/// Please note that each call to [`read()`] may involve a system call, and therefore, using something that implements [`BufRead`], such as [`BufReader`], will be more efficient
/// 
/// [`read()`]: Read::read
/// [onca_core::io]: self
pub trait Read {
    /// Pull some bytes from this source into the specified buffer, returning how many bytes were read.
    /// 
    /// This function does not provide any guarantees about whether it block waiting for data, but if any object needs to block for a read and cannot, it will typically signal this via an [`Err`] return value.
    /// 
    /// If the return value of this method is [`Ok(n)`], then implementations must guarantee that `0 <= n <= buf.len()`.
    /// A nonzero `n` value indicates that the buffer `buf` has been filled in with `n` bytes of data from this source.
    /// If `n` is `0`, then it can indicate one of two senarios:
    /// 
    /// 1. The reader has reached its "end of file" and will likely no longer be able to produce bytes. 
    ///    Note that this does not mean that the reader wiil *always* no longer be able to produce bytes.
    ///    As an example, for a file, it is possible to reach the end of file and get a zero as result, but if more data is appended to the file, future calls to `read` will return more data.
    /// 2. The buffer specified was 0 bytes in length.
    /// 
    /// It is not an error if the returned value `n` is smaller than the buffer size, even when the reader is not at the end of the stream yet.
    /// This may happen for example because fewer bytes are actually available right now (e.g. being close to end-of-file) or because read() was interrupted by a signal.
    /// 
    /// As this trait is safe to implement, callers cannot rely on `n <= buf.len()` for safety.
    /// Extra care needs to be taken when `unsafe` functions are used to access the read bytes.
    /// Callers here to ensure that no unchecked out-of-bounds accesses are possible even if `n > buf.len()`.
    /// 
    /// No guarantees are provided about the contents of `buf` when this function is called, implementations cannot rely on any property of the contents of `buf` being true.
    /// It is recommended that *implementations* only write data to `buf` instead of reading its contents.
    /// 
    /// Corresponding, however, *callers* of this method must not assume any guearantees about how the implementation use `buf`.
    /// The trait is safe to implement, so it is possible that the code that's supposed to write to the buffer might also read from it.
    /// It is your responsibility to make sure that `buf` is initialized before calling `read`.
    /// Calling` read` with an uninitialized `buf` (of the kind one obtains via [`MaybeUninit<T>]) is not safe, and can lead to undefined behavior.
    /// 
    /// [`MaybeUninit`]: core::mem::MaybeUninit
    /// 
    /// # Errors
    /// 
    /// If this function encounters any form of I/O or other error, an error variant will be returned.
    /// If an error is returned, then it must be guaranteed that no bytes were read.
    /// 
    /// An error of  the [`ErrorKind::Interrupted`] kind is non-fatal and the read operation should be retried if there is nothing else to do.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;

    /// Like `read`, except that it reads into a slice of buffers.
    /// 
    /// Data is copied to fill each buffer in order, with the final buffer written to possibly being only partially filled.
    /// This method must behave equivalently to a single call to `read` with concatenated buffers.
    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> Result<usize> {
        default_read_vectored(|b| self.read(b), bufs)
    }

    /// Determines if this `Read`er has an efficient `read_vectored` implementation
    /// 
    /// If a `Read`er does not override the default `read_vectored` implementation, code using it may want to avoid the method all together and coalexce reads into a single buffer for higher performance.
    /// 
    /// The default implementation return `false`
    fn is_read_vectored(&self) -> bool {
        false
    }

    /// Read all bytes until EoF in this source, placing them into `buf`.
    /// 
    /// All bytes read from this source will be appended to the specified buffer `buf`.
    /// This function will continuously call [`read()`] to append more data to `buf` until [`read()`] returns either [`OK(0)`] or an error of a non-[`ErrorKind::Iterrupted`] kind.
    /// 
    /// If successful, this function will return the total number of bytes read
    /// 
    /// # Errors
    /// 
    /// If this function encounters an error of the kind [`ErrorKind::Interrrupted`] , then the error is ignored and the operation will continue.
    /// 
    /// If any other read error is encountered, the this function immediately returns.
    /// Any byte which have already been read will be appended to `buf`
    fn read_to_end(&mut self, buf: &mut DynArray<u8>) -> Result<usize> {
        default_read_to_end(self, buf)
    }

    /// Read all bytes until EoF in this source, appending them to `buf`.
    /// 
    /// If successful, this function returns thenuber of bytes which were read and appended to `buf`
    /// 
    /// # Errors
    /// 
    /// If the data in this stream is not valid UTF-8, then an error is returned and `buf` is unchanged.
    fn read_to_string(&mut self, buf: &mut String) -> Result<usize> {
        default_read_to_string(self, buf)
    }

    /// Read the exact number of bytes required to fill `buf`.
    /// 
    /// This function reads as many bytes as necessary to completely fill the specified buffer `buf`.
    /// 
    /// No guarantees are provided about the contents of `buf` when this function is called, implementations cannot rely on any property of the contents of `buf` being true.
    /// It is recommended that implementations only write data to `buf` instead of reading its constents.
    /// The documentation on [`read`] has a more detailed explanation on this subject.
    /// 
    /// # Errors
    /// 
    /// If this function encounters an error of the kind [`ErrorKind::Interrupted`], then the error is ignored and the operation will continue.
    /// 
    /// If this function encounters an end-of-file before completely filling the buffer, it returns an error of the kind [!ErrorKind::UnexpectedEoF`].
    /// The contents of `buf` are unspecified in this case.
    /// 
    /// If any other read error is encountered, then this function immediately returns.
    /// The contents of `buf` are unspecified in this case.
    /// 
    /// If this function returns an error, it is unspecified how many bytes it has read, but it will never read more that would be necessary to completely fill the buffer
    /// 
    /// [`read`]: Read::read
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        default_read_exact(self, buf)
    }

    /// Pull some bytes from this source into the specified buffer.
    /// 
    /// This is equivalent to the [`read`](Read::read) method, except that it is passed a [`ReadBuf`] rather than `[u8]` to allow use with uninitialized buffers.
    /// The new data will be appended to any existing contents of `buf`.
    /// 
    /// The drefault implementation delegated to `read`
    fn read_buf(&mut self, cursor: BorrowedCursor<'_>) -> Result<()> {
        default_read_buf(|b| self.read(b), cursor)
    }

    /// Read the exact number of bytes required to fill `buf`.
    /// 
    /// This is equivalent to the [`read_exact`](Read::read_exact) method, except that it is passed [`ReadBuf`] rather than `[u8]` to allow use with uninitlaized buffers.
    fn read_buf_exact(&mut self, mut cursor: &mut BorrowedCursor<'_>) -> Result<()> {
        while cursor.capacity() > 0 {
            let prev_written = cursor.written();
            match self.read_buf(cursor.reborrow()) {
                Ok(()) => {},
                Err(e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            }

            if cursor.written() == prev_written {
                return Err(error::const_io_error!(ErrorKind::UnexpectedEof, "failed to fill buffer"));
            }
        }

        Ok(())
    }

    /// Creates a "by reference" adaptor for this instance of `Read`
    /// 
    /// The returned adapter also implements `Read` and will simple borrow this current reader.
    fn by_ref(&mut self) -> &mut Self 
    where
        Self : Sized
    {
        self
    }

    /// Transforms this `Read` instance to an [`Iterator`] over its bytes.
    /// 
    /// The returned type implements [`Iterator`] where the [`Item`] is <code>[Result]<[u8], [io::Error]></code>.
    /// The yeilded item is [`Ok`] if a byte was successfully read and [`Err`] otherwise.
    /// EOF is mapped to returning [`None`] from this iterator
    fn bytes(self) -> Bytes<Self>
    where
        Self : Sized
    {
        Bytes { inner: self }
    }

    /// Creates an adapter which will chain this stream with another.
    /// 
    /// The returned `Read` instance will first read all bytes from this object until EOF is encountered.
    /// Afterwards the output is equivalend to the output of `next`.
    fn chain<R: Read>(self, next: R) -> Chain<Self, R>
    where
        Self : Sized
    {
        Chain { first: self, second: next, done_first: false }
    }

    /// Creates an adapter which will read at most `limit` bytes from it.
    /// 
    /// This function returns a new instance of `Read` which will read at most `limit` bytes, after which it will always return EoF ([`Ok(0`]).
    /// Any read errors will not count towards the number of bytes and future calls to [`read()`] may succeed.
    /// 
    /// [`Ok(0)`]: Ok
    /// [`read()`]: Read::read
    fn take(self, limit: u64) -> Take<Self>
    where
        Self : Sized
    {
        Take { inner: self, limit }
    }
}

/// Read all bytes from a [reader][Read] into a new [`String`].
/// 
/// This is a convenience functiohn for [`Read::read_to_string`].
/// Using this function avoids having to create a variable first and provides more type safety since you can only get the buffer out if there were no errors.
/// (If you use [`Read::read_to_string`] you have to remember to check wehtehr the read succeeded because otherwise your fudder will be emtpy or only partially full).
/// 
/// # Performance
/// 
/// The downside of this function's increased ease of use and type safety is that it gives you less control over perforamance. 
/// For example, you can't pre_allocate memory like you can useing [`String::with_capacity`] and [`Read::read_to_string`].
/// Also, you can't re-use the buffer if an error occurs while reading.
/// 
/// In many cases, this function's performance will be adequate and the ease of use and type safety tradeoffs will be worth it.
/// However, there are cases where you need more control over performance, and in those cases you should definitiely use [`Read::read_to_string`] directly.
/// 
/// Note that in some special cases, such as when reading files, this function will pre
/// jallocate memory based on the size of the input it is reading.
/// In those cases, the performance should be as good as if you had uses [`Read::read_to_string`] with a manually pre-allocated buffer.
/// 
/// # Errors
/// 
/// This function forces you to handle erros because the output (the `String`) is wrapped in a [`Result`]. 
/// See [`Read::read_to_string`] for the erros that can occur.
/// If any error occurs, you will get an [`Err`], so you don't have to worry about your buffer bing empty or partially full.
pub fn read_to_string<R: Read>(mut reader: R) -> Result<String> {
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;
    Ok(buf)
}

/// Read all bytes from a [reader][Read] into a new [`String`] with a pre-allocated capacity.
/// 
/// Please see the documentation of [`read_to_string`] for more details
pub fn read_to_string_with_capacity<R: Read>(mut reader: R, capacity: usize) -> Result<String> {
    let mut buf = String::with_capacity(capacity);
    reader.read_to_string(&mut buf)?;
    Ok(buf)
}

/// A trait for objects which are byte-oriented sinks.
/// 
/// Implementors of the`Write` trait are sometimes called 'writers'.
/// 
/// Writers are defined by two required methods, [`write`] and [`flush`]:
/// 
/// * The [`write`] method will attempt to write some data into the object, returning how many bytes were successfully written
/// * the [`flush`] method is useful for adapters and explicit buffer themselves for ensuring that all buffered data has been pushed out to the `true sink`
/// 
/// Writers are intended to be composable with one another. Many implementors throughout [`onca_core::io`] take and provide types which implement the `Write` trait.
/// 
/// [`write`]: Write::write
/// [`flush`]: Write::flush
/// [`onca_core::io`]: self
pub trait Write {
    /// Write a buffer into this writer, returning how many bytes were written.
    /// 
    /// This function will attempt to write the entire contents of `buf`, but the entire write might not succeed, or the write may also generate an error.
    /// A call to `write` represents *at most one* attempt to write to any wrapped object.
    /// 
    /// Calls to `write` are not guaranteed to block waiting for data to be written, and a write which would otherwise block can be indicated through an [`Err`] variant.
    /// 
    /// If the return value is [`Ok(n)`], then it must be guaranteed that `n <= buf.len()`.
    /// A return value of `0` typically means that the underlying object is no longer able to accept bytes and will likely not be able to in the future as well, or that the buffer provided is empty.
    /// 
    /// # Errors
    /// 
    /// Each call to `write` may generate an I/O error indicating that the operation could not be completed.
    /// If an error is returned then no bytes in the buffer were written to this writer.
    /// 
    /// It is **not** considered an error if the entire buffer could not be written to this writer.
    /// 
    /// An error of the [`ErrorKind::Interrupted`] kind is non-fatal and the write operation should be retired if there is nothing else to do.
    fn write(&mut self, buf: &[u8]) -> Result<usize>;

    /// Like [`write`], except that it writes from a slice of buffers.
    /// 
    /// Data is copied from each buffer in order, with the final buffer read from possibly being only partially consumed.
    /// THis method must behave as a call to [`write`] with the buffers concatenated would.
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> Result<usize> {
        default_write_vectored(|b| self.write(b), bufs)
    }

    /// Determines if this `Write`r has an efficient [`write_vectored`] implementation.
    /// 
    /// If a `Write`r does not override the default [`write_vectored`] implementation, code using it may want to avoid the method all together and coalesce writes into a single buffer for higher performace.
    /// 
    /// The default implemtantion returns `false`.
    /// 
    /// [`write_vectored`]: Write::write_vectored
    fn is_write_vectored(&self) -> bool {
        false
    }

    /// Flush this output stream, ensuring that all intermediately buffered contents reach their destination
    /// 
    /// # Errors
    /// 
    /// It is considered an error if not all bytes could be written due to I/O errors of EoF being reached.
    fn flush(&mut self) -> Result<()>;

    /// Attempts to write an entire buffer into this writer.
    /// 
    /// The method will continuously call [`write`] until there is no more data to be written or an error of a non-[`ErrorKind::Interrupted`] kind is returned.
    /// This method will not reutrn until the entire buffer has been succesfully written or such an erro occurs.
    /// The first error that is not of a ['ErrorKind::Interrupted`] kind generted from this method will be returned.
    /// 
    /// If the buffer contains no data, this will never call [`write`]
    /// 
    /// # Errors
    /// 
    /// This function will return the first error of a non-[`ErrorKind::Interrupted`] kind that [`write`].
    /// 
    /// [`write`]: Write::write
    fn write_all(&mut self, mut buf: &[u8]) -> Result<()> {
        while !buf.is_empty() {
            match self.write(buf) {
                Ok(0) => {
                    return Err(error::const_io_error!(
                        ErrorKind::WriteZero,
                        "failed to write whole buffer"
                    ));
                },
                Ok(n) => buf = &buf[n..],
                Err(ref e) if e.kind() == ErrorKind::Interrupted => {},
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    /// Attempts to write multiple buffers into this writer.
    /// 
    /// This method will continuously call [`write_vectored`] until there is no more data to be written or an error of a non-[`ErrorKind::Interrupted`] kind is returned.
    /// This method will not return until all buffers have been succesfully written or such an error occurs.
    /// The first error that is not of a [`ErrorKind::Interrupted`] kind generated from this method will be returned.
    /// 
    /// If the buffer contains no data, this will never call [`write_vectored`].
    /// 
    /// # Notes
    /// 
    /// Unlike [`write_vectored`], this takes a *mutable* reference to a slice of [`IoSlice`]s, not an immutable one. THat's because we need to modify the slice to keep track of the bytes already written.
    /// 
    /// Once this function returns, the contents of `bufs` are unspecified, as this depends on how many calls to [`write_vectored`] were necessary.
    /// It is best to understand this function as taking ownership of `bufs` and to not use `buds` afterwards.
    /// The underlying buffers, to which the [`IoSlice`]s point (but not the [`IoSlice`]s themselves), are unchanged and can be reused.
    /// 
    /// [`write_vectored`]: Write::write_vectored
    fn write_all_vectored(&mut self, mut bufs: &mut [IoSlice<'_>]) -> Result<()> {
        // Guarantee that bufs is empty if it contains no data, to avoid calling write_vectored if there is no data to be written
        IoSlice::advance_slices(&mut bufs, 0);
        while !bufs.is_empty() {
            match self.write_vectored(bufs) {
                Ok(0) => {
                    return Err(error::const_io_error!(
                        ErrorKind::WriteZero,
                        "failed t write whole buffer"
                    ));
                },
                Ok(n) => IoSlice::advance_slices(&mut bufs, n),
                Err(ref e) if e.kind() == ErrorKind::Interrupted => {},
                Err(e) => return Err(e)
            }
        }
        Ok(())
    }

    /// Writes a formatted string into this writer, returning any errorencountered.
    /// 
    /// This method is primarily used to interface with the [`format_args!()`] macro, and it is rare that this should be explicitly called.
    /// The [`write!()`] macro should be favored to invoke this method instead.
    /// 
    /// This function internally uses the [`write_all`] method on this trait and hence will continuously write data so long as no erros are received.
    /// This also means that partial writes are not indicated in this signature.
    /// 
    /// # Errors
    /// 
    /// This function will return any I/) error reproted while formatting
    /// 
    /// [`write_all`]: Write::write_all
    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> Result<()> {
        // Create a shim which translates a Write t a fmt::Write and saves off I/O errors. instad of discarding them
        struct Adapter<'a, T: ?Sized + 'a> {
            inner : &'a mut T,
            error : Result<()>
        }

        impl<T: Write + ?Sized> fmt::Write for Adapter<'_, T> {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                match self.inner.write_all(s.as_bytes()) {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        self.error = Err(e);
                        Err(fmt::Error)
                    }
                }
            }
        }

        let mut output = Adapter { inner: self, error: Ok(()) };
        match fmt::write(&mut output, fmt) {
            Ok(()) => Ok(()),
            Err(..) => {
                // Check if the error came from the underlying `Write` or not
                if output.error.is_err() {
                    output.error
                } else {
                    Err(error::const_io_error!(ErrorKind::Uncategorized, "formatter error"))
                }
            }
        }
    }

    /// Creates a "by reference" adapter for this instance of `Write`.
    /// 
    /// The returned adapter also implements `Write` and will simply borrow this current writer
    fn by_ref(&mut self) -> &mut Self
    where
        Self : Sized
    {
        self
    }
}

/// Enumeration of possible methods t oseek within an I/O object.
/// 
/// It is used by the [`Seek`] trait.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SeekFrom {
    /// Sets the offst to th provided number of bytes
    Start(u64),

    /// Sets the offset to the size of this object plus the specified number of bytes
    /// 
    /// It is possible to seek beyond the end of an object, but it's an error to seek before byte 0
    End(i64),

    /// Sets the offset to the curent position plus the specified number of bytes.
    /// 
    /// It is possible to seek beyond the end of an object, but it's an error to seek before byte 0.
    Current(i64)
}

/// The `Seek` trait provides a cursor which can be moved within a stream of bytes.
/// 
/// The stream typically has a fixed size, allowing seeking releative to either end or the current offset.
pub trait Seek {
    /// Seek to an offset, in bytes, in a stream.
    /// 
    /// A seek beyond the end of a stream is allowed, but behavior is defined by the implementation.
    /// 
    /// If the seek operation completed successfully, this method returns the new position from the start of the stream.
    /// That position can be used later with [`SeekFrom::Start`].
    /// 
    /// # Errors
    /// 
    /// Seeking can fail, for exapmle because it might involce flushing a buffer.
    /// 
    /// Seeking to a negative offset is considered an error
    fn seek(&mut self, pos: SeekFrom) -> Result<u64>;

    /// Rewind to the beginning of a stream.
    /// 
    /// This is a convenience method, equivalent to `seek(SeekFrom::Start(0))`.
    /// 
    /// # Errors
    /// 
    /// Rewinding can fail, for example because it might involve flushing a buffer.
    fn rewind(&mut self) -> Result<()> {
        self.seek(SeekFrom::Start(0))?;
        Ok(())
    }

    /// Returns the length of this steram (in bytes)
    /// 
    /// This method is implemented using up to three seek operations.
    /// If this method return successfully, the seek position is unchanged (i.e. the position before calling this method is the same as afterwards).
    /// However, if this method returns an error, the seek position is unspecified
    /// 
    /// If you need to obtain the length of *many* streams and you don't care about the seek position afterwards, 
    /// you can recude the number of seek operation by simple calling `seek(SeekFrom::End(0))` and using its return value (it is also the stream length).
    /// 
    /// Note that length of a stream can chagne over time (for example, when data is appended to a file).So calling this method multiple times does not necessarily return the same lengths each time.
    fn stream_len(&mut self) -> Result<u64> {
        let old_pos = self.stream_position()?;
        let len = self.seek(SeekFrom::End(0))?;

        // Avoid seeking a third time when we were already at the end of the stream.
        // The branch is usually way cheapter than a seek operation.
        if old_pos != len {
            self.seek(SeekFrom::Start(old_pos))?;
        }

        Ok(len)
    }

    /// Returns the current seek positon from the start of the stream.
    /// 
    /// This is equivalent to `seek(SeekFrom::Current(0))`.
    fn stream_position(&mut self) -> Result<u64> {
        self.seek(SeekFrom::Current(0))
    }
}

/// A `BufRead` is a type of `Read`er which has an internal buffer, allowing it to perform extra ways of reading.
/// 
/// For example, reading line-by-line is inefficient without using a buffer, so if you want to read by line, you'll need `BufRead`, which includes a [`read_line`] method as well as a [`lines`] iterator
/// 
/// [`read_line`]: BufRead::read_line
/// [`lines`]: BufRead::lines
pub trait BufRead : Read {
    /// Returns the contents of the internal buffer, filling it with more data from the inner reader if it is empty.
    /// 
    /// This function is a lower-level call.
    /// It needs to be paired with the [`consume`] mthod to function properly.
    /// When calling this method, none of the contents will be "read" in the sense that later calling `read` may return the same contents.
    /// As such, [`consume`] must be called with the number of bytes that are consumed from this buffer to ensure that the bytes are never returned twice.
    /// 
    /// An empty buffer reteruned indicsated that the stream has reached EoF.
    /// 
    /// # Errors
    /// 
    /// This function will return an I/O error if the underlying reader was read, but returned an error.
    /// 
    /// [`consume`]: BufRead::consume
    fn fill_buf(&mut self) -> Result<&[u8]>;

    /// Tells this butter taht `amt` bytes have ben consumes from the buffer, so they could no longer be returned in calls to `read`.
    /// 
    /// This function is a lower-level call. It needs to be paired with the [`fill_buf`] method to fundtion properly.
    /// This function deos not perform any I/O, ti simply informs this object that some amount of its buffer, returned from [`fill_buf`], has been consumed and should no longer be returned.
    /// As such, this  function may do odd things if [`fill_buf`] isn't called before calling it.
    /// 
    /// The `amt` must be `<=` the number of bytes in the buffer returned by [`fill_buf`]
    /// 
    /// [`fill_buf`]: BufRead::fill_buf
    fn consume(&mut self, amt: usize);

    /// Check if the underlying `Read` has any data left to be read.
    /// 
    /// This function may fill the buffer to check for data, so this function returns `Result<bool>`, not `bool`.
    /// 
    /// Default implemtnation calls `fill_buf` and checks that returend slice is empty (which means that there is not data left, since EoF has been reached).
    fn has_data_left(&mut self) -> Result<bool> {
        self.fill_buf().map(|b| !b.is_empty())
    }


    /// Read all bytes into `buf` until the delimiter `byte` of EoF is reached
    /// 
    /// This function will read bytes from the underlying stream until the delimiter of EoF is found.
    /// Once found, all bytes up to, and including, the felimiter (if found) will be appended to `buf`
    /// 
    /// This fuction is blocking and should be used carefully: it is possible for an attacker to continuously send bytes without ever sending the delimiter or EoF.
    /// 
    /// # Errors
    /// 
    /// This function will ignore all intances of [`ErrorKind::Interrupted`] and will othersie return any errors returned by [`fill_buf`].
    /// If an I/O error is encountered, then all bytes read so far will be present in `buf` and its length will have been adjusted properly.
    /// 
    /// [`fill_buf`]: BufRead::fill_buf
    fn read_until(&mut self, byte: u8, buf: &mut DynArray<u8>) -> Result<usize> {
        read_until(self, byte, buf)
    }

    /// Read all bytes until a newline (the `0x0A` byte) is reached, and append them to the provided buffer.
    /// You do not need to clear the buffer before appending.
    /// 
    /// This function will read bytes fro mteh underlying stream until the newline delimiter (the `0x0A`) or EoF is found. 
    /// Once found, all bytes up to, and including, the delimiter (if found) will be appended to `buf`.
    /// 
    /// If successful, this functin will return the total number of bytes read.
    /// 
    /// If this function returns [`Ok(0)`], the stream has reached EoF.
    /// 
    /// This fuction is blocking and should be used carefully: it is possible for an attacker to continuously send bytes without ever sending the delimiter or EoF.
    /// 
    /// # Errors
    /// 
    /// This function has the same error semantics as [`read_until`] and will also return an error if the read bytes are not valid UTF-8.
    /// If an I/O error is encountered, then `buf` may contain some bytes already read in the event that all data read so far was valid UTF-8.
    /// 
    /// [`Ok(0)`]: Ok
    /// [`read_until`]: BufRead::read_until
    fn read_line(&mut self, buf: &mut String) -> Result<usize> {
        unsafe { append_to_string(buf, |b| read_until(self, b'\n', b)) }
    }

    /// Returns an iterator over the contents of this reader split on the byte `byte`.
    /// 
    /// The iterator returned from this function will return instances of <code>[io::Result]<[DynArray]\<u8>></code>.
    /// Each dynarray returned will *not* have the delimiter byte at the end.
    /// 
    /// Each allocated dynarray will use the allocator provided to this function.
    /// 
    /// This function will yield errors whenever [`read_until`] would have also yielded an error.
    /// 
    /// [io::Result]: self::Result "io::result"
    /// [`read_until`]: BufRead::read_until
    fn split(self, byte: u8) -> Split<Self>
    where
        Self : Sized
    {
        Split { buf: self, delim: byte, alloc_id: get_active_alloc().get_id(), mem_tag: get_active_mem_tag() }
    }

    /// returns an iterator over the lines of this reader.
    /// 
    /// This iterator returned from this function will yield instances of <code>[io::Result]<[String]></code>.
    /// Each string returned will *not* have a newline byte (the `0x0A` byte) or `CRLF` (`0x0D`, `0x0A` bytes) at the end.
    /// 
    /// [io::Result]: self::Result "io::Result"
    fn lines(self) -> Lines<Self>
    where
        Self : Sized
    {
        Lines { buf: self, alloc_id: get_active_alloc().get_id(), mem_tag: get_active_mem_tag() }
    }
}

fn read_until<R: BufRead + ?Sized>(r: &mut R, delim: u8, buf: &mut DynArray<u8>) -> Result<usize> {
    let mut read = 0;
    loop {
        let (done, used) = {
            let available = match r.fill_buf() {
                Ok(n) => n,
                Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => return Err(e)
            };
            // PERF(jel): rust's impl has a special impl on posix using libc, would it be faster than slice::memchr ??
            match memchr::memchr(delim, available) {
                Some(i) => {
                    buf.extend_from_slice(&available[..=i]);
                    (true, i + 1)
                },
                None => {
                    buf.extend_from_slice(available);
                    (false, available.len())
                }
            }
        };
        r.consume(used);
        read += used;
        if done || used == 0 {
            return Ok(read);
        }
    }
}

/// A buffer type used with `Read::read_vectored`.
/// 
/// It is a wrapper around an `&mut [u8]`, while unlike [`std::io::IoSliceMut`] it does not guarantee ABI compatibility, 
/// it can be converted to the platform specific representation using a simple set of casts when needed, and it would at most add 2 moves at the site of usage.
#[repr(transparent)]
pub struct IoSliceMut<'a>(&'a mut [u8]);

unsafe impl<'a> Send for IoSliceMut<'a> {}
unsafe impl<'a> Sync for IoSliceMut<'a> {}

impl<'a> fmt::Debug for IoSliceMut<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl<'a> IoSliceMut<'a> {
    /// Create a new `IoSliceMut` wrapping a byte slice
    /// 
    /// # Panics
    /// 
    /// A slice larger than 4GiB may panic on Windows
    #[inline]
    pub fn new(buf: &'a mut [u8]) -> IoSliceMut<'a> {
        IoSliceMut(buf)
    }

    /// Advance the internal cursor of the slice.
    /// 
    /// Also see [`IoSliceMut::advance_slices`] to advance the cursor of multiple buffers.
    /// 
    /// # Panics
    /// 
    /// Panics when trying to advance beyond the end of the slice
    #[inline]
    pub fn advance(&'a mut self, n: usize) {
        self.0 = &mut self.0[n..];
    }

    /// Advance a slice of slices
    /// 
    /// Shrinks the slice to remove any `IoSliceMut`s that are fully advanced over.
    /// If the cursor ends up in the middle of an `IoSliceMut`, is it modified to start at that cursor/
    /// 
    /// For example, if we have a slice of two 8-byte `IoSliceMuts` and we advance by 10 bytes, the result will only include the second `IoSliceMut`, advanced by 2 bytes.
    /// 
    /// # Panics
    /// 
    /// Panics whe trying to advance beyond the end of the slices
    #[inline]
    pub fn advance_slices(bufs: &'a mut &mut [IoSliceMut<'a>], n: usize) {
        // Number of buffers to remove.
        let mut remove = 0;
        // Total length of all the to be removed buffers
        let mut accumulated_len = 0;
        for buf in bufs.iter() {
            if accumulated_len + buf.len() > n {
                break;
            } else {
                accumulated_len += buf.len();
                remove += 1;
            }
        }

        *bufs = &mut mem::replace(bufs, &mut [])[remove..];
        if bufs.is_empty() {
            assert!(n == accumulated_len, "advancing io slices beyond their length");
        } else {
            bufs[0].advance(n - accumulated_len);
        }
    }
}

impl<'a> Deref for IoSliceMut<'a> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> DerefMut for IoSliceMut<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

/// A buffer type used with `Write::write_vectored`.
/// 
/// It is a wrapper around an `&mut [u8]`, while unlike [`std::io::IoSlice`] it does not guarantee ABI compatibility, 
/// it can be converted to the platform specific representation using a simple set of casts when needed, and it would at most add 2 moves at the site of usage.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct IoSlice<'a>(&'a [u8]);

unsafe impl<'a> Send for IoSlice<'a> {}
unsafe impl<'a> Sync for IoSlice<'a> {}

impl<'a> fmt::Debug for IoSlice<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl<'a> IoSlice<'a> {
    /// Cratees a new `IoSlice` wrapping a byte slice
    /// 
    /// # Panics
    /// 
    /// A slice larger than 4GiB may panic on Windows
    #[must_use]
    #[inline]
    pub fn new(buf: &'a [u8]) -> IoSlice<'a> {
        IoSlice(buf)
    }

    /// Advanced the internal cursor of the slice.
    /// 
    /// Also see [`IoSlice::advance_slices`] to advance the cursors of multiple buffers
    /// 
    /// # Panics
    /// 
    /// Panics when trying to advance beyond the end of the slice
    #[inline]
    pub fn advance(&mut self, n: usize) {
        self.0 = &self.0[n..];
    }

    /// Advance a slice of slices
    /// 
    /// Shrinks the slice to remove any `IoSlice`s thatr are fully advanced over.
    /// If hte curesor ends up in the middle of an `IoSlice`, it is modified to start at that cursor.
    /// 
    /// For example, if we have a slice of two 8-byte `IoSlice`s, and we advance by 10 bytes, the result will only include the second `IoSlice`, advanced by 2 bytes.
    /// 
    /// # Panics
    /// 
    /// Panics when trying to advance beyond the end of the slice
    #[inline]
    pub fn advance_slices(bufs: &mut &mut [IoSlice<'a>], n: usize) {
        // Number of buffers to remove.
        let mut remove = 0;
        // Total length of all the to be removed buffers.
        let mut accumulated_len = 0;
        for buf in bufs.iter() {
            if accumulated_len + buf.len() > n {
                break;
            } else {
                accumulated_len += buf.len();
                remove += 1;
            }
        }

        *bufs = &mut mem::replace(bufs, &mut [])[remove..];
        if bufs.is_empty() {
            assert!(n == accumulated_len, "advancing io slices beyond their length");
        } else {
            bufs[0].advance(n - accumulated_len)
        }
    }
}

impl<'a> Deref for IoSlice<'a> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

/// Adapter to chain together two readers.
/// 
/// This struct is generally created by calling [`chain`] on a reader.
/// Please see the documentation of [`chain`] for more details.
/// 
/// [`chain`]: Read::chain
#[derive(Debug)]
pub struct Chain<T, U> {
    first      : T,
    second     : U,
    done_first : bool
}

impl<T, U> Chain<T, U> {
    /// Consumes the `Chain`, returning the wrapped readers
    pub fn into_inner(self) -> (T, U) {
        (self.first, self.second)
    }

    /// Gets references to the underlying readers in this `Chain`
    pub fn get_ref(&self) -> (&T, &U) {
        (&self.first, &self.second)
    }

    /// Gets mutable references to the underlying readers in this `Chain`
    /// 
    /// Care should be taken to avoid modifying the internal I/O state of the underlying readers as doing so may corrupt the internal state of this `Chain`
    pub fn get_mut(&mut self) -> (&mut T, &mut U) {
        (&mut self.first, &mut self.second)
    }
}

impl<T: Read, U: Read> Read for Chain<T, U> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if !self.done_first {
            match self.first.read(buf)? {
                0 if !buf.is_empty() => self.done_first = true,
                n => return Ok(n),
            }
        }
        self.second.read(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> Result<usize> {
        if !self.done_first {
            match self.first.read_vectored(bufs)? {
                0 if bufs.iter().any(|b| !b.is_empty()) => self.done_first = true,
                n => return Ok(n),
            }
        }
        self.second.read_vectored(bufs)
    }
}

impl<T: BufRead, U: BufRead> BufRead for Chain<T, U> {
    fn fill_buf(&mut self) -> Result<&[u8]> {
        if !self.done_first {
            match self.first.fill_buf()? {
                buf if buf.is_empty() => {
                    self.done_first = true;
                },
                buf => return Ok(buf)
            }
        }
        self.second.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        if !self.done_first { self.first.consume(amt) } else { self.second.consume(amt) }
    }
}

impl<T, U> SizeHint for Chain<T, U> {
    fn lower_bound(&self) -> usize {
        SizeHint::lower_bound(&self.first) + SizeHint::lower_bound(&self.second)
    }

    fn upper_bound(&self) -> Option<usize> {
        match (SizeHint::upper_bound(&self.first), SizeHint::upper_bound(&self.second)) {
            (Some(first), Some(second)) => first.checked_add(second),
            _ => None
        }
    }
}

/// Reader adapter which limits the bytes read from an underlying reader.
/// 
/// This struct is genrally crated by calling [`take`] on a reader.
/// Please see the documetnation of [`take`] for more details.
/// 
/// [`take`]: Read::take
pub struct Take<T> {
    inner : T,
    limit : u64
}

impl<T> Take<T> {
    /// Returns the number of bytes that can be read before this instance will return EoF.
    /// 
    /// # Note
    /// 
    /// This instance may reach `EoF` after reading fewer bytes than indicated by this method, if the underlying [`Read`] instance reaches EoF.
    pub fn limit(&self) -> u64 {
        self.limit
    }

    /// Sets the number of bytes taht can be read before this instance will return EoF.
    /// This is the same as constructing a new `Take` instance, so the amount of bytes read and the previous limit value don't matter when calling this method
    pub fn set_limit(&mut self, limit: u64) {
        self.limit = limit;
    }

    /// Consumes the `Take`, returning the wrapped reader.
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Gets a reference to the underlying reader.
    pub fn get_ref(&self) -> &T {
        &self.inner
    }

    /// Get a mutable reference to the underlyign reader
    /// 
    /// Care should be taken to avoid modifying the itnernal I/O state of the underlying reader as doing so may correupt the internal limit of this `Take`
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<T: Read> Read for Take<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        // Don't call into inner reader at all at EoF because it may still block
        if self.limit == 0 {
            return Ok(0);
        }

        let max = cmp::min(buf.len() as u64, self.limit) as usize;
        let n = self.inner.read(&mut buf[..max])?;
        assert!(n as u64 <= self.limit, "number of read bytes exceeds limit");
        self.limit -= n as u64;
        Ok(n)
    }

    fn read_buf(&mut self, mut cursor: BorrowedCursor<'_>) -> Result<()> {
        // Don't call into our inner reader at all at EoF, because it may still block
        if self.limit == 0 {
            return Ok(());
        }

        if self.limit <= cursor.capacity() as u64 {
            let limit = cmp::min(self.limit, usize::MAX as u64) as usize;
            let extra_init = cmp::min(limit, cursor.init_ref().len());

            // SAFETY: no uninit data is written to ibuf
            let ibuf = unsafe { &mut cursor.as_mut()[..limit] };
            let mut sliced_buf : BorrowedBuf<'_> = ibuf.into();

            // SAFETY: extra_init bytes of ibuf are know to be initialized
            unsafe {
                sliced_buf.set_init(extra_init);
            }

            let mut cur = sliced_buf.unfilled();
            self.inner.read_buf(cur.reborrow())?;

            let new_init = cur.init_ref().len();
            let filled = sliced_buf.len();

            // cur / slice_buf / ibuf must drop here

            unsafe {
                // SAFETY: filled bytes have been filled and therefore initialized
                cursor.advance(filled);
                // SAFETY: new_init bytes of buf's unfilled buffer have been initialized
                cursor.set_init(new_init);
            }

            self.limit -= filled as u64
        } else {
            let written = cursor.written();
            self.inner.read_buf(cursor.reborrow())?;
            self.limit -= (cursor.written() - written) as u64;
        }

        Ok(())
    }
}

impl<T: BufRead> BufRead for Take<T> {
    fn fill_buf(&mut self) -> Result<&[u8]> {
        // Don't call into inner reader at all at EoF because it may still block
        if self.limit == 0 {
            return Ok(&[]);
        }

        let buf = self.inner.fill_buf()?;
        let cap = cmp::min(buf.len() as u64, self.limit) as usize;
        Ok(&buf[..cap])
    }

    fn consume(&mut self, amt: usize) {
        // Don't call into inner reader at all at EoF because it may still block
        let amt = cmp::min(amt as u64, self.limit) as usize;
        self.limit -= amt as u64;
        self.inner.consume(amt);
    }
}

impl<T> SizeHint for Take<T> {
    fn lower_bound(&self) -> usize {
        cmp::min(SizeHint::lower_bound(&self.inner) as u64, self.limit) as usize
    }

    fn upper_bound(&self) -> Option<usize> {
        match SizeHint::upper_bound(&self.inner) {
            Some(upper_bound) => Some(cmp::min(upper_bound as u64, self.limit) as usize),
            None => self.limit.try_into().ok()
        }
    }
}

/// An iterator over `u8` values of a reader.
/// 
/// THis struct is genrally created by calling [`bytes`] on a reader.
/// Please see the documentation of [`bytes`] for more details.
/// 
/// [`bytes`]: Read::bytes
#[derive(Debug)]
pub struct Bytes<R> {
    inner : R
}

impl<R: Read> Iterator for Bytes<R> {
    type Item = Result<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut byte = 0;
        loop {
            return match self.inner.read(slice::from_mut(&mut byte)) {
                Ok(0) => None,
                Ok(..) => Some(Ok(byte)),
                Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => Some(Err(e))
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        SizeHint::size_hint(&self.inner)
    }
}


trait SizeHint {
    fn lower_bound(&self) -> usize;
    fn upper_bound(&self) -> Option<usize>;
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.lower_bound(), self.upper_bound())
    }
}

impl<T> SizeHint for T {
    #[inline]
    default fn lower_bound(&self) -> usize {
        0
    }

    #[inline]
    default fn upper_bound(&self) -> Option<usize> {
        None
    }
}

impl<T> SizeHint for &mut T {
    #[inline]
    fn lower_bound(&self) -> usize {
        SizeHint::lower_bound(&**self)
    }

    #[inline]
    fn upper_bound(&self) -> Option<usize> {
        SizeHint::upper_bound(&**self)
    }
}

impl<T> SizeHint for HeapPtr<T> {
    #[inline]
    fn lower_bound(&self) -> usize {
        SizeHint::lower_bound(&**self)
    }

    #[inline]
    fn upper_bound(&self) -> Option<usize> {
        SizeHint::upper_bound(&**self)
    }
}

impl SizeHint for &[u8] {
    #[inline]
    fn lower_bound(&self) -> usize {
        self.len()
    }

    #[inline]
    fn upper_bound(&self) -> Option<usize> {
        Some(self.len())
    }
}

/// An iterator over the contents of an instance of `BufRead` split on a particular byte.
/// 
/// This struct is generally created by calling [`split`] on a `BufRead`.
/// Please see the documentation of [`split`] for more details.
/// 
/// [`split`]: BufRead::split
#[derive(Debug)]
pub struct Split<B> {
    buf      : B,
    delim    : u8,
    alloc_id : u16,
    mem_tag  : MemTag
}

impl<B: BufRead> Iterator for Split<B> {
    type Item = Result<DynArray<u8>>;

    fn next(&mut self) -> Option<Self::Item> {
        let _scope_alloc = ScopedAlloc::new(UseAlloc::Id(self.alloc_id));
        let _scope_mem_tag = ScopedMemTag::new(self.mem_tag);

        let mut buf = DynArray::new();
        match self.buf.read_until(self.delim, &mut buf) {
            Ok(0) => None,
            Ok(_) => {
                if buf[buf.len() - 1] == self.delim {
                    buf.pop();
                }
                Some(Ok(buf))
            },
            Err(e) => Some(Err(e))
        }
    }
}

/// An iterator over the lines of an instance of `BufRead`.
/// 
/// This struct is generally crated by calling [`lines`] on a `BufRead`.
/// Please see the documentation of [`lines`] for more details.
/// 
/// [`lines`]: BufRead::lines
#[derive(Debug)]
pub struct Lines<B> {
    buf      : B,
    alloc_id : u16,
    mem_tag  : MemTag,
}

impl<B: BufRead> Iterator for Lines<B> {
    type Item = Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        let _scope_alloc = ScopedAlloc::new(UseAlloc::Id(self.alloc_id));
        let _scope_mem_tag = ScopedMemTag::new(self.mem_tag);

        let mut buf = String::new();
        match self.buf.read_line(&mut buf) {
            Ok(0) => None,
            Ok(_) => {
                if buf.ends_with('\n') {
                    buf.pop();
                    if buf.ends_with('\r') {
                        buf.pop();
                    }
                }
                Some(Ok(buf))
            }
            Err(e) => Some(Err(e))
        }
    }
}