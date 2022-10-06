use core::fmt;
use crate::io::{self, BorrowedCursor, BufRead, IoSlice, IoSliceMut, Read, Seek, SizeHint, Write};

/// A reader which is always at EoF
/// 
/// This struct is generally crated by calling [`empty()`].
/// Plese see the documentation of [`empty()`] for more details.
#[non_exhaustive]
#[derive(Clone, Copy, Default)]
pub struct Empty;

/// Constructs a new handle to an empty reader.
/// 
/// All reads from teh returned reader will return <code>[Ok]\(0)</code>
#[must_use]
pub const fn emtpy() -> Empty {
    Empty
}

impl Read for Empty {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        Ok(0)
    }

    #[inline]
    fn read_buf(&mut self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        Ok(())
    }
}

impl BufRead for Empty {
    #[inline]
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        Ok(&[])
    }

    fn consume(&mut self, _: usize) {
    }
}

impl Seek for Empty {
    #[inline]
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        Ok(0)
    }

    #[inline]
    fn stream_len(&mut self) -> io::Result<u64> {
        Ok(0)
    }

    #[inline]
    fn stream_position(&mut self) -> io::Result<u64> {
        Ok(0)
    }
}

impl fmt::Debug for Empty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Empty").finish_non_exhaustive()
    }
}

impl SizeHint for Empty {
    #[inline]
    fn lower_bound(&self) -> usize {
        0
    }

    #[inline]
    fn upper_bound(&self) -> Option<usize> {
        Some(0)
    }
}

/// A reader which yields one byte over and over and over and over and ...
pub struct Repeat {
    byte: u8
}

/// Createa an instance of a reader that infinitely repeats one byte.
/// 
/// All reads from this reader will succeed by filling the specified buffer with the given byte.
#[must_use]
pub const fn repeat(byte: u8) -> Repeat {
    Repeat { byte }
}

impl Read for Repeat {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        for slot in &mut *buf {
            *slot = self.byte;
        }
        Ok(buf.len())
    }

    fn read_buf(&mut self, mut cursor: BorrowedCursor<'_>) -> io::Result<()> {
        // SAFETY: No uninit bytes are being written
        for slot in unsafe { cursor.as_mut() } {
            slot.write(self.byte);
        }

        let remaining = cursor.capacity();

        // SAFETY: the entire unfilled portion of buf has been initialized
        unsafe {
            cursor.advance(remaining);
        }

        Ok(())
    }

    #[inline]
    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        let mut nwritten = 0;
        for buf in bufs {
            nwritten += self.read(buf)?;
        }
        Ok(nwritten)
    }

    #[inline]
    fn is_read_vectored(&self) -> bool {
        true
    }
}

impl SizeHint for Repeat {
    #[inline]
    fn lower_bound(&self) -> usize {
        usize::MAX
    }

    #[inline]
    fn upper_bound(&self) -> Option<usize> {
        None
    }
}

impl fmt::Debug for Repeat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Repeat").finish_non_exhaustive()
    }
}

/// A writer which will move data into the void
/// 
/// This struct is generally crated by calling [`sink`].
/// Please see the documentation of [`sink()`] for more details
pub struct Sink;

/// Create an instance of a writer which will successfully consume all data.
/// 
/// All calls to [`write`] on the returned instance will return `Ok(buf.len())` and the contents of the buffer will not be inspected
/// 
/// [`write`]: Write::write
#[must_use]
pub const fn sink() -> Sink {
    Sink
}

impl Write for Sink {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        let total_len = bufs.iter().map(|b| b.len()).sum();
        Ok(total_len)
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

impl fmt::Debug for Sink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Sink").finish_non_exhaustive()
    }
}