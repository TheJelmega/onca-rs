use windows::Win32::Foundation::GetLastError;

pub mod sync;
pub mod time;

pub fn errno() -> u32 {
    unsafe { GetLastError().0 }
}