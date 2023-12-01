use std::{hash::Hasher, ptr::copy_nonoverlapping, mem::{transmute, size_of}};

use super::Hasher128;

/// MD5 hash
/// 
/// Info can be found at: https://en.wikipedia.org/wiki/MD5
/// 
/// # Note
/// 
/// Unlike the specification, this implementation does not support individual bits, only full bytes,
/// as this would otherwise significantly slow down writing data to the internal buffer, 
/// since it needs to be copied byte-per-byte, to make sure they line up with the current offset in the buffer.
pub struct MD5 {
    block:     [u8; Self::BLOCK_SIZE],
    state:     [u32; Self::STATE_SIZE],
    num_bytes: u8,
    size:      u64,
}

impl MD5 {
    const BLOCK_SIZE: usize = 64;
    const STATE_SIZE: usize = 4;
    const LAST_BLOCK_SIZE: usize = Self::BLOCK_SIZE - 8;

    const SHIFTS: [u8; 16] = [
        7, 12, 17, 22,
        5,  9, 14, 20,
        4, 11, 16, 23,
        6, 10, 15, 21,
    ];

    const SINES: [u32; 64] = [
        0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee,
        0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
        0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be,
        0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
        0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa,
        0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
        0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed,
        0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
        0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c,
        0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
        0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05,
        0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
        0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039,
        0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
        0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1,
        0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391,
    ];

    pub fn new() -> Self {
        MD5 {
            block: [0; Self::BLOCK_SIZE],
            state: [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476],
            num_bytes: 0,
            size:      0
        }
    }
    fn hash_block(state: &mut [u32; Self::STATE_SIZE], block: &mut [u8; Self::BLOCK_SIZE]) {
        let mut a = state[0];
        let mut b = state[1]; 
        let mut c = state[2];
        let mut d = state[3];
        let block: &mut [u32; 16] = unsafe { transmute(block) };

        // Convert elements from little endian to machine endian for calculations
        for i in 0..16 {
            block[i] = u32::from_le(block[i]);
        }

        for i in 0..64 {
            let (f, g, h) = if i < 16 {
                (
                    (b & c) | (!b & d),
                    i & 0xF,
                    Self::SHIFTS[i & 0x3]
                )
            } else if i < 32 {
                (
                    (d & b) | (!d & c),
                    (5 * i + 1) & 0xF,
                    Self::SHIFTS[4 + (i & 0x3)]
                )
            } else if i < 48 {
                (
                    b ^ c ^ d,
                    (3 * i + 5) & 0xF,
                    Self::SHIFTS[8 + (i & 0x3)]
                )
            } else {
                (
                    c ^ (b | !d),
                    (7 * i) & 0xF,
                    Self::SHIFTS[12 + (i & 0x3)]
                )
            };

            let f = f.wrapping_add(a).wrapping_add(Self::SINES[i]).wrapping_add(block[g]);
            a = d;
            d = c;
            c = b;
            b = b.wrapping_add(f.rotate_left(h as u32));
        }

        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
    }
}

impl Hasher for MD5 {
    fn finish(&self) -> u64 {
        // Since MD5 is little endian, we assume that the buffer we get is in memory as
        // 00112233445566778899AABBCCDDEEFF
        // So we will just take the lower bytes and return them interpreted as little-endian
        let hash = self.finish128();
        let mut buf = [0; 8];
        unsafe { copy_nonoverlapping(hash.as_ptr(), buf.as_mut_ptr(), 8) };
        u64::from_le_bytes(buf)
    }

    fn write(&mut self, mut bytes: &[u8]) {
        self.size += bytes.len() as u64;

        let mut num_bytes = self.num_bytes as usize;
        if num_bytes + bytes.len() >= Self::BLOCK_SIZE {
            let mut space_left = Self::BLOCK_SIZE - num_bytes;
            while bytes.len() >= space_left {
                unsafe { copy_nonoverlapping(bytes.as_ptr(), self.block.as_mut_ptr().add(num_bytes), space_left) };

                Self::hash_block(&mut self.state, &mut self.block);

                bytes = &bytes[space_left..];
                num_bytes = 0;
                space_left = Self::BLOCK_SIZE;
            }
        } 

        // Copy the remainder into the buffer and set the current lenght
        unsafe { copy_nonoverlapping(bytes.as_ptr(), self.block.as_mut_ptr().add(num_bytes), bytes.len()) };
        self.num_bytes = (num_bytes + bytes.len()) as u8;
    }
}

impl Hasher128 for MD5 {
    fn finish128(&self) -> [u8; 16] {
        let mut num_bytes = self.num_bytes as usize;
        let mut block = [0; Self::BLOCK_SIZE];

        unsafe { copy_nonoverlapping(self.block.as_ptr(), block.as_mut_ptr(), num_bytes) };

        let mut state = self.state;
        
        if num_bytes > Self::LAST_BLOCK_SIZE {
            block[num_bytes] = 0x80;
            Self::hash_block(&mut state, &mut block);
            block = [0; Self::BLOCK_SIZE];
        } else {
            block[num_bytes] = 0x80;
        }

        let size_bytes: [u8; 8] = unsafe { transmute((self.size * 8).to_le()) };
        unsafe { copy_nonoverlapping(size_bytes.as_ptr(), block.as_mut_ptr().add(Self::LAST_BLOCK_SIZE), size_of::<u64>()) };
        Self::hash_block(&mut state, &mut block);

        unsafe { transmute(state) }
    }
}

#[cfg(test)]
mod tests {
    use std::hash::Hasher;

    use crate::hashing::Hasher128;

    use super::MD5;

    #[test]
    pub fn md5() {
        let mut md5 = MD5::new();
        let data = "".as_bytes();
        md5.write(&data);
        let hash = md5.finish128();
        let expected = [0xd4, 0x1d, 0x8c, 0xd9, 0x8f, 0x00, 0xb2, 0x04, 0xe9, 0x80, 0x09, 0x98, 0xec, 0xf8, 0x42, 0x7e];
        assert_eq!(hash, expected);
        
        // 43 bytes
        let mut md5 = MD5::new();
        let data = "The quick brown fox jumps over the lazy dog".as_bytes();
        md5.write(&data);
        let hash = md5.finish128();
        let expected = [0x9e, 0x10, 0x7d, 0x9d, 0x37, 0x2b, 0xb6, 0x82, 0x6b, 0xd8, 0x1d, 0x35, 0x42, 0xa4, 0x19, 0xd6];
        assert_eq!(hash, expected);

        // 44 bytes
        let mut md5 = MD5::new();
        let data = "The quick brown fox jumps over the lazy dog.".as_bytes();
        md5.write(&data);
        let hash = md5.finish128();
        let expected = [0xe4, 0xd9, 0x09, 0xc2, 0x90, 0xd0, 0xfb, 0x1c, 0xa0, 0x68, 0xff, 0xad, 0xdf, 0x22, 0xcb, 0xd0];
        assert_eq!(hash, expected);

        // 60 bytes (overflow into empty block with size)
        let mut md5 = MD5::new();
        let data = "Lorem ipsum dolor sit amet, consectetur adipiscing elit dui.".as_bytes();
        md5.write(&data);
        let hash = md5.finish128();
        let expected = [0x7b, 0x12, 0xa9, 0x84, 0xe9, 0x36, 0x35, 0xe1, 0x98, 0x2f, 0x28, 0x74, 0x81, 0x11, 0x1e, 0xee];
        assert_eq!(hash, expected);

        // 64 bytes (overflow into empty block starting with 1 and with size)
        let mut md5 = MD5::new();
        let data = "Lorem ipsum dolor sit amet, consectetur adipiscing elit blandit.".as_bytes();
        md5.write(&data);
        let hash = md5.finish128();
        let expected = [0x57, 0xbb, 0x63, 0xbe, 0xaa, 0xf0, 0x16, 0xbf, 0x69, 0x9d, 0x5b, 0x45, 0x51, 0x27, 0x07, 0x8c];
        assert_eq!(hash, expected);

        // 96 bytes
        let mut md5 = MD5::new();
        let data = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec vel nisi magna. Duis aliquam leo.".as_bytes();
        md5.write(&data);
        let hash = md5.finish128();
        let expected = [0x0c, 0xb2, 0xa6, 0x41, 0x7c, 0x3e, 0x0b, 0x4b, 0xe9, 0x76, 0x03, 0xc3, 0xc1, 0x3d, 0x1d, 0xad];
        assert_eq!(hash, expected);

        // 256 bytes
        let mut md5 = MD5::new();
        let data = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Aenean vel elit justo. Ut quis felis vitae nisi malesuada malesuada nec eu massa. Suspendisse fringilla nulla id tristique commodo. Pellentesque nec nisi ut elit pretium tincidunt quis non dolor dui.".as_bytes();
        md5.write(&data);
        let hash = md5.finish128();
        let expected = [0x48, 0x4c, 0x5b, 0xb8, 0xb0, 0xa4, 0x4a, 0x11, 0xfb, 0xb0, 0xaa, 0x8c, 0x61, 0x94, 0xa9, 0x97];
        assert_eq!(hash, expected);
    }
}

