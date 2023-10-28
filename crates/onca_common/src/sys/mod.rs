use core::sync::atomic::{AtomicU8, Ordering, AtomicBool};

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

static IS_SYSTEM_INITIALIZED : AtomicBool = AtomicBool::new(false);

pub fn init_system() -> Result<(), &'static str> {
    assert!(is_on_main_thread(), "System can only be initialized on the main thread");
    if IS_SYSTEM_INITIALIZED.load(Ordering::Relaxed) {
        return Ok(());
    }

    os::init_system()?;

    IS_SYSTEM_INITIALIZED.store(true, Ordering::Release);
    Ok(())
} 

pub fn shutdown_system() {
    if !IS_SYSTEM_INITIALIZED.load(Ordering::Relaxed) {
        return;
    }

    os::shutdown_system();
}

/// Application handle
pub type AppHandle = os::AppHandle;

/// Get the current application handle
pub fn get_app_handle() -> AppHandle {
    os::get_app_handle()
}