use std::ops::*;


#[derive(Clone, Copy, Debug)]
pub struct BitSet<const COUNT: usize> 
// We need a constraint here to make the compiler happy
where
    [u64; (COUNT + 63) / 64]:
{
    bits: [u64; (COUNT + 63) / 64],
}

impl<const COUNT: usize> BitSet<COUNT> where
    [u64; (COUNT + 63) / 64]:
{
    /// Number of bits in the bitset
    pub const BIT_COUNT : usize = COUNT; 

    /// Number of bytes needed to store all bits
    /// 
    /// # NOTE
    /// 
    /// This does not reflect the size of the bitset, as for alignment reasons, the bitset will always be a multiple of 64 bits, or 8 bytes
    pub const BYTE_COUNT : usize = (COUNT + 7) / 8;

    /// Number of full u64s used
    pub const NUM_FULL_U64 : usize = COUNT / 64;

    pub fn new() -> Self {
        Self { bits: [0; (COUNT + 63) / 64] }
    }

    /// Set the given bit.
    pub fn set(&mut self, idx: usize, set: bool) {
        let (byte_idx, bit_idx) = Self::indices(idx);
        let mask = 1u64 << bit_idx;
        if set {
            self.bits[byte_idx] |= mask;
        } else {
            self.bits[byte_idx] &= !mask;
        };
    }

    /// Enable a bit
    pub fn enable(&mut self, idx: usize) {
        let (byte_idx, bit_idx) = Self::indices(idx);
        self.bits[byte_idx] |= 1u64 << bit_idx;
    }

    /// Disable a bit
    pub fn disable(&mut self, idx: usize) {
        let (byte_idx, bit_idx) = Self::indices(idx);
        self.bits[byte_idx] &= !(1u64 << bit_idx);
    }

    /// Get the given bit.
    pub fn get(&self, idx: usize) -> bool {
        let (byte_idx, bit_idx) = Self::indices(idx);
        ((self.bits[byte_idx] >> bit_idx) & 0x1) != 0
    }

    /// Flip the value of the given bit.  
    pub fn flip(&mut self, idx: usize) -> bool {
        let (byte_idx, bit_idx) = Self::indices(idx);
        let mask = 1u64 << bit_idx;
        self.bits[byte_idx] ^= mask;
        self.bits[byte_idx] & mask == mask
    }

    /// Check if all bits are set
    pub fn all(&self) -> bool {
        let bits_left : usize = Self::BIT_COUNT & 63;
        let last_bits_mask = 0xFFFF_FFFF_FFFF_FFFF << (63 - bits_left);

        let last_idx = self.bits.len() - 1;
        for i in 0..last_idx {
            if self.bits[i] != u64::MAX {
                return false;
            }
        }
        self.bits[last_idx] == last_bits_mask
    }

    /// Check if any bits are set
    pub fn any(&self) -> bool {
        let (front, back) = self.get_u64_and_bytes();
        for bytes in front {
            if *bytes != 0 {
                return true;
            }
        }
        for byte in back {
            if *byte != 0 {
                return true;
            }
        }
        false
    }

    /// Check if no bits are set
    pub fn none(&self) -> bool {
        !self.any()
    }

    /// Count the number of bits set to 1
    pub fn count_ones(&self) -> usize {
        let mut acc = 0usize;
        let (front, back) = self.get_u64_and_bytes();
        for bytes in front {
            acc += bytes.count_ones() as usize;
        }
        for byte in back {
            acc += byte.count_ones() as usize;
        }
        acc
    }

    /// Count the number of bits set to 0
    pub fn count_zeros(&self) -> usize {
        let mut acc = 0usize;
        let (front, back) = self.get_u64_and_bytes();
        for bytes in front {
            acc += bytes.count_zeros() as usize;
        }
        for byte in back {
            acc += byte.count_zeros() as usize;
        }
        acc
    }

    /// Clear all bits to 0
    pub fn clear(&mut self) {
        for bytes in &mut self.bits {
            *bytes = 0;
        }
    }

    /// Set all bits
    pub fn set_all(&mut self) {
        let bits_left : usize = Self::BIT_COUNT & 63;
        let last_bits_mask = 0xFFFF_FFFF_FFFF_FFFF << (63 - bits_left);

        let last_idx = self.bits.len() - 1;
        for i in 0..last_idx {
            self.bits[i] = u64::MAX;
        }
        self.bits[last_idx] = last_bits_mask;
    }
    
    /// Get an iterator to the bitset
    pub fn iter(&self) -> Iter<'_, COUNT> {
        Iter { bitset: &self, idx: 0, end: COUNT }
    }

    /// Get an iterator over all bits that are set to 1
    pub fn iter_ones(&self) -> IterOnes<COUNT> {
        IterOnes { bitset: &self, idx: 0, end: COUNT }
    }

    /// Get an iterator over all bits that are set to 1
    pub fn iter_zeros(&self) -> IterZeros<COUNT> {
        IterZeros { bitset: &self, idx: 0, end: COUNT }
    }

    #[inline(always)]
    fn indices(idx: usize) -> (usize, usize) {
        debug_assert!(idx < COUNT);
        (
            idx / 64,
            63 - (idx & 63)
        )
    }

    fn from_u8_arr(bits: [u8; (COUNT + 7) / 8]) -> Self {
        let mut res = BitSet::new();
        let bytes_ptr = res.bits.as_mut_ptr() as *mut u8;
        for i in 0..Self::BYTE_COUNT {
            unsafe { *bytes_ptr.add(i) = bits[i] };
        }
        res
    }

    fn get_u64_and_bytes_mut(&mut self) -> (&mut [u64], &mut [u8]) {
        let back_start = COUNT - Self::NUM_FULL_U64 * 8;
        let back_len = COUNT & 63;
        
        let back = unsafe { core::slice::from_raw_parts_mut(self.bits.as_mut_ptr().add(Self::NUM_FULL_U64) as *mut u8, back_len) };
        let front = &mut self.bits[..Self::NUM_FULL_U64];
        (front, back)
    }

    fn get_u64_and_bytes(&self) -> (&[u64], &[u8]) {
        let back_start = COUNT - Self::NUM_FULL_U64 * 8;
        let back_len = COUNT & 63;
        
        let back = unsafe { core::slice::from_raw_parts(self.bits.as_ptr().add(Self::NUM_FULL_U64) as *const u8, back_len) };
        let front = &self.bits[..Self::NUM_FULL_U64];
        (front, back)
    }
}

impl<const COUNT: usize> Not for &BitSet<COUNT> where
    [u64; (COUNT + 63) / 64]:
{
    type Output = BitSet<COUNT>;

    fn not(self) -> Self::Output {
        let mut res = BitSet::<COUNT>::new();
        for i in 0..self.bits.len() {
            res.bits[i] = !self.bits[i];
        }
        res
    }
}

impl<const COUNT: usize> BitOr for &BitSet<COUNT> where
    [u64; (COUNT + 63) / 64]:
{
    type Output = BitSet<COUNT>;

    fn bitor(self, rhs: Self) -> Self::Output {
        let mut res = BitSet::<COUNT>::new();
        for i in 0..self.bits.len() {
            res.bits[i] = self.bits[i] | rhs.bits[i];
        }
        res
    }
}

impl<const COUNT: usize> BitXor for &BitSet<COUNT> where
    [u64; (COUNT + 63) / 64]:
{
    type Output = BitSet<COUNT>;

    fn bitxor(self, rhs: Self) -> Self::Output {
        let mut res = BitSet::<COUNT>::new();
        for i in 0..self.bits.len() {
            res.bits[i] = self.bits[i] ^ rhs.bits[i];
        }
        res
    }
}

impl<const COUNT: usize> BitAnd for &BitSet<COUNT> where
    [u64; (COUNT + 63) / 64]:
{
    type Output = BitSet<COUNT>;

    fn bitand(self, rhs: Self) -> Self::Output {
        let mut res = BitSet::<COUNT>::new();
        for i in 0..self.bits.len() {
            res.bits[i] = self.bits[i] & rhs.bits[i];
        }
        res
    }
}

impl<const COUNT: usize> BitOrAssign<&Self> for BitSet<COUNT> where
    [u64; (COUNT + 63) / 64]:
{
    fn bitor_assign(&mut self, rhs: &Self) {
        for i in 0..self.bits.len() {
            self.bits[i] |= rhs.bits[i];
        }
    }
}

impl<const COUNT: usize> BitXorAssign<&Self> for BitSet<COUNT> where
    [u64; (COUNT + 63) / 64]:
{
    fn bitxor_assign(&mut self, rhs: &Self) {
        for i in 0..self.bits.len() {
            self.bits[i] ^= rhs.bits[i];
        }
    }
}

impl<const COUNT: usize> BitAndAssign<&Self> for BitSet<COUNT> where
    [u64; (COUNT + 63) / 64]:
{
    fn bitand_assign(&mut self, rhs: &Self) {
        for i in 0..self.bits.len() {
            self.bits[i] &= rhs.bits[i];
        }
    }
}

impl<const COUNT: usize> IntoIterator for BitSet<COUNT> where
    [u64; (COUNT + 63) / 64]:
{
    type Item = bool;
    type IntoIter = IntoIter<COUNT>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter{ bitset: self, idx: 0, end: COUNT }
    }
}

impl<'a, const COUNT: usize> IntoIterator for &'a BitSet<COUNT> where
    [u64; (COUNT + 63) / 64]:
{
    type Item = bool;
    type IntoIter = Iter<'a, COUNT>;

    fn into_iter(self) -> Self::IntoIter {
        Iter{ bitset: self, idx: 0, end: COUNT }
    }
}

//--------------------------------------------------------------

pub struct Iter<'a, const COUNT: usize> where
    [u64; (COUNT + 63) / 64]:
{
    bitset : &'a BitSet<COUNT>,
    idx    : usize,
    end    : usize
}

impl<const COUNT: usize> Iterator for Iter<'_, COUNT> where
    [u64; (COUNT + 63) / 64]:
{
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.end {
            let idx = self.idx;
            self.idx -= 1;
            Some(self.bitset.get(idx))
        } else {
            None
        }
    }
}

impl<const COUNT: usize> DoubleEndedIterator for Iter<'_, COUNT> where
    [u64; (COUNT + 63) / 64]:
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.idx < self.end {
            self.end -= 1;
            Some(self.bitset.get(self.end))
        } else {
            None
        }
    }
}

impl<const COUNT: usize> ExactSizeIterator for Iter<'_, COUNT> where
    [u64; (COUNT + 63) / 64]:
{
    fn len(&self) -> usize {
        COUNT
    }
}

//--------------------------------------------------------------

pub struct IterOnes<'a, const COUNT: usize> where
    [u64; (COUNT + 63) / 64]:
{
    bitset : &'a BitSet<COUNT>,
    idx    : usize,
    end    : usize
}

impl<const COUNT: usize> Iterator for IterOnes<'_, COUNT> where
    [u64; (COUNT + 63) / 64]:
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        while self.idx < self.end {
            let idx = self.idx;
            self.idx += 1;
            if self.bitset.get(idx) {
                return Some(idx);
            }
        }
        None
    }
}

impl<const COUNT: usize> DoubleEndedIterator for IterOnes<'_, COUNT> where
    [u64; (COUNT + 63) / 64]:
{
    fn next_back(&mut self) -> Option<Self::Item> {
        while self.idx < self.end {
            self.end -= 1;
            if self.bitset.get(self.end) {
                return Some(self.end);
            }
        }
        None
    }
}

impl<const COUNT: usize> ExactSizeIterator for IterOnes<'_, COUNT> where
    [u64; (COUNT + 63) / 64]:
{
    fn len(&self) -> usize {
        COUNT
    }
}

//--------------------------------------------------------------

pub struct IterZeros<'a, const COUNT: usize> where
    [u64; (COUNT + 63) / 64]:
{
    bitset: &'a BitSet<COUNT>,
    idx:    usize,
    end:    usize
}

impl<const COUNT: usize> Iterator for IterZeros<'_, COUNT> where
    [u64; (COUNT + 63) / 64]:
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        while self.idx < self.end {
            let idx = self.idx;
            self.idx += 1;
            if !self.bitset.get(self.idx) {
                return Some(idx);
            }
        }
        None
    }
}

impl<const COUNT: usize> DoubleEndedIterator for IterZeros<'_, COUNT> where
    [u64; (COUNT + 63) / 64]:
{
    fn next_back(&mut self) -> Option<Self::Item> {
        while self.idx < self.end {
            self.end -= 1;
            if !self.bitset.get(self.end) {
                return Some(self.end);
            }
        }
        None
    }
}

impl<const COUNT: usize> ExactSizeIterator for IterZeros<'_, COUNT> where
    [u64; (COUNT + 63) / 64]:
{
    fn len(&self) -> usize {
        COUNT
    }
}

//--------------------------------------------------------------

pub struct IntoIter<const COUNT: usize> where
    [u64; (COUNT + 63) / 64]:
{
    bitset: BitSet<COUNT>,
    idx:    usize,
    end:    usize,
}

impl<const COUNT: usize> Iterator for IntoIter<COUNT>  where
    [u64; (COUNT + 63) / 64]:
{
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.end {
            Some(self.bitset.get(self.idx))
        } else {
            None
        }
    }
}

impl<const COUNT: usize> DoubleEndedIterator for IntoIter<COUNT> where
    [u64; (COUNT + 63) / 64]:
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.idx < self.end {
            self.end -= 1;
            Some(self.bitset.get(self.end))
        } else {
            None
        }
    }
}

impl<const COUNT: usize> ExactSizeIterator for IntoIter<COUNT> where
    [u64; (COUNT + 63) / 64]:
{
    fn len(&self) -> usize {
        COUNT
    }
}
