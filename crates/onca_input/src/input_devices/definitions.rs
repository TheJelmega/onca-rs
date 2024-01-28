use std::collections::HashMap;

use onca_common_macros::flags;
use onca_hid::{self as hid, VendorProduct};
use onca_logging::{log_error, log_warning};
use onca_toml::{self as toml, Toml};

use crate::LOG_INPUT_CAT;


#[derive(Clone, Copy, Debug)]
pub struct UsageDef {
    /// HID usage
    pub usage:  hid::Usage,
    /// Report index
    pub report: u16,
}

impl UsageDef {
    pub fn new(usage: hid::Usage, report: u16) -> Self {
        Self { usage, report }
    }
}

/// Input device definition kind
#[flags]
pub enum DefinitionKind {
    /// Gamepad
    Gamepad,
    /// Touch
    Touch,
    /// Sensors
    Sensor,
}

impl DefinitionKind {
    pub fn from_single_str(name: &str) -> DefinitionKind {
        match name {
            "gamepad" => DefinitionKind::Gamepad,
            "touch"   => DefinitionKind::Touch,
            "sensor"  => DefinitionKind::Sensor,
            _         => DefinitionKind::None,
        }
    }
}

/// Input device axis definition
#[derive(Clone, Copy, Debug)]
pub struct DefinitionAxis {
    pub x: UsageDef,
    pub y: Option<UsageDef>,
    pub z: Option<UsageDef>,
}

#[derive(Clone, Copy, Debug)]
pub struct DefinitionDpadDiagonals {
    pub up_left:    UsageDef,
    pub down_left:  UsageDef,
    pub up_right:   UsageDef,
    pub down_right: UsageDef,
}

#[derive(Clone, Copy, Debug)]
pub enum DefinitionDPad {
    Hat{
        usage:   UsageDef,
        neutral: u8,
    },
    Buttons {
        up:    UsageDef,
        down:  UsageDef,
        left:  UsageDef,
        right: UsageDef,
        diags: Option<DefinitionDpadDiagonals>,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DefinitionSensors {
    pub gyroscope:     Option<DefinitionAxis>,
    pub accelerometer: Option<DefinitionAxis>,
}

/// Input device HID definition
#[derive(Clone)]
pub struct InputDeviceDefinition {
    pub kind:           DefinitionKind,
    pub name:           String,
    pub vendor_product: VendorProduct,
    pub buttons:        HashMap<String, UsageDef>,
    pub axes:           HashMap<String, DefinitionAxis>,
    pub dpad:           Option<DefinitionDPad>,
    pub touch:          Vec<DefinitionAxis>,
    pub sensors:        DefinitionSensors,
}

pub fn parse_definitions(toml: &Toml) -> Vec<InputDeviceDefinition> {
    let mut defs = Vec::new();

    let src_defs = toml.get("definition");
    if let Some(toml::Item::Array(def_items)) = src_defs {
        for item in def_items {
            if let Some(def) = parse_definition(&item) {
                defs.push(def);
            }
        }
    }

    defs
}

fn parse_definition(item: &toml::Item) -> Option<InputDeviceDefinition> {
    let item = match item {
        toml::Item::Table(item) => item,
        _ => {
            log_error!(LOG_INPUT_CAT, parse_definition, "Expected a table to parse a device definition");
            return None;
        }
    };

    let kind = if let Some(kind_str) = item.get::<String>("kind") {
        let kind = DefinitionKind::from_single_str(&kind_str);
        if kind.is_none() {
            log_error!(LOG_INPUT_CAT, parse_definition, "Unknown input device definition kind");
                return None;
        }
        kind
    } else if let Some(kind_arr) = item.get::<Vec<toml::Item>>("kind") {
        let mut kind = DefinitionKind::None;
        for item in kind_arr {
            match item {
                toml::Item::String(name) => {
                    let tmp = DefinitionKind::from_single_str(&name);
                    if tmp.is_none() {
                        log_warning!(LOG_INPUT_CAT, "Skipping invalid input definition kind '{name}'");
                    }
                    kind |= tmp;
                },
                _ => {
                    log_warning!(LOG_INPUT_CAT, "Skipping invalid input definition kind");
                }
            }
        }
        kind
    } else {
        log_error!(LOG_INPUT_CAT, parse_definition, "An input device definition requires a kind to be specified");
        return None;
    };

    let vid = if let Some(vid) = item.get::<i64>("vid") {
        vid
    } else {
        log_error!(LOG_INPUT_CAT, parse_definition, "An input device definition requires a pid (product id) to be specified");
        return None;
    };

    let pid = if let Some(pid) = item.get::<i64>("pid") {
        pid
    } else {
        log_error!(LOG_INPUT_CAT, parse_definition, "An input device definition requires a pid (product id) to be specified");
        return None;
    };

    let name = if let Some(name) = item.get::<String>("name") {
        name.clone()
    } else {
        log_warning!(LOG_INPUT_CAT, "No device name, defaulting to 'generic {kind}'");
        format!("generic {kind}")
    };

    let mut buttons = HashMap::new();
    if let Some(button_table) = item.get::<toml::Table>("buttons") {
        for entry in button_table {
            match parse_usage(entry.1, "button") {
                Ok(button) => _ = buttons.insert(entry.0.to_string(), button),
                Err(err) => log_error!(LOG_INPUT_CAT, parse_definition, "Error processing device `{name}` (VID: {vid:X}, PID: {pid:X}): {err}"),
            }
        }
    }

    let mut axes = HashMap::new();
    if let Some(axis_table) = item.get::<toml::Table>("axes") {
        for entry in axis_table {
            match parse_axis(entry.1) {
                Ok(axis) => _ = axes.insert(entry.0.to_string(), axis),
                Err(err) => log_error!(LOG_INPUT_CAT, parse_definition, "{err}"),
            }
        }
    }

    let dpad = match item.get::<toml::Table>("dpad") {
        Some(table) => match parse_dpad(table) {
            Ok(dpad) => Some(dpad),
            Err(err) => {
                log_error!(LOG_INPUT_CAT, parse_definition, "Error parsing device dpad `{name}` (VID: {vid:X}, PID: {pid:X}): {err}");
                None
            },
        },
        None => None,
    };

    let touch = match item.get::<toml::Table>("touch") {
        Some(table) => match parse_touch(table) {
            Ok(touch) => touch,
            Err(err) => {
                log_error!(LOG_INPUT_CAT, parse_definition, "Error parsing device touch `{name}` (VID: {vid:X}, PID: {pid:X}): {err}");
                Vec::new()
            },
        },
        None => Vec::new(),
    };

    let sensors = match item.get::<toml::Table>("sensors") {
        Some(table) => match parse_sensors(table) {
            Ok(sensors) => sensors,
            Err(err) => {
                log_error!(LOG_INPUT_CAT, parse_definition, "Error parsing device sensors `{name}` (VID: {vid:X}, PID: {pid:X}): {err}");
                DefinitionSensors {
                    gyroscope: None,
                    accelerometer: None,
                }
            },
        },
        None => DefinitionSensors {
            gyroscope: None,
            accelerometer: None,
        },
    };

    Some(InputDeviceDefinition {
        kind,
        name,
        vendor_product: VendorProduct::from_u16(*vid as u16, *pid as u16),
        buttons,
        axes,
        dpad,
        touch,
        sensors,
    })
}

fn parse_usage(item: &toml::Item, iden: &str) -> Result<UsageDef, String> {
    if let toml::Item::Table(table) = item {
        let page = table.get::<i64>("page").map_or(Err(format!("Missing `page` for {iden} definition")), |val| Ok(*val as u16))?;
        let usage = table.get::<i64>("usage").map_or(Err(format!("Missing `usage` for {iden} definition")), |val| Ok(*val as u16))?;
        let hid_usage = hid::Usage::from_u16(page, usage);

        let report = table.get::<i64>("report").map_or(0, |val| *val as u16);

        Ok(UsageDef::new(hid_usage, report))
    } else {
        Err(format!("Expected inline table with page/usage for {iden} definition"))
    }
}

fn parse_axis(item: &toml::Item) -> Result<DefinitionAxis, String> {
    let table = match item {
        toml::Item::Table(table) => table,
        _ => return Err("Expected an inline table for an axis definition".to_string()),
    };

    let z = table.get_item("z").map_or(Ok(None), |item| parse_usage(item, "z-axis").map(|val| Some(val)))?;
    let y = table.get_item("y").map_or(Ok(None), |item| parse_usage(item, "y-axis").map(|val| Some(val)))?;
    let x = table.get_item("x").map_or_else(
        || parse_usage(item, "axis"),
        |item| parse_usage(item, "x-axis")
    )?;

    Ok(DefinitionAxis { x, y, z })
}
 
fn parse_dpad(table: &toml::Table) -> Result<DefinitionDPad, String> {
    match table.get::<String>("kind").map_or(Err("Missing `kind` dpad definition".to_string()), |val| Ok(val))?.as_str() {
        "hat" => {
            let hat_item = table.get_item("hat").map_or(Err("Missing `hat` for dpad definition".to_string()), |val| Ok(val))?;
            let neutral = table.get::<i64>("neutral").map_or(0, |val| *val as u8);
            Ok(DefinitionDPad::Hat{ usage: parse_usage(hat_item, "dpad hat")?, neutral })
        },
        "buttons" => {
            let up = table.get_item("up").map_or(Err("Missing 'up' for button dpad".to_string()), |item| parse_usage(item, "dpad up"))?;
            let down = table.get_item("down").map_or(Err("Missing 'down' for button dpad".to_string()), |item| parse_usage(item, "dpad down"))?;
            let left = table.get_item("left").map_or(Err("Missing 'left' for button dpad".to_string()), |item| parse_usage(item, "dpad left"))?;
            let right = table.get_item("right").map_or(Err("Missing 'right' for button dpad".to_string()), |item| parse_usage(item, "dpad right"))?;

            let up_left = table.get_item("up_left");
            let up_right = table.get_item("up_right");
            let down_left = table.get_item("down_left");
            let down_right = table.get_item("down_right");

            let any_diag = up_left.is_some() || up_right.is_some() || down_left.is_some() || down_right.is_some();
            let diags = if any_diag {
                let up_left = up_left.map_or(Err("Missing `up_left` for button dpad with diagonal".to_string()), |item| parse_usage(item, "dpad up_left"))?;
                let up_right = up_right.map_or(Err("Missing `up_right` for button dpad with diagonal".to_string()), |item| parse_usage(item, "dpad up_left"))?;
                let down_left = down_left.map_or(Err("Missing `down_left` for button dpad with diagonal".to_string()), |item| parse_usage(item, "dpad up_left"))?;
                let down_right = down_right.map_or(Err("Missing `down_right` for button dpad with diagonal".to_string()), |item| parse_usage(item, "dpad up_left"))?;

                Some(DefinitionDpadDiagonals {
                    up_left,
                    down_left,
                    up_right,
                    down_right,
                })
            } else {
                None
            };

            Ok(DefinitionDPad::Buttons {
                up,
                down,
                left,
                right,
                diags
            })
        },
        _ => {
            Err("Dpad's kind needs to be eitehr 'hat' or 'buttons".to_string())
        }
    }
}

fn parse_touch(table: &toml::Table) -> Result<Vec<DefinitionAxis>, String> {
    let touch_item = table.get_item("touch").map_or(Err("Missing `touch` for the touch section".to_string()), |item| Ok(item))?;
    match touch_item {
        toml::Item::Array(arr) => {
            let mut axes = Vec::new();
            for item in arr {
                axes.push(parse_axis(item)?);
            }
            Ok(axes)
        },
        toml::Item::Table(_) => {
            let axis = parse_axis(touch_item)?;
            Ok(vec![axis])
        },
        _ => Err("Expected an inline table or an array for the `touch` element".to_string())
    }
}

fn parse_sensors(table: &toml::Table) -> Result<DefinitionSensors, String> {
    let gyroscope = table.get_item("gyroscope").map_or(Ok(None), |item| parse_axis(item).map(|val| Some(val)))?;
    let accelerometer = table.get_item("accelerometer").map_or(Ok(None), |item| parse_axis(item).map(|val| Some(val)))?;
    Ok(DefinitionSensors {
        gyroscope,
        accelerometer,
    })
}