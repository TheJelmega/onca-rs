// Portions of the project have been copied from parking_lot and is copyrighted by Amanieu d'Antra under the MIT license (located in: '3rd-party-licenses/parking_lot')

use super::raw_rwlock::RawRwLock;
use super::lock_imp;

/// A reader-writer lock
/// 
/// This type of lock allows a number of reader or at most one writer at any point in time.
/// The write portion of thi lock typically allows modification of the underlying data (exclusive access) and the read portion of this lock typically allows for read-only access (shared access).
/// 
/// This lock uses a task-fair locking policy which avoids both reader and writer starvation.
/// This meanstha readers trying to acquire the lock will block even if the lock is unlocked when tehr are writer waiting to acquire the lock.
/// Because of this, attempt to recursively acquire a read lock with a single thread may result in a deadlock.
/// 
/// The type parameter `T` represents the data that this lock protexts.
/// It is required that `T` satisfies `Send` to be shared across threads and `Sync` to allow concurrentaccess through readers.
/// The RAII guard returned from the locking methods implement `Deref` (and `DerefMut` for the `write` methods) to allow access to the contained of the lock.
/// 
/// # Fairness
/// 
/// A typical unfair lock can often end up in a situation where a single thread quicly acquires and releases the same lock in succession, which can starve other threads witing to acquire the rwlock.
/// While this improves throughput because it doesn't force a context switch when a thread tries to re-acquire a rwlock it has just released, this can starve other threads
/// 
/// This rwlock uses [eventual fairness](https://trac.webkit.org/changeset/203350) to ensuer that th lock will be fair on average without sacrificint throughput.
/// This is done by forcing a fair unlock on average every 0.5ms, which will forse the lock to go to the next thread waiting for the rwlock.
/// 
/// Additionally, any critical section longer than 1ms will always use a fair unlock, which has a negligible impact on thoughput considering the lenght of the critical section.
/// 
/// YOu can also force a fair unlock by calling `RwLockReadGuard::unlock_fair` or `RwLockWriteGuard::unlock_fair` when unlocking a mutex instead of simple dorpping the guard.
/// 
/// # Differences from the standard library `RwLock`:
/// 
/// - Supports atomically downgrading a write lock into a read lock.
/// - Task-fair locking policty instead of an unspecified platform default.
/// - No poisoning, the lock is released normally on panic.
/// - Only requires 1 word of space, whereas the standard library boxes the `RwLock` due to platform limitiations.
/// - Can be statically constructed.
/// - Does not require any drop glue when dropped.
/// - Inline fast path for the uncontended case.
/// - Effecient handling of micro-contention using adaptive spinning.
/// - Allows raw locking & unlocking without a guard.
/// - Supports eventual fairness so that the rwlock is fair on average.
/// - Optionally allows making the rwlock fair by calling `RwLockReadGuard::unlock_fair` or `RwLockWriteGuard::unlock_fair`.
pub type RwLock<T> = lock_imp::RwLock<RawRwLock, T>;

/// RAII structure used to release the shared read access of a lock when dropped.
pub type RwLockReadGuard<'a, T> = lock_imp::RwLockReadGuard<'a, RawRwLock, T>;

/// RAII structure used to release the exclusive write access of a lock when dropped.
pub type RwLockWriteGuard<'a, T> = lock_imp::RwLockWriteGuard<'a, RawRwLock, T>;

/// An RAII read lock guard returned by `RwLockReadGuard::map`, which can point to a subfield of the protected data.
/// 
/// The main difference between `MappedRwLockReadGuard` and `RwLockReadGuard` is that the former doesn't support temporarily unlocking and re-locking, since that could introduce soundness issues if the locked boject is modified by another thread.
pub type MappedRwLockReadGuard<'a, T> = lock_imp::MappedRwLockReadGuard<'a, RawRwLock, T>;

/// An RAII read lock guard returned by `RwLockWriteGuard::map`, which can point to a subfield of the protected data.
/// 
/// The main difference between `MappedRwLockWriteGuard` and `RwLockWriteGuard` is that the former doesn't support temporarily unlocking and re-locking, since that could introduce soundness issues if the locked boject is modified by another thread.
pub type MappedRwLockWriteGuard<'a, T> = lock_imp::MappedRwLockReadGuard<'a, RawRwLock, T>;

/// RAII structure used to release the upgradable read access of a lock when dropped.
pub type RwLockUpgradableReadGuard<'a, T> = lock_imp::RwLockUpgradableReadGuard<'a, RawRwLock, T>;