use windows::{
    core::PCSTR,
    Win32::{
        Foundation::{HMODULE, GetLastError, FreeLibrary},
        System::LibraryLoader::{LoadLibraryA, GetProcAddress},
    },
};

use crate::strings::String;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DynLibHandle(HMODULE);

impl DynLibHandle {
    pub fn handle(&self) -> HMODULE {
        self.0
    }
}

pub(crate) fn load(s: &str) -> Result<DynLibHandle, i32> {
    unsafe {
        let res = LoadLibraryA(PCSTR(s.as_ptr()));
        match res {
            Ok(handle) => Ok(DynLibHandle(handle)),
            Err(err) => Err(err.code().0),
        }
    }
}

pub(crate) fn close(handle: DynLibHandle) -> Result<(), i32> {
    unsafe {
        match FreeLibrary(handle.0) {
            Ok(_) => Ok(()),
            Err(err) => Err(err.code().0),
        }
    }
}

pub(crate) fn get_proc_address(handle: DynLibHandle, proc_name: &str) -> Option<fn()> {
    unsafe {
        let proc = GetProcAddress(handle.0, PCSTR(proc_name.as_ptr()));
        proc.map(|proc| core::mem::transmute(proc))
    }
}