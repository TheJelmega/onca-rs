

mod fnv;
use std::hash::Hasher;

pub use fnv::*;

mod md5;
pub use md5::*;

mod sha1;
pub use sha1::*;




pub trait Hasher128: Hasher {
    /// returns the hash value for the values written so far.
    /// 
    /// Depsite its name, the method does not reset the hasher's internal state.
    /// Additinal `write`s will continue from the current value.
    /// If you need to start a fresh hash value, you will have to create a new hasher. 
    fn finish128(&self) -> [u8; 16];
}

pub trait Hasher160: Hasher {
    /// returns the hash value for the values written so far.
    /// 
    /// Depsite its name, the method does not reset the hasher's internal state.
    /// Additinal `write`s will continue from the current value.
    /// If you need to start a fresh hash value, you will have to create a new hasher. 
    fn finish160(&self) -> [u8; 20];
}