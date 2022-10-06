extern crate alloc;

use core::{
    slice,
    ptr,
    hash,
    fmt,
    str::*,
    ops::*,
    char::decode_utf16,
    iter::{FromIterator, from_fn},
    str::pattern::Pattern,
    ops::{RangeBounds, Range}, 
};
use std::{
    collections::TryReserveError, 
};

use crate::{
    alloc::{UseAlloc, Allocator},
    collections::DynArray,
    mem::MEMORY_MANAGER,
};

#[derive(Debug, PartialEq, Eq)]
pub struct FromUtf8Error {
    bytes: DynArray<u8>,
    error: Utf8Error
}

#[derive(Debug)]
pub struct FromUtf16Error(());

#[derive(PartialOrd, Eq, Ord)]
pub struct String {
    arr : DynArray<u8>
}

impl String {
    // TODO(jel): const new
    /// Create a new empty string
    #[inline]
    #[must_use]
    pub fn new(alloc: UseAlloc) -> Self {
        Self { arr: DynArray::new(alloc) }
    }

    /// Create a new empty string with a minimum given capacity
    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize, alloc: UseAlloc) -> Self {
        Self { arr: DynArray::with_capacity(capacity, alloc) }
    }

    /// Create a string from raw utf8 bytes if the utf8 bytes are valid, otherwise return an error
    #[inline]
    pub fn from_utf8(arr: DynArray<u8>) -> Result<String, FromUtf8Error> {
        match std::str::from_utf8(&arr) {
            Ok(..) => Ok(String{ arr }),
            Err(e) => Err(FromUtf8Error{ bytes: arr, error: e }),
        }
    }

    /// Creat a string from raw utf8 bytes, including invalid characters
    /// 
    /// Unlike `std::String`, we cannot return a `Cow<'_, str>` for 2 reasons
    ///   - str already has ToOwned implemented, returning `std::string::String`'
    ///   - We would be missing info about the request allocator, because of how `Cow::Borrowed` works
    #[must_use]
    pub fn from_utf8_lossy(v: &[u8], alloc: UseAlloc) -> String {
        let mut iter = Utf8Chunks::new(v);

        let first_valid = if let Some(chunk) = iter.next() {
            let valid = chunk.valid();
            if chunk.invalid().is_empty() {
                debug_assert_eq!(valid.len(), v.len());
                let mut s = String::new(alloc);
                s.push_str(valid);
                return s;
            }
            valid
        } else {
            return String::new(alloc);
        };

        const REPLACEMENT: &str = "\u{FFFD}";

        let mut res = String::with_capacity(v.len(), alloc);
        res.push_str(first_valid);
        res.push_str(REPLACEMENT);

        for chunk in iter {
            res.push_str(chunk.valid());
            if !chunk.invalid().is_empty() {
                res.push_str(REPLACEMENT)
            }
        }

        res
    }

    /// Decode a UTF-16-enocoded vector `v` into a `string`, returning [`Err`]
    pub fn from_utf16(v: &[u16], alloc: UseAlloc) -> Result<String, FromUtf16Error> {
        // From rust's std impl
        // // This isn't done via collect::<Result<_, _>>() for performance reasons.
        // // FIX_ME: the function can be simplified again when #48994 is closed.
        let mut ret = String::with_capacity(v.len(), alloc);
        for c in decode_utf16(v.iter().cloned()) {
            if let Ok(c) = c {
                ret.push(c);
            } else {
                return Err(FromUtf16Error(()));
            }
        }
        Ok(ret)
    }

    /// Decode a UTF-16-encoded slice `v` into a `String`, raplxing inclaid data with [the replacement character (`U+FFFD`)][U+FFFD]
    /// 
    /// [U+FFFD]: core::char::REPLACEMENT_CHARACTER
    #[inline]
    #[must_use]
    pub fn from_utf16_lossy(v: &[u16]) -> String {
        decode_utf16(v.iter().cloned()).map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER)).collect()
    }

    /// Convert a `str` into a `String` with a given allocator
    #[inline]
    #[must_use]
    pub fn from_str(s: &str, alloc: UseAlloc) -> Self {
        let mut res = String::new(alloc);
        res.push_str(s);
        res
    }

    /// Convert a dynamic array of bytes to a `String` without checking that the string contains valid UTF-8
    #[inline]
    #[must_use]
    pub unsafe fn from_utf8_unchecked(bytes: DynArray<u8>) -> String {
        Self { arr: bytes }
    }

    /// Converts a 'String' into a byte dynamic array
    #[inline]
    #[must_use]
    pub fn into_bytes(self) -> DynArray<u8> {
        self.arr
    }

    /// Extracts a string slice containing the entire `String`
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        self
    }

    /// Extracts a string slice containing the entire `String`
    #[inline]
    #[must_use]
    pub fn as_mut_str(&mut self) -> &mut str {
        self
    }

    /// Appends a given string slice onto the end of this `String`
    #[inline]
    pub fn push_str(&mut self, string: &str) {
        self.arr.extend_from_slice(string.as_bytes())
    }

    /// Copies elements fronm `src` range to the end of the string
    pub fn extend_from_within<R>(&mut self, src: R)
        where R: RangeBounds<usize>
    {
        let rsc @ Range { start, end } = slice::range(src, ..self.len());

        assert!(self.is_char_boundary(start));
        assert!(self.is_char_boundary(end));

        self.arr.extend_from_within(start..end)
    }

    /// Return the `String`'s capacity, in bytes
    #[inline]
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.arr.capacity()
    }

    /// Reserves capacity for at least `additional` bytes more that the current length.
    /// The allocator may reserve more space to speculatively avoid frequent allocations.
    /// After calling 'reserve' capacity will be greater than or equal to `self.len() + additional`.
    /// Does nothing if already sufficient
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.arr.reserve(additional)
    }

    /// Reserves the minimum capacity for at least 'additional' bytes more than the current length.
    /// Unlike [`reserve`], this will not deliberately over-allocate to speculatively avoid frequent allocations.
    /// After calling `reserve_exact`, capacity will be greater than or equal to `self.len() + additional`. 
    /// Does nothing if the capacity is already sufficient
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.arr.reserve_exact(additional)
    }

    /// Tries to reserve for at least `additional` byte more than the current lenght.
    /// Teh allocator may reserve more space to speculatively avoid freguent allocations. After calling `try_reserve`, capacity will be greater than or equal to `self.len() + additional` if it returns `OK(())`.
    /// Does nothing if capacity is already sufficient.
    /// This method preserves the contents even if an error occurs
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.arr.try_reserve(additional)
    }

    /// Tries to reserve the minimum capacity for at least `additional` bytes more than the current length.
    /// Unlike [`try_reserve`], this will not deliberately over-allocate to speculatively avoid frequent allocations.
    /// After calling `try_reserve_exact`, capacity will be greater than or equal to `self.len() + additional` if it returns `Ok(())`.
    /// Does notheing if the capacity is already sufficient
    /// 
    /// Note that the allocator may five the collection more space than it request.
    /// Therefore, capacity can not be relied upon to be pecisely minimal.
    /// Prefer [`try_reserve`] if future insertions are expected.
    #[inline]
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.arr.try_reserve_exact(additional)
    }

    /// Shrinks the capacity of this `String` to match its length
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.arr.shrink_to_fit()
    }

    /// Shrinks the capacity of this `String` with a lower bound
    /// 
    /// The capacity will remain at least as large as both the length and supplied value
    /// 
    /// If the current capacity is less than the lower limit, this is a no-op
    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.arr.shrink_to(min_capacity)
    }

    /// Appends the given [`char`] to the end of this `String`.
    #[inline]
    pub fn push(&mut self, ch: char) {
        match ch.len_utf8() {
            1 => self.arr.push(ch as u8),
            _ => self.arr.extend_from_slice(ch.encode_utf8(&mut [0; 4]).as_bytes()),
        }
    }

    /// Returns a byte slice of this `String`'s contents
    /// 
    /// The inverse of this method is [`from_utf8`]
    #[inline]
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.arr
    }

    /// Shortens the `String`to the specified length
    /// 
    /// If `new_len` is greater than the string's curent length, this has no effect
    /// 
    /// Note that this method has no effect on the allocated capacity of the string
    /// 
    /// # Panics
    /// 
    /// Panics if `new_len` does not lie on a [`char`] boundary
    #[inline]
    pub fn truncate(&mut self, new_len: usize) {
        if new_len <= self.len() {
            assert!(self.is_char_boundary(new_len));
            self.arr.truncate(new_len)
        }
    }

    /// removes the last character from the string buffer and returns it
    /// 
    /// returns [`None`] if this `String` is empty
    #[inline]
    pub fn pop(&mut self) -> Option<char> {
        let ch = self.chars().rev().next()?;
        let new_len = self.len() - ch.len_utf8();
        unsafe {
            self.arr.set_len(new_len)
        }
        Some(ch)
    }

    /// Removes a [`char`] from this `String` at a byte position and returns it.
    /// 
    /// This is an *O*(*n*) operation, as it requires copying every element in the buffer
    /// 
    /// # Panics
    /// 
    /// Panics id `idx` is larger than or equal to the `String`'s length, or if it does not lie on a [`char`] boundary
    #[inline]
    pub fn remove(&mut self, idx: usize) -> char {
        let ch = match self[idx..].chars().next() {
            Some(ch) => ch,
            None => panic!("Cannot remove a char from the end of a string"),
        };

        let next = idx + ch.len_utf8();
        let len = self.len();
        unsafe {
            ptr::copy(self.arr.as_ptr().add(next), self.arr.as_mut_ptr().add(idx), len - next);
            self.arr.set_len(len - (next - idx));
        }
        ch
    }

    /// Remove all matches of patter `pat` in the `String`
    pub fn remove_matches<'a, P>(&'a mut self, pat: P)
        where P: for<'x> Pattern<'x>
    {
        use core::str::pattern::Searcher;

        let rejections = {
            let mut searcher = pat.into_searcher(self);
            // Per Searcher::next:
            //
            // A match result needs to contain the whole matched patter, however Reject results may be split up into arbitrary many adjecent fragments.
            // Both ranges may have zero length
            //
            //
            // In practice the implementation of Searcher::nect_match tend to be more efficient, so we use it here and do some work to invert matches into rejections since thatt's what we want to copy below
            let mut front = 0;
            let rejections : DynArray<_> = from_fn(|| {
                let (start, end) = searcher.next_match()?;
                let prev_front = front;
                front = end;
                Some((prev_front, start))
            })
            .collect();
            rejections.into_iter().chain(core::iter::once((front, self.len())))
        };

        let mut len = 0;
        let ptr = self.arr.as_mut_ptr();

        for (start, end) in rejections {
            let count = end - start;
            if start != len {
                // SAFETY: per Searcher::next
                //
                // The stream of Match and Reject values up to the Done will contain index ranges that are adjacent, non-overlapping, covering the whole haystack and laying on utf8 boundaries.
                unsafe { 
                    ptr::copy(ptr.add(start), ptr.add(len), count)
                }
            }
            len += count;
        }

        unsafe{
            self.arr.set_len(len)
        }
    }

    /// RRetains only the character specified by the predicate
    /// 
    /// In other words, remove all characters `c` such that `f(c)` returns `false`.
    /// THis method operates in place, visiting each character exactly once in the original order, and preserves the order of the retained characters
    pub fn retain<F>(&mut self, mut f: F)
        where F: FnMut(char) -> bool
    {
        struct SetLenOnDrop<'a> {
            s: &'a mut String,
            idx: usize,
            del_bytes: usize
        }

        impl<'a> Drop for SetLenOnDrop<'a> {
            fn drop(&mut self) {
                let new_len = self.idx - self.del_bytes;
                debug_assert!(new_len < self.s.len());
                unsafe { self.s.arr.set_len(new_len) };
            }
        }

        let len = self.len();
        let mut guard = SetLenOnDrop { s: self, idx: 0, del_bytes: 0 };

        while guard.idx < len {
            let ch = 
                // SAFETY: `guard.idx` is positive-or-zero  and less than len so the `get_unchecked` is in bounds. `self` is a valid UTF-8 like string and the returned stlice strts at a unicode codepoint, so the `chars` always return one character.
                unsafe { guard.s.get_unchecked(guard.idx..len).chars().next().unwrap_unchecked() };
            let ch_len = ch.len_utf8();

            if !f(ch) {
                guard.del_bytes += ch_len;
            } else if guard.del_bytes > 0 {
                // SAFERY: `guard.idx` is in bound and `guard.del_bytes` represents the number of bytes that are erased from the string so the resulting `guard.idx - guard.del_bytes` always represents a valid unicode code point
                //
                // `guead.del_bytes` >= `ch.len_utf8()`, so taking a slice with `ch.len_utd8)` len is safe.
                ch.encode_utf8(unsafe {
                    slice::from_raw_parts_mut(
                        guard.s.as_mut_ptr().add(guard.idx - guard.del_bytes),
                        ch.len_utf8()
                    )
                });

                // Point idx to the next char
                guard.idx += ch_len;
            }
        }
        drop(guard);
    }

    /// Insert a character into this `String` at a byte position
    /// 
    /// This is an *O*(*n*) operation as it requres copying every element in the buffer.
    /// 
    /// # Panics
    /// 
    /// Panics if `idx` is larger than the `String`'s length, or if it does not lie on a [`char`] boundary
    #[inline]
    pub fn insert(&mut self, idx: usize, ch: char) {
        assert!(self.is_char_boundary(idx));
        let mut bits = [0; 4];
        let bits = ch.encode_utf8(&mut bits).as_bytes();

        unsafe {
            self.insert_bytes(idx, bits);
        }
    }

    unsafe fn insert_bytes(&mut self, idx: usize, bytes: &[u8]) {
        let len = self.len();
        let amt = bytes.len();
        self.arr.reserve(amt);

        unsafe {
            ptr::copy(self.arr.as_ptr().add(idx), self.arr.as_mut_ptr().add(idx + amt), len - idx);
            ptr::copy_nonoverlapping(bytes.as_ptr(), self.arr.as_mut_ptr().add(idx), amt);
            self.arr.set_len(len + amt);
        }
    }

    /// Insert a string slice into this `String` at a byte position
    /// 
    /// This is an *O*(*n*) operation as it requres copying every element in the buffer.
    /// 
    /// # Panics
    /// 
    /// Panics if `idx` is larger than the `String`'s length, or if it does not lie on a [`char`] boundary
    #[inline]
    pub fn insert_str(&mut self, idx: usize, string: &str) {
        assert!(self.is_char_boundary(idx));
        unsafe {
            self.insert_bytes(idx, string.as_bytes());
        }
    }

    /// Returns a mutable reference to the contents of this `String`.
    /// 
    /// # Safety
    /// 
    /// This function is unsafe, because the returned `&mut DynArray` allows writing bytes which are not valid UTF-8.
    /// If this constraint is violated, using the original `String` after dropping the `&mut DynArray` may violate memory safety, as anything using the String assumes that the `String`'s are valid UTF-8
    #[inline]
    pub unsafe fn as_mut_dynarr(&mut self) -> &mut DynArray<u8> {
        &mut self.arr
    }

    /// Get the length of this `String`, in bytes, not [`char`]s or graphemes.
    /// In other words, it might not be what a human considers the length of the string
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.arr.len()
    }

    /// Returns `true` if this `String` has a length of zero, `false` otherwise
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the id of the allocator used by this `String`
    #[inline]
    #[must_use]
    pub fn allocator_id(&self) -> u16 {
        self.arr.allocator_id()
    }

    /// Get the allocator used by this `String`
    #[inline]
    #[must_use]
    pub fn allocator(&mut self) -> &mut dyn Allocator {
        self.arr.allocator()
    }

    /// Splits the string into two at the given byte index
    /// 
    /// Returns a newly allocated `String` with the same allocator as this `String`.
    /// `self` contains bytes `[0, at)`, and the returned string constains bytes `[at, len)`.
    /// `at` must be on the boundary of a UTF-8 codepoint.
    /// 
    /// # Panics
    /// 
    /// Panics if `at` isn't on a `UTF-8` codepoint boundary, or if it's beyond the last codepoint of the string
    #[inline]
    #[must_use = "use `.truncate()` if you don't need the other half"]
    pub fn split_off(&mut self, at: usize) -> String {
        assert!(self.is_char_boundary(at));
        let other = self.arr.split_off(at);
        unsafe { String::from_utf8_unchecked(other) }
    }

    /// Truncate this `String`, removing all contents
    /// 
    /// While this means the `String` will have a length of zero, it does not touch its capacity
    #[inline]
    pub fn clear(&mut self) {
        self.arr.clear()
    }

    /// Removes the specified range from the string in bulk, returning all removed characters as an iterator
    /// 
    /// The returned iterator keeps a mutalbe borrow on the strin to optimize its implementation.
    /// 
    /// # Panics
    /// 
    /// Panics if the starting point or end point do not lie on a [`char`] boundary, of if they're out of bounds
    /// 
    /// # Leaking
    /// 
    /// If the returned iterator goes out of scope without being dropped (due to [`core::mem::forget`], for example), the string may still contain a copy of any drained characters, or may have lost characters arbitrarily, including characters outside the range
    pub fn drain<R>(&mut self, range: R) -> Drain<'_>
        where R : RangeBounds<usize>
    {
        // Memory safety
        //
        // The String version of Deain does not have thememory safety issues of hte dynarray version. The data is just plain bytes.
        // Because the range removal happens in Drop, if the Drain iterator is leaked, the removal will not happen
        let Range { start, end } = slice::range(range, ..self.len());
        assert!(self.is_char_boundary(start));
        assert!(self.is_char_boundary(end));

        // Take out two simultaneous borrows. The &mut String won't be accessed until iteration is over, in Drop
        let self_ptr = self as *mut _;
        // SAFERY: `slice::range` and `is_char_boundary` do the appropriate bounds checks
        let chars_iter = unsafe { self.get_unchecked(start..end) }.chars();

        Drain { start, end, iter: chars_iter, string: self_ptr }
    }

    /// Removes the specified range in the string, and replace it with the given string.
    /// The given string doesn't need to be the same length as the range
    /// 
    /// # Panics
    /// 
    /// Panics if the starting point or end point do not lie on a [`char`] boundary, or if they're out of bounds
    pub fn replace_range<R>(&mut self, range: R, replace_with: &str)
        where R : RangeBounds<usize>
    {
        // Memory safety
        //
        // `replace_range` does not have the memory safety issues of a dynarray `splice`, as the data is just plain bytes

        // WARNING: Inlining this variable would be unsound (#81138)
        // NOTE(jel): The reasoning behind this is that calling `range.start_bound()` multiple times could return different result, caused by internal mutability
        let start = range.start_bound();
        match start {
            std::ops::Bound::Included(&n) => assert!(self.is_char_boundary(n)),
            std::ops::Bound::Excluded(&n) => assert!(self.is_char_boundary(n + 1)),
            std::ops::Bound::Unbounded => {},
        }

        // WARNING: Inlining this variable would be unsound (#81138)
        // NOTE(jel): The reasoning behind this is that calling `range.start_bound()` multiple times could return different result, caused by internal mutability
        let end = range.end_bound();
        match end {
            std::ops::Bound::Included(&n) => assert!(self.is_char_boundary(n + 1)),
            std::ops::Bound::Excluded(&n) => assert!(self.is_char_boundary(n)),
            std::ops::Bound::Unbounded => {},
        }

        // WARNING: Inlining this variable would be unsound (#81138)
        // We assume the bounds reported by `range` remian the same, but an adversarial implementation could change between calls
        unsafe { self.as_mut_dynarr() }.splice((start, end), replace_with.bytes());
    }
}

// TODO(jel): How to make this work on string slices directly? As it seems that the `alloc` crate only is allowed to do that, cause it's a special crate
// This implements functions that are otherwise done via `str`, but which would have returned a rust `String`, instead of an onca `String`
impl String {
    /// Replaces all matches of a patter with another string.
    /// 
    /// `replace` creates a new `String`, and copies the data from this string into it.
    /// While doing so, it attempts to find matches of a pattern.
    /// If it finds any, it replaces them with the replacement string slice
    #[inline]
    #[must_use = "this returns the replaced string as a new allocation, without modifying the original"]
    pub fn replace<'a, P: Pattern<'a>>(&'a self, from: P, to: &str) -> String {
        let mut res = String::new(UseAlloc::Id(self.allocator_id()));
        let mut last_end = 0;
        for (start, part) in self.match_indices(from) {
            res.push_str(unsafe { self.get_unchecked(last_end..start) });
            res.push_str(to);
            last_end = start + part.len();
        }
        res.push_str(unsafe{ self.get_unchecked(last_end..self.len()) });
        res
    }

    /// Replaces the first N matches of a patter with another string.
    /// 
    /// `replacen` create a new `String`, and copeis the data from this string into it.
    /// While doing so, it attempts to find matches of a pattern.
    /// If it finds any, it replaces them with the replacement string slice at the most `count` times
    pub fn replacen<'a, P: Pattern<'a>>(&'a self, pat: P, to: &str, count: usize) -> String {
        // Hope to reduce the times of re-allocation
        let mut res = String::with_capacity(32, UseAlloc::Id(self.allocator_id()));
        let mut last_end = 0;
        for (start, part) in self.match_indices(pat).take(count) {
            res.push_str(unsafe { self.get_unchecked(last_end..start) });
            res.push_str(to);
            last_end = start + part.len();
        }
        res.push_str(unsafe{ self.get_unchecked(last_end..self.len()) });
        res
    }

    /// Returns the lowercase equivalent of this `String`, as a new `String`
    /// 
    /// 'Lowercase' is defined according to the terms of the Unicode Derived Core Properties `Lowercase`
    /// 
    /// Since some charactes can expend into multiple characters when changing case, this functions returns a `String` instead of modifying the paramter in-place
    pub fn to_lowercase(&self) -> String {
        let out = Self::convert_while_ascii(self.as_bytes(), u8::to_ascii_lowercase, UseAlloc::Id(self.allocator_id()));

        // Safety: we know this is a valid char boundary since out.len() is only progressed if ascii bytes are found
        let rest = unsafe { self.get_unchecked(out.len()..) };

        // Safety: We have written only valid ASCII to out dynarray
        let mut s = unsafe { String::from_utf8_unchecked(out) };

        for (i, c) in rest[..].char_indices() {
            if c == 'Σ' {
                // Σ maps to σ, except at the end of a word wher it maps to ς.
                // This is hte only conditional (contextual), but language-independent mapping in `SpecialCasing.txt`, so hard-code it rather thn have a generic "condition" mechanism
                // See https://github.com/rust-lang/rust/issues/26035
                map_uppercase_sigma(rest, i, &mut s);
            } else {
                match core::unicode::conversions::to_lower(c) {
                    [a, '\0', _] => s.push(a),
                    [a, b, '\0'] => {
                        s.push(a);
                        s.push(b);
                    },
                    [a, b, c] => {
                        s.push(a);
                        s.push(b);
                        s.push(c);
                    }
                }
            }
        }

        return s;

        fn map_uppercase_sigma(from: &str, i: usize, to: &mut String) {
            // See https://www.unicode.org/versions/Unicode7.0.0/ch03.pdf#G33992
            // for the definition of `Final_Sigma`.
            debug_assert!('Σ'.len_utf8() == 2);
            let is_word_final = case_ignorable_then_cased(from[..i].chars().rev()) &&
                                !case_ignorable_then_cased(from[i + 2..].chars());
            to.push_str(if is_word_final { "ς" } else { "σ" });
        }

        fn case_ignorable_then_cased<I: Iterator<Item = char>>(iter: I) -> bool {
            use core::unicode::{Case_Ignorable, Cased};
            match iter.skip_while(|&c| Case_Ignorable(c)).next() {
                Some(c) => Cased(c),
                None => false
            }
        }
    }

    /// Returns the uppercase equivalent of this string slice, as a new `String`
    /// 
    /// 'Uppercase' is defined according to the terms of the Unicode Derived Core Property `Uppercase`.
    /// 
    /// SInce some characters can expand into multiple characters when changing the case, this funciton reutns a `String` instead of modigying the paramter in-place.
    #[must_use = "this returns the uppercase string as a new String, without modigying the original"]
    pub fn to_uppercase(&self) -> String {
        let out = Self::convert_while_ascii(self.as_bytes(), u8::to_ascii_uppercase, UseAlloc::Id(self.allocator_id()));

        // Safety: we know this is a valid char boundary since out.len() is only progressed if ascii bytes are found
        let rest = unsafe { self.get_unchecked(out.len()..) };

        // Safety: We have written only valid ASCII to out vec
        let mut s = unsafe { String::from_utf8_unchecked(out) };

        for c in rest.chars() {
            match core::unicode::conversions::to_upper(c) {
                [a, '\0', _] => s.push(a),
                [a, b, '\0'] => {
                    s.push(a);
                    s.push(b);
                },
                [a, b, c] => {
                    s.push(a);
                    s.push(b);
                    s.push(c);
                }
            }
        }
        s
    }

    // NOTE(jel): re-implements iter.repeat(n), as we don't use std::Vec
    /// Create a new `String` by repeating a string `n` times.
    /// 
    /// # Panics
    /// 
    /// This function will panic if th capacity would overflow
    #[must_use]
    pub fn repeat(&self, n: usize) -> String {
        if n == 0 {
            return String::new(UseAlloc::Id(self.allocator_id()));
        }

        // If `n` is larger than zero, it can be split as
        // `n = 2^expn + rem (2^expn > rem, expn >= 0, rem >= 0)`.
        // `2^expn` is the number represented by the leftmost '1' bit of `n`,
        // and `rem` is the remaining part of `n`.

        let capacity = self.len().checked_mul(n).expect("capacity overflow");
        let mut buf = DynArray::<u8>::with_capacity(capacity, UseAlloc::Id(self.allocator_id()));

        // `2^exp` repetition is done by doubling `buf` `expn`-times
        buf.extend(self.as_bytes());
        {
            let mut m = n >> 1;
            // if m > 0, ther eare remaining bits up to the leftmost `1`.
            while m > 0 {
                unsafe {
                    ptr::copy_nonoverlapping(buf.as_ptr(), (buf.as_mut_ptr().add(buf.len())), buf.len());
                    let buf_len = buf.len();
                    buf.set_len(buf_len * 2);
                }

                m >>= 1;
            }
        }

        // `rem` (`= n - 2^expn`) repetition is done by copying `rem` repetitions from buf itself
        let rem_len = capacity - buf.len();
        if rem_len > 0 {
            unsafe {
                // This is non-overlapping since `2^expn > rem`
                ptr::copy_nonoverlapping(buf.as_ptr(), buf.as_mut_ptr().add(buf.len()), rem_len);
                // `buf.len() + rem_len` eqauls to `but.capacity` (`= self.len() * n`).
                buf.set_len(capacity);
            }
        }

        unsafe { String::from_utf8_unchecked(buf) }
    }

    /// Returns a copy of this string where each character is mapped to its ASCII upper case equivalent
    /// 
    /// ASCII letters 'a' to 'z' are mapped to 'A' to 'Z'.
    /// 
    /// To uppercase the value in-place, use [`make_ascii_uppercase`].
    /// 
    /// To uppercase ASCII characters in addition to non-ASCII characters, use [`to_uppercase`]
    /// 
    /// [`make_ascii_uppercase`]: str::make_ascii_uppercase
    /// ['to_uppercase`]: #method.to_uppercase
    #[must_use = "to uppercase the value in-place, use `make_ascii_uppercase()`"]
    #[inline]
    pub fn to_ascii_uppercase(&self) -> String {
        let mut bytes = self.arr.clone();
        bytes.make_ascii_uppercase();
        // make_ascii uppercase) preserves the UTF-8 invarient
        unsafe { String::from_utf8_unchecked(bytes) }
    }

    /// Returns a copy of this string where each character is mapped to its ASCII lower case equivalent
    /// 
    /// ASCII letters 'A' to 'Z' are mapped to 'a' to 'z'.
    /// 
    /// To lowercase the value in-place, use [`make_ascii_lowercase`].
    /// 
    /// To lowercase ASCII characters in addition to non-ASCII characters, use [`to_lowercase`]
    /// 
    /// [`make_ascii_lowercase`]: str::make_ascii_uppercase
    /// ['to_lowercase`]: #method.to_uppercase
    #[must_use = "to lowercase the value in-place, use `make_ascii_lowercase()`"]
    #[inline]
    pub fn to_ascii_lowercase(&self) -> String {
        let mut bytes = self.arr.clone();
        bytes.make_ascii_lowercase();
        // make_ascii uppercase) preserves the UTF-8 invarient
        unsafe { String::from_utf8_unchecked(bytes) }
    }

    fn convert_while_ascii(b: &[u8], convert: fn(&u8) -> u8, alloc: UseAlloc) -> DynArray<u8> {
        let mut out = DynArray::with_capacity(b.len(), alloc);

        const USIZE_SIZE : usize = core::mem::size_of::<usize>();
        const MAGIC_UNROLL : usize = 2;
        const N : usize = USIZE_SIZE * MAGIC_UNROLL;
        const NONASCII_MASK : usize = usize::from_ne_bytes([0x80; USIZE_SIZE]);

        let mut i = 0;
        unsafe {
            while i + N <= b.len() {
                let in_chunk = b.get_unchecked(i..i + N);
                let out_chunk = out.spare_capacity_mut().get_unchecked_mut(i..i + N);

                let mut bits = 0;
                for j in 0..MAGIC_UNROLL {
                    // read the bytes ` usize at a time (unaligned since we haven't checked the alignment)
                    // safety: in_chunk are valid bytes in the range
                    bits |= in_chunk.as_ptr().cast::<usize>().add(j).read_unaligned();
                }

                if bits & NONASCII_MASK != 0 {
                    break;
                }

                // peform the case conversions on N bytes
                for j in 0..N {
                    let out = out_chunk.get_unchecked_mut(j);
                    out.write(convert(in_chunk.get_unchecked(j)));
                }
                i += N;
            }
            out.set_len(i);
        }

        out
    }
}

impl FromUtf8Error {
    /// Returns a slice of [`u8`]s that were attempted to convert to a string
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..]
    }

    /// Return the bytes that were attempted to convert to a `String`
    /// 
    /// This method is carefully constructed to avoid alloction.
    /// It will consume the error, moving out the bytes, so that a copy of the bytes does not need to be made.
    #[must_use = "`self` will be dropped if the result is not used"]
    pub fn into_bytes(self) -> DynArray<u8> {
        self.bytes
    }

    /// Fetch a `Utf8Error` to get more details about the conversion failure.
    /// 
    /// The [`Utf8Error`] type provided by [`std::str`] represents an error tha tmay occur when converting a slice of [`u8`]s to a [`&str`].
    /// In this sense, it's an analogue to `FromUtf8Error`. 
    /// See its documentation for more details on using it
    #[must_use]
    pub fn utf8_error(&self) -> Utf8Error {
        self.error
    }
}

impl fmt::Display for FromUtf8Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.error, f)
    }
}

impl fmt::Display for FromUtf16Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt("invalid utf-16: lone surrogate found", f)
    }
}

impl Clone for String {
    fn clone(&self) -> Self {
        Self { arr: self.arr.clone() }
    }

    fn clone_from(&mut self, source: &Self) {
        self.arr.clone_from(&source.arr)
    }
}

impl FromIterator<char> for String {
    /// Creat a new string from an iterator, using the default allocator
    fn from_iter<I: IntoIterator<Item = char>>(iter: I) -> Self {
        let mut buf = String::new(UseAlloc::Default);
        buf.extend(iter);
        buf
    }
}

impl<'a> FromIterator<&'a char> for String {
    /// Creat a new string from an iterator, using the default allocator
    fn from_iter<I: IntoIterator<Item = &'a char>>(iter: I) -> Self {
        let mut buf = String::new(UseAlloc::Default);
        buf.extend(iter);
        buf
    }
}

impl<'a> FromIterator<&'a str> for String {
    /// Creat a new string from a [`str`], using the default allocator
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
        let mut buf = String::new(UseAlloc::Default);
        buf.extend(iter);
        buf
    }
}

impl FromIterator<String> for String {
    /// Creat a new string from an iterator, using the default allocator
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        let mut iterator = iter.into_iter();

        // Because we're iterating over `String`s, we can avoid at least one allocation by getting the first string from the iterator and appending to it all the subsequent strings
        match iterator.next() {
            None => String::new(UseAlloc::Default),
            Some(mut buf) => {
                buf.extend(iterator);
                buf
            },
        }
    }
}

impl Extend<char> for String {
    fn extend<T: IntoIterator<Item = char>>(&mut self, iter: T) {
        let iterator = iter.into_iter();
        let (lower_bound, _) = iterator.size_hint();
        self.reserve(lower_bound);
        iterator.for_each(move |c| self.push(c));
    }

    //#[inline]
    //fn extend_one(&mut self, ch: char) {
    //    self.push(ch);
    //}

    //#[inline]
    //fn extend_reserve(&mut self, additional: usize) {
    //    self.reserve(additional)
    //}
}

impl<'a> Extend<&'a char> for String {
    fn extend<T: IntoIterator<Item = &'a char>>(&mut self, iter: T) {
        self.extend(iter.into_iter().cloned())
    }

    //#[inline]
    //fn extend_one(&mut self, &ch: &'a char) {
    //    self.push(ch);
    //}

    //#[inline]
    //fn extend_reserve(&mut self, additional: usize) {
    //    self.reserve(additional)
    //}
}

impl<'a> Extend<&'a str> for String {
    fn extend<T: IntoIterator<Item = &'a str>>(&mut self, iter: T) {
        iter.into_iter().for_each(move |s| self.push_str(s))
    }

    //#[inline]
    //fn extend_one(&mut self, item: &'a str) {
    //    self.push_str(s)
    //}
}

impl Extend<String> for String {
    fn extend<T: IntoIterator<Item = String>>(&mut self, iter: T) {
        iter.into_iter().for_each(move |s| self.push_str(&s))
    }
}

// impls unstable 'pattern' feature (#27721)
impl<'a, 'b> Pattern<'a> for &'b String {
    type Searcher = <&'b str as Pattern<'a>>::Searcher;

    fn into_searcher(self, haystack: &'a str) -> Self::Searcher {
        self[..].into_searcher(haystack)
    }

    #[inline]
    fn is_contained_in(self, haystack: &'a str) -> bool {
        self[..].is_contained_in(haystack)
    }

    #[inline]
    fn is_prefix_of(self, haystack: &'a str) -> bool {
        self[..].is_prefix_of(haystack)
    }

    #[inline]
    fn is_suffix_of(self, haystack: &'a str) -> bool {
        self[..].is_suffix_of(haystack)
    }

    #[inline]
    fn strip_prefix_of(self, haystack: &'a str) -> Option<&'a str> {
        self[..].strip_prefix_of(haystack)
    }

    #[inline]
    fn strip_suffix_of(self, haystack: &'a str) -> Option<&'a str> {
        self[..].strip_suffix_of(haystack)
    }
}

impl PartialEq for String {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(&self[..], &other[..])
    }

    #[inline]
    fn ne(&self, other: &Self) -> bool {
        PartialEq::ne(&self[..], &other[..])
    }
}

macro_rules! impl_eq {
    ($lhs:ty, $rhs:ty) => {
        #[allow(unused_lifetimes)]
        impl<'a, 'b> PartialEq<$rhs> for $lhs {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool {
                PartialEq::eq(&self[..], &other[..])
            }

            #[inline]
            fn ne(&self, other: &$rhs) -> bool {
                PartialEq::ne(&self[..], &other[..])
            }
        }

        #[allow(unused_lifetimes)]
        impl<'a, 'b> PartialEq<$lhs> for $rhs {
            #[inline]
            fn eq(&self, other: &$lhs) -> bool {
                PartialEq::eq(&self[..], &other[..])
            }

            #[inline]
            fn ne(&self, other: &$lhs) -> bool {
                PartialEq::ne(&self[..], &other[..])
            }
        }
    };
}
impl_eq!{ String, str }
impl_eq!{ String, &'a str }

// TODO(jel): impl const Default for String
impl Default for String {
    /// Create as an empty 'String` with the default allocator
    fn default() -> Self {
        Self::new(UseAlloc::Default)
    }
}

impl fmt::Display for String {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl fmt::Debug for String {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl hash::Hash for String {
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
        (**self).hash(hasher)
    }
}

/// Implements the `+` operator for concatenating two strings.
/// 
/// This consumes the `String` on the left-hand side and re-uses its buffer (growing it if neccesary).
/// This is done to avoid allocating a new `String` and copying the entire contents on every operation, which would lead to *O*(*n*^2) running time when building a *n*-byte string by repeated concatinations
/// 
/// The string on the right-hand side is only borrowed; its contents are copied into the returned `String`
impl Add<&str> for String {
    type Output = String;

    fn add(mut self, rhs: &str) -> Self::Output {
        self.push_str(rhs);
        self
    }
}

/// Implements the `+=` operator for appending to a `String`
/// 
/// This has the same behavior as the [`push_str`][String::push_str] method.
impl AddAssign<&str> for String {
    #[inline]
    fn add_assign(&mut self, rhs: &str) {
        self.push_str(rhs)
    }
}

impl Index<Range<usize>> for String {
    type Output = str;

    #[inline]
    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self[..][index]
    }
}

impl Index<RangeTo<usize>> for String {
    type Output = str;

    #[inline]
    fn index(&self, index: RangeTo<usize>) -> &Self::Output {
        &self[..][index]
    }
}

impl Index<RangeFrom<usize>> for String {
    type Output = str;

    #[inline]
    fn index(&self, index: RangeFrom<usize>) -> &Self::Output {
        &self[..][index]
    }
}

impl Index<RangeFull> for String {
    type Output = str;

    #[inline]
    fn index(&self, index: RangeFull) -> &Self::Output {
        unsafe { core::str::from_utf8_unchecked(&self.arr) }
    }
}

impl Index<RangeInclusive<usize>> for String {
    type Output = str;

    #[inline]
    fn index(&self, index: RangeInclusive<usize>) -> &Self::Output {
        Index::index(&**self, index)
    }
}

impl Index<RangeToInclusive<usize>> for String {
    type Output = str;
    
    #[inline]
    fn index(&self, index: RangeToInclusive<usize>) -> &Self::Output {
        Index::index(&**self, index)
    }
}

impl IndexMut<Range<usize>> for String {
    #[inline]
    fn index_mut(&mut self, index: Range<usize>) -> &mut Self::Output {
        &mut self[..][index]
    }
}

impl IndexMut<RangeTo<usize>> for String {
    #[inline]
    fn index_mut(&mut self, index: RangeTo<usize>) -> &mut Self::Output {
        &mut self[..][index]
    }
}

impl IndexMut<RangeFrom<usize>> for String {
    #[inline]
    fn index_mut(&mut self, index: RangeFrom<usize>) -> &mut Self::Output {
        &mut self[..][index]
    }
}

impl IndexMut<RangeFull> for String {
    #[inline]
    fn index_mut(&mut self, index: RangeFull) -> &mut Self::Output {
        unsafe { core::str::from_utf8_unchecked_mut(&mut *self.arr) }
    }
}

impl IndexMut<RangeInclusive<usize>> for String {
    #[inline]
    fn index_mut(&mut self, index: RangeInclusive<usize>) -> &mut Self::Output {
        IndexMut::index_mut(&mut **self, index)
    }
}

impl IndexMut<RangeToInclusive<usize>> for String {
    #[inline]
    fn index_mut(&mut self, index: RangeToInclusive<usize>) -> &mut Self::Output {
        IndexMut::index_mut(&mut **self, index)
    }
}

impl Deref for String {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { core::str::from_utf8_unchecked(&self.arr) }
    }
}

impl DerefMut for String {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { core::str::from_utf8_unchecked_mut(&mut *self.arr) }
    }
}

impl FromStr for String {
    type Err = core::convert::Infallible;

    /// Create a `String` from a `str` using the default allocator
    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(String::from(s))
    }
}

/// A trait for converting a value to a `String`
/// 
/// This trait is automatically implemented for any type which implements the [`Display`] trait. As such, `ToString` shouldn't be implemented directly: [`Display`] should be implemented instead, and you ge the ToString implementation for free
/// 
/// ['Display']: fmt::Display
pub trait ToString {
    /// Converts the given value to a `String` with the default allocator
    fn to_string(&self) -> String {
        self.to_string_with_alloc(UseAlloc::Default)
    }
    
    /// Converts the given value to a `String` with the given allocator
    fn to_string_with_alloc(&self, alloc: UseAlloc) -> String;
}

/// # Panics
/// 
/// In this implementation, the 'to_string' method panics if the `Display` implementation returns an error.
/// This indicated an incorrect `Display` implementation since `fmt::Write for String` never returns an error itself
impl<T: fmt::Display + ?Sized> ToString for T {
    #[inline]
    default fn to_string_with_alloc(&self, alloc: UseAlloc) -> String {
        let mut buf = String::new(alloc);
        let mut formatter = core::fmt::Formatter::new(&mut buf);
        // Bypass format_args!() to avoid wrtie_str with zero-length strs
        fmt::Display::fmt(self, &mut formatter).expect("a Display implementation returned an error unexpectedly");
        buf
    }
}

impl ToString for char {
    fn to_string_with_alloc(&self, alloc: UseAlloc) -> String {
        let mut buf = String::new(alloc);
        buf.push_str(self.encode_utf8(&mut [0; 4]));
        buf
    }
}

impl ToString for u8 {
    fn to_string_with_alloc(&self, alloc: UseAlloc) -> String {
        let mut buf = String::with_capacity(3, alloc);
        let mut n = *self;
        if n >= 10 {
            if n >= 100 {
                buf.push((b'0' + n / 100) as char);
                n %= 100;
            }
            buf.push((b'0' + n / 10) as char);
            n %= 10;
        }
        buf.push((b'0' + n) as char);
        buf
    }
}

impl ToString for i8 {
    fn to_string_with_alloc(&self, alloc: UseAlloc) -> String {
        let mut buf = String::with_capacity(4, alloc);
        if self.is_negative() {
            buf.push('-')
        }

        let mut n = self.unsigned_abs();
        if n >= 10 {
            if n >= 100 {
                buf.push('1');
                n -= 100;
            }
            buf.push((b'0' + n / 10) as char);
            n %= 10;
        }
        buf.push((b'0' + n) as char);
        buf
    }
}

impl ToString for str {
    #[inline]
    fn to_string_with_alloc(&self, alloc: UseAlloc) -> String {
        let mut buf = String::new(alloc);
        buf.push_str(self);
        buf
    }
}

impl ToString for String {
    fn to_string_with_alloc(&self, alloc: UseAlloc) -> String {
        self.to_owned()
    }
}

impl AsRef<str> for String {
    fn as_ref(&self) -> &str {
        self
    }
}

impl AsMut<str> for String {
    fn as_mut(&mut self) -> &mut str {
        self
    }
}

impl AsRef<[u8]> for String {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl From<&str> for String {
    /// Converts a `&str` into a [`String`].
    /// 
    /// The result is allocated on the heap
    #[inline]
    fn from(s: &str) -> Self {
        let mut buf = String::new(UseAlloc::Default);
        buf.push_str(s);
        buf
    }
}

impl From<&mut str> for String {
    /// Converts a `&mut str` into a [`String`].
    /// 
    /// The result is allocated on the heap
    #[inline]
    fn from(s: &mut str) -> Self {
        let mut buf = String::new(UseAlloc::Default);
        buf.push_str(s);
        buf
    }
}

impl From<&String> for String {
    /// Converts a `&String` into a [`String`].
    /// 
    /// This clones `s` and returns the clone
    #[inline]
    fn from(s: &String) -> Self {
        s.clone()
    }
}

impl From<String> for DynArray<u8> {
    /// Converts the fiven [`String`] to a dynarray [`DynArray`] that hold values of type [`u8`]
    fn from(string: String) -> Self {
        string.into_bytes()
    }
}

impl fmt::Write for String {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.push_str(s);
        Ok(())
    }

    #[inline]
    fn write_char(&mut self, c: char) -> fmt::Result {
        self.push(c);
        Ok(())
    }
}

/// A draining iterator for 'String'
/// 
/// This struct is created by the [`drain`] method on [`String`]. See it's documentation for more
/// 
/// [`drain`]: String::drain
pub struct Drain<'a> {
    /// Will be used as &'a mut String in the destructor
    string : *mut String,
    /// Start of part to remove
    start  : usize,
    /// End of part to remove
    end    : usize,
    /// Current remaining range to remove
    iter   : Chars<'a>
}

unsafe impl Sync for Drain<'_> {}
unsafe impl Send for Drain<'_> {}

impl Drop for Drain<'_> {
    fn drop(&mut self) {
        unsafe {
            // Use DynArray::drain. "Reaffirm" the bounds checks to avoid panic code being inserted again
            let self_arr = (*self.string).as_mut_dynarr();
            if self.start <= self.end && self.end <= self_arr.len() {
                self_arr.drain(self.start..self.end);
            }
        }
    }
}

impl<'a> Drain<'a> {
    /// Returns the remaining (sub)string of this iterator as a slice
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.iter.as_str()
    }
}

impl<'a> AsRef<str> for Drain<'a> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'a> AsRef<[u8]> for Drain<'a> {
    fn as_ref(&self) -> &[u8] {
        self.as_str().as_bytes()
    }
}

impl Iterator for Drain<'_> {
    type Item = char;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn last(mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
}

impl DoubleEndedIterator for Drain<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
}

impl From<char> for String {
    /// Allocatoed an owned [`String`] from a single character
    #[inline]
    fn from(c: char) -> Self {
        let mut buf = String::new(UseAlloc::Default);
        buf.push(c);
        buf
    }
}
