use crate::os;

pub mod thread_id;
pub use thread_id::*;


/// Get the current OS error
pub fn errno() -> u32 {
    os::errno()
}

/// Ensure that the OS is using UTF-8 for the application
/// 
/// This is mainly used to make sure the codepage is set correctly on windows
pub fn ensure_utf8() -> Result<(), u32> {
    os::ensure_utf8()
}

/// Application handle
pub struct AppHandle(os::AppHandle);

impl AppHandle {
    /// Get the OS handle from the application handle
    pub fn os_handle(self) -> os::AppHandle {
        self.0
    }
}

/// Get the current application handle
pub fn get_app_handle() -> AppHandle {
    AppHandle(os::get_app_handle())
}