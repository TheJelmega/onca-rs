mod parking_lot;


use windows::{
    core::PCWSTR, 
    Win32::{
        Foundation::*, 
        System::{WindowsProgramming::INFINITE, Threading::*}}
    };
use core::{
    cell::UnsafeCell,
    ptr::null_mut,
    sync::atomic::{AtomicUsize, AtomicPtr, Ordering}
};
use crate::{
    sync::*,
    mem::HeapPtr,
    time::Instant,
};

pub use parking_lot::{ThreadParker, UnparkHandle};

//-----------------------------------------------------------------------------------------------------------------------------

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