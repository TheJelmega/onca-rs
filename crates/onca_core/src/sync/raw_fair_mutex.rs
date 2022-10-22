// Portions of the project have been copied from parking_lot and is copyrighted by Amanieu d'Antra under the MIT license (located in: '3rd-party-licenses/parking_lot')

use super::raw_mutex::RawMutex;
use super::lock_imp::{self, RawMutexFair};

/// Raw fair mutex type backed by the parking_lot
pub struct RawFairMutex(RawMutex);

unsafe impl lock_imp::RawMutex for RawFairMutex {
    const INIT : Self = RawFairMutex(<RawMutex as lock_imp::RawMutex>::INIT);

    type GuardMarker = <RawMutex as lock_imp::RawMutex>::GuardMarker;

    #[inline]
    fn lock(&self) {
        self.0.lock()
    }

    #[inline]
    fn try_lock(&self) -> bool {
        self.0.try_lock()
    }

    #[inline]
    unsafe fn unlock(&self) {
        self.0.unlock_fair()
    }

    #[inline]
    fn is_locked(&self) -> bool {
        self.0.is_locked()
    }
}

unsafe impl lock_imp::RawMutexFair for RawFairMutex {
    #[inline]
    unsafe fn unlock_fair(&self) {
        self.0.unlock_fair()
    }

    #[inline]
    unsafe fn bump(&self) {
        self.0.bump()
    }
}

unsafe impl lock_imp::RawMutexTimed for RawFairMutex {
    #[inline]
    fn try_lock_for(&self, timeout: std::time::Duration) -> bool {
        self.0.try_lock_for(timeout)
    }

    #[inline]
    fn try_lock_until(&self, timeout: std::time::Instant) -> bool {
        self.0.try_lock_until(timeout)
    }
}