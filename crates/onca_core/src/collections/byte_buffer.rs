use core::{
    ops::{RangeBounds, Deref, DerefMut, Index, IndexMut},
    mem::MaybeUninit,
    hash::Hash, slice::{SliceIndex, self},
};
use std::{vec, io};

use crate::alloc::{Layout, GetAllocatorId};

use super::{
    bitset::IntoIter,
    impl_slice_partial_eq,
};

/// A buffer containing data as an untyped block of bytes
#[derive(Default, Debug)]
pub struct ByteBuffer(Vec<u8>);

impl ByteBuffer {
    /// Create a new [`ByteBuffer`]
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Create a new [`ByteBuffer`] with a given capacity
    #[must_use]
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    /// Get the [`ByteBuffer`]'s capacity
    #[inline]
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    /// Reserve additional space in the [`ByteBuffer`]
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional)
    }

    /// Try and reserve additional space in the [`ByteBuffer`]
    /// 
    /// ## Errors
    /// 
    /// Returns a `TryReserveError` if it was not possible to allocate additional
    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), std::collections::TryReserveError> {
        self.0.try_reserve(additional)
    }

    /// Reserve an exact amount of additional bytes in the [`ByteBuffer`]
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.0.reserve_exact(additional)
    }

    /// Reserve an exact amount of additional bytes in the [`ByteBuffer`]
    #[inline]
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), std::collections::TryReserveError> {
        self.0.try_reserve_exact(additional)
    }

    /// Shrink the [`ByteBuffer`] to its size
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit()
    }

    /// Shrink the [`ByteBuffer`] to either its size of the given minimum capacity, which ever is larger
    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.0.shrink_to(min_capacity)
    }

    /// Truncate the [`ByteBuffer`] to the given length
    #[inline]
    pub fn truncate(&mut self, len: usize) {
        self.0.truncate(len)
    }

    /// Get the [`ByteBuffer`] as a slice
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }
    
    /// Get the [`ByteBuffer`] as a mutable slice
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        self.0.as_mut_slice()
    }

    /// Get the [`ByteBuffer`] as a pointer
    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.0.as_ptr()
    }

    /// Get the [`ByteBuffer`] as a mutable pointer
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.0.as_mut_ptr()
    }

    /// Set the length of the [`ByteBuffer`]
    /// 
    /// ## Safety
    /// 
    /// The user needs to make sure that the buffer, up to the given length, contains valid data
    #[inline]
    pub unsafe fn set_len(&mut self, new_len: usize) {
        self.0.set_len(new_len)
    }

    /// Push a byte into the buffer
    #[inline]
    pub fn push(&mut self, byte: u8) {
        self.0.push(byte)
    }

    /// Pop a byte from the [`ByteBuffer`]
    #[inline]
    pub fn pop(&mut self) -> Option<u8> {
        self.0.pop()
    }

    /// Append a [`ByteBuffer`] to this [`ByteBuffer`]
    #[inline]
    pub fn append(&mut self, other: &mut ByteBuffer) {
        self.0.append(&mut other.0)
    }

    /// Drain a range of bytes from the [`ByteBuffer`]
    #[inline]
    pub fn drain<R: RangeBounds<usize>>(&mut self, range: R) -> vec::Drain<'_, u8> {
        self.0.drain(range)
    }

    /// Clear the [`ByteBuffer`]
    #[inline]
    pub fn clear(&mut self) {
        self.0.clear()
    }

    /// Get the length of the [`ByteBuffer`]
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check whether the [`ByteBuffer`] is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Split the [`ByteBuffer`] at a given location and return the data after it as its own [`ByteBuffer`]
    #[inline]
    pub fn split_off(&mut self, at: usize) -> Self {
        Self(self.0.split_off(at))
    }

    /// Resize the [`ByteBuffer`] with a given byte
    #[inline]
    pub fn resize(&mut self, new_len: usize, byte: u8) {
        self.0.resize(new_len, byte)
    }

    /// Extend the [`ByteBuffer`] from a slice
    #[inline]
    pub fn extend_from_slice(&mut self, other: &[u8]) {
        self.0.extend_from_slice(other)
    }

    /// Extend the [`ByteBuffer`] using a range of bytes that is already in the [`ByteBuffer`]
    #[inline]
    pub fn extend_from_within<R: RangeBounds<usize>>(&mut self, src: R) {
        self.0.extend_from_within(src)
    }

    /// Get the unused capacity of the [`ByteBuffer`] as a mutable slice
    #[inline]
    pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        self.0.spare_capacity_mut()
    }

    /// Split the [`ByteBuffer`] at its length into a mutable slice with valid data and a mutable slice with the uninitialized spare capacity
    #[inline]
    pub fn split_at_spare_mut(&mut self) -> (&mut [u8], &mut [MaybeUninit<u8>]) {
        self.0.split_at_spare_mut()
    }

    /// Replace a range of bytes in the [`ByteBuffer`] with a given range of data
    #[inline]
    pub fn splice<R, I>(&mut self, range: R, replace_with: I) -> vec::Splice<'_, I::IntoIter> where
        R: RangeBounds<usize>,
        I : IntoIterator<Item = u8>
    {
        self.0.splice(range, replace_with)
    }

    /// Get the id of the allocator used by the [`ByteBuffer`]
    #[inline]
    #[must_use]
    pub fn allocator_id(&self) -> u16 {
        self.0.allocator_id()
    }

    /// Convert the [`ByteBuffer`] into a `HeapPtr` with a slice
    #[inline]
    pub fn into_heap_slice(self) -> Box<[u8]> {
        self.0.into_boxed_slice()
    }

    /// Pad the [`ByteBuffer`] with `0` to have a length that's a multiple of a given base
    pub fn pad_to_multiple(&mut self, base: usize) {
        let new_len = self.len().next_multiple_of(base);
        self.resize(new_len, 0);
    }

    /// Write a value to the [`ByteBuffer`], as a raw bytes
    pub fn write_raw<T: Copy>(&mut self, value: T) {
        let len = core::mem::size_of_val(&value);
        let data = &value as *const T as *const u8;
        let slice = unsafe { core::slice::from_raw_parts(data, len) };
        self.extend_from_slice(slice);
    }
}

impl Deref for ByteBuffer {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &*(self.0)
    }
}

impl DerefMut for ByteBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *(self.0)
    }
}

impl Clone for ByteBuffer {
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.0.clone_from(&source.0)
    }
}

impl Hash for ByteBuffer {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<I: SliceIndex<[u8]>> Index<I> for ByteBuffer {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        self.0.index(index)
    }
}

impl<I: SliceIndex<[u8]>> IndexMut<I> for ByteBuffer {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

impl FromIterator<u8> for ByteBuffer {
    #[inline]
    fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
        Self(FromIterator::from_iter(iter))
    }
}

impl IntoIterator for ByteBuffer {
    type Item = u8;
    type IntoIter = vec::IntoIter<u8>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a ByteBuffer {
    type Item = &'a u8;
    type IntoIter = slice::Iter<'a, u8>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut ByteBuffer {
    type Item = &'a mut u8;
    type IntoIter = slice::IterMut<'a, u8>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl Extend<u8> for ByteBuffer {
    #[inline]
    fn extend<T: IntoIterator<Item = u8>>(&mut self, iter: T) {
        self.0.extend(iter)
    }
}

impl<'a> Extend<&'a u8> for ByteBuffer {
    #[inline]
    fn extend<T: IntoIterator<Item = &'a u8>>(&mut self, iter: T) {
        self.0.extend(iter)
    }
}

impl AsRef<ByteBuffer> for ByteBuffer {
    #[inline]
    fn as_ref(&self) -> &ByteBuffer {
        self
    }
}

impl AsMut<ByteBuffer> for ByteBuffer {
    #[inline]
    fn as_mut(&mut self) -> &mut ByteBuffer {
        self
    }
}

impl AsRef<[u8]> for ByteBuffer {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self
    }
}

impl AsMut<[u8]> for ByteBuffer {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        self
    }
}

impl From<&[u8]> for ByteBuffer {
    #[inline]
    fn from(value: &[u8]) -> Self {
        Self(From::from(value))
    }
}

impl From<&mut [u8]> for ByteBuffer {
    #[inline]
    fn from(value: &mut [u8]) -> Self {
        Self(From::from(value))
    }
}

impl<const N: usize> From<[u8; N]> for ByteBuffer {
    #[inline]
    fn from(value: [u8; N]) -> Self {
        Self(From::from(value))
    }
}

impl_slice_partial_eq!{ [] ByteBuffer, ByteBuffer }
impl_slice_partial_eq!{ [] ByteBuffer, [u8] }
impl_slice_partial_eq!{ [] ByteBuffer, &[u8] }
impl_slice_partial_eq!{ [] ByteBuffer, &mut [u8] }
impl_slice_partial_eq!{ [const M: usize] ByteBuffer, [u8; M] }
impl_slice_partial_eq!{ [const M: usize] ByteBuffer, &[u8; M] }
impl_slice_partial_eq!{ [const M: usize] ByteBuffer, &mut [u8; M] }
impl_slice_partial_eq!{ [] [u8], ByteBuffer }
impl_slice_partial_eq!{ [] &[u8], ByteBuffer }
impl_slice_partial_eq!{ [] &mut [u8], ByteBuffer }
impl_slice_partial_eq!{ [const N: usize] [u8; N], ByteBuffer }
impl_slice_partial_eq!{ [const N: usize] &[u8; N], ByteBuffer }
impl_slice_partial_eq!{ [const N: usize] &mut [u8; N], ByteBuffer }

impl io::Write for ByteBuffer {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        self.0.write_vectored(bufs)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        self.0.is_write_vectored()
    }

    #[inline]
    fn write_all(&mut self, mut buf: &[u8]) -> io::Result<()> {
        self.0.write_all(buf)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}