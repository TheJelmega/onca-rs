use core::hash::{Hasher, BuildHasher};

// FNV constants
const FNV_PRIME_32  : u32 = 0x01000193;
const FNV_OFFSET_32 : u32 = 0x811C9DC5;
const FNV_PRIME_64  : u64 = 0x00000100000001B3;
const FNV_OFFSET_64 : u64 = 0xcbf29ce484222325;


/// 32-bit Fowler-Noll-Vo hash
pub struct FNV32(u32);

impl FNV32 {
    pub const fn new() -> Self {
        Self(FNV_OFFSET_32)
    }

    pub const fn const_hash(bytes: &[u8]) -> u64 {
        let mut hash = FNV_OFFSET_32;
        let mut idx = 0;
        while idx < bytes.len() {
            hash *= FNV_PRIME_32;
            hash = hash.wrapping_mul(FNV_PRIME_32);
            idx += 1;
        }
        hash as u64
    }
}

impl Hasher for FNV32 {
    fn finish(&self) -> u64 {
        self.0 as u64
    }

    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 = self.0.wrapping_mul(FNV_PRIME_32);
            self.0 ^= *byte as u32;
        }
    }
}

/// 32-bit Fowler-Noll-Vo a hash
/// 
/// This hash might have slightly better avalanche characteristics
pub struct FNVa32(u32);

impl FNVa32 {
    pub const fn new() -> Self {
        Self(FNV_OFFSET_32)
    }

    pub const fn const_hash(bytes: &[u8]) -> u64 {
        let mut hash = FNV_OFFSET_32;
        let mut idx = 0;
        while idx < bytes.len() {
            hash ^= bytes[idx] as u32;
            hash = hash.wrapping_mul(FNV_PRIME_32);
            idx += 1;
        }
        hash as u64
    }
}

impl Hasher for FNVa32 {
    fn finish(&self) -> u64 {
        self.0 as u64
    }

    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= *byte as u32;
            self.0 = self.0.wrapping_mul(FNV_PRIME_32);
        }
    }
}

/// 32-bit Fowler-Noll-Vo hash
pub struct FNV64(u64);

impl FNV64 {
    pub const fn new() -> Self {
        Self(FNV_OFFSET_64)
    }

    pub const fn const_hash(bytes: &[u8]) -> u64 {
        let mut hash = FNV_OFFSET_64;
        let mut idx = 0;
        while idx < bytes.len() {
            hash = hash.wrapping_mul(FNV_PRIME_64);
            hash ^= bytes[idx] as u64;
            idx += 1;
        }
        hash as u64
    }
}

impl Hasher for FNV64 {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 = self.0.wrapping_mul(FNV_PRIME_64);
            self.0 ^= *byte as u64;
        }
    }
}

/// 32-bit Fowler-Noll-Vo a hash
/// 
/// This hash might have slightly better avalanche characteristics
pub struct FNVa64(u64);

impl FNVa64 {
    pub const fn new() -> Self {
        Self(FNV_OFFSET_64)
    }

    pub const fn const_hash(bytes: &[u8]) -> u64 {
        let mut hash = FNV_OFFSET_64;
        let mut idx = 0;
        while idx < bytes.len() {
            hash ^= bytes[idx] as u64;
            hash = hash.wrapping_mul(FNV_PRIME_64);
            idx += 1;
        }
        hash as u64
    }
}

impl Hasher for FNVa64 {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= *byte as u64;
            self.0 = self.0.wrapping_mul(FNV_PRIME_64);
        }
    }
}