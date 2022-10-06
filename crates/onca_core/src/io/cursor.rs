use super::prelude::*;

use core::cmp;
use std::convert::TryInto;
use crate::{
    io::{self, BorrowedCursor, ErrorKind, IoSlice, IoSliceMut, SeekFrom}, 
    collections::DynArray,
    mem::HeapPtr
};

/// A 'Cursor` wraps an in-memeory buffer and provides it with a [`Seek`] implementation.
/// 
/// `Cursor`s are used with in-memory buffers, anything implementing <code>[AsRef]<\[u8]></code>, to allow them to implement [`Read`] and/or [`Write`], 
/// allowing thse buffers to be used anywhere you might use a reader or writer that does acutal I/O.
/// 
/// Onca implements some I/O tratis on various types which are commonly used as a buffer, like <code>Cursor<[DynArray]\<u8>></code> and <code>Cursor<[&\[u8\]][bytes]></code>.
#[derive(PartialEq, Eq, Default, Debug)]
pub struct Cursor<T> {
    inner: T,
    pos: u64
}

impl<T> Cursor<T> {
    /// Creates a new cursor wrapping the provided underlying in-memeory buffer
    /// 
    /// Cursor initial position is `0` even if the underlying buffer (e.g. [`DynArray`]) is not empty.
    /// So writing to the cursor start with overwriting [`DynArray`] content, not with appending to it.
    pub const fn new(inner: T) -> Cursor<T> {
        Cursor { inner, pos: 0 }
    }

    /// Consumes this cursor, returning the underlying value
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Gets a reference to the underlying value in this error
    pub const fn get_ref(&self) -> &T {
        &self.inner
    }

    /// Returns the current position of this cursor.
    pub const fn position(&self) -> u64 {
        self.pos
    }

    /// Sets the position of this cursor.
    pub fn stream_position(&mut self, pos: u64) {
        self.pos = pos;
    }
}

impl<T: AsRef<[u8]>> Cursor<T> {
    /// Returns the remaining slice.
    pub fn remaining_slice(&self) -> &[u8] {
        let len = self.pos.min(self.inner.as_ref().len() as u64);
        &self.inner.as_ref()[(len as usize)..]
    }

    /// Returns `true` if the remaining slice is empty.
    pub fn is_empty(&self) -> bool {
        self.pos >= self.inner.as_ref().len() as u64
    }
}

impl<T: Clone> Clone for Cursor<T> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone(), pos: self.pos }
    }

    fn clone_from(&mut self, source: &Self) {
        self.inner.clone_from(&source.inner);
        self.pos = source.pos;
    }
}

impl<T: AsRef<[u8]>> io::Seek for Cursor<T> {
    fn seek(&mut self, seek: SeekFrom) -> io::Result<u64> {
        let (base_pos, offset) = match seek {
            SeekFrom::Start(n) => {
                self.pos = n;
                return Ok(n);
            },
            SeekFrom::End(n) => (self.inner.as_ref().len() as u64, n),
            SeekFrom::Current(n) => (self.pos, n),
        };

        match base_pos.checked_add_signed(offset) {
            Some(n) => {
                self.pos = n;
                Ok(self.pos)
            }
            None => Err(io::const_io_error!(
                ErrorKind::InvalidInput,
                "invalid seek to a negative or overflowing position"
            )),
        }
    }

    fn stream_len(&mut self) -> io::Result<u64> {
        Ok(self.inner.as_ref().len() as u64)
    }

    fn stream_position(&mut self) -> io::Result<u64> {
        Ok(self.pos)
    }
}

impl<T: AsRef<[u8]>> Read for Cursor<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = Read::read(&mut self.remaining_slice(), buf)?;
        self.pos += n as u64;
        Ok(n)
    }

    fn read_buf(&mut self, mut cursor: BorrowedCursor<'_>) -> io::Result<()> {
        let prev_written = cursor.written();
        Read::read_buf(&mut self.fill_buf()?, cursor.reborrow());
        self.pos += (cursor.written() - prev_written) as u64;
        Ok(())
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        let mut nread = 0;
        for buf in bufs {
            let n = self.read(buf)?;
            nread += n;
            if n < buf.len() {
                break;
            }
        }
        Ok(nread)
    }

    fn is_read_vectored(&self) -> bool {
        true
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        let n = buf.len();
        Read::read_exact(&mut self.remaining_slice(), buf)?;
        self.pos += n as u64;
        Ok(())
    }
}

impl<T: AsRef<[u8]>> BufRead for Cursor<T> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        Ok(self.remaining_slice())
    }

    fn consume(&mut self, amt: usize) {
        self.pos += amt as u64;
    }
}

// Non-resizing write implementation
#[inline]
fn slice_write(pos_mut: &mut u64, slice: &mut [u8], buf: &[u8]) -> io::Result<usize> {
    let pos = cmp::min(*pos_mut, slice.len() as u64);
    let amt = (&mut slice[(pos as usize)..]).write(buf)?;
    *pos_mut += amt as u64;
    Ok(amt)
}

#[inline]
fn slice_write_vectored(pos_mut: &mut u64, slice: &mut [u8], bufs: &[IoSlice<'_>]) -> io::Result<usize> {
    let mut nwritten = 0;
    for buf in bufs {
        let n = slice_write(pos_mut, slice, buf)?;
        nwritten += n;
        if n < buf.len() {
            break;
        }
    }
    Ok(nwritten)
}

/// Reserves the required space, and pads the dynarray with 0s if necessary.
fn reserve_and_pad(pos_mut: &mut u64, arr: &mut DynArray<u8>, buf_len: usize) -> io::Result<usize> {
    let pos : usize = (*pos_mut).try_into().map_err(|_| {
        io::const_io_error!(
            ErrorKind::InvalidInput,
            "cursor position exceeds maximum possible vector length"
        )
    })?;

    // For safety reasonds, we don't want these numbers to overflow, otherwise our allocation won't be enough
    let desired_cap = pos.saturating_add(buf_len);
    if desired_cap > arr.capacity() {
        // We want our arr's total capacity to have room for (pos + buf_len) bytes. Reserve allocated based on additional elements from the length, so we need to reserve the difference
        arr.reserve(desired_cap - arr.len());
    }
    // Pad if pos is above the current len
    if pos > arr.len() {
        let diff = pos - arr.len();
        // Unfortunately, `resize()` would suffice, but the optimizer does not realize the `reserve` if does can be eliminated.
        // So we do it manually to eliminate that extra branch
        let spare = arr.spare_capacity_mut();
        debug_assert!(spare.len() >= diff);
        // Safety: we have allocated enough capacity for this, and we are only writing, not reading
        unsafe {
            spare.get_unchecked_mut(..diff).fill(core::mem::MaybeUninit::new(0));
            arr.set_len(pos)
        }
    }
    Ok(pos)
}

/// Writes the slice to the vec without allocating
/// 
/// # Safety
/// 
/// `arr` must have `buf.len()` spare capacity
unsafe fn vec_write_unchecked(pos: usize, arr: &mut DynArray<u8>, buf: &[u8]) -> usize {
    debug_assert!(arr.capacity() >= pos + buf.len());
    arr.as_mut_ptr().add(pos).copy_from(buf.as_ptr(), buf.len());
    pos + buf.len()
}

/// Resizing write implementation for [`Cursor`].
/// 
/// The cursor is allowed to hav a pre-allocated and initialized dynarray body, but with a position of 0. This means the [`Write`] will overwrite the contents of he vec.
/// 
/// This also allows the dynarray's body to be empty, but with a position or N.
/// This means that [`Write`] will pad the dynarray with 0 initially, before writing anything from tha point
fn dynarr_write(pos_mut: &mut u64, arr: &mut DynArray<u8>, buf: &[u8]) -> io::Result<usize> {
    let buf_len = buf.len();
    let mut pos = reserve_and_pad(pos_mut, arr, buf_len)?;

    // Write the buf, then progresss the arr forward if necessary
    // Safety: we had ensured that the capacity is available, adn that all byte get written up to pos
    unsafe {
        pos = vec_write_unchecked(pos, arr, buf);
        if pos > arr.len() {
            arr.set_len(pos);
        }
    }

    // Bump us forward
    *pos_mut += buf_len as u64;
    Ok(buf_len) 
}

/// Resizing write_vectored implemetnation for [`Cursor`]
/// 
/// The Cursor is allowed to have a pre-allocated and initialized dynarray body, but with a position of 0.
/// This means the [`Write`] will overwrite the contents of the dynarray.
/// 
/// This also allows for the dynarray body to be empty, but with a position of N.
/// This means that [`Write`] will pad the dynarray with 0 initially, before writing anything from that point.
fn dynarr_write_vectored(pos_mut: &mut u64, arr: &mut DynArray<u8>, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
    // For safety reasons, we don't want this  sum to overflow ever.
    // If this saturated, the reserve should panic to avoid any unsound writing.
    let buf_len = bufs.iter().fold(0usize, |a, b| a.saturating_add(b.len()));
    let mut pos = reserve_and_pad(pos_mut, arr, buf_len)?;

    // Write teh buf, the progress the dynarray forward if necessary
    // Safety: we have ensured taht the capacity is available and that all bytes get written up to the last pos
    unsafe {
        for buf in bufs {
            pos = vec_write_unchecked(pos, arr, buf);
        }
        if pos > arr.len() {
            arr.set_len(pos);
        }
    }

    // Bump us forward
    *pos_mut += buf_len as u64;
    Ok(buf_len)
}

impl Write for Cursor<&mut [u8]> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        slice_write(&mut self.pos, self.inner, buf)
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        slice_write_vectored(&mut self.pos, self.inner, bufs)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        true
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Write for Cursor<&mut DynArray<u8>> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        dynarr_write(&mut self.pos, self.inner, buf)
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        dynarr_write_vectored(&mut self.pos, self.inner, bufs)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        true
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Write for Cursor<DynArray<u8>> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        dynarr_write(&mut self.pos, &mut self.inner, buf)
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        dynarr_write_vectored(&mut self.pos, &mut self.inner, bufs)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        true
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Write for Cursor<HeapPtr<[u8]>> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        slice_write(&mut self.pos, &mut self.inner, buf)
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        slice_write_vectored(&mut self.pos, &mut self.inner, bufs)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        true
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<const N: usize> Write for Cursor<[u8; N]> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        slice_write(&mut self.pos, &mut self.inner, buf)
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        slice_write_vectored(&mut self.pos, &mut self.inner, bufs)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        true
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}