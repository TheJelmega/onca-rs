use core::{
    sync::atomic::{AtomicBool, Ordering},
    ptr
};

use windows::{
    Win32::{
        Foundation::{GetLastError, HINSTANCE, OLE_E_WRONGCOMPOBJ, RPC_E_CHANGED_MODE},
        System::{
            LibraryLoader::GetModuleHandleA,
            Console::{SetConsoleOutputCP, SetConsoleCP}, Ole::{OleUninitialize, OleInitialize}
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

static OLE_INITIALIZED : AtomicBool = AtomicBool::new(false);

pub(crate) fn init_system() -> Result<(), &'static str> {
    if !OLE_INITIALIZED.load(Ordering::Relaxed) {
        // Setup OLE
        unsafe {
            let ole_res = OleInitialize(ptr::null_mut());
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