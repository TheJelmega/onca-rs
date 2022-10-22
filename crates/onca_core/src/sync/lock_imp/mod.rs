// Portions of the project have been copied from parking_lot and is copyrighted by Amanieu d'Antra under the MIT license (located in: '3rd-party-licenses/parking_lot')
//! This module provides type-safe and fully features `Mutex` and `RwLock` types which wrap a simple raw mutex or rwlock type.
//! This has several benegits: not only does it eliminate a large poition of work in implementing custom lock types.
//! It also allows users to write code which is generic with regards to different lock implementations.
//! 
//! Basic use of this module is very straightforward:
//! 
//! 1. Create a raw lock type. This should only contain the lock state, not any dat aprotexted by the lock.
//! 2. Implement the `RawMutex` trait for your custom lock primitive.
//! 3. Export your mutex as a type alias for `lock_impl::Mutex`, and your mutex guard as a type alias for `lock_api::MutexGuard`.
//!    See the [example](#example) below for details
//! 
//! This process is simialr for RwLocks, except that two guards need to be exported instad of one.
//! (Or 3 guards if your type supports upgradable read locks, see [extension traits](#extension-traits) below for details)
//! 
//! # Example
//! 
//! ```
//! use onca_core::sync::lock_imp::{RawMutex, Mutex, GuardSend};
//! use core::sync::atomic::{AtomicBool, Ordering};
//! 
//! // 1. Define our raw lock type
//! pub struct RawSpinLock(AtomicBool);
//! 
//! // 2. Implement RawMutex for this type
//! unsafe impl RawMutex for RawSpinLock {
//!     const INIT: RawSpinLock = RawSpinLock(AtomicBool::new(false));
//! 
//!     // A spinlock guard can be sent to another thread and unlocked there
//!     type GuardMarker = GuardSend;
//! 
//!     fn lock(&self) {
//!         // Note: This isn't the best way of implementing a spinlock, but it suffices for the sake of this example
//!         while !self.try_lock() {}
//!     }
//! 
//!     fn try_lock(&self) -> bool {
//!         self.0
//!             .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
//!             .is_ok()
//!     }
//! 
//!     unsafe fn unlock(&self) {
//!         self.0.store(false, Ordering::Release);
//!     }
//! }
//! 
//! // 3. Export the wrappers. These are the types that your users will actually use.
//! pub type Spinlock<T> = Mutex<RawSpinLock, T>;
//! pub type SpinlockGuard<'a, T> = MutexGuard<'a, RawSpinlock, T>;
//! ```
//! 
//! # Extension traits
//! 
//! In addition to basic locking & unlocking functionality, yo have the option of exposing additional functionality in your lock types by implementing additional traits for it.
//! Examples of extensions features include:
//! 
//! - Fail unlocking (`RawMutexFair`, `RawRwLockFair`)
//! - Lock timeouts ('RawMutexTimed`, `RawRwLockTimed`)
//! - Drowgradable write locks (`RawRwLockDowngradable`)
//! - Recustive read locks (`RawRwLockRecursive`)
//! - Upgradable read locks (`RawRwLockUpgradable`)

// TODO: parking_lot refers to scopeguard ???

/// Marker type which indicates that the type for a lock is `Send`.
pub struct GuardSend(());

// Marker type which indicates that the type for a lock is not `Send`.
pub struct GuardNoSend(*mut ());

unsafe impl Sync for GuardNoSend {}

mod mutex;
pub use mutex::*;

mod remutex;
pub use remutex::*;

mod rwlock;
pub use rwlock::*;

