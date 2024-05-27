#![allow(unused)]
#![feature(allocator_api)]


use std::{
    alloc::Global,
    vec::Vec,
};

use criterion::{criterion_group, Criterion, black_box};

use onca_common::{collections::*, mem::{AllocStorage, StorageSingleSlicedWrapper}};

fn dynarr_new(c: &mut Criterion) {
    c.bench_function("DynArr::new", |b| b.iter(|| {
        DynArr::<u32>::new()
    }));
    c.bench_function("Vec::new", |b| b.iter(|| {
        Vec::<u32>::new()
    }));
    c.bench_function("DynArr::with_capacity(64)", |b| b.iter(|| {
        DynArr::<u32>::with_capacity(64)
    }));
    c.bench_function("Vec::with_capacity(64)", |b| b.iter(|| {
        Vec::<u32>::with_capacity(64)
    }));
}

fn dynarr_reserve(c: &mut Criterion) {
    c.bench_function("DynArr::reserve", |b| b.iter(|| {
        let mut arr = DynArr::<u32>::new();
        arr.reserve(32);
        arr
    }));
    c.bench_function("DynArr::reserve", |b| b.iter(|| {
        let mut arr = Vec::<u32>::new();
        arr.reserve(32);
        arr
    }));
}

fn dynarr_push(c: &mut Criterion) {
    c.bench_function("DynArr::push(100) no reserve", |b| b.iter(|| {
        let mut arr = DynArr::<u32>::new();
        for i in 0..100 {
            arr.push(i);
        }
        arr
    }));
    c.bench_function("DynArr::push(100) reserve", |b| b.iter(|| {
        let mut arr = DynArr::<u32>::new();
        arr.reserve(100);
        for i in 0..100 {
            arr.push(i);
        }
        arr
    }));

    c.bench_function("Vec::push(100) no reserve", |b| b.iter(|| {
        let mut arr = Vec::<u32>::new();
        for i in 0..100 {
            arr.push(i);
        }
        arr
    }));
    c.bench_function("Vec::push(100) reserve", |b| b.iter(|| {
        let mut arr = Vec::<u32>::new();
        arr.reserve(100);
        for i in 0..100 {
            arr.push(i);
        }
        arr
    }));
}

fn dynarr_index(c: &mut Criterion) {
    let arr = dynarr![5; 100];
    c.bench_function("DynArr::index(100)", |b| b.iter(|| {
        for i in 0..100 {
            black_box(arr[i]);
        }
    }));

    let vbuf = vec![5; 100];
    c.bench_function("Vec::index(100)", |b| b.iter(|| {
        for i in 0..100 {
            black_box(vbuf[i]);
        }
    }));
}

criterion_group!(dynarr,
    // dynarr_new,
    // dynarr_reserve,
    //dynarr_push,
    dynarr_index
);