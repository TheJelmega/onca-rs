#![allow(unstable_features)]
#![feature(allocator_api)]

mod hash;
mod dynarr;


use criterion::criterion_main;


criterion_main!(dynarr::dynarr);