use windows::{
    core::PCSTR,
    Win32::{
        Foundation::{HINSTANCE, GetLastError},
        System::LibraryLoader::{LoadLibraryA, FreeLibrary, GetProcAddress},
    },
};

use crate::strings::String;



#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DynLibHandle(HINSTANCE);

impl DynLibHandle {
    pub fn handle(&self) -> HINSTANCE {
        self.0
    }
}

pub fn load(s: &String) -> Result<DynLibHandle, i32> {
    unsafe {
        let res = LoadLibraryA(PCSTR(s.as_ptr()));
        match res {
            Ok(handle) => Ok(DynLibHandle(handle)),
            Err(err) => Err(err.code().0),
        }
    }
}

pub fn close(handle: DynLibHandle) -> Result<(), i32> {
    unsafe {
        let res = FreeLibrary(handle.0).as_bool();
        if res {
            Ok(())
        } else {
            Err(GetLastError().0 as i32)
        }
    }
}

pub fn get_proc_address(handle: DynLibHandle, proc_name: &String) -> Option<fn()> {
    unsafe {
        let proc = GetProcAddress(handle.0, PCSTR(proc_name.as_ptr()));
        proc.map(|proc| core::mem::transmute(proc))

    }
}