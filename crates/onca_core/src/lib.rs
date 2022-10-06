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
#![feature(hasher_prefixfree_extras)]

// Used by string.rs
#![feature(utf8_chunks)]
#![feature(slice_range)]
#![feature(pattern)]
#![feature(fmt_internals)]
#![feature(unicode_internals)]

// Used by io
#![feature(strict_provenance)]
#![feature(slice_internals)]
#![feature(ptr_as_uninit)]
#![feature(maybe_uninit_slice)]
#![feature(maybe_uninit_write_slice)]
#![feature(mixed_integer_ops)]

// General
#![feature(min_specialization)]


mod bytes;

pub mod alloc;
pub mod os;
pub mod sync;
pub mod mem;
pub mod collections;
pub mod strings;

pub mod io;

pub use bytes::*;
