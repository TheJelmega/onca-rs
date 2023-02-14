use std::{fs::*, io::*, mem::take};

enum SubSections {
    None,
    Vendor,
    HidPages,
}

struct Vendor {
    id      : u16,
    name    : String,
    devices : Vec<Device>
}

struct Device {
    id   : u16,
    name : String,
}

struct HidUsagePage {
    id     : u8,
    name   : String,
    usages : Vec<HidUsage>
}

struct HidUsage {
    id   : u16,
    name : String
}


/// This will be in onca_hid/build.rs
// TODO: Better data structure, as this is a sparse array
fn main() {
    let input_file = File::open("src/usb.ids").unwrap();
    let input = BufReader::new(input_file);

    let table_output_file = File::create("src/hid.generated.rs").unwrap();
    let mut table_output = BufWriter::new(table_output_file);

    let mut version : String = String::new();
    let mut sub_section = SubSections::None;
 
    let mut vendors : Vec<Vendor> = Vec::new();
    let mut cur_vendor : Option<Vendor> = None;
    let mut cur_device : Option<Device> = None;

    let mut hid_usage_pages : Vec<HidUsagePage> = Vec::new();
    let mut cur_usage_page : Option<HidUsagePage> = None;
    let mut cur_usage : Option<HidUsage> = None;

    for line in input.lines() {
        let line = line.unwrap();

        if line.is_empty() {
            continue;
        }
        
        if line.starts_with("#") {
            if line.starts_with("# Version: ") {
                version = line[11..].to_string();
            }
            continue;
        }

        if !line.starts_with("\t") {
            if line.starts_with("HUT") {
                sub_section = SubSections::HidPages;

                let id = u8::from_str_radix(&line[4..6], 16).unwrap();
                let name = to_escaped_string(&line[8..]);

                if let Some(mut usage_page) = take(&mut cur_usage_page) {
                    if let Some(usage) = take(&mut cur_usage) {
                        usage_page.usages.push(usage);
                    }

                    hid_usage_pages.push(usage_page);
                }

                cur_usage_page = Some(HidUsagePage { id, name, usages: Vec::new() });

            } else if let Ok(id) = u16::from_str_radix(&line[..4], 16) {
                sub_section = SubSections::Vendor;

                if let Some(mut vendor) = take(&mut cur_vendor) {
                    if let Some(device) = take(&mut cur_device) {
                        vendor.devices.push(device);
                    }

                    vendors.push(vendor);
                }

                let name = to_escaped_string(&line[6..]);

                cur_vendor = Some(Vendor{ id, name, devices: Vec::new() });
            } else {
                sub_section = SubSections::None;
            }
        } else {
            match sub_section {
                SubSections::None => (),
                SubSections::Vendor => {
                    if !line.starts_with("\t\t") {
                        let cur_vendor = match &mut cur_vendor {
                            Some(vendor) => vendor,
                            None => panic!("No vendor, input is malformed"),
                        };
                        
                        if let Some(device) = take(&mut cur_device) {
                            cur_vendor.devices.push(device);
                        }
                        
                        let name = to_escaped_string(&line[7..]);
                        let err_text = std::format!("Failed to parse id for \"{}\" ('{}')", name, &line[1..5]);
                        
                        let id = u16::from_str_radix(&line[1..5], 16).expect(&err_text);
                        cur_device = Some(Device { id,  name });
                        
                    }
                },
                SubSections::HidPages => {
                    let cur_usage_page = match &mut cur_usage_page {
                        Some(page) => page,
                        None => panic!("No usage page, input is malformed"),
                    };
                    
                    let mut name = to_escaped_string(&line[6..]);
                    let err_text = std::format!("Failed to parse id for \"{}\" ('{}')", name, &line[1..4]);

                    // There is a mistake in the some versions of the linux USB id repository
                    if name == "Keypad 8 and Page Up" {
                        name = "Keypad 9 and Page Up".to_string();
                    }

                    let id = u16::from_str_radix(&line[1..4], 16).expect(&err_text);

                    cur_usage_page.usages.push(HidUsage { id, name });
                },
            }
        }

        
    }

    if let Some(mut vendor) = take(&mut cur_vendor) {
        if let Some(device) = take(&mut cur_device) {
            vendor.devices.push(device);
        }
        
        vendors.push(vendor);
    }
    if let Some(mut usage_page) = take(&mut cur_usage_page) {
        if let Some(usage) = take(&mut cur_usage) {
            usage_page.usages.push(usage);
        }

        hid_usage_pages.push(usage_page);
    }


    // Write out data

    // Vendors
    table_output.write_fmt(format_args!(
r"//! Generated HID tables
//! 
//! Data retrieved from http://www.linux-usb.org/usb-ids.html version {version}

use crate::{{UsbVendorId, UsbVendor, UsbDeviceId, UsbDevice, HidUsagePageId, HidUsagePage, HidUsage, HidUsageId}};
"
    )).unwrap();

    //------------------------------------------------------------------------------------------------------------------------------
    // USB VENDORS AND DEVICES
    //------------------------------------------------------------------------------------------------------------------------------

    table_output.write_fmt(format_args!(r"
pub(crate) const USB_VENDORS: &[UsbVendor] = &[
"
        )).unwrap();

    for vendor in vendors {
        table_output.write_fmt(format_args!("\tUsbVendor {{ id: UsbVendorId::new(0x{:04X}), name: \"{}\", devices: ", vendor.id, vendor.name)).unwrap();

        if vendor.devices.is_empty() {
            table_output.write_fmt(format_args!("None }},\n")).unwrap();
        } else {
            table_output.write_fmt(format_args!("Some(&[\n")).unwrap();
            for device in vendor.devices {
                table_output.write_fmt(format_args!("\t\tUsbDevice {{ id: UsbDeviceId::new(0x{:04X}), name: \"{}\"}},\n", device.id, device.name)).unwrap();
            }
            table_output.write_fmt(format_args!("\t]) }},\n")).unwrap();
        }
    }

    table_output.write_fmt(format_args!(r"
];
"
    )).unwrap();

    //------------------------------------------------------------------------------------------------------------------------------
    // HID USAGES AND PAGES
    //------------------------------------------------------------------------------------------------------------------------------

    table_output.write_fmt(format_args!(r"
pub(crate) const HID_USAGE_PAGES : &[HidUsagePage] = &[
"
    )).unwrap();

    for page in &hid_usage_pages {
        table_output.write_fmt(format_args!("\tHidUsagePage {{ id: HidUsagePageId::new(0x{:04X}), name: \"{}\", usages: ", page.id, page.name)).unwrap();

        if page.usages.is_empty() {
            table_output.write_fmt(format_args!("None }},\n")).unwrap();
        } else {
            table_output.write_fmt(format_args!("Some(&[\n")).unwrap();
            for usage in &page.usages {
                table_output.write_fmt(format_args!("\t\tHidUsage {{ id: HidUsageId::new(0x{:04X}), name: \"{}\"}},\n", usage.id, usage.name)).unwrap();
            }
            table_output.write_fmt(format_args!("\t]) }},\n")).unwrap();
        }
    }

    table_output.write_fmt(format_args!(r"
];
"
    )).unwrap();

    //------------------------------------------------------------------------------------------------------------------------------
    // BUILD.RS CARGO NOTIFICATIONS
    //------------------------------------------------------------------------------------------------------------------------------

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/usb.ids");
}

fn to_escaped_string(s: &str) -> String {
    s.to_owned().replace('\\', "\\\\").replace('\"', "\\\"")
}