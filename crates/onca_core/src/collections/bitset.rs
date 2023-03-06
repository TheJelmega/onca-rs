use std::ops::*;


#[derive(Clone, Copy)]
pub struct BitSet<const COUNT: usize> 
// We need a constraint here to make the compiler happy
where
    [u8; (COUNT + 7) / 8]:
{
    bits: [u8; (COUNT + 7) / 8],
}

impl<const COUNT: usize> BitSet<COUNT>
where
    [u8; (COUNT + 7) / 8]:,
    [u64; COUNT / 64]:
{
    /// Number of bits in the bitset
    pub const BIT_COUNT : usize = COUNT; 

    /// Number of bytes needed to store all bits
    pub const BYTE_COUNT : usize = (COUNT + 7) / 8;

    const U64_COUNT : usize = COUNT / 64;
    const U8_START : usize = Self::U64_COUNT * 8;

    pub fn new() -> Self {
        Self { bits: [0u8; (COUNT + 7) / 8] }
    }

    /// Set the given bit.
    pub fn set(&mut self, idx: usize, set: bool) {
        let (byte_idx, bit_idx) = Self::indices(idx);
        let mask = 1u8 << bit_idx;
        if set {
            self.bits[byte_idx] |= mask;
        } else {
            self.bits[byte_idx] &= !mask;
        };
    }

    /// Enable a bit
    pub fn enable(&mut self, idx: usize) {
        let (byte_idx, bit_idx) = Self::indices(idx);
        self.bits[byte_idx] |= 1u8 << bit_idx;
    }

    /// Disable a bit
    pub fn disable(&mut self, idx: usize) {
        let (byte_idx, bit_idx) = Self::indices(idx);
        self.bits[byte_idx] &= !(1u8 << bit_idx);
    }

    /// Get the given bit.
    pub fn get(&self, idx: usize) -> bool {
        let (byte_idx, bit_idx) = Self::indices(idx);
        ((self.bits[byte_idx] >> bit_idx) & 0x1) != 0
    }

    /// Flip the value of the given bit.  
    pub fn flip(&mut self, idx: usize) -> bool {
        let (byte_idx, bit_idx) = Self::indices(idx);
        let mask = 1u8 << bit_idx;
        self.bits[byte_idx] ^= mask;
        self.bits[byte_idx] & mask == mask
    }

    /// Check if all bits are set
    pub fn all(&self) -> bool {
        // Can't use const here, so let will have to do, and let the compiler optimize it
        let full_bytes : usize = Self::BIT_COUNT / 8;
        let bits_left : usize = Self::BIT_COUNT & 0x7;

        let u64_arr = self.bits_as_u64();
        for i in 0..Self::U64_COUNT {
            if u64_arr[i] != u64::MAX {
                return false;
            }
        }

        for i in Self::U8_START..full_bytes {
            if self.bits[i] != 0xFF {
                return false;
            }
        }
        if bits_left != 0 {
            self.bits[full_bytes] == 0xFF << bits_left
        } else {
            true
        }
    }

    /// Check if any bits are set
    pub fn any(&self) -> bool  {
        let u64_arr = self.bits_as_u64();
        for i in 0..Self::U64_COUNT {
            if u64_arr[i] != u64::MAX {
                return false;
            }
        }

        for i in Self::U8_START..Self::BYTE_COUNT {
            if self.bits[i] != 0 {
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
        let mut u64_arr = self.bits_as_u64();
        for i in 0..BitSet::<COUNT>::U64_COUNT {
            acc += u64_arr[i].count_ones() as usize;
        }
        for i in BitSet::<COUNT>::U8_START..BitSet::<COUNT>::BYTE_COUNT {
            acc += self.bits[i].count_ones() as usize;
        }
        acc
    }

    /// Count the number of bits set to 0
    pub fn count_zeros(&self) -> usize {
        let mut acc = 0usize;
        let mut u64_arr = self.bits_as_u64();
        for i in 0..BitSet::<COUNT>::U64_COUNT {
            acc += u64_arr[i].count_zeros() as usize;
        }
        for i in BitSet::<COUNT>::U8_START..BitSet::<COUNT>::BYTE_COUNT {
            acc += self.bits[i].count_zeros() as usize;
        }
        acc
    }

    /// Clear all bits to 0
    pub fn clear(&mut self) {
        let mut u64_arr = self.bits_as_u64_mut();
        for i in 0..BitSet::<COUNT>::U64_COUNT {
            u64_arr[i] = 0;
        }
        for i in BitSet::<COUNT>::U8_START..BitSet::<COUNT>::BYTE_COUNT {
            self.bits[i] = 0;
        }
    }

    /// Set all bits
    pub fn set_all(&mut self) {
        let mut u64_arr = self.bits_as_u64_mut();
        for i in 0..BitSet::<COUNT>::U64_COUNT {
            u64_arr[i] = u64::MAX;
        }
        for i in BitSet::<COUNT>::U8_START..BitSet::<COUNT>::BYTE_COUNT {
            self.bits[i] = u8::MAX;
        }
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
            idx / 8,
            7 - (idx & 0x7)
        )
    }

    fn from_arr(bits: [u8; (COUNT + 7) / 8]) -> Self {
        Self { bits }
    }

    fn bits_as_u64(&self) -> &[u64; COUNT / 64] {
        let u64_ptr = &self.bits as *const _ as *const u64;
        unsafe { &*(u64_ptr as *const [u64; COUNT / 64]) }
    }

    fn bits_as_u64_mut(&mut self) -> &mut [u64; COUNT / 64] {
        let u64_ptr = &mut self.bits as *mut _ as *mut u64;
        unsafe { &mut *(u64_ptr as *mut [u64; COUNT / 64]) }
    }
}

impl<const COUNT: usize> Not for &BitSet<COUNT>
where
    [u8; (COUNT + 7) / 8]:,
    [u64; COUNT / 64]:
{
    type Output = BitSet<COUNT>;

    fn not(self) -> Self::Output {
        let mut res = BitSet::<COUNT>::new();

        let mut res_u64 = res.bits_as_u64_mut();
        let self_u64 = self.bits_as_u64();

        for i in 0..BitSet::<COUNT>::U64_COUNT {
            res_u64[i] = !self_u64[i];
        }
        for i in BitSet::<COUNT>::U8_START..BitSet::<COUNT>::BYTE_COUNT {
            res.bits[i] = !self.bits[i];
        }
        res
    }
}

impl<const COUNT: usize> BitOr for &BitSet<COUNT>
where
    [u8; (COUNT + 7) / 8]:,
    [u64; COUNT / 64]:
{
    type Output = BitSet<COUNT>;

    fn bitor(self, rhs: Self) -> Self::Output {
        let mut res = BitSet::<COUNT>::new();

        let mut res_u64 = res.bits_as_u64_mut();
        let self_u64 = self.bits_as_u64();
        let rhs_u64 = rhs.bits_as_u64();

        for i in 0..BitSet::<COUNT>::U64_COUNT {
            res_u64[i] = self_u64[i] | res_u64[i];
        }
        for i in BitSet::<COUNT>::U8_START..BitSet::<COUNT>::BYTE_COUNT {
            res.bits[i] = self.bits[i] | rhs.bits[i];
        }
        res
    }
}

impl<const COUNT: usize> BitXor for &BitSet<COUNT>
where
    [u8; (COUNT + 7) / 8]:,
    [u64; COUNT / 64]:
{
    type Output = BitSet<COUNT>;

    fn bitxor(self, rhs: Self) -> Self::Output {
        let mut res = BitSet::<COUNT>::new();

        let mut res_u64 = res.bits_as_u64_mut();
        let self_u64 = self.bits_as_u64();
        let rhs_u64 = rhs.bits_as_u64();

        for i in 0..BitSet::<COUNT>::U64_COUNT {
            res_u64[i] = self_u64[i] ^ res_u64[i];
        }
        for i in BitSet::<COUNT>::U8_START..BitSet::<COUNT>::BYTE_COUNT {
            res.bits[i] = self.bits[i] ^ rhs.bits[i];
        }
        res
    }
}

impl<const COUNT: usize> BitAnd for &BitSet<COUNT>
where
    [u8; (COUNT + 7) / 8]:,
    [u64; COUNT / 64]:
{
    type Output = BitSet<COUNT>;

    fn bitand(self, rhs: Self) -> Self::Output {
        let mut res = BitSet::<COUNT>::new();

        let mut res_u64 = res.bits_as_u64_mut();
        let self_u64 = self.bits_as_u64();
        let rhs_u64 = rhs.bits_as_u64();

        for i in 0..BitSet::<COUNT>::U64_COUNT {
            res_u64[i] = self_u64[i] & res_u64[i];
        }
        for i in BitSet::<COUNT>::U8_START..BitSet::<COUNT>::BYTE_COUNT {
            res.bits[i] = self.bits[i] & rhs.bits[i];
        }
        res
    }
}

impl<const COUNT: usize> BitOrAssign<&Self> for BitSet<COUNT>
where
    [u8; (COUNT + 7) / 8]:,
    [u64; COUNT / 64]:
{
    fn bitor_assign(&mut self, rhs: &Self) {
        let mut self_u64 = self.bits_as_u64_mut();
        let rhs_u64 = rhs.bits_as_u64();

        for i in 0..BitSet::<COUNT>::U64_COUNT {
            self_u64[i] |= rhs_u64[i];
        }
        for i in BitSet::<COUNT>::U8_START..BitSet::<COUNT>::BYTE_COUNT {
            self.bits[i] |= rhs.bits[i];
        }
    }
}

impl<const COUNT: usize> BitXorAssign<&Self> for BitSet<COUNT>
where
    [u8; (COUNT + 7) / 8]:,
    [u64; COUNT / 64]:
{
    fn bitxor_assign(&mut self, rhs: &Self) {
        let mut self_u64 = self.bits_as_u64_mut();
        let rhs_u64 = rhs.bits_as_u64();

        for i in 0..BitSet::<COUNT>::U64_COUNT {
            self_u64[i] ^= rhs_u64[i];
        }
        for i in BitSet::<COUNT>::U8_START..BitSet::<COUNT>::BYTE_COUNT {
            self.bits[i] ^= rhs.bits[i];
        }
    }
}

impl<const COUNT: usize> BitAndAssign<&Self> for BitSet<COUNT>
where
    [u8; (COUNT + 7) / 8]:,
    [u64; COUNT / 64]:
{
    fn bitand_assign(&mut self, rhs: &Self) {
        let mut self_u64 = self.bits_as_u64_mut();
        let rhs_u64 = rhs.bits_as_u64();

        for i in 0..BitSet::<COUNT>::U64_COUNT {
            self_u64[i] &= rhs_u64[i];
        }
        for i in BitSet::<COUNT>::U8_START..BitSet::<COUNT>::BYTE_COUNT {
            self.bits[i] &= rhs.bits[i];
        }
    }
}

impl<const COUNT: usize> IntoIterator for BitSet<COUNT>
where
    [u8; (COUNT + 7) / 8]:,
    [u64; COUNT / 64]:
{
    type Item = bool;
    type IntoIter = IntoIter<COUNT>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter{ bitset: self, idx: 0, end: COUNT }
    }
}

impl<'a, const COUNT: usize> IntoIterator for &'a BitSet<COUNT>
where
    [u8; (COUNT + 7) / 8]:,
    [u64; COUNT / 64]:
{
    type Item = bool;
    type IntoIter = Iter<'a, COUNT>;

    fn into_iter(self) -> Self::IntoIter {
        Iter{ bitset: self, idx: 0, end: COUNT }
    }
}

pub struct Iter<'a, const COUNT: usize>
where
    [u8; (COUNT + 7) / 8]:,
    [u64; COUNT / 64]:
{
    bitset : &'a BitSet<COUNT>,
    idx    : usize,
    end    : usize
}

impl<const COUNT: usize> Iterator for Iter<'_, COUNT>
where
    [u8; (COUNT + 7) / 8]:,
    [u64; COUNT / 64]:
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

impl<const COUNT: usize> DoubleEndedIterator for Iter<'_, COUNT>
where
    [u8; (COUNT + 7) / 8]:,
    [u64; COUNT / 64]:
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

impl<const COUNT: usize> ExactSizeIterator for Iter<'_, COUNT>
where
    [u8; (COUNT + 7) / 8]:,
    [u64; COUNT / 64]:
{
    fn len(&self) -> usize {
        COUNT
    }
}

//--------------------------------------------------------------

pub struct IterOnes<'a, const COUNT: usize>
where
    [u8; (COUNT + 7) / 8]:,
    [u64; COUNT / 64]:
{
    bitset : &'a BitSet<COUNT>,
    idx    : usize,
    end    : usize
}

impl<const COUNT: usize> Iterator for IterOnes<'_, COUNT>
where
    [u8; (COUNT + 7) / 8]:,
    [u64; COUNT / 64]:
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        while self.idx < self.end {
            let idx = self.idx;
            self.idx += 1;
            if self.bitset.get(self.idx) {
                return Some(idx);
            }
            self.idx += 1;
        }
        None
    }
}

impl<const COUNT: usize> DoubleEndedIterator for IterOnes<'_, COUNT>
where
    [u8; (COUNT + 7) / 8]:,
    [u64; COUNT / 64]:
{
    fn next_back(&mut self) -> Option<Self::Item> {
        while self.idx < self.end {
            self.end -= 1;
            if self.bitset.get(self.idx) {
                return Some(self.end);
            }
        }
        None
    }
}

impl<const COUNT: usize> ExactSizeIterator for IterOnes<'_, COUNT>
where
    [u8; (COUNT + 7) / 8]:,
    [u64; COUNT / 64]:
{
    fn len(&self) -> usize {
        COUNT
    }
}

//--------------------------------------------------------------

pub struct IterZeros<'a, const COUNT: usize>
where
    [u8; (COUNT + 7) / 8]:,
    [u64; COUNT / 64]:
{
    bitset : &'a BitSet<COUNT>,
    idx    : usize,
    end    : usize
}

impl<const COUNT: usize> Iterator for IterZeros<'_, COUNT>
where
    [u8; (COUNT + 7) / 8]:,
    [u64; COUNT / 64]:
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        while self.idx < self.end {
            let idx = self.idx;
            self.idx += 1;
            if !self.bitset.get(self.idx) {
                return Some(idx);
            }
            self.idx += 1;
        }
        None
    }
}

impl<const COUNT: usize> DoubleEndedIterator for IterZeros<'_, COUNT>
where
    [u8; (COUNT + 7) / 8]:,
    [u64; COUNT / 64]:
{
    fn next_back(&mut self) -> Option<Self::Item> {
        while self.idx < self.end {
            self.end -= 1;
            if !self.bitset.get(self.idx) {
                return Some(self.end);
            }
        }
        None
    }
}

impl<const COUNT: usize> ExactSizeIterator for IterZeros<'_, COUNT>
where
    [u8; (COUNT + 7) / 8]:,
    [u64; COUNT / 64]:
{
    fn len(&self) -> usize {
        COUNT
    }
}

//--------------------------------------------------------------

pub struct IntoIter<const COUNT: usize>
where
    [u8; (COUNT + 7) / 8]:,
    [u64; COUNT / 64]:
{
    bitset : BitSet<COUNT>,
    idx    : usize,
    end    : usize,
}

impl<const COUNT: usize> Iterator for IntoIter<COUNT> 
where
    [u8; (COUNT + 7) / 8]:,
    [u64; COUNT / 64]:
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

impl<const COUNT: usize> DoubleEndedIterator for IntoIter<COUNT>
where
    [u8; (COUNT + 7) / 8]:,
    [u64; COUNT / 64]:
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

impl<const COUNT: usize> ExactSizeIterator for IntoIter<COUNT>
where
    [u8; (COUNT + 7) / 8]:,
    [u64; COUNT / 64]:
{
    fn len(&self) -> usize {
        COUNT
    }
}
