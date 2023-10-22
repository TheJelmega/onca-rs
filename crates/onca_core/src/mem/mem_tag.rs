


/// Memory tag
/// 
/// A memory tag consists of:
/// - 16-bit plugin ID (not all plugin will use tracking)
/// - 10-bit category
/// -  6-bit sub-category
/// ```
/// +----------------+----------+------+
/// |    plugin id   | category | sub  |
/// +----------------+----------+------+
/// 0                16         26     32
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct MemTag(u32);

impl MemTag {
    const PLUGIN_ID_SHIFT: u32 = 16;
    const PLUGIN_ID_MASK: u32 = 0xFFFF;
    const CATEGORY_SHIFT: u32 = 6;
    const CATEGORY_MASK: u32 = 0x3FF;
    const SUB_CAT_SHIFT: u32 = 0;
    const SUB_CAT_MASK: u32 = 0x3F;


    pub fn new(plugin_id: u16, category: u16, sub_category: u8) -> Self {
        Self(
            (plugin_id    as u32 & Self::PLUGIN_ID_MASK) << Self::PLUGIN_ID_SHIFT |
            (category     as u32 & Self::CATEGORY_MASK ) << Self::CATEGORY_SHIFT  |
            (sub_category as u32 & Self::SUB_CAT_MASK  ) << Self::SUB_CAT_SHIFT
        )
    }

    /// Get the plugin id
    pub fn plugin_id(self) -> u16 {
        (self.0 >> Self::PLUGIN_ID_SHIFT) as u16
    }

    /// Get the category
    pub fn category(self) -> u16 {
        ((self.0 >> Self::CATEGORY_SHIFT) & Self::CATEGORY_MASK) as u16
    }

    /// Get the sub-category
    pub fn sub_category(self) -> u8 {
        ((self.0 >> Self::SUB_CAT_SHIFT) & Self::SUB_CAT_MASK) as u8
    }
}