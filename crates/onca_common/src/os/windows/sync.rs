
//-----------------------------------------------------------------------------------------------------------------------------

use windows::Win32::System::Threading::Sleep;

/// Yeild the rest of the current timeslice to the OS
#[inline]
pub fn thread_yield() {
    unsafe {
        // We don't use SwitchToThread here because it doesn't consider all
        // threads in the system and the thread we are waiting for may not get
        // selected.
        Sleep(0);
    }
}


//-----------------------------------------------------------------------------------------------------------------------------