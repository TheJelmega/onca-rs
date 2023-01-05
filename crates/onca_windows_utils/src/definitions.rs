#![allow(non_snake_case)]

use windows::Win32::{Foundation::POINTL, Graphics::Gdi::DEVMODE_FIELD_FLAGS};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DEVMODEA_0_0 {
    pub dmOrientation   : i16,
    pub dmPaperSize     : i16,
    pub dmPaperLength   : i16,
    pub dmPaperWidth    : i16,
    pub dmScale         : i16,
    pub dmCopies        : i16,
    pub dmDefaultSource : i16,
    pub dmPrintQuality  : i16,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DEVMODEA_0_1 {
    pub dmPosition           : POINTL,
    pub dmDisplayOrientation : u32,
    pub dmDisplayFixedOutput : u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union DEVMODEA_0 {
    pub Anonymous0 : DEVMODEA_0_0,
    pub dmPosition : POINTL,
    pub Anonymous1 : DEVMODEA_0_1,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union DEVMODEA_1 {
    pub dmDisplayFlags : u32,
    pub dmNup          : u32,
}

// TODO(jel): Use windows crate version once it's fixed (broken in 0.43.0)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct DEVMODEA {
    pub dmDeviceName       : [u8; 32],
    pub dmSpecVersion      : u16,
    pub dmDriverVersion    : u16,
    pub dmSize             : u16,
    pub dmDriverExtra      : u16,
    pub dmFields           : DEVMODE_FIELD_FLAGS,
    pub Anonymous0         : DEVMODEA_0,
    pub dmColor            : i16,
    pub dmDuplex           : i16,
    pub dmYResolution      : i16,
    pub dmTTOption         : i16,
    pub dmCollate          : i16,
    pub dmFormName         : [u8; 32],
    pub dmLogPixels        : u16,
    pub dmBitsPerPel       : u32,
    pub dmPelsWidth        : u32,
    pub dmPelsHeight       : u32,
    pub Anonymous1         : DEVMODEA_1,
    pub dmDisplayFrequency : u32,
    pub dmICMMethod        : u32,
    pub dmICMIntent        : u32,
    pub dmMediaType        : u32,
    pub dmDitherType       : u32,
    pub dmReserved1        : u32,
    pub dmReserved2        : u32,
    pub dmPanningWidth     : u32,
    pub dmPanningHeight    : u32,
}

impl Default for DEVMODEA {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}