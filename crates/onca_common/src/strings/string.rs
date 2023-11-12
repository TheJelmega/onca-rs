extern crate alloc;

use core::{
    slice,
    ptr,
    hash,
    fmt::{self, Write},
    str::*,
    ops::*,
    char::decode_utf16,
    iter::{FromIterator, from_fn},
    ops::{RangeBounds, Range}, 
};
use std::collections::TryReserveError;

use crate::{
    alloc::{AllocId, ScopedAlloc},
    mem,
};

pub use std::string::*;

pub trait StringExtensions {
    fn null_terminate(&mut self);

    fn from_null_terminated_utf16_lossy(s: &[u16]) -> String;
    unsafe fn from_null_terminated_utf8_unchecked_u8(s: &[u8]) -> String;
    unsafe fn from_null_terminated_utf8_unchecked_i8(s: &[i8]) -> String;

    fn as_null_terminated_bytes(&self) -> Vec<u8>;
}

impl StringExtensions for String {
    fn null_terminate(&mut self) {
        let vec = unsafe { self.as_mut_vec() };
        vec.push(0);
        vec.pop();
    }

    fn from_null_terminated_utf16_lossy(s: &[u16]) -> String {
        let len = s.iter().position(|byte| *byte == 0).unwrap_or(s.len());
        let mut res = Self::from_utf16_lossy(&s[..len]);
        res.null_terminate();
        res
    }

    unsafe fn from_null_terminated_utf8_unchecked_u8(s: &[u8]) -> String {
        let len = s.iter().position(|byte| *byte == 0).unwrap_or(s.len());
        let mut res = Self::from_utf8_unchecked(s[..len].iter().map(|byte| *byte).collect());
        res.null_terminate();
        res
    }

    unsafe fn from_null_terminated_utf8_unchecked_i8(s: &[i8]) -> String {
        let len = s.iter().position(|byte| *byte == 0).unwrap_or(s.len());
        let mut res = Self::from_utf8_unchecked(s[..len].iter().map(|byte| *byte as u8).collect());
        res.null_terminate();
        res
    }

    fn as_null_terminated_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::from(self.as_bytes());
        bytes.push(0);
        bytes.pop();
        bytes
    }

    
}