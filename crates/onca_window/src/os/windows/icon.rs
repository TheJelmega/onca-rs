use onca_core::prelude::*;
use onca_logging::log_warning;
use windows::{
    Win32::{
        UI::WindowsAndMessaging::{
            HICON, LoadImageA, IMAGE_BITMAP, LR_LOADFROMFILE, DestroyIcon, CopyIcon
        },
        Foundation::{HMODULE, GetLastError}
    },
    core::PCSTR
};

use crate::{PhysicalSize, LOG_CAT};

pub struct OSIcon {
    hicon: HICON
}

impl OSIcon {
    pub(crate) fn from_path(path: &str, size: Option<PhysicalSize>) -> OSIcon {
        unsafe {
            let _scope_alloc = ScopedAlloc::new(AllocId::TlsTemp);

            let (width, height) = size.map(|size | (size.width as i32, size.height as i32)).unwrap_or((0, 0));
            let path = String::from(path);
            let hicon = LoadImageA(
                HMODULE(0),
                PCSTR(path.as_ptr()),
                IMAGE_BITMAP,
                width, height,
                LR_LOADFROMFILE
            );

            match hicon {
                Ok(icon) => OSIcon { hicon: HICON(icon.0) },
                Err(err) => {
                    log_warning!(LOG_CAT, "Failed to load icon '{path}'. (hresult: {:X})", err.code().0);
                    return OSIcon { hicon: HICON(0) }
                },
            }
        }
    }

    pub(crate) fn hicon(&self) -> HICON {
        self.hicon
    }
}

impl Drop for OSIcon {
    fn drop(&mut self) {
        unsafe {
            if !self.hicon.is_invalid() {
                let res = DestroyIcon(self.hicon).as_bool();
                if !res {
                    log_warning!(LOG_CAT, "Failed to destoy icon with handle '{:X}' (err: {:X})", self.hicon.0, GetLastError().0);
                }
            }
        }
    }
}

impl Clone for OSIcon {
    fn clone(&self) -> Self {
        unsafe {
            let hicon = CopyIcon(self.hicon);

            match hicon {
                Ok(hicon) => OSIcon { hicon },
                Err(err) => {
                    log_warning!(LOG_CAT, "Failed to copy icon. (hresult: {:X})", err.code().0);
                    OSIcon { hicon: HICON(0) }
                },
            }
        }
    }
}