//! An encapsulation of `BufReader`'s buffer management logic
//! 
//! This module factors out he basic functionality of `BufReader` in order to protect two core invariants:
//! * `filled` bytes of `buf` are always initialized
//! * `pos` is always <= `filled`
//! Since this module encapsulates the buffer management logiv, we can ensure that the range `pos..filled` is always a valid index into the initialized region of the buffer.
//! This mean that user code which wants to do reads from a `BufReader` via `buffer` + `consume` can do so without encountering any runtime bounds checks
use core::{cmp, mem::MaybeUninit};
use crate::{io::{self, BorrowedBuf, Read}, mem::HeapPtr, alloc::UseAlloc};

pub struct Buffer {
    // The buffer.
    buf    : HeapPtr<[MaybeUninit<u8>]>,
    // The current seek offset into `buf`, must always be <= `filled`
    pos    : usize,
    // Each call to `fill_buf`` sets `filled` to indicate how many bytes at the start of `buf` are initialized with bytes from a read.
    filled : usize
}

impl Buffer {
    #[inline]
    pub fn with_capacity(capacity: usize, alloc: UseAlloc) -> Self {
        let buf = HeapPtr::new_uninit_slice(capacity, alloc);
        Self { buf, pos: 0, filled: 0 }
    }

    #[inline]
    pub fn buffer(&self) -> &[u8] {
        // SAFETY: self.pos and self.cap are valid, and self.cap => self.pos, and that region is initialized because those are all invariants of the type.
        unsafe { MaybeUninit::slice_assume_init_ref(self.buf.get_unchecked(self.pos..self.filled)) }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.buf.len()
    }

    #[inline]
    pub fn filled(&self) -> usize {
        self.filled
    }

    #[inline]
    pub fn pos(&self) -> usize {
        self.pos
    }

    #[inline]
    pub fn discard_buffer(&mut self) {
        self.pos = 0;
        self.filled = 0;
    }

    #[inline]
    pub fn consume(&mut self, amt: usize) {
        self.pos = cmp::min(self.pos + amt, self.filled);
    }

    /// If there are `mat` bytes available in the buffer, pass a slice containing those bytes to `visitor` and return true.
    /// If there are not enough bytes available, return false.
    #[inline]
    pub fn consume_with<V>(&mut self, amt: usize, mut visitor: V) -> bool 
    where
        V : FnMut(&[u8]),
    {
        if let Some(claimed) = self.buffer().get(..amt) {
            visitor(claimed);
            // If the indexing into self.buffer() succeeds, amt must be a valid increment.
            self.pos += amt;
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn unconsume(&mut self, amt: usize) {
        self.pos = self.pos.saturating_sub(amt);
    }

    #[inline]
    pub fn fill_buf<R: Read>(&mut self, mut reader: R) -> io::Result<&[u8]> {
        // If we've reached the end of our internal buffer, then we need to fetch some more data from the reader.
        // Branch using `>=` instead of the more correct `==` to tell he compiler that the pos..cap slice is always valid.
        if self.pos >= self.filled {
            debug_assert!(self.pos == self.filled);

            let mut buf = BorrowedBuf::from(&mut *self.buf);
            // SAFETY: `self.filled` bytes will always have been initialized.
            unsafe {
                buf.set_init(self.filled);
            }

            reader.read_buf(buf.unfilled())?;

            self.filled = buf.len();
            self.pos = 0;
        }
        Ok(self.buffer())
    }
}