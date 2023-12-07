use core::{
    sync::atomic::{AtomicBool, Ordering},
    ptr
};

use windows::{
    Win32::{
        Foundation::{GetLastError, HMODULE, OLE_E_WRONGCOMPOBJ, RPC_E_CHANGED_MODE},
        System::{
            LibraryLoader::GetModuleHandleA,
            Console::{SetConsoleOutputCP, SetConsoleCP},
            Ole::{OleUninitialize, OleInitialize}
        }, 
        Globalization::{CP_UTF8, GetACP}
    },
    core::PCSTR
};

use crate::sync::Mutex;

pub mod sync;
pub mod time;
pub mod thread;
pub mod dynlib;
pub mod misc;
pub mod sys_info;

pub(crate) fn errno() -> u32 {
    match unsafe { GetLastError() } {
        Ok(_) => 0,
        Err(err) => err.code().0 as u32,
    }
}

pub(crate) fn ensure_utf8() -> Result<(), u32> {
    unsafe {
        let acp = GetACP();
        if acp != CP_UTF8 {
            return Err(0);
        }
        
        SetConsoleOutputCP(CP_UTF8).map_err(|err| err.code().0 as u32)?;
        SetConsoleCP(CP_UTF8).map_err(|err| err.code().0 as u32)
    }
}

#[derive(Clone, Copy)]
pub struct AppHandle {
    hmodule : HMODULE
}

impl AppHandle {
    pub fn hmodule(&self) -> HMODULE {
        self.hmodule
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
                    let app_handle = AppHandle{ hmodule: hinstance };
                    *locked_handle = Some(app_handle);
                    app_handle
                },
                Err(_) => panic!("Failed to get application handle"),
            }
        },
    }
}

static OLE_INITIALIZED : AtomicBool = AtomicBool::new(false);

pub(crate) fn init_system() -> Result<(), &'static str> {
    if !OLE_INITIALIZED.load(Ordering::Relaxed) {
        // Setup OLE
        unsafe {
            let ole_res = OleInitialize(None);
            match ole_res {
                Ok(_) => {
                    OLE_INITIALIZED.store(true, Ordering::Relaxed);
                },
                Err(err) => {
                    let err_code = err.code();
                    match err_code {
                        OLE_E_WRONGCOMPOBJ => {
                            return Err("COMPOBJ.DLL and OLE2.DLL are incompatible (err: OLE_E_WRONGCOMPOBJ)");
                        },
                        RPC_E_CHANGED_MODE => {
                            return Err("Trying to initialize OLE which already initialized COM to be multi-threaded. OLE requires STA (Single Threaded Apartments). (err: RPC_E_CHANGED_MODE)");
                        }
                        _ => return Err("Failed to initialize OLE")
                    }
                },
            }
        }
    }

    Ok(())
}

pub(crate) fn shutdown_system() {
    // Use relaxed, as this can only be called from the main thread
    if OLE_INITIALIZED.load(Ordering::Relaxed) {
        unsafe { OleUninitialize() };
        OLE_INITIALIZED.store(false, Ordering::Relaxed);
    }
}