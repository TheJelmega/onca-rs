use std::{hash::Hasher, ptr::copy_nonoverlapping, mem::{transmute, size_of}};

use super::Hasher160;

/// SHA-1 hash
/// 
/// Info can be found at: https://en.wikipedia.org/wiki/SHA-1
/// 
/// # Note
/// 
/// Unlike the specification, this implementation does not support individual bits, only full bytes,
/// as this would otherwise significantly slow down writing data to the internal buffer, 
/// since it needs to be copied byte-per-byte, to make sure they line up with the current offset in the buffer.
pub struct SHA1 {
    block:     [u8; Self::BLOCK_SIZE],
    state:     [u32; Self::STATE_SIZE],
    num_bytes: u8,
    size:      usize,
}

impl SHA1 {
    const BLOCK_SIZE: usize = 64;
    const STATE_SIZE: usize = 5;
    const LAST_BLOCK_SIZE: usize = Self::BLOCK_SIZE - 8;

    /// Create a new SHA-1 hasher
    pub fn new() -> SHA1 {
        SHA1 {
            block: [0; Self::BLOCK_SIZE],
            state: [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0],
            num_bytes: 0,
            size: 0,
        }
    }

    fn hash_block(state: &mut [u32; Self::STATE_SIZE], block: &[u8; Self::BLOCK_SIZE]) {
        let block: [u32; 16] = unsafe { transmute(*block) };

        // Convert elements from big endian to machine endian for calculations
        let mut w = [0; 80];
        for i in 0..16 {
            w[i] = u32::from_be(block[i]);
        }

        // Extend 16 32-bit words into 80 32-bit words
        for i in 16..80 {
            // NOTE: SHA-0 does not have the rotate_left(1)
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }

        let mut a = state[0];
        let mut b = state[1];
        let mut c = state[2];
        let mut d = state[3];
        let mut e = state[4];

        for i in 0..80 {
            let (f, k) = if i < 20 {
                (
                    (b & c) | (!b & d),
                    0x5A827999
                )
            } else if i < 40 {
                (
                    b ^ c ^ d,
                    0x6ED9EBA1
                )
            } else if i < 60 {
                (
                    (b & c) | (b & d) | (c & d),
                    0x8F1BBCDC
                )
            } else {
                (
                    b ^ c ^ d,
                    0xCA62C1D6
                )
            };

            let tmp = a.rotate_left(5).wrapping_add(f).wrapping_add(e).wrapping_add(k).wrapping_add(w[i]);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = tmp;
        }

        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
        state[4] = state[4].wrapping_add(e);
    }
}

impl Hasher for SHA1 {
    fn finish(&self) -> u64 {
        // Since SHA-1 is big endian, we assume that the buffer we get is in memory as
        // 33221100FFEEDDCCBBAA99887766554433221100
        // Se we will just take the upper bytes and return them interpreted as big-endian
        let state = self.finish160();
        let mut buf = [0; 8];
        unsafe { copy_nonoverlapping(state.as_ptr().offset(12), buf.as_mut_ptr(), 8) };
        u64::from_be_bytes(buf)
    }

    fn write(&mut self, mut bytes: &[u8]) {
        self.size += bytes.len();

        let mut num_bytes = self.num_bytes as usize;
        if num_bytes + bytes.len() >= Self::BLOCK_SIZE {
            let mut space_left = Self::BLOCK_SIZE - num_bytes;
            while bytes.len() >= space_left {
                unsafe { copy_nonoverlapping(bytes.as_ptr(), self.block.as_mut_ptr().add(num_bytes), space_left) };

                Self::hash_block(&mut self.state, &self.block);        

                bytes = &bytes[space_left..];
                num_bytes = 0;
                space_left = Self::BLOCK_SIZE;
            }

            // Final write to the current buffer for curren data (at start of the block)
            unsafe { copy_nonoverlapping(bytes.as_ptr(), self.block.as_mut_ptr(), bytes.len()) };
        } else {
            // Since we don't hit the buffer boundary yet, just copy into the buffer
        }

        // Copy the remainder into the buffer and set the current lenght
        unsafe { copy_nonoverlapping(bytes.as_ptr(), self.block.as_mut_ptr().add(num_bytes), bytes.len()) };
        self.num_bytes = (num_bytes + bytes.len()) as u8;
    }

    // Overloads to ensure Big-Endianess

    fn write_u8(&mut self, i: u8) {
        self.write(&[i])
    }

    fn write_u16(&mut self, i: u16) {
        self.write(&i.to_be_bytes())
    }

    fn write_u32(&mut self, i: u32) {
        self.write(&i.to_be_bytes())
    }

    fn write_u64(&mut self, i: u64) {
        self.write(&i.to_be_bytes())
    }

    fn write_u128(&mut self, i: u128) {
        self.write(&i.to_be_bytes())
    }

    fn write_usize(&mut self, i: usize) {
        self.write(&i.to_be_bytes())
    }

    fn write_i8(&mut self, i: i8) {
        self.write_u8(i as u8)
    }

    fn write_i16(&mut self, i: i16) {
        self.write_u16(i as u16)
    }

    fn write_i32(&mut self, i: i32) {
        self.write_u32(i as u32)
    }

    fn write_i64(&mut self, i: i64) {
        self.write_u64(i as u64)
    }

    fn write_i128(&mut self, i: i128) {
        self.write_u128(i as u128)
    }

    fn write_isize(&mut self, i: isize) {
        self.write_usize(i as usize)
    }
}

impl Hasher160 for SHA1 {
    fn finish160(&self) -> [u8; 20] {
        let mut num_bytes = self.num_bytes as usize;
        let mut block = [0; Self::BLOCK_SIZE];

        unsafe { copy_nonoverlapping(self.block.as_ptr(), block.as_mut_ptr(), num_bytes) };

        let mut state = self.state;

        if num_bytes > Self::LAST_BLOCK_SIZE {
            block[num_bytes] = 0x80;
            Self::hash_block(&mut state, &block);
            block = [0; Self::BLOCK_SIZE];
        } else {
            block[num_bytes] = 0x80;
        }

        let size_bytes: [u8; 8] = unsafe { transmute((self.size * 8).to_be()) };
        unsafe { copy_nonoverlapping(size_bytes.as_ptr(), block.as_mut_ptr().add(Self::LAST_BLOCK_SIZE), size_of::<u64>()) };
        Self::hash_block(&mut state, &block);

        // Convert result into big-endian
        let tmp = state.map(|val| val.to_be());
        unsafe { transmute(tmp) }
    }
}


#[cfg(test)]
mod tests {
    use std::hash::Hasher;
    use crate::hashing::Hasher160;

    use super::SHA1;


    #[test]
    fn sha1() {
        let mut sha1 = SHA1::new();
        let data = "".as_bytes();
        sha1.write(data);
        let hash = sha1.finish160();
        // result in big-endian: da39a3ee5e6b4b0d3255bfef95601890afd80709
        let expected = [0xda, 0x39, 0xa3, 0xee, 0x5e, 0x6b, 0x4b, 0x0d, 0x32, 0x55, 0xbf, 0xef, 0x95, 0x60, 0x18, 0x90, 0xaf, 0xd8, 0x07, 0x09];
        assert_eq!(hash, expected);
        
        // 43 bytes
        let mut sha1 = SHA1::new();
        let data = "The quick brown fox jumps over the lazy dog".as_bytes();
        sha1.write(data);
        let hash = sha1.finish160();
        // result in big-endian: 2fd4e1c67a2d28fced849ee1bb76e7391b93eb12
        let expected = [0x2f, 0xd4, 0xe1, 0xc6, 0x7a, 0x2d, 0x28, 0xfc, 0xed, 0x84, 0x9e, 0xe1, 0xbb, 0x76, 0xe7, 0x39, 0x1b, 0x93, 0xeb, 0x12];
        assert_eq!(hash, expected);

        // 43 bytes
        let mut sha1 = SHA1::new();
        let data = "The quick brown fox jumps over the lazy cog".as_bytes();
        sha1.write(data);
        let hash = sha1.finish160();
        // result in big-endian: de9f2c7fd25e1b3afad3e85a0bd17d9b100db4b3
        let expected = [0xde, 0x9f, 0x2c, 0x7f, 0xd2, 0x5e, 0x1b, 0x3a, 0xfa, 0xd3, 0xe8, 0x5a, 0x0b, 0xd1, 0x7d, 0x9b, 0x10, 0x0d, 0xb4, 0xb3];
        assert_eq!(hash, expected);

        // 60 bytes (overflow into empty block with size)
        let mut sha1 = SHA1::new();
        let data = "Lorem ipsum dolor sit amet, consectetur adipiscing elit dui.".as_bytes();
        sha1.write(&data);
        let hash = sha1.finish160();
        // result in big-endian: 40b6681dc4deed037a061acd5d319853499332ea
        let expected = [0x40, 0xb6, 0x68, 0x1d, 0xc4, 0xde, 0xed, 0x03, 0x7a, 0x06, 0x1a, 0xcd, 0x5d, 0x31, 0x98, 0x53, 0x49, 0x93, 0x32, 0xea];
        assert_eq!(hash, expected);

        // 64 bytes (overflow into empty block starting with 1 and with size)
        let mut sha1 = SHA1::new();
        let data = "Lorem ipsum dolor sit amet, consectetur adipiscing elit blandit.".as_bytes();
        sha1.write(&data);
        let hash = sha1.finish160();
        // result in big-endian: 1eced717eb2267d70427b41073132f6610fce7ba
        let expected = [0x1e, 0xce, 0xd7, 0x17, 0xeb, 0x22, 0x67, 0xd7, 0x04, 0x27, 0xb4, 0x10, 0x73, 0x13, 0x2f, 0x66, 0x10, 0xfc, 0xe7, 0xba];
        assert_eq!(hash, expected);

        // 96 bytes
        let mut sha1 = SHA1::new();
        let data = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec vel nisi magna. Duis aliquam leo.".as_bytes();
        sha1.write(&data);
        let hash = sha1.finish160();
        // result in big-endian: 7648ee02b7e49618ff4522558c6d7edc59171f22
        let expected = [0x76, 0x48, 0xee, 0x02, 0xb7, 0xe4, 0x96, 0x18, 0xff, 0x45, 0x22, 0x55, 0x8c, 0x6d, 0x7e, 0xdc, 0x59, 0x17, 0x1f, 0x22];
        assert_eq!(hash, expected);

        // 256 bytes
        let mut sha1 = SHA1::new();
        let data = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Aenean vel elit justo. Ut quis felis vitae nisi malesuada malesuada nec eu massa. Suspendisse fringilla nulla id tristique commodo. Pellentesque nec nisi ut elit pretium tincidunt quis non dolor dui.".as_bytes();
        sha1.write(&data);
        let hash = sha1.finish160();
        // result in big-endian: e478c993ca89ec1abbab98a6f1f291be4f841c88
        let expected = [0xe4, 0x78, 0xc9, 0x93, 0xca, 0x89, 0xec, 0x1a, 0xbb, 0xab, 0x98, 0xa6, 0xf1, 0xf2, 0x91, 0xbe, 0x4f, 0x84, 0x1c, 0x88];
        assert_eq!(hash, expected);
    }
}