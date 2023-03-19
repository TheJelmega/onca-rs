use core::mem::size_of;
use onca_core::{
    prelude::*,
    collections::BTreeSet,
    utils
};
use onca_logging::log_error;
use windows::{
    Win32::{
        Foundation::{RECT, LPARAM, BOOL, GetLastError, HWND, POINT},
        Graphics::Gdi::{
            EnumDisplayMonitors, GetMonitorInfoA, MonitorFromWindow, MonitorFromPoint, MonitorFromRect, 
            HMONITOR, HDC, MONITORINFOEXA, MONITORINFO,
            MONITOR_DEFAULTTONULL, EnumDisplaySettingsExA, ENUM_CURRENT_SETTINGS, DEVMODEA, ENUM_DISPLAY_SETTINGS_MODE, DM_BITSPERPEL, DM_PELSWIDTH, DM_PELSHEIGHT, DM_DISPLAYFREQUENCY, EnumDisplayDevicesA, DISPLAY_DEVICEA, EDS_RAWMODE,
        },
        UI::{
            WindowsAndMessaging::{MONITORINFOF_PRIMARY},
            HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI}
        },
    },
    core::PCSTR
};

use crate::{Monitor, LOG_CAT, WindowSettings, MonitorRect, MonitorMode, PhysicalSize, MonitorModeOrdWrapper};

#[derive(Clone, Copy)]
pub struct MonitorHandle(HMONITOR);

impl MonitorHandle {
    pub fn hmonitor(&self) -> HMONITOR {
        self.0
    }
}

unsafe extern "system" fn monitor_enum_proc(hmonitor: HMONITOR, _dc: HDC, _rect: *mut RECT, lparam: LPARAM) -> BOOL {
    let monitors = &mut *(lparam.0 as *mut DynArray<Monitor>);
    let monitor = get_monitor(hmonitor, false);
    if let Some(mon) = monitor {
        monitors.push(mon);
    }
    BOOL(1)
}

unsafe extern "system" fn primary_monitor_enum_proc(hmonitor: HMONITOR, _dc: HDC, _rect: *mut RECT, lparam: LPARAM) -> BOOL {
    let monitor = &mut *(lparam.0 as *mut Option<Monitor>);
    *monitor = get_monitor(hmonitor, true);
    BOOL(monitor.is_none() as i32)
}

unsafe fn get_monitor(hmonitor: HMONITOR, want_primary: bool) -> Option<Monitor> {
    // Both values are identical
    let mut dpi_x = 0;
    let mut dpi_y = 0;

    let res = GetDpiForMonitor(hmonitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y);
    let dpi = match res {
        Ok(_) => dpi_x as u16,
        Err(err) => {
            log_error!(LOG_CAT, get_monitor, "Failed to retrieve monitor DPI, setting DPI to default {} (HRESULT err: {})", WindowSettings::DEFAULT_DPI, err.code().0);
            WindowSettings::DEFAULT_DPI
        },
    };

    let mut monitor_info = MONITORINFOEXA::default();
    monitor_info.monitorInfo.cbSize = size_of::<MONITORINFOEXA>() as u32;

    let res = GetMonitorInfoA(hmonitor, &mut monitor_info as *mut _ as *mut MONITORINFO).as_bool();
    if res {
        let primary = (monitor_info.monitorInfo.dwFlags & MONITORINFOF_PRIMARY) == MONITORINFOF_PRIMARY;
        if want_primary && !primary {
            return None;
        }
        
        let dev_name = *(&monitor_info.szDevice as *const _ as *const [u8; 32]);
        let mon_rc = monitor_info.monitorInfo.rcMonitor;
        let work_rc = monitor_info.monitorInfo.rcWork;

        let mut i = 0;
        let mut res = true;
        // window-rs DEVMODEA struct is incorrect, so we are using our own definition for now
        let mut dev_mode = DEVMODEA::default();
        dev_mode.dmSize = size_of::<DEVMODEA>() as u16;

        // All available modes
        let mut monitor_btree = BTreeSet::new();
        while res {
            res = EnumDisplaySettingsExA(PCSTR(dev_name.as_ptr()), ENUM_DISPLAY_SETTINGS_MODE(i), &mut dev_mode as *mut _ as *mut DEVMODEA, EDS_RAWMODE).as_bool();
            
            let mut mon_mode = MonitorMode::default();
            if utils::is_flag_set(dev_mode.dmFields, DM_BITSPERPEL) {
                mon_mode.bits_per_pixel = dev_mode.dmBitsPerPel as u8;
            }
            if utils::is_flag_set(dev_mode.dmFields, DM_PELSWIDTH | DM_PELSHEIGHT) {
                mon_mode.size = PhysicalSize::new(dev_mode.dmPelsWidth as u16, dev_mode.dmPelsHeight as u16);
            }
            if utils::is_flag_set(dev_mode.dmFields, DM_DISPLAYFREQUENCY) {
                mon_mode.refesh_rate = dev_mode.dmDisplayFrequency;
            }
            
            monitor_btree.insert(MonitorModeOrdWrapper(mon_mode));
            i += 1;
        }
        let monitor_modes = DynArray::from_iter(monitor_btree.into_iter().map(|val| val.0));

        // Current settings
        EnumDisplaySettingsExA(PCSTR(dev_name.as_ptr()), ENUM_CURRENT_SETTINGS, &mut dev_mode as *mut _ as *mut DEVMODEA, EDS_RAWMODE).as_bool();
        let refresh_rate = if dev_mode.dmFields & DM_DISPLAYFREQUENCY == DM_DISPLAYFREQUENCY {
            dev_mode.dmDisplayFrequency as f32
        } else {
            0f32
        };

        let mut display_dev = DISPLAY_DEVICEA::default();
        display_dev.cb = size_of::<DISPLAY_DEVICEA>() as u32;
        EnumDisplayDevicesA(PCSTR(monitor_info.szDevice.as_ptr() as *const u8), 0, &mut display_dev, 0);

        Some(Monitor {
            os_handle: MonitorHandle(hmonitor),
            mon_rect: MonitorRect { x: mon_rc.left, y: mon_rc.top, width: (mon_rc.right - mon_rc.left) as u16, height: (mon_rc.bottom - mon_rc.top) as u16 },
            work_rect: MonitorRect { x: work_rc.left, y: work_rc.top, width: (work_rc.right - work_rc.left) as u16, height: (work_rc.bottom - work_rc.top) as u16 },
            refresh_rate,
            dpi,
            primary,
            dev_name,
            name: core::mem::transmute(display_dev.DeviceString),
            modes: monitor_modes,
        })
    } else {
        log_error!(LOG_CAT, get_monitor, "Failed to retrieve monitor info");
        None
    }
}

pub(crate) fn enumerate_monitors() -> DynArray<Monitor> {
    unsafe {
        let mut monitors = DynArray::new();

        let lparam = LPARAM(&mut monitors as *mut DynArray<Monitor> as isize);
        let res = EnumDisplayMonitors(HDC(0), None, Some(monitor_enum_proc), lparam).as_bool();
        if !res {
            let err_code = GetLastError().0;
            log_error!(LOG_CAT, enumerate_monitors, "Failed to enumerate monitors (err: {err_code})");
        }

        monitors
    }
}

pub(crate) fn primary_monitor() -> Option<Monitor> {
    unsafe {
        let mut monitor : Option<Monitor> = None;
        
        let lparam = LPARAM(&mut monitor as *mut Option<Monitor> as isize);
        EnumDisplayMonitors(HDC(0), None, Some(primary_monitor_enum_proc), lparam);
        monitor
    }
}

pub(crate) fn get_monitor_from_hwnd(hwnd: HWND) -> Option<Monitor> {
    unsafe {
        let hmonitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONULL);
        if hmonitor.is_invalid() {
            None
        } else {
            get_monitor(hmonitor, false)
        } 
    }
}

pub(crate) fn get_monitor_at(x: i32, y: i32) -> Option<Monitor> {
    unsafe {
        let hmonitor = MonitorFromPoint(POINT { x, y }, MONITOR_DEFAULTTONULL);
        if hmonitor.is_invalid() {
            None
        } else {
            get_monitor(hmonitor, false)
        } 
    }
}

pub(crate) fn get_monitor_from_largest_overlap(rect: MonitorRect) -> Option<Monitor> {
    unsafe {
        let rect = RECT { left: rect.x, top: rect.y, right: rect.x + rect.width as i32, bottom: rect.y + rect.height as i32  };
        let hmonitor = MonitorFromRect(&rect, MONITOR_DEFAULTTONULL);
        if hmonitor.is_invalid() {
            None
        } else {
            get_monitor(hmonitor, false)
        } 
    }
}

pub(crate) fn get_monitor_rect(hmon: HMONITOR) -> Option<MonitorRect> {
    unsafe {
        if hmon.is_invalid() {
            return None;
        }

        let mut monitor_info = MONITORINFOEXA::default();
        monitor_info.monitorInfo.cbSize = size_of::<MONITORINFOEXA>() as u32;
        
        let res = GetMonitorInfoA(hmon, &mut monitor_info as *mut _ as *mut MONITORINFO).as_bool();
        if !res {
            return None;
        }   

        let mon_rect = monitor_info.monitorInfo.rcMonitor;
        Some(MonitorRect {
            x: mon_rect.left,
            y: mon_rect.top,
            width: (mon_rect.right - mon_rect.left) as u16,
            height: (mon_rect.bottom - mon_rect.top) as u16,
        })
    }
}
