use onca_common::{prelude::*, utils};
use crate::{os, Window, PhysicalSize};

pub type OSMonitorHandle = os::MonitorHandle;

/// Monitor rect
#[derive(Clone, Copy, Default, Debug)]
pub struct MonitorRect {
    pub x      : i32,
    pub y      : i32,
    pub width  : u16,
    pub height : u16
}

/// Monitor graphics mode
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct MonitorMode {
    pub bits_per_pixel : u8,
    pub refesh_rate    : u32,
    pub size           : PhysicalSize,
}

/// Monitor info
pub struct Monitor {
    pub(crate) os_handle    : os::MonitorHandle,
    pub(crate) mon_rect     : MonitorRect,
    pub(crate) work_rect    : MonitorRect,
    pub(crate) dpi          : u16,
    pub(crate) refresh_rate : f32,
    pub(crate) primary      : bool,
    pub(crate) dev_name     : [u8; 32],
    pub(crate) name         : [u8; 128],
    pub(crate) modes        : Vec<MonitorMode>,
}

impl Monitor {
    /// Enumerate over all attached monitors and return a array of them.
    pub fn enumerate_monitors() -> Vec<Monitor> {
        os::monitor::enumerate_monitors()
    }

    /// Get the primary monitor. If a monitor cannot be detected, return `None`.
    pub fn primary() -> Option<Monitor> {
        os::monitor::primary_monitor()
    }

    /// Get the monitor with the largest overlap with a window. If a monitor cannot be detected, return `None`.
    pub fn from_window(window: Window) -> Option<Monitor> {
        os::monitor::get_monitor_from_hwnd(window.os_handle().hwnd())
    }

    /// Get the monitor with the largest overlap of the given rect. If the rect does not overlap a monitor, return `None`.
    pub fn from_largest_overlap(rect: MonitorRect) -> Option<Monitor> {
        os::monitor::get_monitor_from_largest_overlap(rect)
    }

    /// Get the monitor at a certain point. If the point does not fall inside a monitor, return `None`.
    pub fn at(x: i32, y: i32) -> Option<Monitor> {
        os::monitor::get_monitor_at(x, y)
    }

    /// Get the monitors OS handle.
    pub fn os_handle(&self) -> OSMonitorHandle {
        self.os_handle
    }

    /// Get the full monitor rect.
    pub fn monitor_rect(&self) -> &MonitorRect {
        &self.mon_rect
    }

    /// Get the work area rect (work area is the full monitor, without always-on-top OS elements e.g. task bar).
    pub fn work_rect(&self) -> &MonitorRect {
        &self.mon_rect
    }

    /// Get the monitor's position.
    pub fn position(&self) -> (i32, i32) {
        (self.mon_rect.x, self.mon_rect.y)
    }

    /// Get the monitor's size.
    pub fn size(&self) -> (u16, u16) {
        (self.mon_rect.width, self.mon_rect.height)
    }

    /// Get the working area's position.
    pub fn work_position(&self) -> (i32, i32) {
        (self.work_rect.x, self.work_rect.y)
    }

    /// Get the working area's size.
    pub fn work_size(&self) -> (u16, u16) {
        (self.work_rect.width, self.work_rect.height)
    }

    /// Get the monitors refresh rate.
    pub fn refresh_rate(&self) -> f32 {
        self.refresh_rate
    }

    /// Get the monitor's dpi.
    pub fn dpi(&self) -> u16 {
        self.dpi
    }

    /// Check if the monitor is the primary monitor.
    pub fn is_primary(&self) -> bool {
        self.primary
    }

    /// Get the monitor's name.
    pub fn dev_name(&self) -> &str {
        utils::null_terminated_arr_to_str_unchecked(&self.dev_name)
    }

    /// Get the monitor's name.
    pub fn name(&self) -> &str {
        utils::null_terminated_arr_to_str_unchecked(&self.name)
    }

    /// Get all available monitor modes
    pub fn modes(&self) -> &Vec<MonitorMode> {
        &self.modes
    }
}



#[derive(PartialEq, Eq)]
pub(crate) struct MonitorModeOrdWrapper(pub(crate) MonitorMode);

impl PartialOrd for MonitorModeOrdWrapper {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.0.size.width.partial_cmp(&other.0.size.width) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }match self.0.size.height.partial_cmp(&other.0.size.height) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.0.refesh_rate.partial_cmp(&other.0.refesh_rate) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.0.bits_per_pixel.partial_cmp(&other.0.bits_per_pixel)
    }
}

impl Ord for MonitorModeOrdWrapper {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.0.size.width.cmp(&other.0.size.width) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }match self.0.size.height.cmp(&other.0.size.height) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match self.0.refesh_rate.cmp(&other.0.refesh_rate) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.0.bits_per_pixel.cmp(&other.0.bits_per_pixel)
    }
}
