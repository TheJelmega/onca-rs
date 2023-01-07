mod buffer;

use core::fmt;
use crate::{
    io::{self, BorrowedCursor, BufRead, IoSliceMut, Read, Seek, SeekFrom, SizeHint, DEFAULT_BUF_SIZE},
    alloc::{UseAlloc, ScopedAlloc, ScopedMemTag},
    collections::DynArray
};
use buffer::Buffer;

/// THe `BufReader` struct adds buffering to any header
/// 
/// It can be excessively inefficient to work directly with a [`Read`] instance.
/// For example, some implementations [`read`][`Read::read`] will result in a system call everytime it gets called.
/// A `BufReader` performs large, infrequent reads on the underlying [`Read`] and maintains an in-memory buffer of the result.
/// 
/// `BufReader` can improve the speed of programs tha make *small* and *repeated* read calls to the same reader.
/// It does not help when reading very large amoungs at once, or reading just one or a few times.
/// It also provides no advantage when rading from a source that is already in memory, like a <code>[DynArray]<u8></code>.
/// 
/// When trhe `BufReader` is dropped, the contents of its buffer will be discarded.
/// Createing multiple instance of a `BufReader` on the same stream can cause data loss.
/// Reading from the underlying reader after unwrapping the `BufReader` with [`BufReader::into_inner`] can also cause data loss.
pub struct BufReader<R> {
    inner : R,
    buf   : Buffer,
}

impl<R: Read> BufReader<R> {
    /// Creates a new `BufRewader` wit ha default buffer capacity. The default is currently 8KB
    pub fn new(inner: R) -> Self {
        Self::with_capacity(inner, DEFAULT_BUF_SIZE)
    }

    /// Creates a new `BufReader<R>` with the specified buffer capacity
    pub fn with_capacity(inner: R, capacity: usize) -> Self {
        Self { inner, buf: Buffer::with_capacity(capacity) }
    }
}

impl<R> BufReader<R> {
     /// Gets a reference to the underlying reader.
     /// 
     /// It is inadvisable to directly read from the underlying reader
     pub fn get_ref(&self) -> &R {
        &self.inner
     }

     /// Gets a mutable reference to the underlying reader.
     /// 
     /// It is inadvisable to directly read from the underlying reader
     pub fn get_mut(&mut self) -> &mut R {
        &mut self.inner
     }

     /// Returns a reference to the internally buffered data.
     /// 
     /// Unlike [`fill_buf`], this will not attemp to fill th buffer if it is emtpy
     /// 
     /// [`fill_buf`]: BufRead::fill_buf
     pub fn buffer(&self) -> &[u8] {
        self.buf.buffer()
     }

     /// Returns the number of bytes the internal buffer can hold at once
     pub fn capacity(&self) -> usize {
        self.buf.capacity()
     }

     /// Unwraps this `BufReader`, returning the underlying reader.
     /// 
     /// Note that any leftover data in the internal buffer is lost.
     /// Therefore, a following read from the underlying reader may lead to data loss.
     pub fn into_inner(self) -> R {
        self.inner
     }

     /// Invalidate all data in the internal buffer
     #[inline]
     fn discard_buffer(&mut self) {
        self.buf.discard_buffer()
     }
}

impl<R: Seek> BufReader<R> {
    pub fn seek_relative(&mut self, offset: i64) -> io::Result<()> {
        let pos = self.buf.pos() as u64;
        if offset < 0 {
            if let Some(_) = pos.checked_sub((-offset) as u64) {
                self.buf.unconsume((-offset) as usize);
                return Ok(());
            }
        } else if let Some(new_pos) = pos.checked_add(offset as u64) {
            if new_pos <= self.buf.filled() as u64 {
                self.buf.consume(offset as usize);
                return Ok(());
            }
        }
        self.seek(SeekFrom::Current(offset)).map(drop)
    }
}

impl<R: Read> Read for BufReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // If we don't have any buffered data and we're doing a massive read (larger than our internal buffer), bypass our internal buffer entirely.
        if self.buf.pos() == self.buf.filled() && buf.len() >= self.capacity() {
            self.discard_buffer();
            return self.inner.read(buf);
        }
        let nread = {
            let mut rem = self.fill_buf()?;
            rem.read(buf)?
        };
        self.consume(nread);
        Ok(nread)
    }

    fn read_buf(&mut self, mut cursor: BorrowedCursor<'_>) -> io::Result<()> {
        // If we don't have any buffered data and we're doing a massive read (larger than our internal buffer), bypass our internal buffer entirely.
        if self.buf.pos() == self.buf.filled() && cursor.capacity() >= self.capacity() {
            self.discard_buffer();
            return self.inner.read_buf(cursor);
        }

        let prev = cursor.written();

        let mut rem = self.fill_buf()?;
        rem.read_buf(cursor.reborrow());

        self.consume(cursor.written() - prev); // Slice impl of read_buf is known to never unfill `buf`

        Ok(())
    }

    // Small read_exacts from a BufReader ar extremely common when using something like a deserializer.
    //The default implementation calls read in a loop, which results in surprsingly poor code generation for the common path where the buffer has enough bytes to fill teh passed-in buffer
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        if self.buf.consume_with(buf.len(), |claimed| buf.copy_from_slice(claimed)) {
            Ok(())
        } else {
            crate::io::default_read_exact(self, buf)
        }
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        let total_len = bufs.iter().map(|b| b.len()).sum::<usize>();
        if self.buf.pos() == self.buf.filled() && total_len >= self.capacity() {
            self.discard_buffer();
            return self.inner.read_vectored(bufs);
        }
        let nread = {
            let mut rem = self.fill_buf()?;
            rem.read_vectored(bufs)?
        };
        self.consume(nread);
        Ok(nread)
    }

    fn is_read_vectored(&self) -> bool {
        self.inner.is_read_vectored()
    }

    // The inner reader might have an optimized `read_to_end`.
    // Drain out buffer and then delegate to teh inner implementation
    fn read_to_end(&mut self, buf: &mut crate::collections::DynArray<u8>) -> io::Result<usize> {
        let inner_buf = self.buffer();
        buf.extend_from_slice(inner_buf);
        let nread = inner_buf.len();
        self.discard_buffer();
        Ok(nread + self.inner.read_to_end(buf)?)
    }

    // The inner reader might have an optimized `read_to_string`.
    // Drain out buffer and then delegate to teh inner implementation
    /// `read_to_string` may temporarily allocate memory and it will use the same allocator as used by the [`String`]
    fn read_to_string(&mut self, buf: &mut crate::strings::String) -> io::Result<usize> {
        // In the general `else` case below we must read bytes into a side buffer, check that they are valid UTF-8, and then append them to `buf`.
        // This requires a potentially large memcpy.

        // If `buf` is empty -- the most common case -- we can leverage `append_to_string` to read directly into `buf`'s internal byte buffer, saving an allocation and a memcpy.
        if buf.is_empty() {
            // `append_to_string`'s safety relies on the buffer only being appended to, since it only checks the UTF-8 validity of hte new data. 
            // If there were existing constent in `buf`, then an untrustworthy reader (i.e. self.inner) could not only append bytes, but also modify existing bytes and renderthem invalid.
            // On the other hand, if `buf` is empty then by definition any writed mused be appended and `append_to_string` will validate all of the new bytes.
            unsafe { crate::io::append_to_string(buf, |b| self.read_to_end(b)) }
        } else {
            let _scope_alloc = ScopedAlloc::new(UseAlloc::Id(buf.allocator_id()));
            let _scope_mem_tag = ScopedMemTag::new(buf.mem_tag());

            // We cannot append our byte buffer directly onto the `buf` String as there could be an incomplete UTF-8 sequence that has only been partially read.
            // We must read everything into a side buffer first and then call `from_utf8` on the complete buffer
            let mut bytes = DynArray::new();
            self.read_to_end(&mut bytes)?;
            let string = core::str::from_utf8(&bytes).map_err(|_| {
                io::const_io_error!(
                    io::ErrorKind::InvalidData,
                    "stream did not contain valid UTF-8"
                )
            })?;
            *buf += string;
            Ok(string.len())
        }
    }
}

impl<R: Read> BufRead for BufReader<R> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.buf.fill_buf(&mut self.inner)
    }

    fn consume(&mut self, amt: usize) {
        self.buf.consume(amt)
    }
}

impl<R: fmt::Debug> fmt::Debug for BufReader<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BufReader")
            .field("reader", &self.inner)
            .field("buffer", &format_args!("{}/{}", self.buf.filled() - self.buf.pos(), self.buf.capacity()))
        .finish()
    }
}

impl<R: Seek> Seek for BufReader<R> {
    /// Seek to an offset, in bytes, in the underlying reader.
    /// 
    /// The position used for seeking with <code>[SeekFrom::Current]/(_)</code> is the position the underlying reader would be at, if the `BufReader<R>` had no internal buffer.
    /// 
    /// Seeking alwyas discards the internal buffer, even if the seek position would otherwise fall within. 
    /// This guarenteees that calling [`BufReader::into_inner()`] immediately after a seek yields the underlying reader at the same position.
    /// 
    /// To seek without discarding the internal buffer, use [`BufReader::seek_relative`].
    /// 
    /// See [`onca_core::io::Seek`] for more details.
    /// 
    /// Note: In the edge case where you're seeking with <code>[SeekFrom::Current]](_)</code> where `n` minus the internal buffer lnegth overflow an `i64` two seeks will be performed instead of one.
    /// If teh second seek returns [`Err`], the underlying reader will be left at the same position iw ould have been if you called `seeks` with <code>[SeekFrom::Current]](0)</code>
    /// 
    /// [`onca_core::io::Seek`]: Seek
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let result = if let SeekFrom::Current(n) = pos {
            let remainder = (self.buf.filled() - self.buf.pos()) as i64;
            // It should be safe to assume that remainder fits within an i64 as the relative means we managed to allocate 8 exibytes and that's absurd.
            // But it's not out of the realm of possibility for some weird underlying reader to support seeking by i64::MIN so we need to handle underflow when subtracting the remainder
            if let Some(offset) = n.checked_sub(remainder) {
                self.inner.seek(SeekFrom::Current(offset))?
            } else {
                // seek backward by our remiander, and then by the offset
                self.inner.seek(SeekFrom::Current(-remainder));
                self.discard_buffer();
                self.inner.seek(SeekFrom::Current(n))?
            }
        } else {
            // Seeking with Start/End doesn't care about our buffer length
            self.inner.seek(pos)?
        };

        self.discard_buffer();
        Ok(result)
    }

    /// Returns the current seek position from the start of the stream.
    /// 
    /// The value returned is equivalent to `self.seek(SeekFrom::Current(0))`, but does not flush the internal buffer.
    /// Due to this optimization, the function does no guarantee that calling `.into_inner()` immediately afterwards will yield the underlying reader at the same time.
    /// Use [`BufReader::seek`] instead if you require that guarantee
    /// 
    /// # Panics
    /// 
    /// Theis function will panic if the position of the inner reader is smaller than the amoung of buffered data.
    /// Tat can happen if the inner reader has an incorrect implementation of [`Seek::stream_position`], or if the position has gone out of sync due to calling [`Seek::seek`] directly on the underlying container.
    fn stream_position(&mut self) -> io::Result<u64> {
        let remainder = (self.buf.filled() - self.buf.pos()) as u64;
        self.inner.stream_position().map(|pos| {
            pos.checked_sub(remainder).expect(
                "overflow when sutracting remaining buffer size from inner stream position"
            )
        })
    }
}

impl<R> SizeHint for BufReader<R> {
    #[inline]
    fn lower_bound(&self) -> usize {
        SizeHint::lower_bound(self.get_ref()) + self.buffer().len()
    }

    #[inline]
    fn upper_bound(&self) -> Option<usize> {
        SizeHint::upper_bound(self.get_ref()).and_then(|up| self.buffer().len().checked_add(up))
    }
}