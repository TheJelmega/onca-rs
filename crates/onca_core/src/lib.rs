#![allow(unused)]

// Used by alloc/heap_ptr/unique/rc/arc.rs
#![feature(coerce_unsized)]
#![feature(unsize)]
// Used by layout.rs
#![feature(int_roundings)]
#![feature(int_log)]
// Used by collections/*.rs
#![feature(allocator_api)]
#![feature(btreemap_alloc)]
#![feature(min_specialization)]
#![feature(hasher_prefixfree_extras)]

// Used by string.rs
#![feature(utf8_chunks)]
#![feature(slice_range)]
#![feature(pattern)]
#![feature(fmt_internals)]
#![feature(unicode_internals)]


mod bytes;

pub mod alloc;
pub mod os;
pub mod sync;
pub mod mem;
pub mod collections;
pub mod strings;

pub use bytes::*;
