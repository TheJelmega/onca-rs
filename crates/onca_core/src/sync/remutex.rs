// Portions of the project have been copied from parking_lot and is copyrighted by Amanieu d'Antra under the MIT license (located in: '3rd-party-licenses/parking_lot')

use super::raw_mutex::RawMutex;
use core::num::NonZeroUsize;
use super::lock_imp::{self, GetThreadId};

/// Implementation of the `GetThreadId` trait for `lock_imp::ReentrantMutex`
pub struct RawThreadId;

unsafe impl GetThreadId for RawThreadId {
    const INIT: Self = RawThreadId;

    fn nonzero_thread_id(&self) -> NonZeroUsize {
        // The address of the thread-local variable is guaranteed to be unique to the current thread, and is also guaranteed to be non-zero.
        // The variable has to have a non-zero size to guarantee it has a unique address for each thread.
        thread_local!(static KEY : u8 = 0);
        KEY.with(|x| {
            NonZeroUsize::new(x as *const _ as usize)
                .expect("thread-local variable address is null")
        })
    }
}

/// A mutex which can be recursively locked by a single thread.
/// 
/// This type is identical to `Mutex` except for the following points:
/// 
/// - Locking multiple times from the same thread will work correctly instead of deadlocking.
/// - `ReentrantMutexGuard` does not give mutable references to the locked data.
///   Use a `RefCell` if you need this.
pub type ReentrantMutex<T> = lock_imp::ReentrantMutex<RawMutex, RawThreadId, T>;

/// An RAII implementation of a "scoped lock" of a reentrant mutex.
/// When this structure is dropped (falls out of scope), the lock will be unlocked.
/// 
/// The data protected by teh mutex can be accessed thorugh thei guard via its `Deref` implementation.
pub type ReentrantMutexGuard<'a, T> = lock_imp::ReentrantMutexGuard<'a, RawMutex, RawThreadId, T>;

// An RAII mutex guard returned by `ReentrantMutexGuard::map`, which can point to a subfield of the protexted data.
/// 
/// The main difference between `ReentrantMutexGuard` and `ReentrantMutex` is that the former doesn't support temporarily unlocking and re-locking, 
/// since that could introduce soundness issues if the locked object is modified by another thread
pub type MappedReentrantMutexGuard<'a, T> = lock_imp::MappedReentrantMutexGuard<'a, RawMutex, RawThreadId, T>;
