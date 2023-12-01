use criterion::{criterion_main, criterion_group, Criterion};

use std::hash::Hasher;
use onca_common::hashing::{FNV32, MD5};

const LOREM_IPSUM_128: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Suspendisse cursus, velit sed porta feugiat, metus nibh porta accumsan.";
const LOREM_IPSUM_1024: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Maecenas tincidunt lacus faucibus, ultricies tellus a, hendrerit nunc. In dignissim ante vel interdum rutrum. Fusce vel odio rhoncus, consequat arcu sed, mattis sem. Aenean lacus est, maximus eu nulla et, porta sollicitudin nulla. Praesent facilisis faucibus sapien et laoreet. Sed euismod elit vitae suscipit vestibulum. Curabitur iaculis erat lectus, at iaculis velit sodales ac. Aenean ante arcu, lobortis vel laoreet in, accumsan ut odio. Nunc ac congue justo. Etiam aliquet ullamcorper tortor, non aliquam lacus finibus sed. Praesent non lacinia est. Aenean sit amet nisl at mi fermentum aliquet ac vitae felis. Aliquam sit amet dictum nisi. Duis dignissim tempor viverra. Fusce tempus orci quis egestas rhoncus. Integer gravida metus vitae blandit pretium. Nulla a pulvinar arcu. Suspendisse consequat finibus ultricies. Suspendisse potenti. Integer eget sollicitudin est, eu tincidunt velit. Cras interdum nisi eget molestie dictum. Nullam mollis tortor nec ex.";

fn fnv_benchmark(c: &mut Criterion) {
    c.bench_function("fnv-1 32-bit: 4 bytes", |b| b.iter(|| {
        let mut hasher = FNV32::new();
        hasher.write_u32(0x12345678);
        hasher.finish()
    }));
    
    c.bench_function("fnv-1 32-bit: 8 bytes", |b| b.iter(|| {
        let mut hasher = FNV32::new();
        hasher.write_u64(0x123456789ABCDEF);
        hasher.finish()
    }));

    c.bench_function("fnv-1 32-bit: 128 bytes", |b| b.iter(|| {
        let mut hasher = FNV32::new();
        hasher.write(LOREM_IPSUM_128.as_bytes());
        hasher.finish()
    }));

    c.bench_function("fnv-1 32-bit: 1024 bytes", |b| b.iter(|| {
        let mut hasher = FNV32::new();
        hasher.write(LOREM_IPSUM_1024.as_bytes());
        hasher.finish()
    }));
}


fn md5_benchmark(c: &mut Criterion) {
    c.bench_function("md5: 4 bytes", |b| b.iter(|| {
        let mut hasher = MD5::new();
        hasher.write_u32(0x12345678);
        hasher.finish()
    }));
    
    c.bench_function("md5: 8 bytes", |b| b.iter(|| {
        let mut hasher = MD5::new();
        hasher.write_u64(0x123456789ABCDEF);
        hasher.finish()
    }));

    c.bench_function("md5: 128 bytes", |b| b.iter(|| {
        let mut hasher = MD5::new();
        hasher.write(LOREM_IPSUM_128.as_bytes());
        hasher.finish()
    }));

    c.bench_function("md5: 1024 bytes", |b| b.iter(|| {
        let mut hasher = MD5::new();
        hasher.write(LOREM_IPSUM_1024.as_bytes());
        hasher.finish()
    }));
}

fn sha1_benchmark(c: &mut Criterion) {
    c.bench_function("sha-1: 4 bytes", |b| b.iter(|| {
        let mut hasher = MD5::new();
        hasher.write_u32(0x12345678);
        hasher.finish()
    }));
    
    c.bench_function("sha-1: 8 bytes", |b| b.iter(|| {
        let mut hasher = MD5::new();
        hasher.write_u64(0x123456789ABCDEF);
        hasher.finish()
    }));

    c.bench_function("sha-1: 128 bytes", |b| b.iter(|| {
        let mut hasher = MD5::new();
        hasher.write(LOREM_IPSUM_128.as_bytes());
        hasher.finish()
    }));

    c.bench_function("sha-1: 1024 bytes", |b| b.iter(|| {
        let mut hasher = MD5::new();
        hasher.write(LOREM_IPSUM_1024.as_bytes());
        hasher.finish()
    }));
}

criterion_group!(hash, /*fnv_benchmark, md5_benchmark,*/ sha1_benchmark);
criterion_main!(hash);