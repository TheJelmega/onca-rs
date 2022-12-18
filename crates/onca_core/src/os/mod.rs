

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub use self::windows as os_imp;


pub fn errno() -> u32 {
    os_imp::errno()
}