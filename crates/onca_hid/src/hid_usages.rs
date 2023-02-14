use core::fmt;

use crate::hid_data::HID_USAGE_PAGES;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct UsagePageId(u16);

impl UsagePageId {
	pub const fn new(id: u16) -> Self {
		Self(id)
	}

	pub const fn as_u16(self) -> u16 {
		self.0
	}
}

impl fmt::Debug for UsagePageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("HidUsageId").field(&format_args!("{:X}", self.0)).finish()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct UsageId(u16);

impl UsageId {
	pub const fn new(id: u16) -> Self {
		Self(id)
	}

	pub const fn as_u16(self) -> u16 {
		self.0
	}
}

impl fmt::Debug for UsageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("HidUsageId").field(&format_args!("{:X}", self.0)).finish()
    }
}

#[derive(Debug)]
pub struct HidUsagePage {
    pub id     : UsagePageId,
    pub name   : &'static str,
    pub usages : Option<&'static [HidUsage]>
}

impl HidUsagePage {
    /// Get a USB vendor from its ID.
    /// 
    /// If the ID does not point to a valid vendor, `None` will be returned.
    pub fn new(usage_page_id: UsagePageId) -> Option<&'static HidUsagePage> { 
        match HID_USAGE_PAGES.binary_search_by_key(&usage_page_id, |vendor| vendor.id) {
            Ok(idx) => Some(&HID_USAGE_PAGES[idx]),
            Err(_) => None,
        }
    }

    /// Get a USB device for the current vendor drom its ID.
    /// 
    /// If the vendor does not have any devices, or if the ID does not point to a valid device, `None` will be returned.
    pub fn get_usage(&self, usage_id: UsageId) -> Option<&HidUsage> {
        match self.usages {
            Some(usages) => {
                match usages.binary_search_by_key(&usage_id, |dev| dev.id) {
                    Ok(idx) => Some(&usages[idx]),
                    Err(_) => None,
                }
            },
            None => None,
        }
    }
}

impl fmt::Display for HidUsagePage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name)
    }
}

#[derive(Debug)]
pub struct HidUsage {
    pub id   : UsageId,
    pub name : &'static str,
}

impl fmt::Display for HidUsage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name)
    }
}


#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Usage {
    pub page  : UsagePageId,
    pub usage : UsageId,
}

impl Usage {
    pub fn new(page: UsagePageId, usage: UsageId) -> Self {
        Self { page, usage }
    }

    pub fn from_u16(page: u16, usage: u16) -> Self {
        Self::new(UsagePageId(page), UsageId(usage))
    }
}

impl fmt::Display for Usage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match HidUsagePage::new(self.page) {
            Some(page) => {
                match page.get_usage(self.usage) {
                    Some(usage) => f.write_fmt(format_args!("{{ page: \"{}\", usage: \"{}\" }}", page.name, usage.name)),
                    None => f.write_fmt(format_args!("{{ page: \"{}\", usage: {:04X} }}", page.name, self.usage.0)),
                }
            },
            None => f.write_fmt(format_args!("{{ page: {:04X} usage: {:04X} }}", self.page.0, self.usage.0)),
        }
    }
}