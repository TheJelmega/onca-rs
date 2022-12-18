use windows::Win32::Foundation::GetLastError;

pub mod sync;

pub fn errno() -> u32 {
    unsafe { GetLastError().0 }
}