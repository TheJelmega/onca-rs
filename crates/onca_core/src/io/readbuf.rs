use core::{
    cmp,
    fmt,
    mem::{self, MaybeUninit}
};
use crate::io::{Result, Write};

/// A borrowed byte buffer which is incrementally filled and initialized
/// 
/// This type is a sort of "double cursro".
/// It tracks three regions in the buffer: 
/// a region at the beginning of he buffer that has bee logivally filled with data,
/// a region that has been initialized at some point, but not yet logically filled,
/// and a region atthe end that is fully uninitialized.
/// The filled region is guaranteed to be a subset of the initialized region.
/// 
/// In sumary, the contents of the buffer can be visualized as:
/// ```not_rust
/// [              capacity              ]
/// [  filled  |        unfilled         ]
/// [    initialized     | uninitialized ]
/// ```
/// 
/// A `BorrowedBuf` is crated around some existing data (or capacity for data) via a unique reference ('&mut`).
/// The `BorrowedBuf` can be configured (e.g. using `clear` and `set_init`), but cannot be directly written.
/// To write into the buffer, use `unfilled` to create a `BorrowedCursor`.
/// The cursor has write -only access to the unfilled portion of the buffer (you can think of it as a write-only iterator).
pub struct BorrowedBuf<'data> {
    /// The buffer's underlying data.
    buf    : &'data mut [MaybeUninit<u8>],
    /// The length of `self.buf` which is known to be filled.
    filled : usize,
    /// The length of `self.buf` which is knows to be initialized.
    init   : usize,
}

impl fmt::Debug for BorrowedBuf<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BorrowedBuf")
            .field("filled", &self.filled)
            .field("init", &self.init)
            .field("capacity", &self.capacity())
        .finish()
    }
}

/// Create a new `BorrowedBuf` from a fully initialized slice
impl <'data> From<&'data mut [u8]> for BorrowedBuf<'data> {
    fn from(slice: &'data mut [u8]) -> Self {
        let len = slice.len();

        BorrowedBuf { 
            buf: unsafe { (slice as *mut [u8]).as_uninit_slice_mut().unwrap() }, 
            filled: 0, 
            init: len
        }
    }
}

/// Crate a new `BorrowedBuf` from an unitialized buffer.
/// 
/// Use `set_init` if part of the buffer is known to be already initialized.
impl<'data> From<&'data mut [MaybeUninit<u8>]> for BorrowedBuf<'data> {
    fn from(buf: &'data mut [MaybeUninit<u8>]) -> Self {
        BorrowedBuf { buf, filled: 0, init: 0 }
    }
}

impl<'data> BorrowedBuf<'data> {
    /// Returns the total capacity of the buffer
    #[inline]
    pub fn capacity(&self) -> usize {
        self.buf.len()
    }

    /// Returns the length of the filled part of the buffer.
    #[inline]
    pub fn len(&self) -> usize {
        self.filled
    }

    /// Returns the length og the initialized part of the buffer.
    #[inline]
    pub fn init_len(&self) -> usize {
        self.init
    }

    /// Returns a shared reference to the filled portion of the buffer.
    #[inline]
    pub fn filled(&self) -> &[u8] {
        // SAFETY: We only slice the filled part of the buffer, which is already valid
        unsafe { MaybeUninit::slice_assume_init_ref(&self.buf[..self.filled]) }
    }

    /// Returns a cursor over the unfilled part of the buffer.
    #[inline]
    pub fn unfilled<'this>(&'this mut self) -> BorrowedCursor<'this> {
        BorrowedCursor {
            start: self.filled,
            // SAFETY: we never assinginto `BorrowedCursor::buf`, so treating its lifetime covaraintly is safe
            buf: unsafe { mem::transmute::<&'this mut BorrowedBuf<'data>, &'this mut BorrowedBuf<'this>>(self) }
        }
    }

    /// Clears the buffer, resetting the filled region to empty
    /// 
    /// The number of initialized bytes is not changed, and the contents of the buffer is not modified.
    #[inline]
    pub fn clear(&mut self) -> &mut Self {
        self.filled = 0;
        self
    }

    /// Assert that the first `n` bytes of the buffer are initialized.
    /// 
    /// `BorrowedBuf` assumes that bytes are never de-initialized, so this method does nothrin when called with fewer bytes than are already known to be intialized.
    /// 
    /// # Safety
    /// 
    /// The caller must ensure that the first `n` unfilled bytes of the buffer have already been initialized
    #[inline]
    pub unsafe fn set_init(&mut self, n: usize) -> &mut Self {
        self.init = cmp::max(self.init, n);
        self
    }
}

/// A writable view of the unfilled portion of a [`BorrowedBuf`](BorrowedBuf).
/// 
/// Provied access to the initialized and unitialized parts of the underlying `BorrowedBuf`.
/// Data  can be written directly to the cursor by using [`append`](BorrowedCurso::append) or indirectly by gettin a slice o part of all of the cursor writing inot the slice.
/// In the indirect case, the caller must call [`advance`](BorrwoedCursor::advance) after writeing to inform the cursor how many bytes have been written.
/// 
/// Once data is written to the cursor, it becomes part of the filled portion of the underlying `BorrwedBuf` and can no longer be accessed or re-written by the cursor.
/// I.e. the cursor tracks the unfilled part o the underlying `BOrrowedBuf`.
/// 
/// The lifetime `'a` is a bound on the lifetime of the underlying buffer (which means it is a bound on he data in tha buffer by trasitivity)
#[derive(Debug)]
pub struct BorrowedCursor<'a> {
    /// The underlying buffer.
    // Safety invariant: we treat the type of buf as covariant in the lifetime of `BorrwoedBuf` when we cratre a `BorrowedCursor`.
    // This is only safe if we never replace `buf` by assigning into it, so don't do that!
    buf   : &'a mut BorrowedBuf<'a>,
    /// The length of the filled portion of the underlyin buffer at the time of the cursor's creation.
    start : usize
}

impl<'a> BorrowedCursor<'a> {
    /// Reborrow this cursor by cloning it with a smaller lifetime.
    /// 
    /// Since teh cursor maintains unique access to tis underlying buffer, the borrowed cursor is not accesible while the new cursor exists.
    #[inline]
    pub fn reborrow<'this>(&'this mut self) -> BorrowedCursor<'this> {
        BorrowedCursor { 
            // SAFETY: we never assinginto `BorrowedCursor::buf`, so treating its lifetime as covariant is safe
            buf: unsafe { mem::transmute::<&'this mut BorrowedBuf<'a>, &'this mut BorrowedBuf<'this>>(self.buf) }, 
            start: self.start
        }
    }

    /// Returns the available space in the cursor
    #[inline]
    pub fn capacity(&self) -> usize {
        self.buf.capacity() - self.buf.filled
    }

    /// Returns the number of bytes written to this cursor since it was crated from a `BorrowBuf`
    /// 
    /// Note that if this cursor is a reborrowed clone of another, then the count returend is hte count written via either cursor, not the count since the cursor was reborrowed
    #[inline]
    pub fn written(&self) -> usize {
        self.buf.filled - self.start
    }

    /// Returns a shared reference to the initialized portion of the cursor
    #[inline]
    pub fn init_ref(&self) -> &[u8] {
        // SAFETY: We only slice the initialized part of the buffer, which is always valid
        unsafe { MaybeUninit::slice_assume_init_ref(&self.buf.buf[self.buf.filled..self.buf.init]) }
    }

    /// Return a mutable reference to the initialized portion of the cursor.
    #[inline]
    pub fn init_mut(&mut self) -> &mut [u8] {
        // SAFETY: We only slice the initialized part of the buffer, which is always valid
        unsafe { MaybeUninit::slice_assume_init_mut(&mut self.buf.buf[self.buf.filled..self.buf.init]) }
    }

    /// Return a mutable reference tothe uninitialized part of the cursor.
    /// 
    /// It is safe to uninitialize any of these bytes.
    #[inline]
    pub fn uninit_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        &mut self.buf.buf[self.buf.init..]
    }

    /// Returns a mutable reference to the whole cursor
    /// 
    /// # Safety
    /// 
    /// The caller must not uninitialize any bytes int ehinitlaized portion of the cursor
    #[inline]
    pub unsafe fn as_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        &mut self.buf.buf[self.buf.filled..]
    }
    
    /// Advance the cursor by asserting that `n` bytes were filled.
    /// 
    /// After advancing, the `n` bytes are no longer accessible via the cursor and can only be accessed via the underlying buffer.
    /// I.e. the buffer's  filled portion grows by `n` elements and its unfilled portion (and the capacity of the cursor) shrinks by `n` elements.
    /// 
    /// # Safety
    /// 
    /// The caller must ensure that the first `n` bytes of the dcurosr have been properly initialized.
    #[inline]
    pub unsafe fn advance(&mut self, n: usize) -> &mut Self {
        self.buf.filled += n;
        self.buf.init = cmp::max(self.buf.init, self.buf.filled);
        self
    }

    /// Initializes all bytes in the cursor
    #[inline]
    pub fn ensure_init(&mut self) -> &mut Self {
        for byte in self.uninit_mut() {
            byte.write(0);
        }
        self.buf.init = self.buf.capacity();
        self
    }

    /// Asserts that the first `n` unfilled bytes of the cursor are initialized.
    /// 
    /// `BorrowedBuf` assumes that bytes are never de-initialized, so this method does nothing when called with fewer bytes tha are already known to be initialized
    /// 
    /// # Safety
    /// 
    /// The caller must ensure that the first `n` bytes of the buffer have already been initialized
    #[inline]
    pub unsafe fn set_init(&mut self, n: usize) -> &mut Self {
        self.buf.init = cmp::max(self.buf.init, self.buf.filled + n);
        self
    }

    /// Append data to the cursor, advancing position within its buffer.
    /// 
    /// # Panics
    /// 
    /// Panics if `self.capacity()` is less than `buf.len()`.
    #[inline]
    pub fn append(&mut self, buf: &[u8]) {
        assert!(self.capacity() >= buf.len());

        // SAFETY: we do not de-initialize any of the elements of the slice
        unsafe {
            MaybeUninit::write_slice(&mut self.as_mut()[..buf.len()], buf);
        }

        // SAFETY: We just added the entire contents of buf to the filled section
        unsafe {
            self.set_init(buf.len());
        }
        self.buf.filled += buf.len();
    }
}

impl<'a> Write for BorrowedCursor<'a> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.append(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}