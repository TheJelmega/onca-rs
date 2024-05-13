#![allow(unused)]
#![allow(incomplete_features)]

// NOTE: Currently, we have a lot of features we need for the implementations, we should be looking to reduce this in the future.
//            While not having to use unstable features might not be possible, we might be able to reduce the amount we need

#![feature(generic_const_exprs)]
#![feature(specialization)]
#![feature(unsize)]
#![feature(const_trait_impl)]
#![feature(effects)]

#![feature(allocator_api)]
#![feature(alloc_layout_extra)]
#![feature(ptr_alignment_type)]
#![feature(strict_provenance)]
#![feature(ptr_metadata)]
#![feature(const_alloc_error)]
#![feature(layout_for_ptr)]
#![feature(const_try)]
#![feature(const_alloc_layout)]
#![feature(const_mut_refs)]
#![feature(const_ptr_write)]
#![feature(const_intrinsic_copy)]
#![feature(const_refs_to_cell)]
#![feature(const_slice_from_raw_parts_mut)]
#![feature(const_ptr_as_ref)]
#![feature(hint_assert_unchecked)]

#![feature(vec_split_at_spare)]
#![feature(can_vector)]

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
pub mod index_handle;

pub mod event_listener;

pub use bytes::*;
pub mod prelude;