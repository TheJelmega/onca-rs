
use crate::{os, PhysicalSize};


pub struct Icon {
    os_icon : os::OSIcon,
}

impl Icon {
    pub fn from_path(path: &str, size: Option<PhysicalSize>) -> Icon {
        Icon { os_icon: os::OSIcon::from_path(path, size) }
    }

    pub(crate) fn get_os_icon(&self) -> &os::OSIcon {
        &self.os_icon
    }
}