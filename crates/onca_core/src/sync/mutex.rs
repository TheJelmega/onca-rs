// Portions of the project have been copied from parking_lot and is copyrighted by Amanieu d'Antra under the MIT license (located in: '3rd-party-licenses/parking_lot')

use super::raw_mutex::RawMutex;
use super::lock_imp;

/// A mutual exclusion primitive useful for protecting shared data.
/// 
/// This mutex will block threads waiting for the lock to become available.
/// The mutex can be statically initialized or created by the `new` constructor.
/// Each mutex has atype paramter which represent the data that it is proteting.
/// The data can only be accessed through the RAII guards returned from `lock` and `try_lock`, which guarantees that the data is only ever accessed when the mutex is locked.
/// 
/// # Fairness
/// 
/// A typical unfair lock can often end up in a situation where a single thread quickly acquires and releases the same mutex is succession, which can starve other threads waiting to acquire the mutex.
/// While this improves throughput, because it doesn't force a context whitch when a thread tries to re-acquire a mutes it has just released, this can starve other threads.
/// 
/// This mutex uses [eventual fairness](https://trac.webkit.org/changeset/203350) to ensure that the lock will be fair unlock on average every 0.5ms, which will force the lock to go to the next thread waiting for the mutex.
/// 
/// Additionally, any critical section longer than 1ms will always use a fair unlock, which has a negligible impact on throughput considering the length of the critical section.
/// 
/// You can also force a fair unlock by calling `MutexGuard::unlock_fair` when unlocking a mutex instead of simple dropping the mutex
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
/// - Supports eventual fairness so that the mutex is fair on average.
/// - Optimally allows making the mutex fair by calling `MutexGuard::unlock_fair`
pub type Mutex<T> = lock_imp::Mutex<RawMutex, T>;

/// An RAII implementation of a "scoped lock" of a mutex.
/// When this structure is dropped (falls out of scope), the lock will be unlocked.
/// 
/// The data protected by the mutex can be accessed through this guard via its `Deref` and `DeferMut` implementations.
pub type MutexGuard<'a, T> = lock_imp::MutexGuard<'a, RawMutex, T>;

/// An RAII mutex guard returned by `MutexGuard::map`, which can point to a subfield of the protexted data.
/// 
/// The main difference between `MappedMutexGuard` and `MutexGuard` is that the former doesn't support temporarily unlocking and re-locking, 
/// since that could introduce soundness issues if the locked object is modified by another thread
pub type MappedMutexGuard<'a, T> = lock_imp::MappedMutexGuard<'a, RawMutex, T>;