use core::num::NonZeroUsize;
use onca_common::prelude::*;
use onca_logging::log_warning;

use crate::{DeviceType, LOG_INPUT_CAT, DeviceTypeMatchSupport, DeviceHandle};

pub enum SchemeItem {
    /// The device is required for this mapping
    Required(DeviceType),
    /// Either device can be used, but one is required to exists
    Either(Vec<DeviceType>),
    /// The device is optional for this mapping
    Optional(DeviceType),
}

impl SchemeItem {
    fn discriminant(&self) -> u8 {
        match self {
            SchemeItem::Required(_) => 0,
            SchemeItem::Either(_) => 1,
            SchemeItem::Optional(_) => 2,
        }
    }
}

// TODO: Interned string
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ControlSchemeID(String);

impl Default for ControlSchemeID {
    fn default() -> Self {
        Self(Default::default())
    }
}

/// Control scheme
/// 
/// A control scheme is limited to 64 items.
pub struct ControlScheme {
    identifier : ControlSchemeID,
    /// Items
    items      : Vec<SchemeItem>,
    /// Index of the first `Optional` item (this is also the number of required/either devices)
    opt_start  : Option<NonZeroUsize>,
}

impl ControlScheme {
    pub const MAX_SCHEME_ITEMS : usize = 64;

    /// Create a new control scheme
    /// 
    /// # Errors
    /// 
    /// Return an error when 0 or more than 64 items are passed, or when duplicate device 
    pub fn new(identifier: ControlSchemeID, mut items: Vec<SchemeItem>) -> Result<Self, Vec<SchemeItem>> {
        if !items.is_empty() && items.len() <= Self::MAX_SCHEME_ITEMS {
            if items.iter().position(|val| !matches!(val, SchemeItem::Optional(_))).is_none() {
                log_warning!(LOG_INPUT_CAT, "Trying to create an input scheme with a Required/Either item");
                return Err(items);
            }

            // Put `Required` items first, then `Either` items, and finally `Optional` items
            items.sort_by(|a, b| a.discriminant().cmp(&b.discriminant()) );
            let opt_start = items.iter().position(|val| matches!(val, SchemeItem::Optional(_))).and_then(|val| NonZeroUsize::new(val));

            Ok(Self{ identifier, items, opt_start })
        } else {
            Err(items)
        }
    }

    /// Try to create a control set for the layout from the currently available devices.
    /// 
    /// Returns an option with the created control set, if the control scheme can be created
    /// 
    /// If there is not exact match for a device, but there is a device with the support needed for the wanted control scheme, the first device that fits will be selected.
    pub fn create_control_set<'a, F>(&self, available_devices: &Vec<DeviceHandle>, get_dev_type: F) -> Option<ControlSet>
        where F : Fn(DeviceHandle) -> DeviceType
    {
        let mut scheme_devs = Vec::new();
        scheme_devs.resize(self.items.len(), DeviceHandle::Invalid);
        let mut cur_matches = Vec::new();
        cur_matches.resize(self.items.len(), DeviceTypeMatchSupport::None);

        for dev in available_devices {
            let dev_type = get_dev_type(*dev);

            for (idx, item) in self.items.iter().enumerate() {
                let match_support = match item {
                    SchemeItem::Required(ty) => dev_type.match_or_supports(ty),
                    SchemeItem::Either(types) => types.iter().fold(DeviceTypeMatchSupport::None, |acc, ty| acc.max(dev_type.match_or_supports(ty))),
                    SchemeItem::Optional(ty) => dev_type.match_or_supports(ty),
                };
                if cur_matches.len() <= idx || cur_matches[idx] != match_support {
                    cur_matches[idx] = match_support;
                    scheme_devs[idx] = *dev;
                }
            }
        }

        let has_all_required = scheme_devs[..self.opt_start.map_or(scheme_devs.len(), |x| x.get())].iter().find(|handle| !matches!(handle, DeviceHandle::Invalid)).is_none();

        if has_all_required {
            None
        } else {
            Some(ControlSet{ scheme: self.identifier.clone(), devices: scheme_devs })
        }
    }

    pub fn identifier(&self) -> &ControlSchemeID {
        &self.identifier
    }
}

/// Control set containing device handles for current layout
pub struct ControlSet {
    pub(crate) scheme  : ControlSchemeID,
    pub(crate) devices : Vec<DeviceHandle>,
}

impl ControlSet {
    pub fn scheme_identifier(&self) -> &ControlSchemeID {
        &self.scheme
    }

    pub fn devices(&self) -> &Vec<DeviceHandle> {
        &self.devices
    }

    pub(crate) fn take_devices(&mut self) -> Vec<DeviceHandle> {
        core::mem::take(&mut self.devices)
    }
}