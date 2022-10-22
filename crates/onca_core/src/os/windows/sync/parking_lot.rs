// Portions of the project have been copied from parking_lot and is copyrighted by Amanieu d'Antra under the MIT license (located in: '3rd-party-licenses/parking_lot')

use core::{
    ffi,
    mem,
    ptr::null_mut,
    sync::atomic::{AtomicUsize, AtomicPtr, Ordering}
};
use crate::{
    mem::HeapPtr, 
    alloc::UseAlloc,
    time::Instant,
};
use windows::
    { Win32::{
        Foundation::{GetLastError, BOOL, ERROR_TIMEOUT},
        System::{
            WindowsProgramming::INFINITE,
            Threading::{WaitOnAddress, WakeByAddressSingle}
        }
    },
};

/// Helpter type for putting a trhead to sleep uitl some other thread wakes it
pub struct ThreadParker {
    key     : AtomicUsize
}

impl ThreadParker {
    pub const IS_CHEAP_TO_CONSTRUCT : bool = true;

    #[inline]
    pub const fn new() -> ThreadParker {
        // Initialize the backend here to ensure we don't get any panics later on, which could lease synchronization primitives in a broken state
        ThreadParker {
            key: AtomicUsize::new(0)
        }
    }

    /// Prepares the parker.
    /// This should be called before adding it to the queue
    #[inline]
    pub fn prepare_park(&self) {
        self.key.store(1, Ordering::Relaxed);
    }

    /// Checks if the park timed out.
    /// This should be called while holding the queue lock after park_until has returned false.
    #[inline]
    pub fn timed_out(&self) -> bool {
        self.key.load(Ordering::Relaxed) != 0
    }

    /// Parks the thread until it is unparked.
    /// This should be called after it has been added to the queue, after unlocking the queue
    #[inline]
    pub unsafe fn park(&self) {
        while self.key.load(Ordering::Acquire) != 0 {
            let r = self.wait_on_address(&self.key, INFINITE);
            debug_assert!(r);
        }
    }

    /// Parks the thread until it is unparked or the timout is reached.
    /// This should be called after it has been added to the queue, after unlocking the queue.
    /// Return true if we were unperked and false if we timed out
    #[inline]
    pub unsafe fn park_until(&self, timeout: Instant) -> bool {
        while self.key.load(Ordering::Acquire) != 0 {
            let now = Instant::now();

            // Early exit when past end time
            if timeout <= now {
                return false;
            }

            let diff = timeout - now;
            let timeout = diff
                .as_secs()
                .checked_mul(1000)
                .and_then(|x| x.checked_add((diff.subsec_nanos() as u64 + 999_999) / 1_000_000))
                .map(|ms| {
                    if ms > u32::MAX as u64 {
                        INFINITE
                    } else {
                        ms as u32
                    }
                })
                .unwrap_or(INFINITE);

            let cmp = 1usize;
            if !self.wait_on_address(&self.key, timeout) {
                debug_assert_eq!(unsafe{ GetLastError() }, ERROR_TIMEOUT);
            }
        }
        true
    }

    /// Locks the parker to prevent the target thread from exiting.
    /// This is necessary to ensure that thread-local TrheadData object remain valid.
    /// This should be called while holding the queue lock
    #[inline]
    pub unsafe fn unpark_lock(&self) -> UnparkHandle {
        // We dont' need to lock anything, just clear the state.
        self.key.store(0, Ordering::Release);

        UnparkHandle { key: &self.key as *const _ }
    }

    fn wait_on_address(&self, key: &AtomicUsize, timout: u32) -> bool {
        let cmp = 1usize;
        unsafe {    
            WaitOnAddress(
            key as *const _ as *mut ffi::c_void,
            &cmp as *const _ as *mut ffi::c_void,
            mem::size_of::<usize>(),
            timout
            ).as_bool() 
        }
    }
}


/// Handle for a thread that is about to be unparked.
/// We need to mark the thread as unparked while holding the queue lock, but we delay the actual unparking until after the queue lock is released.
pub struct UnparkHandle {
    key          : *const AtomicUsize
}

impl UnparkHandle {
    /// Wakes up the parked thread.
    /// This should be called after the queue lock is released to avoid blocking for too long.
    #[inline]
    pub fn unpark(self) {
        unsafe { WakeByAddressSingle(self.key as *mut ffi::c_void) };
    }
}