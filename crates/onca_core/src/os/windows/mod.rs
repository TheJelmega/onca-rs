use windows::{
    Win32::{
        Foundation::{GetLastError, HINSTANCE},
        System::{
            LibraryLoader::GetModuleHandleA,
            Console::{SetConsoleOutputCP, SetConsoleCP}
        }, 
        Globalization::{CP_UTF8, GetACP}
    },
    core::PCSTR
};

use crate::sync::Mutex;

pub mod sync;
pub mod time;
pub mod thread;

pub(crate) fn errno() -> u32 {
    unsafe { GetLastError().0 }
}

pub(crate) fn ensure_utf8() -> Result<(), u32> {
    unsafe {
        let acp = GetACP();
        if acp != CP_UTF8 {
            return Err(0);
        }
        
        let res = SetConsoleOutputCP(CP_UTF8).as_bool();
        if !res {
            return Err(errno());
        }

        let res = SetConsoleCP(CP_UTF8).as_bool();
        if !res {
            return Err(errno());
        }
        
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct AppHandle {
    hinstance : HINSTANCE
}

impl AppHandle {
    pub fn hinstance(&self) -> HINSTANCE {
        self.hinstance
    }
}

pub(crate) fn get_app_handle() -> AppHandle {
    static HANDLE : Mutex<Option<AppHandle>> = Mutex::new(None);

    let mut locked_handle = HANDLE.lock();
    match *locked_handle {
        Some(handle) => handle,
        None => {
            let hinstance = unsafe { GetModuleHandleA(PCSTR(core::ptr::null())) };
            match hinstance {
                Ok(hinstance) => {
                    let app_handle = AppHandle{ hinstance };
                    *locked_handle = Some(app_handle);
                    app_handle
                },
                Err(_) => panic!("Failed to get application handle"),
            }
        },
    }
}