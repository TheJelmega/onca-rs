use core::fmt;
use crate::io::{self, buffered::linewritershim, BufWriter, IntoInnerError, IoSlice, Write};

use super::linewritershim::LineWriterShim;

/// Wraps a writer and buffers output to it, flushing whenever a newline (`0x0A`, `'\n'`) is detected.
/// 
/// The [`BufWriter`] struct wraps a writer and buffers its output.
/// But it only does this batched write when it goes out of scope, or when the internal buffer is full.
/// Sometimes, you'd prefer to write each line as it's completed, rather than the entire buffer at one.
/// Enter `LineWriter`, it does exactly that.
/// 
/// Like [`BufWriter`], a `LineWriter`'s buffer will also be flushed when the `LineWriter` goes out of scope or when its internal buffer is full.
/// 
/// If there's still a partial line in the buffer when the `LineWriter` is dropped , it will flush those contents
pub struct LineWriter<W: Write> {
    inner : BufWriter<W>
}

impl<W: Write> LineWriter<W> {
    
    /// Creates a new `LineWriter`
    pub fn new(inner: W) -> Self {
        // Lines typically aren't that long, don't use a giant buffer
        Self::with_capacity(inner, 1024)
    }

    /// Creates a new `LineWriter` with at least the specified capacity for the internal buffer.
    pub fn with_capacity(inner: W, capacity: usize) -> Self {
        LineWriter { inner: BufWriter::with_capacity(inner, capacity) }
    }


    /// Get a reference to the underlying writer
    pub fn get_ref(&self) -> &W {
        self.inner.get_ref()
    }

    /// Gets a reference to the underlying writer.
    /// 
    /// Caution must be taken when calling methods on teh mutable reference returned as extra writes could corrupt the output stream
    pub fn get_mut(&mut self) -> &mut W {
        self.inner.get_mut()
    }

    /// Unwraps this `LineWriter`, returning the underlying writer.
    /// 
    /// The internl buffer is written out before returning the writer
    /// 
    /// # Errors
    /// 
    /// An [`Err`] will be returned if an error occurs, while flushing the buffer.
    pub fn into_inner(self) -> Result<W, IntoInnerError<LineWriter<W>>> {
        self.inner.into_inner().map_err(|err| err.new_wrapped(|inner| LineWriter { inner: inner }))
    }
}

impl<W: Write> Write for LineWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        LineWriterShim::new(&mut self.inner).write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        LineWriterShim::new(&mut self.inner).write_vectored(bufs)
    }

    fn is_write_vectored(&self) -> bool {
        self.inner.is_write_vectored()
    }

    fn write_all(&mut self, mut buf: &[u8]) -> io::Result<()> {
        LineWriterShim::new(&mut self.inner).write_all(buf)
    }

    fn write_all_vectored(&mut self, mut bufs: &mut [IoSlice<'_>]) -> io::Result<()> {
        LineWriterShim::new(&mut self.inner).write_all_vectored(bufs)
    }

    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> io::Result<()> {
        LineWriterShim::new(&mut self.inner).write_fmt(fmt)
    }
}

impl<W: Write + fmt::Debug> fmt::Debug for LineWriter<W> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LineWriter")
            . field("writer", &self.get_ref())
            .field("buffer", &format_args!("{}/{}", self.inner.buffer().len(), self.inner.capacity()))
        .finish_non_exhaustive()
    }
}