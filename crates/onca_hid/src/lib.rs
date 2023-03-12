#![feature(rustc_attrs)]

use core::{
	num::NonZeroU32,
	mem,
	fmt,
	ops::{self, RangeInclusive, RangeBounds},
};

use onca_core::time::Duration;

use onca_core::prelude::*;
use onca_logging::{LogCategory, log_warning};

mod os;

mod vendor_device;
use os::OSDevice;
pub use vendor_device::{UsbVendorId, UsbVendor, UsbDeviceId, UsbDevice, VendorProduct};

mod hid_usages;
pub use hid_usages::{UsagePageId, HidUsagePage, HidUsage, UsageId, Usage};

#[path = "hid.generated.rs"]
mod hid_data;

pub const LOG_HID_CAT : LogCategory = LogCategory::new("Hid");

// USB devices can have at most 126 character strings
pub const MAX_HID_STRING_LEN : usize = 126;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Identifier {
    /// Vendor and product
    pub vendor_device : VendorProduct,
	/// version
	pub version       : u16,
    /// HID usage
    pub usage         : Usage,
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("HID({}, usage: {})",
            self.vendor_device,
            self.usage))
    }
}

pub struct Capabilities {
	pub input_report_byte_len    : u16,
	pub output_report_byte_len   : u16,
	pub feature_report_byte_len  : u16,
	pub num_collection_nodes     : u16,
	pub num_input_button_caps    : u16,
	pub num_input_value_caps     : u16,
	pub num_input_data_indices   : u16,
	pub num_output_button_caps   : u16,
	pub num_output_value_caps    : u16,
	pub num_output_data_indices  : u16,
	pub num_feature_button_caps  : u16,
	pub num_feature_value_caps   : u16,
	pub num_feature_data_indices : u16,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeviceHandle(usize);

impl DeviceHandle {
	pub fn new(raw: usize) -> DeviceHandle {
		DeviceHandle(raw)
	}

	pub fn is_valid(&self) -> bool {
		self.0 != 0 && self.0 != usize::MAX
	}
}

impl fmt::Debug for DeviceHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("DeviceHandle").field(&format_args!("{:X}",self.0)).finish()
    }
}

pub(crate) enum PreparseDataInternal {
	Address(usize),
	Blob(DynArray<u8>)
}

/// Hid preparsed data
pub struct PreparseData(pub(crate) PreparseDataInternal);

impl PreparseData {
	pub fn new_blob(blob: DynArray<u8>) -> Self {
		PreparseData(PreparseDataInternal::Blob(blob))
	}

	pub(crate) fn get_address(&self) -> usize {
		match &self.0 {
		    PreparseDataInternal::Address(addr) => *addr,
		    PreparseDataInternal::Blob(blob) => blob.as_ptr() as usize,
		}
	}
}

#[repr(transparent)]
#[rustc_layout_scalar_valid_range_start(128)]
#[rustc_nonnull_optimization_guaranteed]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct VendorCollectionType(u8);

impl VendorCollectionType {
	pub unsafe fn new_unchecked(ty: u8) -> Self {
		VendorCollectionType(ty)
	}

	pub fn new(ty: u8) -> Option<Self> {
		if ty >= 0x80 {
			Some(unsafe { Self::new_unchecked(ty) })
		} else {
			None
		}
	}
}

pub enum CollectionKind {
	/// Collection containing physically related values, e.g. group of axes.
	Physical,
	/// Collection containing application defined collection, e.g. mouse or keyboard.
	Application,
	/// Collection containing logically related values, e.g. an association between a data buffer and a byte count.
	Logical,
	/// Report
	Report,
	/// Names array
	NamedArray,
	/// Usage switch
	UsageSwitch,
	/// Usage modified
	UsageModified,
	/// Collection specified by the vendor
	Vendor(VendorCollectionType)
}

impl CollectionKind {
	pub fn from_u8(val: u8) -> Option<Self> {
		match val {
			0 => Some(Self::Physical),
			1 => Some(Self::Application),
			2 => Some(Self::Logical),
			3 => Some(Self::Report),
			4 => Some(Self::NamedArray),
			5 => Some(Self::UsageSwitch),
			6 => Some(Self::UsageModified),
			7..=127 => None,
			vendor => Some(Self::Vendor(unsafe { VendorCollectionType::new_unchecked(vendor) }))
		}
	}
}

/// Hid top-level collection
pub struct TopLevelCollection<'a> {
	/// All collections in the top-level collections
	nodes : DynArray<CollectionNode<'a>>,
}

impl<'a> TopLevelCollection<'a> {
	pub(crate) fn new(mut nodes: DynArray<CollectionNode<'a>>, children: DynArray<DynArray<u16>>) -> Self {
		debug_assert!(nodes.len() > 0, "TopLevelCollection::new() should never be called if there are no nodes");

		// Setup references to nodes
		for (id, children) in children.iter().enumerate() {
			for child_id in children {
				let child = &nodes[*child_id as usize] as * const CollectionNode<'a>;

				// SAFETY: All nodes have the same lifetime as the collection, as `self.nodes` cannot be modified, so they will always refer to valid children.
				nodes[id].children.push(unsafe { &*child });
			}
		}
		Self { nodes: nodes }
	}

	/// Get the node representing the top level collection
	pub fn get_top_node(&self) -> &CollectionNode<'a> {
		&self.nodes[0]
	}

	/// Get the node at a given id
	pub fn get_id(&self, id: u16) -> Option<&'a CollectionNode<'a>> {
		for node in &self.nodes {
			if node.ids.contains(&id) {
				// SAFETY: Node will live as long ass the top level collection
				return Some(unsafe { &*(node as *const CollectionNode<'a>) });
			}
		}
		None
	}
}


/// Hid collection
pub struct CollectionNode<'a> {
	/// Collection id range
	// NOTE: this is not the element idx into `TopLevelCollection::nodes`
	pub ids        : ValueRange<u16>,
	/// All usages for this collection (as defined by aliassed delimiters)
	pub usages     : DynArray<Usage>,
	/// Kind/type of collection
	pub kind       : CollectionKind,
	/// Child collections
	pub children   : DynArray<&'a CollectionNode<'a>>,
}

impl CollectionNode<'_> {
	pub fn get_prefered_usage(&self) -> Usage {
		*self.usages.last().expect("Invalid collection node, `usages` should never be empty")
	}

	pub fn usage_for_id(&self, id: u16) -> Option<Usage> {
		if self.ids.contains(&id) {
			let idx = (id - self.ids.start) as usize;
			Some(self.usages[idx])
		} else {
			None
		}
	}
}

/// Inclusive range (wihout taking the space for the additional bool in RangeInclusive)
#[derive(Clone, Copy)]
pub struct ValueRange<T> {
	pub start : T,
	pub end   : T,
}

impl<T> ValueRange<T> {
	pub fn as_inclusive_range(self) -> RangeInclusive<T> {
		self.start..=self.end
	}
}

impl<T: PartialOrd> ValueRange<T> {
	pub fn contains(&self, val: &T) -> bool {
		<Self as RangeBounds<T>>::contains(&self, val)
	}
}

impl<T> ops::RangeBounds<T> for ValueRange<T> {
    fn start_bound(&self) -> ops::Bound<&T> {
        ops::Bound::Included(&self.start)
    }

    fn end_bound(&self) -> ops::Bound<&T> {
        ops::Bound::Included(&self.end)
    }
}

impl<T: Copy> From<ops::RangeInclusive<T>> for ValueRange<T> {
    fn from(range: ops::RangeInclusive<T>) -> Self {
        ValueRange { start: *range.start(), end: *range.end() }
    }
}

/// The value stored in a data element
pub enum DataValue {
	/// Button
	Button(bool),
	/// Raw value
	Value(u32)
}

/// An individual data element that can be retrieved or set in a report
pub struct Data {
	pub index : u16,
	pub value : DataValue
}

/// Hid report types
#[derive(Clone, Copy, Debug)]
pub enum ReportType {
	Input,
	Output,
	Feature
}
pub(crate) const NUM_REPORT_TYPES : usize = ReportType::Feature as usize + 1;

pub(crate) enum ReportData<'a> {
	Slice(&'a [u8]),
	Blob(DynArray<u8>)
}

impl ReportData<'_> {
	pub fn get_data(&self) -> &[u8] {
		match self {
		    ReportData::Slice(slice) => *slice,
		    ReportData::Blob(arr) => arr,
		}
	}

	pub fn get_mut_data(&mut self) -> &mut [u8] {
		match self {
		    ReportData::Slice(_) => 
				panic!("Slices should never be able to be accessed mutably, if this happens, there is an issue in the onca_hid implementation"),
		    ReportData::Blob(arr) => arr,
		}
	}
}

pub struct InputReport<'a> {
	data : ReportData<'a>,
	device : &'a Device
}

impl<'a> InputReport<'a> {
	/// Create an input report from raw slice
	pub unsafe fn from_raw_slice(raw: &'a [u8], device: &'a Device) -> InputReport<'a> {
		InputReport { data: ReportData::Slice(raw), device }
	}

	/// Get the usage of all button that are currently set to 'on'
	pub fn get_buttons(&self) -> Option<DynArray<Usage>> {
		os::get_buttons(self.device, 0, ReportType::Input, self.data.get_data())
	}

	/// Get the usage of all button that are currently set to 'on', for a specific collection
	pub fn get_buttons_for_collection(&self, collection_id: u16)  -> Option<DynArray<Usage>> {
		os::get_buttons(self.device, collection_id, ReportType::Input, self.data.get_data())
	}

	/// Get the usage of all button that are currently set to 'on', for a specific usage page
	pub fn get_buttons_for_page(&self, page: UsagePageId) -> Option<DynArray<UsageId>> {
		os::get_buttons_for_page(self.device, page, 0, ReportType::Input, self.data.get_data())
	}

	/// Get the usage of all button that are currently set to 'on', for a specific usage page and collection
	pub fn get_buttons_for_page_and_collection(&self, page: UsagePageId, collection_id: u16) -> Option<DynArray<UsageId>> {
		os::get_buttons_for_page(self.device, page, collection_id, ReportType::Input, self.data.get_data())
	}

	/// Get the raw value(s) for the given usage
	pub fn get_raw_value(&self, usage: Usage, collection_id: Option<u16>) -> Option<RawValue> {
		os::get_raw_value(self.device, usage, collection_id.unwrap_or_default(), ReportType::Input, self.data.get_data())
	}

	/// Get the scaled value and its logical range for the given usage
	pub fn get_scaled_value(&self, usage: Usage, collection_id: Option<u16>) -> Option<i32> {
		os::get_scaled_value(self.device, usage, collection_id.unwrap_or_default(), ReportType::Input, self.data.get_data())
	}

	/// Get data from the report, this will return all buttons that are on and all values
	pub fn get_data(&self) -> Option<DynArray<Data>> {
		os::get_data(self.device, ReportType::Input, self.data.get_data())
	}
}

pub struct OutputReport<'a> {
	data   : ReportData<'a>,
	device : &'a Device
}

impl OutputReport<'_> {
	/// Set buttons in the report
	pub fn set_buttons(&mut self, page: UsagePageId, usages: &mut [UsageId]) {
		os::set_buttons(self.device, page, 0, usages, ReportType::Output, self.data.get_mut_data())
	}

	/// Set buttons in the report
	pub fn set_buttons_for_collection(&mut self, page: UsagePageId, collection_id: u16, usages: &mut [UsageId]) {
		os::set_buttons(self.device, page, collection_id, usages, ReportType::Output, self.data.get_mut_data())
	}

	/// Unet buttons in the report
	pub fn unset_buttons(&mut self, page: UsagePageId, usages: &mut [UsageId]) {
		os::unset_buttons(self.device, page, 0, usages, ReportType::Output, self.data.get_mut_data())
	}

	/// Unset buttons in the report
	pub fn unset_buttons_for_collection(&mut self, page: UsagePageId, collection_id: u16, usages: &mut [UsageId]) {
		os::unset_buttons(self.device, page, collection_id, usages, ReportType::Output, self.data.get_mut_data())
	}

	/// Set a value in the report
	pub fn set_value(&mut self, usage: Usage, raw_value: u32) {
		os::set_value(self.device, usage, 0, raw_value, ReportType::Output, self.data.get_mut_data());
	}

	/// Set a value in the report
	pub fn set_value_for_collection(&mut self, usage: Usage, collection_id: u16, raw_value: u32) {
		os::set_value(self.device, usage, collection_id, raw_value, ReportType::Output, self.data.get_mut_data());
	}

	/// Set a value in the report
	pub fn set_values(&mut self, usage: Usage, raw_values: &[u8]) {
		os::set_values(self.device, usage, 0, raw_values, ReportType::Output, self.data.get_mut_data());
	}

	/// Set a value in the report
	pub fn set_values_for_collection(&mut self, usage: Usage, collection_id: u16, raw_values: &[u8]) {
		os::set_values(self.device, usage, collection_id, raw_values, ReportType::Output, self.data.get_mut_data());
	}

	/// Set data in the report
	pub fn set_data(&mut self, data: &[Data]) {
		os::set_data(self.device, data, ReportType::Output, self.data.get_mut_data())
	}
}

pub struct FeatureReport<'a> {
	data   : ReportData<'a>,
	device : &'a Device
}

impl FeatureReport<'_> {
	/// Get the usage of all button that are currently set to 'on'
	pub fn get_buttons(&self) -> Option<DynArray<Usage>> {
		os::get_buttons(self.device, 0, ReportType::Input, self.data.get_data())
	}

	/// Get the usage of all button that are currently set to 'on', for a specific collection
	pub fn get_buttons_for_collection(&self, collection_id: u16)  -> Option<DynArray<Usage>> {
		os::get_buttons(self.device, collection_id, ReportType::Input, self.data.get_data())
	}

	/// Get the usage of all button that are currently set to 'on', for a specific usage page
	pub fn get_buttons_for_page(&self, page: UsagePageId) -> Option<DynArray<UsageId>> {
		os::get_buttons_for_page(self.device, page, 0, ReportType::Input, self.data.get_data())
	}

	/// Get the usage of all button that are currently set to 'on', for a specific usage page and collection
	pub fn get_buttons_for_page_and_collection(&self, page: UsagePageId, collection_id: u16) -> Option<DynArray<UsageId>> {
		os::get_buttons_for_page(self.device, page, collection_id, ReportType::Input, self.data.get_data())
	}

	/// Get the raw value(s) for the given usage
	pub fn get_raw_value(&self, usage: Usage, collection_id: Option<u16>) -> Option<RawValue> {
		os::get_raw_value(self.device, usage, collection_id.unwrap_or_default(), ReportType::Input, self.data.get_data())
	}

	/// Get the scaled value and its logical range for the given usage
	pub fn get_scaled_value(&self, usage: Usage, collection_id: Option<u16>) -> Option<i32> {
		os::get_scaled_value(self.device, usage, collection_id.unwrap_or_default(), ReportType::Input, self.data.get_data())
	}

	/// Get data from the report, this will return all buttons that are on and all values
	pub fn get_data(&self) -> Option<DynArray<Data>> {
		os::get_data(self.device, ReportType::Input, self.data.get_data())
	}

	/// Set buttons in the report
	pub fn set_buttons(&mut self, page: UsagePageId, usages: &mut [UsageId]) {
		os::set_buttons(self.device, page, 0, usages, ReportType::Output, self.data.get_mut_data())
	}

	/// Set buttons in the report
	pub fn set_buttons_for_collection(&mut self, page: UsagePageId, collection_id: u16, usages: &mut [UsageId]) {
		os::set_buttons(self.device, page, collection_id, usages, ReportType::Output, self.data.get_mut_data())
	}

	/// Unet buttons in the report
	pub fn unset_buttons(&mut self, page: UsagePageId, usages: &mut [UsageId]) {
		os::unset_buttons(self.device, page, 0, usages, ReportType::Output, self.data.get_mut_data())
	}

	/// Unset buttons in the report
	pub fn unset_buttons_for_collection(&mut self, page: UsagePageId, collection_id: u16, usages: &mut [UsageId]) {
		os::unset_buttons(self.device, page, collection_id, usages, ReportType::Output, self.data.get_mut_data())
	}

	/// Set a value in the report
	pub fn set_value(&mut self, usage: Usage, raw_value: u32) {
		os::set_value(self.device, usage, 0, raw_value, ReportType::Output, self.data.get_mut_data());
	}

	/// Set a value in the report
	pub fn set_value_for_collection(&mut self, usage: Usage, collection_id: u16, raw_value: u32) {
		os::set_value(self.device, usage, collection_id, raw_value, ReportType::Output, self.data.get_mut_data());
	}

	/// Set a value in the report
	pub fn set_values(&mut self, usage: Usage, raw_values: &[u8]) {
		os::set_values(self.device, usage, 0, raw_values, ReportType::Output, self.data.get_mut_data());
	}

	/// Set a value in the report
	pub fn set_values_for_collection(&mut self, usage: Usage, collection_id: u16, raw_values: &[u8]) {
		os::set_values(self.device, usage, collection_id, raw_values, ReportType::Output, self.data.get_mut_data());
	}

	/// Set data in the report
	pub fn set_data(&mut self, data: &[Data]) {
		os::set_data(self.device, data, ReportType::Output, self.data.get_mut_data())
	}
}

/// Button capabilities (report descriptor)
pub struct ButtonCaps {
	/// Usage page for all usages
	pub usage_page    : UsagePageId,
	/// Report id
	pub report_id     : u8,
	/// Data fields associated with the main item
	pub data_fields   : u16,
	/// Index of the collection the capabilites are part of.
	pub collection_id : u16,
	/// Number of reports
	pub report_count  : u16,
	/// Usages, if the report count is higher that the usage `range`, the last usage is used for all subsequent reports
	pub usage         : ValueRange<UsageId>,
	/// String indices, if the report count is higher that the index `range`, the last index is used for all subsequent reports
	pub string_index  : ValueRange<u16>,
	/// Designators, if the report count is higher that the designator `range`, the last designator is used for all subsequent reports
	pub designator    : ValueRange<u16>,
	/// data indices, if the report count is higher that the index `range`, the last index is used for all subsequent reports
	pub data_index    : ValueRange<u16>,
	/// If `true`, the value provides an absolute range, otherwise the data is relative to the previous value
	pub is_absolute   : bool,
}

/// Value capabilities (report descriptor)
pub struct ValueCaps {
	/// Usage page for all usages
	pub usage_page     : UsagePageId,
	/// Report id
	pub report_id      : u8,
	/// Data fields associated with the main item
	pub data_fields    : u16,
	/// Index of the collection the capabilites are part of.
	pub collection_id  : u16,
	/// Does the value have a 'null' state
	pub has_null       : bool,
	/// Unit exponent
	pub unit_exp       : u32,
	/// Unit type
	pub units          : u32,
	/// Logical value range (raw value range)
	pub logical_range  : ValueRange<i32>,
	/// Physical value range (after scaling)
	pub physical_range : ValueRange<i32>,
	/// Bit size of each field
	pub bit_size       : u16,
	/// Number of reports
	pub report_count   : u16,
	/// Usages, if the report count is higher that the usage `range`, the last usage is used for all subsequent reports
	pub usage          : ValueRange<UsageId>,
	/// String indices, if the report count is higher that the index `range`, the last index is used for all subsequent reports
	pub string_index   : ValueRange<u16>,
	/// Designators, if the report count is higher that the designator `range`, the last designator is used for all subsequent reports
	pub designator     : ValueRange<u16>,
	/// data indices, if the report count is higher that the index `range`, the last index is used for all subsequent reports
	pub data_index     : ValueRange<u16>,
	/// If `true`, the value provides an absolute range, otherwise the data is relative to the previous value
	pub is_absolute    : bool,
}

impl ValueCaps {
	/// Get the maximum value of the raw value (raw value in in range `0..=max`)
	pub fn get_raw_value_max(&self) -> u32 {
		(1u64 << self.bit_size as u64) as u32
	}
}

/// Raw unscaled HID value
pub enum RawValue {
	/// Stored as a pair of raw bits and a bit-size
	/// 
	/// To get a signed representation, use the bitsize to sign-extend the value
	Single(u32, u16),
	/// Sequentially stored values and a bit-size
	/// 
	/// To extract the values, the bit-size indicated the number of bits per packed value. Values are not byte aligned!
	Array(DynArray<u8>, u16)
}

impl RawValue {
	/// Get the first value
	pub fn first(&self) -> u32 {
		match self {
		    RawValue::Single(val, _) => *val,
		    RawValue::Array(arr, bit_size) => {
				// Unlikely to happen
				if arr.len() == 0 {
					return 0;
				}

				let mask = 0xFFFF_FFFF >> (32 - bit_size);
				if arr.len() >= mem::size_of::<u32>() {
					return unsafe { *(arr.as_ptr() as *const u32) & mask };
				}

				match bit_size {
					0 ..=8  => { arr[0] as u32 & mask }
					0 ..=16 => unsafe { *(arr.as_ptr() as *const u16) as u32 & mask }
					17..=24 => unsafe { (*(arr.as_ptr() as *const u16) as u32 | (arr[3] as u32) << 16) & mask }
					_ => unreachable!()
				}
			},
		}
	}
}

/// HID device
pub struct Device {
	os_dev        : OSDevice,
	handle        : DeviceHandle,
	identifier    : Identifier,
	preparse_data : PreparseData,
	capabilities  : Capabilities,
	button_caps   : [DynArray<ButtonCaps>; NUM_REPORT_TYPES],
	value_caps    : [DynArray<ValueCaps>; NUM_REPORT_TYPES],
	read_buffer   : DynArray<u8>,
	owns_handle   : bool,
	read_pending  : bool,
}

impl Device {
	pub fn new_path(path: &str) -> Option<Self> {
		os::open_device(path).and_then(|handle| Self::_new(handle, true))
	}

	/// Create a new HID device.
	/// 
	/// If an invalid handle is passed, `None` will be returned.
	pub fn new_handle(handle: DeviceHandle) -> Option<Self> {
		Self::_new(handle, false)
	}

	/// Create a new HID device from raw data.
	pub fn new_raw(handle: DeviceHandle, preparse_data: PreparseData, identifier: Identifier) -> Option<Self> {
		Self::_new_raw(handle, preparse_data, identifier, false)
	}

	fn _new(handle: DeviceHandle, owns_handle: bool) -> Option<Self> {
		if handle.is_valid() {
			let preparse_data =  match os::get_preparse_data(handle) {
			    Some(data) => data,
			    None => return None,
			};

			let identifier = match os::get_identifier(handle, &preparse_data) {
				Some(iden) => iden,
			    None => return None,
			};

			Self::_new_raw(handle, preparse_data, identifier, owns_handle)
		} else {
			None
		}
	}

	fn _new_raw(handle: DeviceHandle, preparse_data: PreparseData, identifier: Identifier, owns_handle: bool) -> Option<Self> {
		let os_dev = match os::create_os_device(&handle) {
		    Some(os_dev) => os_dev,
		    None => return None,
		};

		let capabilities = match os::get_capabilities(&preparse_data) {
		    Some(caps) => caps,
		    None => return None,
		};

		let button_caps = match os::get_button_capabilities(&preparse_data, &capabilities) {
			Some(caps) => caps,
			None => return None,
		};

		let value_caps = match os::get_value_capabilities(&preparse_data, &capabilities) {
			Some(caps) => caps,
			None => return None,
		};

		Some(Self { os_dev, handle, identifier, preparse_data, capabilities, button_caps, value_caps, read_buffer: DynArray::new(), owns_handle, read_pending: false })
	}

	/// Get the device handle
	pub fn handle(&self) -> DeviceHandle {
		self.handle
	}

	/// Get the device identifier
	pub fn identifier(&self) -> &Identifier {
		&self.identifier
	}

	/// Get the vendor string.
	/// 
	/// This should normally match the string which can be found using `UsbDevice::new(...).name`.
	/// 
	/// If the vendor string could not be retrieved, `None` is returned
	pub fn get_vendor_string(&self) -> Option<String> {
		match os::get_vendor_string(self.handle) {
		    Some(s) => Some(s),
			// If we can't get the string directly from the device, check if we can't get it statically from the know vendors
		    None => self.identifier.vendor_device.get_vendor_string(),
		}
	}

	/// Get the product string.
	/// 
	/// This should normally match the string which can be found using `UsbDevice::new(...).get_device(...).name`.
	pub fn get_product_string(&self) -> Option<String> {
		match os::get_product_string(self.handle) {
		    Some(s) => Some(s),
			// If we can't get the string directly from the device, check if we can't get it statically from the know vendors
		    None => self.identifier.vendor_device.get_device_string(),
		}
	}

	/// Get the serial number string.
	pub fn get_serial_number_string(&self) -> Option<String> {
		os::get_serial_number_string(self.handle)
	}

	/// Get an indexed string.
	pub fn get_indexed_string(&self, index: usize) -> Option<String> {
		os::get_indexed_string(self.handle, index)
	}

	/// Get the number of reports that can fit in the HIDs ring buffer used to queue input reports
	pub fn get_num_input_buffers(&self) -> Option<NonZeroU32> {
		os::get_num_input_buffers(self.handle)
	}

	/// Set the number of reports that can fit in the HIDs ring buffer used to queue input reports
	pub fn set_num_input_buffers(&self, num_buffers: u32) {
		if num_buffers < 2 {
			log_warning!(LOG_HID_CAT, "The HID device requires at minimum 2 input buffers");
		} else {
			os::set_num_input_buffers(self.handle, num_buffers);
		}
	}

	/// Get the device HID capabilities
	pub fn get_capabilities(&self) -> &Capabilities {
		&self.capabilities
	}

	/// Get the device's button capabilities
	pub fn get_button_capabilities(&self, report_type: ReportType) -> &DynArray<ButtonCaps> {
		&self.button_caps[report_type as usize]
	}

	/// Get the button capabilities for a specific usage page and an optional collection id
	pub fn get_button_capabilities_for_page(&self, report_type: ReportType, page: UsagePageId, collection_id: Option<u16>) -> Option<&ButtonCaps> {
		let collection_id = collection_id.unwrap_or_default();

		let mut ret = None;
		for caps in &self.button_caps[report_type as usize] {
			if caps.usage_page == page {
				if collection_id == caps.collection_id {
					return Some(caps);
				} else if collection_id == 0 {
					ret = Some(caps)
				}
			}
		}
		ret
	}

	/// Get the device's value capabilities
	pub fn get_value_capabilities(&self, report_type: ReportType) -> &DynArray<ValueCaps> {
		&self.value_caps[report_type as usize]
	}

	/// Get the value capabilities for a specific usage and an optional collection id
	pub fn get_value_capabilities_for_usage(&self, report_type: ReportType, usage: Usage, collection_id: Option<u16>) -> Option<&ValueCaps> {
		let collection_id = collection_id.unwrap_or_default();

		let mut ret = None;
		for caps in &self.value_caps[report_type as usize] {
			if caps.usage_page == usage.page && caps.usage.contains(&usage.usage) {
				if collection_id == caps.collection_id {
					return Some(caps);
				} else if collection_id == 0 {
					ret = Some(caps)
				}
			}
		}
		ret
	}
	
	/// Get the HID collections for the device
	pub fn get_top_level_collection(&self) -> Option<TopLevelCollection<'_>> {
		os::get_top_level_collection(&self)
	}

	/// Create an output report
	pub fn create_output_report(&self, report_id: u8) -> Option<OutputReport<'_>> {
		let blob = os::create_report_data(self, ReportType::Output, report_id)?;
		Some(OutputReport { data: ReportData::Blob(blob), device: self })
	}

	/// Create a feature report
	pub fn create_feature_report(&self, report_id: u8) -> Option<FeatureReport<'_>> {
		let blob = os::create_report_data(self, ReportType::Output, report_id)?;
		Some(FeatureReport { data: ReportData::Blob(blob), device: self })
	}

	/// Flush the device's input buffer
	pub fn flush_input_queue(&self) {
		os::flush_input_queue(self.handle)
	}

	/// Read an input report
	/// 
	/// If a failure occured while trying to read a report, an `Err` will be returned.
	/// 
	/// If the read is successfull, `Ok(None)` can return, meaning that the io operation is still pending
	pub fn read_input_report(&mut self, timeout: Duration) -> Result<Option<InputReport>, ()> {
		os::read_input_report(self, timeout)
	}

	/// Write an output report
	/// 
	/// If a failure occured while trying to write the report, an error will be returned with the report that could not be written.
	/// 
	/// This function is synchronous and will error if writing takes longer than 1 second.
	pub fn write_output_report<'a>(&mut self, report: OutputReport<'a>) -> Result<(), OutputReport<'a>> {
		os::write_output_report(self, report)
	}

	/// Get the feature report from the device
	pub fn get_feature_report(&mut self) -> Option<FeatureReport> {
		os::get_feature_report(self)
	}

	/// Set the feature report of the device
	/// 
	/// If a failure occured while trying to set the feature report, an error will be returned with the report that could not be set.
	pub fn set_feature_report<'a>(&mut self, report: FeatureReport<'a>) -> Result<(), FeatureReport<'a>> {
		os::set_feature_report(self, report)
	}
}

impl Drop for Device {
    fn drop(&mut self) {
        os::free_preparse_data(&mut self.preparse_data);
		os::destroy_os_device(&mut self.os_dev);

		if self.owns_handle {
			os::close_handle(self.handle);
		}
    }
}

pub fn iterator_devices() {

}