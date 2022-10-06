use core::{
    fmt,
    mem,
    ptr
};
use std::error;
use crate::{io::{self, ErrorKind, IntoInnerError, IoSlice, Seek, SeekFrom, Write, DEFAULT_BUF_SIZE}, collections::DynArray, alloc::UseAlloc};

/// Wraps a writer an buffers its output
/// 
/// It can be excessively inefficient to work directly with something that implements [`Write`].
/// or example, some implementations [`write`][`Write::write`] will result in a system call everytime it gets called.
/// A `BufWriter` keeps an in-memory buffer of data and writes it to an underlying writer at large, infrequent batches.
/// 
/// `BufWriter` can improve the speed of programs that make *small* and *repeated* write call to the same writer.
/// It does not help when writing very large amount at once, or writing just one or a few times.
/// It also provides no advantage when writing to a destination that is in memory, like a <code>[DynArray]\<u8></code>.
/// 
/// It is critical to call [`flush`] before `BufWriter` is fropped.
/// Though dropping will attempt to fluch the constents of the buffer, any error that happen in the process ofdropping will be ignored.
/// Calling [`flush`] ensures that the buffer is empty and thus dropping will not even attempt other operations.
pub struct BufWriter<W: Write> {
    inner    : W,
    // THe buffer
    // Avoid using this like a normal `DynArray` in common code paths.
    // That is, don't use `buf.push`, `buf.extend_from_slice`, or any other method that require bounds checking or the like.
    // This make an enormous difference to perfomance (we may want to stop using a `DynArray` entirely).
    buf      : DynArray<u8>,
    // If the inner writer panics in a call to write, we don't want to write the buffered data a second time in BufWriter's destructor.
    // This falg tells he Drop impl if it should skip the flush.
    panicked : bool,
}

impl<W: Write> BufWriter<W> {
    /// Crates a new `BufWriter` with a default buffe capacity.
    /// The default is currently 8KB.
    pub fn new(inner: W, alloc: UseAlloc) -> Self {
        Self::with_capacity(inner, DEFAULT_BUF_SIZE, alloc)
    }

    /// Creates a new `BufWriter` with a t least the specified buffer capacity.
    pub fn with_capacity(inner: W, capacity: usize, alloc: UseAlloc) -> Self {
        Self { inner, buf: DynArray::with_capacity(capacity, alloc), panicked: false }
    }

    /// Send data in our local buffer into the inner writer, looping as necessary until wither it's all been sent of an error occurs.
    /// 
    /// Because all the data in the buffer has been reported to out ownder as "succesfully written" (by returning nonzero success values from `write`), 
    /// any 0-length writes from `inner` must be reported as i/o errord from this method.
    pub(in crate::io) fn flush_buf(&mut self) -> io::Result<()> {
        // Helper struct to ensure the buffer is updated after all the writes are complete.
        // It track the number of written bytes and drains them all from the front of the buffer when dropped
        struct BufGuard<'a> {
            buffer  : &'a mut DynArray<u8>,
            written : usize,
        }

        impl<'a> BufGuard<'a> {
            fn new(buffer: &'a mut DynArray<u8>) -> Self {
                Self { buffer, written: 0 }
            }

            // The unwritten part of the buffer
            fn remaining(&self) -> &[u8] {
                &&self.buffer[self.written..]
            }

            // Flag some bytes as removed from the front of the buffer
            fn consume(&mut self, amt: usize) {
                self.written += amt;
            }

            // true if all of the bytes have been written
            fn done(&self) -> bool {
                self.written >= self.buffer.len()
            }
        }

        impl Drop for BufGuard<'_> {
            fn drop(&mut self) {
                if self.written > 0 {
                    self.buffer.drain(..self.written);
                }
            }
        }

        let mut guard = BufGuard::new(&mut self.buf);
        while !guard.done() {
            self.panicked = true;
            let r = self.inner.write(guard.remaining());
            self.panicked = false;

            match r {
                Ok(0) => {
                    return Err(io::const_io_error!(
                        ErrorKind::WriteZero,
                        "failed to write the buffered data"
                    ));
                },
                Ok(n) => guard.consume(n),
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {},
                Err(e) => return Err(e)
            }
        }
        Ok(())
    }

    /// Buffer some data without flushing it, regardless of the size of the data. Writes as much as possible without exceeding capacity.
    /// Returns the number of bytes written.
    pub(super) fn write_to_buf(&mut self, buf: &[u8]) -> usize {
        let available = self.spare_capacity();
        let amt_to_buffer = available.min(buf.len());

        // SAFETY: `amt_to_buffer` is <= buffer's spare capacity by contruction
        unsafe {
            self.write_to_buffer_unchecked(&buf[..amt_to_buffer]);
        }

        amt_to_buffer
    }

    /// Gets a reference to the underlying writer.
    pub fn get_ref(&self) -> &W {
        &self.inner
    }

    /// Gets a mutable reference to the underlying writer
    pub fn get_mut(&mut self) -> &mut W {
        &mut self.inner
    }

    /// Returns a reference to the internally buffered data.
    pub fn buffer(&self) -> &[u8] {
        &self.buf
    }

    /// Returns a mutable reference to the internal buffer
    /// 
    /// THis can be used to write data directly into the buffer without triggering writes to the underlying buffer.
    /// 
    /// That the buffer is a `DynArray` is an implementation detail.
    /// Callers should not modify the capacity as there currently is not public API to do so and thus any capacity changes would be unexpected by the user.
    pub(in crate::io) fn buffer_mut(&mut self) -> &mut DynArray<u8> {
        &mut self.buf
    }

    /// Return the number of bytes the internal buffer can hold without flushing.
    pub fn capacity(&self) -> usize {
        self.buf.capacity()
    }

    /// Unwraps this `BufWriter`, returning the underlying writer
    /// 
    /// The buffer is written out before returning the writer
    pub fn into_inner(mut self) -> Result<W, IntoInnerError<BufWriter<W>>> {
        match self.flush_buf() {
            Err(e) => Err(IntoInnerError::new(self, e)),
            Ok(()) => Ok(self.into_parts().0)
        }
    }

    /// Disassembles this `BufWriter`, returning the underlying writer, and any buffered but unwritten data.
    /// 
    /// If the underlying writier panicked, it is not know what portion of the data was written.
    /// In this case, we return `WriterPanicked` for he buffered data (from which the buffer contents can still be recovered).
    /// 
    /// `into_parts` makes not attempt to flush data and cannot fail
    pub fn into_parts(mut self) -> (W, Result<DynArray<u8>, WriterPanicked>) {
        let buf = mem::take(&mut self.buf);
        let buf = if !self.panicked { Ok(buf) } else { Err(WriterPanicked{ buf }) };

        // SAFETY: forget(self) prevents double dropping inner
        let inner = unsafe { ptr::read(&mut self.inner) };
        mem::forget(self);

        (inner, buf)
    }

    // Ensure this function does not get inlined into `write`, so that it remains inlineable and its common path remains as short as possible.
    // If this function ends up being called frequently relative to `write`, it's likely a sign that the client is using an improperly sized buffer or their write patterns are somewhat pathological.
    #[cold]
    #[inline(never)]
    pub fn write_cold(&mut self, buf: &[u8]) -> io::Result<usize> {
        if buf.len() > self.spare_capacity() {
            self.flush_buf();
        }

        // Why not len > capacity? To avoid a needles trip through when the input exactly fills it.
        // We'd just need to flush it to the underlying writer anyway
        if buf.len() >= self.buf.capacity() {
            self.panicked = true;
            let r = self.get_mut().write(buf);
            self.panicked = false;
            r
        } else {
            // Write to the buffer.
            // In this case, we write to the buffer even if it fills it exactly.
            // Doing otherwise would mean flushing the buffer, then writing this input to the inner writer, which in many cases would be a worse strategy.

            // SAFETY: There was either enough spare capacity already, or there wasn't and we flushed the buffer to ensure that there is.
            // In the latter case, we know that there is, because flushing ensured that our entire buffer is space capacity, and we entered this block because the input buffer length is less than that capacity.
            // In either case, it's safe to write the iput buffer to our buffer
            unsafe {
                self.write_to_buffer_unchecked(buf);
            }
            Ok(buf.len())
        }
    }

    // Ensure this function does not get inlined into `write_all`, so that it remains inlineable and its common path remains as short as possible.
    // If this function ends up being called frequently relative to `write_all`, it's likely a sign that the client is using an improperly sized buffer or their write patterns are somewhat pathological.
    #[cold]
    #[inline(never)]
    pub fn write_all_cold(&mut self, buf: &[u8]) -> io::Result<()> {
         // Normally, `write_all` just calls `write` in a loop. 
         // We can do better by calling `self.get_mut().write_all` directly, which avoids round rtips through the buffe in the even of a series of partial writes in some circumstances

         if buf.len() > self.spare_capacity() {
            self.flush_buf();
         }

         // Why not len > capacity? To avoid a needles trip through when the input exactly fills it.
        // We'd just need to flush it to the underlying writer anyway
        if buf.len() >= self.buf.capacity() {
            self.panicked = true;
            let r = self.get_mut().write_all(buf);
            self.panicked = false;
            r
        } else {
            // Write to the buffer.
            // In this case, we write to the buffer even if it fills it exactly.
            // Doing otherwise would mean flushing the buffer, then writing this input to the inner writer, which in many cases would be a worse strategy.

            // SAFETY: There was either enough spare capacity already, or there wasn't and we flushed the buffer to ensure that there is.
            // In the latter case, we know that there is, because flushing ensured that our entire buffer is space capacity, and we entered this block because the input buffer length is less than that capacity.
            // In either case, it's safe to write the iput buffer to our buffer
            unsafe {
                self.write_to_buffer_unchecked(buf);
            }
            Ok(())
        }
    }

    // SAFETY: Requires `buf.len() <= self.buf.capacity - self.buf.len()`, i.e. that input buffer lenght is less than or equal to spare capacity.
    #[inline]
    unsafe fn write_to_buffer_unchecked(&mut self, buf: &[u8]) {
        debug_assert!(buf.len() <= self.spare_capacity());
        let old_len = self.buf.len();
        let buf_len = buf.len();
        let src = buf.as_ptr();
        let dst = self.buf.as_mut_ptr().add(old_len);
        ptr::copy_nonoverlapping(src, dst, buf_len);
        self.buf.set_len(old_len + buf_len);
    }

    #[inline]
    fn spare_capacity(&self) -> usize {
        self.buf.capacity() - self.buf.len()
    }
}

impl<W: Write> Write for BufWriter<W> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // Use < instead of <= to avoid a needless trip through the buffer in some cases.
        // See `write_cold` for details
        if buf.len() < self.spare_capacity(){
            // SAFETY: safe by above conditional
            unsafe {
                self.write_to_buffer_unchecked(buf);
            }

            Ok(buf.len())
        } else {
            self.write_cold(buf)
        }
    }

    #[inline]
    fn write_all(&mut self, mut buf: &[u8]) -> io::Result<()> {
        // Use < instead of <= to avoid a needless trip through the buffer in some cases.
        // See `write_all_cold` for details
        if buf.len() < self.spare_capacity(){
            // SAFETY: safe by above conditional
            unsafe {
                self.write_to_buffer_unchecked(buf);
            }

            Ok(())
        } else {
            self.write_all_cold(buf)
        }
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        // FIXME: Consider applying `#[inline]` / `#inline` optimizations already applied to `write` and `write_all`.
        // The performance benefits can be significant.
        // The rust implementation refers to #79930.
        if self.get_ref().is_write_vectored() {
            // We have to handle the possibility that the toal length of the buffers overflows `usize` (even though it can only happen if multiple `IoSlice`s reference the same underlying buffe, as otherwise the buffers wouldn't fit in memory).
            // If the computation overflows, then surely the input cannot fit in our buffer, so we forward to he inner writer's `write_bectored` method to let it handle it appropriately.
            let saturated_total_len = bufs.iter().fold(0usize, |acc, b| acc.saturating_add(b.len()));

            if saturated_total_len > self.spare_capacity() {
                self.flush_buf();
            }
            if saturated_total_len >= self.buf.capacity() {
                // Forward to our inner writer if the total length of the input is greater than or equal to our buffer capacity.
                // If we would have overflowed, this condition also holds, and we pass it to the inner writer
                self.panicked = true;
                let r = self.get_mut().write_vectored(bufs);
                self.panicked = false;
                r
            } else {
                // `saturated_total_len < self.buf.capacity()` implies that we did not saturate.
                
                // SAFETY: We checked whether or not the spare capacity was large enough above.
                // If it was, then we're safe already.
                // It it wasn't, we flushed, making sufficient room for any input <= the buffe size, which includes the input.
                unsafe {
                    bufs.iter().for_each(|b| self.write_to_buffer_unchecked(b));
                }
                Ok(saturated_total_len)
            }
        } else {
            let mut iter = bufs.iter();
            let mut total_written = if let Some(buf) = iter.by_ref().find(|&buf| !buf.is_empty()) {
                // This is the first non-empty slice to write, so if it does fit in the buffer, we still get to flush and proceed.
                if buf.len() > self.spare_capacity() {
                    self.flush_buf()?;
                }
                if buf.len() >= self.buf.capacity() {
                    // The slice is at least as large as the buffering capacity, so it's better to write if directly, bypassing the buffer.
                    self.panicked = true;
                    let r = self.get_mut().write(buf);
                    self.panicked = false;
                    return r;
                } else {
                    // SAFETY: We checked whether or not the spare capacity was large enough above.
                    // If it was, then wer're safe already.
                    // If it wasn't, we flushed, making sufficient room for any input < the buffer size, which includes this input
                    unsafe {
                        self.write_to_buffer_unchecked(buf);
                    }

                    buf.len()
                }
            } else {
                return Ok(0);
            };
            debug_assert!(total_written != 0);
            for buf in iter {
                if buf.len() <= self.spare_capacity() {
                    // SAFETY: safe by above condition
                    unsafe {
                        self.write_to_buffer_unchecked(buf);
                    }

                    // This cannot overflow `usize`. If we are here, we've written all of hte bytes so far to our buffer, and we've ensured that we never exceed the buffer's capacity.
                    // Therefore, `total_written` <= `self.buf.capacity` <= `usize::MAX`
                    total_written += buf.len();
                }
            }

            Ok(total_written)
        }
    }

    fn is_write_vectored(&self) -> bool {
        true
    }

    fn flush(&mut self) -> io::Result<()> {
        self.flush_buf().and_then(|()| self.get_mut().flush())
    }
}

impl<W: Write + fmt::Debug> fmt::Debug for BufWriter<W> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BufWriter")
            .field("writer", &self.inner)
            .field("buffer", &format_args!("{}/{}", self.buf.len(), self.buf.capacity()))
        .finish()
    }
}

impl<W: Write + Seek> Seek for BufWriter<W> {
    /// Seek to the offset, in bytes, in the underlying writer
    /// 
    /// Seeking always writes out the internal buffer before seeking
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.flush_buf();
        self.get_mut().seek(pos)
    }
}

impl<W: Write> Drop for BufWriter<W> {
    fn drop(&mut self) {
        if !self.panicked {
            // dtors should not panic, so we ignore a failed flush
            let _r = self.flush_buf();
        }
    }
}

/// Error returned for the buffered data from `BufWriter::into_parts`, when the underlying writer has previously panicked.
/// Contains the (possibley partly written) buffered data.
pub struct WriterPanicked {
    buf : DynArray<u8>
}

impl WriterPanicked {
    /// Returns the perhaps unwritten data.
    /// Some of this data may have been written by the panicking call(s) to the underlying writer, so to simply wtite it again is not a good idea
    pub fn into_inner(self) -> DynArray<u8> {
        self.buf
    }

    const DESCRIPTION: &'static str = "BufWriter inner writer panicked, what data remains unwritten is not known";
}

impl error::Error for WriterPanicked {}

impl fmt::Debug for WriterPanicked {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WriterPanicked")
            .field("buffer", &format_args!("{}/{}", self.buf.len(), self.buf.capacity()))
        .finish()
    }
}

impl fmt::Display for WriterPanicked {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Self::DESCRIPTION)
    }
}