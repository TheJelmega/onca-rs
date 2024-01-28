use core::fmt;

use onca_common::prelude::*;

use crate::hid_data::USB_VENDORS;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Hash)]
pub struct UsbVendorId(u16);

impl UsbVendorId {
	pub const fn new(id: u16) -> Self {
		Self(id)
	}

	pub const fn as_u16(self) -> u16 {
		self.0
	}
}

impl fmt::Display for UsbVendorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("0x{:04}", self.0))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Hash)]
pub struct UsbDeviceId(u16);

impl UsbDeviceId {
	pub const fn new(id: u16) -> Self {
		Self(id)
	}

	pub const fn as_u16(self) -> u16 {
		self.0
	}
}

impl fmt::Display for UsbDeviceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("0x{:04}", self.0))
    }
}

#[derive(Debug)]
pub struct UsbVendor {
	pub id      : UsbVendorId,
	pub name    : &'static str,
	pub devices : Option<&'static [UsbDevice]>,
}

impl UsbVendor {
    /// Get a USB vendor from its ID.
    /// 
    /// If the ID does not point to a valid vendor, `None` will be returned.
    pub fn new(vendor_id: UsbVendorId) -> Option<&'static UsbVendor> { 
        match USB_VENDORS.binary_search_by_key(&vendor_id, |vendor| vendor.id) {
            Ok(idx) => Some(&USB_VENDORS[idx]),
            Err(_) => None,
        }
    }

    /// Get a USB device for the current vendor drom its ID.
    /// 
    /// If the vendor does not have any devices, or if the ID does not point to a valid device, `None` will be returned.
    pub fn get_device(&self, device_id: UsbDeviceId) -> Option<&UsbDevice> {
        match self.devices {
            Some(devices) => {
                match devices.binary_search_by_key(&device_id, |dev| dev.id) {
                    Ok(idx) => Some(&devices[idx]),
                    Err(_) => None,
                }
            },
            None => None,
        }
    }
}

impl fmt::Display for UsbVendor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name)
    }
}

#[derive(Debug)]
pub struct UsbDevice {
	pub id: UsbDeviceId,
	pub name: &'static str,
}

impl fmt::Display for UsbDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct VendorProduct {
    pub vendor : UsbVendorId,
    pub device : UsbDeviceId,
}

impl VendorProduct {
    pub fn new(vendor: UsbVendorId, device: UsbDeviceId) -> Self {
        Self { vendor, device }
    }

    pub fn from_u16(vendor: u16, device: u16) -> Self {
        Self::new(UsbVendorId(vendor), UsbDeviceId(device))
    }

    pub fn get_vendor_string(&self) -> Option<String> {
        match UsbVendor::new(self.vendor) {
            Some(vendor) => Some(vendor.name.to_string()),
            None => None,
        }
    }

    pub fn get_device_string(&self) -> Option<String> {
        match UsbVendor::new(self.vendor) {
            Some(vendor) => match vendor.get_device(self.device) {
                Some(device) => Some(device.name.to_string()),
                None => None,
            },
            None => None,
        }
    }
}

impl fmt::Display for VendorProduct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match UsbVendor::new(self.vendor) {
            Some(vendor) => {
                match vendor.get_device(self.device) {
                    Some(device) => f.write_fmt(format_args!("{{ {} {} }}", vendor.name, device.name)),
                    None => f.write_fmt(format_args!("{{ vendor: {}, product: 0x{:04X} }}", vendor.name, self.device.0)),
                }
            },
            None => f.write_fmt(format_args!("{{ vendor: 0x{:04X}, product: 0x{:04X} }}", self.vendor.0, self.device.0)),
        }
    }
}

