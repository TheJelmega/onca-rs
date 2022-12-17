use core::{
    mem,
    cmp,
    fmt
};

// TODO: Combine tag and alloc ID into 1 value? 8-bit category, 8/4-bit subcategory, 4/8-bit differentiator. All 0s or 1s: malloc
// TODO: align at min 2? up to 2^16 alignment max
/// Memory layout
/// 
/// The data is stored as following (size shown in bits)
/// 
/// ```
/// +--------+--------+--------+--------+--------+--------+--------+--------+--------+--------+--------+--------+--------+--------+--------+--------+
/// |                                           size                                                            |         alloc id         | align  |
/// +--------+--------+--------+--------+--------+--------+--------+--------+--------+--------+--------+--------+--------+--------+--------+--------+
/// 0        4        8        12       16       20       24       28       32       36       40       44       48       52       56       60       64
/// MSB                                                                                                                                             LSB
/// ```
/// 
/// where `align` is `log2(alignment) - log2(MIN_ALIGN)` 
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Layout {
    /// Size, alloc_id and log2 of the alignment (real alignment in 2^value)
    packed : u64
}

impl Layout {
    /// Maximum size of an allocaion (1TiB)
    pub const MAX_SIZE : u64 = (1u64 << 40) - 1;
    /// Maximum alignment of an allocation (2^15 bytes)
    pub const MAX_ALIGN : u64 = 1 << 15;
    /// Maximum allococator id
    pub const MAX_ALLOC_ID : u16 = 0x0FFF;
    /// Number of bits to shift to retrieve the allocator id
    pub const ALLOC_ID_SHIFT : usize = 4;
    /// Number of bits to shift to retrieve the size
    pub const SIZE_SHIFT : usize = 16;
    /// Mask for the allocator id
    pub const SIZE_MASK : u64 = 0xFFFF_FFFF_FFFF_0000;
    /// Mask for the allocator id
    pub const ALLOC_ID_MASK : u64 = 0xFFF0;
    /// Mask for the log2(align)
    pub const ALIGN_MASK : u64 = 0x0F;

    pub fn new_raw(size: usize, alloc_id: u16, align: usize) -> Self
    {
        Self { packed: 
            (size as u64) << Self::SIZE_SHIFT |
            (alloc_id << Self::ALLOC_ID_SHIFT) as u64 & Self::ALLOC_ID_MASK |
            align.ilog2() as u64 & Self::ALIGN_MASK
        }
    }

    /// Create a new layout for type `T`
    pub fn new<T>() -> Self {
        let size = mem::size_of::<T>();
        let align = mem::align_of::<T>();
        Self::new_size_align(size, align)
    }

    /// Create a new layout from a `size` and an `alignment`
    pub fn new_size_align(size: usize, align: usize) -> Self {
        assert!(size  != 0                       , "Size needs to be larger than 0");
        assert!(size  <= Self::MAX_SIZE as usize , "Can only allocate up to and including MAX_SIZE bytes");
        assert!(align != 0                       , "Alignment needs to be larger than 0");
        assert!(align <= Self::MAX_ALIGN as usize, "Alignment needs to be smaller or equal to MAX_ALIGN");
        assert!(align.is_power_of_two()          , "Alignment needs to be a power of 2");

        Self::new_raw(size, 0, align)
    }

    /// Create a new layout for an array that can store `count` elements of type `T`
    pub fn array<T>(count: usize) -> Self {
        let size = mem::size_of::<T>() * count;
        let align = mem::align_of::<T>();
        Self::new_size_align(size, align)
    }

    /// Get a 0-size layout
    #[inline]
    pub fn null() -> Self
    {
        Self { packed: 0 }
    }

    /// Get a copy of the layout that is expanded by the given layout
    /// 
    /// The added size is added to the element that it would represent has the correct alignment inside of the layout
    pub fn expand(self, other: Self) -> Self {
        let align = cmp::max(self.align(), other.align());

        let needed_align = other.align();
        // Make sure data is aligned for the next element
        let mut size = self.size().next_multiple_of(needed_align);
        size += other.size();
        Self::new_raw(size, self.alloc_id(), align)
    }

    /// Get a copy of the layout that is expanded by the given layout. The given layout will be directly appneded and its alignment will be ignored
    /// 
    /// The added size is added to the element that it would represent has the correct alignment inside of the layout
    pub fn expand_packed(self, other: Self) -> Self {
        let size = self.size() + other.size();
        Self::new_raw(size, self.alloc_id(), self.align())
    }

    pub fn with_size_multiple_of(&self, factor: u64) -> Self {
        let raw = self.packed;
        let size = (self.size() as u64).next_multiple_of(factor);
        Self{ packed: (raw & (Self::SIZE_MASK as u64)) | size << Self::SIZE_SHIFT }
    }

    /// Get a copy of the layout that is at minimum aligned with the given alignment
    pub fn with_min_align(&self, align: usize) -> Self {
        assert!(align != 0             , "Alignment needs to be larger than 0");
        assert!(align.is_power_of_two(), "Alignment needs to be a power of 2");

        let final_align = cmp::max(self.align(), align);
        Self::new_raw(self.size(), self.alloc_id(), align)
    }

    /// Get the size of the allocation
    #[inline]
    pub fn size(&self) -> usize {
        (self.packed >> Self::SIZE_SHIFT) as usize
    }

    /// Get the pow2 of the alignment
    #[inline]
    pub fn log2_align(&self) -> u8 {
        (self.packed & Self::ALIGN_MASK) as u8
    }

    /// Get the alignment of the allocation
    #[inline]
    pub fn align(&self) -> usize {
        1usize << self.log2_align() 
    }

    /// Get the allocator id
    #[inline]
    pub fn alloc_id(&self) -> u16 { ((self.packed & Self::ALLOC_ID_MASK) >> Self::ALLOC_ID_SHIFT) as u16 }

    /// Set the allocator id
    /// 
    /// This function is mainly used when allocating the memory
    pub fn set_alloc_id(&mut self, id: u16) {
        self.packed &= !Self::ALLOC_ID_MASK; 
        self.packed |= ((id as u64) << Self::ALLOC_ID_SHIFT) & Self::ALLOC_ID_MASK;
    }

    /// Get a copy of the layout with the allocator id set
    pub fn with_alloc_id(&self, id: u16) -> Self {
        let mut layout = *self;
        layout.packed &= !Self::ALLOC_ID_MASK; 
        layout.packed |= ((id as u64) << Self::ALLOC_ID_SHIFT) & Self::ALLOC_ID_MASK;
        layout
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self::null()
    }
}

impl fmt::Debug for Layout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Layout")
            .field("size", &self.size())
            .field("allocator id", &self.alloc_id())
            .field("alignment", &self.align())
        .finish()
    }
}

#[cfg(test)]
mod tests
{
    use core::mem::size_of;
    use super::Layout;

    #[test]
    fn size_check() {
        assert_eq!(size_of::<Layout>(), 8);
    }

    #[test]
    fn create_raw() {
        let layout = Layout::new_size_align(1024, 16);
        assert_eq!(layout.size(), 1024);
        assert_eq!(layout.log2_align(), 4);
        assert_eq!(layout.align(), 16);
    }

    #[test]
    fn create_from_u64() {
        let layout = Layout::new::<u64>();
        assert_eq!(layout.size(), 8);
        assert_eq!(layout.log2_align(), 3);
        assert_eq!(layout.align(), 8);
    }

    #[test]
    fn create_from_u16()  {
        let layout = Layout::new::<u16>();
        assert_eq!(layout.size(), 2);
        assert_eq!(layout.log2_align(), 1);
        assert_eq!(layout.align(), 2);
    }

    #[test]
    fn min_align()  {
        let mut layout = Layout::new::<u16>();
        layout = layout.with_min_align(16);
        assert_eq!(layout.size(), 2);
        assert_eq!(layout.log2_align(), 4);
        assert_eq!(layout.align(), 16 );
    }
}