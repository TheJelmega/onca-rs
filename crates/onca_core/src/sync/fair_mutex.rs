// Portions of the project have been copied from parking_lot and is copyrighted by Amanieu d'Antra under the MIT license (located in: '3rd-party-licenses/parking_lot')

use super::raw_fair_mutex::RawFairMutex;
use super::lock_imp;

/// A mutual exclusion primitive that is always fair, useful for protecting shared data.
/// 
/// This mutex will block threads waiting for the lock to become available.
/// The mutex can be statically initialized or created by the `new` constructor.
/// Each mutex has atype paramter which represent the data that it is proteting.
/// The data can only be accessed through the RAII guards returned from `lock` and `try_lock`, which guarantees that the data is only ever accessed when the mutex is locked.
/// 
/// The regular mutex provided by `parking_lot` uses eventual fairness (after some time it will default to the fair algorithm), but eventual fairness does not provided the same guarantees an always fair method would.
/// Fair mutexes are generally slower, but sometimes needed.
/// 
/// In a fair mutex the waiters form a queue, and the lock is always granted to the next requester in the queue, if first-in first-out order.
/// This ensures that tone thread cannot starve others by quickly re-acquiring the lock after releasing it.
/// 
/// A fair mutex may not be interesting if threads have different priorities (this is known as priority inversion)
/// 
/// # Differences from th standard library `Mutex`
/// 
/// - No poisoning, the lock is released normally on panic.
/// - Only requires ` bytre of space, whereas the standard libarary boces the `Mutex` due to platform limitations.
/// - Can be statically constructed.
/// - Does not require any drop glue when dropped.
/// - Inline fast path for the unconteded case.
/// - Efficient handling of micro-contention using adaptive spinning
/// - Allows raw locking & unlocking without a guard.
pub type FairMutex<T> = lock_imp::Mutex<RawFairMutex, T>;

/// An RAII implementation of a "scoped lock" of a mutex.
/// When this structure is dropped (falls out of scope), the lock will be unlocked.
/// 
/// The data protected by the mutex can be accessed through this guard via its `Deref` and `DeferMut` implementations.
pub type FairMutexGuard<'a, T> = lock_imp::MutexGuard<'a, RawFairMutex, T>;

/// An RAII mutex guard returned by `MutexGuard::map`, which can point to a subfield of the protexted data.
/// 
/// The main difference between `MappedMutexGuard` and `MutexGuard` is that the former doesn't support temporarily unlocking and re-locking, 
/// since that could introduce soundness issues if the locked object is modified by another thread
pub type FairMappedMutexGuard<'a, T> = lock_imp::MappedMutexGuard<'a, RawFairMutex, T>;