#![allow(unused)]

// NOTE(jel): Currently, we have a lot of features we need for the implementations, we should be looking to reduce this in the future.
//            While not having to use unstable features might not be possible, we might be able to reduce the amount we need

// Used by alloc/heap_ptr/unique/rc/arc.rs
#![feature(coerce_unsized)]
#![feature(unsize)]
// Used by layout.rs
#![feature(int_roundings)]
// Used by collections/*.rs
#![feature(allocator_api)]
#![feature(btreemap_alloc)]
#![feature(hasher_prefixfree_extras)]
#![feature(try_reserve_kind)]
#![feature(ptr_sub_ptr)]

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

#![feature(local_key_cell_methods)]


// NEW

// Used an allocators
#![feature(alloc_layout_extra)]

// Used in containers
#![feature(vec_split_at_spare)]
#![feature(can_vector)]


// General
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(specialization, rustc_attrs)]
// NOTE: as `min_specialization` doesn't work for a minor use-case, we use full specialization here, all code added that needs `specialization` and not just `minspecialization` should be mentioned below:
//     - crate::collections::imp::generic_dyn_array: `impl<T, B: DynArrayBuffer<T>> SpecFromIter<T, IntoIter<T, B>> for GenericDynArray<T, B> {`, as `B` needs to be the same, but does not care about the specialization

#![debugger_visualizer(natvis_file = "libonca_core.natvis")]

#[macro_use]
extern crate scopeguard;


mod bytes;
mod os;

pub mod alloc;
pub mod sync;
pub mod mem;
pub mod collections;
pub mod strings;
pub mod io;

pub mod time;

pub mod sys;
pub mod dynlib;

pub mod utils;
pub mod hashing;

pub mod event_listener;

pub use bytes::*;
pub mod prelude;