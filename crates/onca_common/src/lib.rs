#![allow(unused)]

// NOTE: Currently, we have a lot of features we need for the implementations, we should be looking to reduce this in the future.
//            While not having to use unstable features might not be possible, we might be able to reduce the amount we need

// Used an allocators
#![feature(alloc_layout_extra)]

// Used in containers
#![feature(vec_split_at_spare)]
#![feature(can_vector)]

// General
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(specialization)]
// NOTE: as `min_specialization` doesn't work for a minor use-case, we use full specialization here, all code added that needs `specialization` and not just `minspecialization` should be mentioned below:
//     - crate::collections::imp::generic_dyn_array: `impl<T, B: DynArrayBuffer<T>> SpecFromIter<T, IntoIter<T, B>> for GenericDynArray<T, B> {`, as `B` needs to be the same, but does not care about the specialization

#![debugger_visualizer(natvis_file = "libonca_common.natvis")]

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
pub mod fmt;

pub mod time;

pub mod sys;
pub mod dynlib;

pub mod guid;
pub mod utils;
pub mod hashing;

pub mod event_listener;

pub use bytes::*;
pub mod prelude;