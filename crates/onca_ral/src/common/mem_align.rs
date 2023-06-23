use core::fmt;

/// Representation of a memory alignment (always a power of 2)
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug)]
pub struct MemAlign {
    /// log2 of the alignment
    log2 : u8,
}

impl MemAlign {
    /// Create a new memory alignment
    pub const fn new(align: u64) -> MemAlign {
        debug_assert!(align.is_power_of_two(), "Cannot create an alignment that is not a power of 2.");

        MemAlign { log2: align.ilog2() as u8 }
    }

    /// Create a new memory alignment from the log2 of the alignment
    pub const fn from_log2(log2: u8) -> MemAlign {
        MemAlign { log2 }
    }

    /// Get the log2 of the alignment
    pub const fn log2(&self) -> u8 {
        self.log2
    }

    /// Get the actual alignment
    pub const fn alignment(&self) -> u64 {
        1 << self.log2
    }
}

impl Default for MemAlign {
    fn default() -> Self {
        // Default to the alignment of a pointer
        Self { log2: core::mem::align_of::<*const ()>().ilog2() as u8 }
    }
}

impl fmt::Display for MemAlign {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("align: {}", self.alignment()))
    }
}