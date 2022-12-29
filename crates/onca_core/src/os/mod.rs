use cfg_if::cfg_if;

cfg_if!{
    if #[cfg(windows)] {
        pub mod windows;
        pub use self::windows as os_imp;
    } else {

    }
}

/// Get the current OS error
pub fn errno() -> u32 {
    os_imp::errno()
}

/// Ensure that the OS is using UTF-8 for the application
/// 
/// This is mainly used to make sure the codepage is set correctly on windows
pub fn ensure_utf8() -> Result<(), u32> {
    os_imp::ensure_utf8()
}