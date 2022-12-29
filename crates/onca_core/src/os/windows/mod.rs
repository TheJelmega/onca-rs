use windows::{
    Win32::{
        Foundation::GetLastError,
        System::{
            Console::{SetConsoleOutputCP, SetConsoleCP}
        }, 
        Globalization::{CP_UTF8, GetACP}
    },
    core::PCSTR
};

pub mod sync;
pub mod time;

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
}