use core::{
    cell::Cell,
    mem,
    cmp,
    fmt,
};
use cfg_if::cfg_if;

#[cfg(feature = "memory_tracking")]
thread_local! {
    static TLS_PLUGIN_CATEGORY : Cell<u16> = Cell::new(0);
}

pub fn set_tls_mem_tag_plugin_id(plugin_id: u16) {
    cfg_if!{
        if #[cfg(feature = "memory_tracking")] {
            TLS_PLUGIN_CATEGORY.set(plugin_id & MemTag::MAX_PLUGIN_ID);
        }
    }
}

pub fn get_tls_mem_tag_plugin_id() -> u16 {
    cfg_if!{
        if #[cfg(feature = "memory_tracking")] {
            TLS_PLUGIN_CATEGORY.get()
        } else {
            0
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum MemTagState {
    /// Invalid state
    #[default]
    Invalid,
    /// Valid allocation
    Valid,
    /// The allocation had an underflow
    Underflow,
    /// The allocation had an overflow
    Overflow,
    /// The allocation  had both an underflow and overflow
    UnderAndOverflow
}

/// Tag and tracking info
/// 
/// The UTID represents a unique ID for the allocation, in the current category and plugin
/// The UTID (Unique Tracking ID), combined with the plugin id, category, and sub-category form a unique ID for the allocation
/// 
/// Note, the lower 128 values (MSB not set) of the category are defined by onca, the upper 128 values (MSB set), are can be user defined
/// 
/// ```
/// +--------+--------+--------+--------+--------+--------+--------+--------+--------+--------+--------+--------+--------+--------+--------+--------+
/// |  state |        plugin id         |    category     |  sub-category   |                       UTID (Unique Tracking ID)                       |
/// +--------+--------+--------+--------+--------+--------+--------+--------+--------+--------+--------+--------+--------+--------+--------+--------+
/// 0        4        8        12       16       20       24       28       32       36       40       44       48       52       56       60       64
/// MSB                                                                                                                                             LSB
/// ```
/// 
/// State
/// ```
/// +--------+--------+--------+--------+
/// | tracked|      mem tag state       |
/// +--------+--------+--------+--------+
/// 0        1                          4
/// ```
#[derive(Clone, Copy)]
pub struct MemTag {
    #[cfg(feature = "memory_tracking")]
    packed : u64
}

impl MemTag {
    /// Maximum plugin id
    pub const MAX_PLUGIN_ID : u16 = 0x0FFF;
    /// Maximum category
    pub const MAX_CATEGORY : u8 = 0xFF;
    /// Maximum sub-category
    pub const MAX_SUB_CATEGORY : u8 = 0xFF;
    /// Number of bytes to shift to retireve the state
    pub const STATE_SHIFT : usize = 62;
    /// Number of bytes to shift to retrieve the plugin id
    pub const PLUGIN_SHIFT : usize = 48;
    /// Number of bytes to shift to retrieve the category
    pub const CATEGORY_SHIFT : usize = 40;
    /// Number of bytes to shift to retrieve the sub-category
    pub const SUB_CATEGORY_SHIFT : usize = 32;
    /// State mask
    pub const STATE_MASK : u64 = 0x07;
    /// ID mask
    pub const ID_MASK : u64 = 0x0FFF_FFFF_FFFF_FFFF;

    /// Tracked flag
    pub const TRACKED_FLAG : u64 = 1 << 63;

    /// User category bit
    pub const USER_CATEGORY_BIT : u8 = 0x80;

    /// Create a new memory tag
    pub fn new(plugin_id: u16, category: u8, sub_category: u8, utid: u32) -> Self {
        cfg_if!{
            if #[cfg(feature = "memory_tracking")] {
                Self { packed: 
                    ((plugin_id as u64) & (Self::MAX_PLUGIN_ID as u64)) << Self::PLUGIN_SHIFT |
                    (category as u64) << Self::CATEGORY_SHIFT |
                    (sub_category as u64) << Self::SUB_CATEGORY_SHIFT |
                    (utid as u64)
                }
            } else {
                Self {}
            }
        }
    }

    /// Returns `true` if the tag is tracked, `false` otherwise
    pub fn tracked(&self) -> bool {
        cfg_if!{
            if #[cfg(feature = "memory_tracking")] {
                self.packed & Self::TRACKED_FLAG == Self::TRACKED_FLAG
            } else {
                false
            }
        }
    }

    /// Set the tracked flag
    pub fn set_tracked(&mut self, tracked: bool) {
        cfg_if!{
            if #[cfg(feature = "memory_tracking")] {   
                if tracked {
                    self.packed |= Self::TRACKED_FLAG;
                } else {
                    self.packed &= !Self::TRACKED_FLAG;
                }
            }
        }
    }

    /// Get the mem tag state
    pub fn state(&self) -> MemTagState {
        cfg_if!{
            if #[cfg(feature = "memory_tracking")] {   
                unsafe { mem::transmute(((self.packed >> Self::STATE_SHIFT) & Self::STATE_MASK) as u8) }
            } else {
                MemTagState::Invalid
            }
        }
    }

    /// Set the mem tag state
    pub fn set_state(&mut self, state: MemTagState) {
        cfg_if!{
            if #[cfg(feature = "memory_tracking")] {   
                self.packed &= !(Self::STATE_MASK << Self::STATE_SHIFT);
                self.packed |= (state as u64) << Self::STATE_SHIFT;
            }
        }
    }

    /// Get the plugin id
    pub fn plugin_id(&self) -> u16 {
        cfg_if!{
            if #[cfg(feature = "memory_tracking")] {   
                (self.packed >> Self::PLUGIN_SHIFT) as u16
            } else {
                0
            }
        }
    }

    /// Get the plugin id
    pub fn category(&self) -> u8 {
        cfg_if!{
            if #[cfg(feature = "memory_tracking")] {   
                (self.packed >> Self::CATEGORY_SHIFT) as u8
            } else {
                0
            }
        }
    }

    /// Get the plugin id
    pub fn sub_category(&self) -> u8 {
        cfg_if!{
            if #[cfg(feature = "memory_tracking")] {   
                (self.packed >> Self::SUB_CATEGORY_SHIFT) as u8
            } else {
                0
            }
        }
    }

    /// Get the UTID (Unique Tracking ID)
    pub fn utid(&self) -> u32 {
        cfg_if!{
            if #[cfg(feature = "memory_tracking")] {   
                self.packed as u32
            } else {
                0
            }
        }
    }

    /// Get the unique id, including the plugin id, category, and sub-category
    pub fn id(&self) -> u64 {
        cfg_if!{
            if #[cfg(feature = "memory_tracking")] {   
                self.packed & Self::ID_MASK
            } else {
                0
            }
        }
    }

    /// Check if the category is a user category 
    pub fn is_user_category(&self) -> bool {
        self.category() & Self::USER_CATEGORY_BIT == Self::USER_CATEGORY_BIT
    }
}

impl Default for MemTag {
    fn default() -> Self {
        Self::new(0, 0, 0, 0)
    }
}

impl fmt::Debug for MemTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        cfg_if!{
            if #[cfg(feature = "memory_tracking")] {   
                f.debug_struct("MemTag")
                    .field("tracked", &self.tracked())
                    .field("state", &self.state())
                    .field("plugin id", &self.plugin_id())
                    .field("category", &self.category())
                    .field("sub-category", &self.sub_category())
                    .field("utid", &self.utid())
                .finish()
            }
        }
    }
}

impl fmt::Display for MemTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}


pub enum CoreMemTag {
    Unknown,
    String,
    Sync,

    StdCollections,
    Test,
}

impl CoreMemTag {
    /// Core memory tag category
    pub const CATEGORY : u8 = 0;

    /// Create a memory tag from the Core Memory Tag category
    #[inline]
    pub fn to_mem_tag(self) -> MemTag {
        MemTag::new(0 /* TODO */, Self::CATEGORY, self as u8, 0)
    }
}