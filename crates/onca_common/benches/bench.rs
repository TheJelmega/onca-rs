#![allow(unstable_features)]
#![feature(allocator_api)]

mod hash;


use criterion::criterion_main;


criterion_main!(hash::hash);