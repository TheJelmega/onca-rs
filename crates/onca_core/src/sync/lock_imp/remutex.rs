// Portions of the project have been copied from parking_lot and is copyrighted by Amanieu d'Antra under the MIT license (located in: '3rd-party-licenses/parking_lot')
use core::{
    fmt,
    ptr,
    mem::{self, ManuallyDrop},
    cell::{Cell, UnsafeCell},
    marker::PhantomData,
    ops::Deref,
    num::NonZeroUsize,
    sync::atomic::{AtomicUsize, Ordering},
};
use crate::{
    time::{Duration, Instant},
    mem::Arc
};

use super::{mutex::{RawMutex, RawMutexFair, RawMutexTimed}, GuardNoSend};

/// Helper trait which returns a non-zero thread ID.
/// 
/// The simplest way to implement this trait is to return the address of athread-local variable.
/// 
/// # Safety
/// 
/// Implementation of this trait must ensure that no two active threads share the same thread ID.
/// However the ID of a thread that has exited can be re-used wince that thread is no longer active.
pub unsafe trait GetThreadId {
    /// Initial value
    // A "non-constant" const item is a legacy way to supply an initialized value to downstream static items. Can hopefully be replaced with `const fn new() -> Self` at some point.
    const INIT: Self;

    /// Returns a non-zero thread ID which identifies the current thread of execution.
    fn nonzero_thread_id(&self) -> NonZeroUsize;
}

/// A raw mutex typethat wrpas another raw mutex to provide reentrancy.
/// 
/// Although this has the same methods as the [`RawMutex`] trait, it does not implement it, and should not be used in the same way, since this mutex can successfully acquire a lock multiple imtes inthe same thread.
/// Only use this when you know you want a raw mutex tha can belocked reentrantly; you probably want [`ReentrantMutex`] instead.
pub struct RawReentrantMutex<R, G> {
    owner         : AtomicUsize,
    lock_count    : Cell<usize>,
    mutex         : R,
    get_thread_id : G,
}

unsafe impl<R: RawMutex + Send, G: GetThreadId + Send> Send for RawReentrantMutex<R, G> {}
unsafe impl<R: RawMutex + Sync, G: GetThreadId + Sync> Sync for RawReentrantMutex<R, G> {}

impl<R: RawMutex, G: GetThreadId> RawReentrantMutex<R, G> {
    /// Initial value for an unlocked mutex
    pub const INIT : Self = Self::new(); 
    
    /// Create a new `RawReentrantMutex`
    pub const fn new() -> Self {
        Self { 
            owner: AtomicUsize::new(0), 
            lock_count: Cell::new(0), 
            mutex: R::INIT, 
            get_thread_id: G::INIT 
        }
    }
    
    #[inline]
    fn lock_internal<F: FnOnce() -> bool>(&self, try_lock: F) -> bool {
        let id = self.get_thread_id.nonzero_thread_id().get();
        if self.owner.load(Ordering::Relaxed) == id {
            self.lock_count.set(
                self.lock_count
                    .get()
                    .checked_add(1)
                    .expect("ReentrantMutex lock count overflow")
            );
        } else {
            if !try_lock() {
                return false;
            }
            self.owner.store(id, Ordering::Relaxed);
            debug_assert_eq!(self.lock_count.get(), 0);
            self.lock_count.set(1);
        }
        true
    }

    /// Acquires this mutex, blocking if it's held by another thread
    #[inline]
    pub fn lock(&self) {
        self.lock_internal(|| {
            self.mutex.lock();
            true
        });
    }

    /// Attempts to acquire this mutex without blocking.
    /// Returns `true` if the lock was successfully acquired and `false` otherwise.
    #[inline]
    pub fn try_lock(&self) -> bool {
        self.lock_internal(|| self.mutex.try_lock())
    }

    /// Unlocks this mutex.
    /// The inner mutex may not be unlocked if this mutex was acquired previusly in the current thread.
    /// 
    /// # Safety
    /// 
    /// This method may only be called if the mutex is held by the current thread.
    #[inline]
    pub unsafe fn unlock(&self) {
        let lock_count = self.lock_count.get() - 1;
        self.lock_count.set(lock_count);
        if lock_count == 0 {
            self.owner.store(0, Ordering::Relaxed);
            self.mutex.unlock();
        }
    }

    /// Checks whether the mutex is currently locked
    #[inline]
    pub fn is_locked(&self) -> bool {
        self.mutex.is_locked()
    }

    /// Checks whether the mutex is currently held by the curren thread.
    #[inline]
    pub fn is_owned_by_current_thread(&self) -> bool {
        let id = self.get_thread_id.nonzero_thread_id().get();
        self.owner.load(Ordering::Relaxed) == id
    }
}

impl<R: RawMutexFair, G: GetThreadId> RawReentrantMutex<R, G> {
    /// Unlocks this mutex using a fair unlock protocol.
    /// The inner mutex may not be unlocked if this mutes was acquired previusly in the current thread.
    /// 
    /// # Safety
    /// 
    /// This method may only be called if the mutex is held by teh current thread.
    #[inline]
    pub unsafe fn unlock_fair(&self) {
        let lock_count = self.lock_count.get() - 1;
        self.lock_count.set(lock_count);
        if lock_count == 0 {
            self.owner.store(0, Ordering::Relaxed);
            self.mutex.unlock_fair();
        }
    }

    /// Temporarily yields the mutex to a waiting thread if there is one.
    /// 
    /// This method is functionally equivalend to calling `unlock_fair` followed by `lock`, however it can be much more efficient in the case where there are no waiting threads.
    /// 
    /// # Safety
    /// 
    /// This method may only be called if the mutex is held by the current thread.
    #[inline]
    pub unsafe fn bump(&self) {
        if self.lock_count.get() == 1 {
            let id = self.owner.load(Ordering::Relaxed);
            self.owner.store(0, Ordering::Relaxed);
            self.mutex.bump();
            self.owner.store(id, Ordering::Relaxed);
        }
    }
}

impl<R: RawMutexTimed, G: GetThreadId> RawReentrantMutex<R, G> {
    /// Attempts to acquire this lock until a timeout is reached
    #[inline]
    pub fn try_lock_until(&self, timeout: Instant) -> bool {
        self.lock_internal(|| self.mutex.try_lock_until(timeout))
    }

    /// Attempts to acquire this lock until a timeout is reached
    #[inline]
    pub fn try_lock_for(&self, timeout: Duration) -> bool {
        self.lock_internal(|| self.mutex.try_lock_for(timeout))
    }
}

/// A mutual exclusion primitive useful for protecting shared data.
/// 
/// This mutex will block threads for the lock to become available.
pub struct ReentrantMutex<R, G, T: ?Sized> {
    raw  : RawReentrantMutex<R, G>,
    data : UnsafeCell<T>,
}

unsafe impl<R: RawMutex + Send, G: GetThreadId + Send, T: ?Sized + Send> Send for ReentrantMutex<R, G, T> {}
unsafe impl<R: RawMutex + Sync, G: GetThreadId + Sync, T: ?Sized + Sync> Sync for ReentrantMutex<R, G, T> {}

impl<R: RawMutex, G: GetThreadId, T> ReentrantMutex<R, G, T> {
    /// Creates a new mutex in an unlocked state ready for use
    #[inline]
    pub const fn new(val: T) -> ReentrantMutex<R, G, T> {
        ReentrantMutex { 
            raw: RawReentrantMutex::new(),
            data: UnsafeCell::new(val)
        }
    }

    /// Consume this mutex, returning the underlying data
    #[inline]
    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }
}

impl<R, G, T> ReentrantMutex<R, G, T> {
    /// Creates a new mutex based on a pre-existing raw mutex
    /// 
    /// This allows creating a mutex in a constant context
    pub const fn const_new(raw_mutex: R, get_thread_id: G, val: T) -> ReentrantMutex<R, G, T> {
        ReentrantMutex { 
            data: UnsafeCell::new(val),
            raw: RawReentrantMutex { 
                owner: AtomicUsize::new(0),
                lock_count: Cell::new(0),
                mutex: raw_mutex,
                get_thread_id
            }, 
        }
    }
}

impl<R: RawMutex, G: GetThreadId, T: ?Sized> ReentrantMutex<R, G, T> {
    /// # Safety
    /// 
    /// THe lock must be held when calling this method
    unsafe fn guard(&self) -> ReentrantMutexGuard<'_, R, G, T> {
        ReentrantMutexGuard { remutex: self, marker: PhantomData }
    }

    /// Acquires a reentrant mutex, blocking the current thread until it is able to do so
    /// 
    /// If th mutex is held by another thread, then this function will block the local thread until it is available to acquire the mutex.
    /// If the mutex is already held by the current thread, then this funciton will increment the lock reference count and return immediately.
    /// Upon returning, teh thread is the only thread with the mutex held. An RAII guard is returned t allow scoped unlock of the lock.
    /// When the guard goes out of scope, the mutex will be unlocked.
    #[inline]
    pub fn lock(&self) -> ReentrantMutexGuard<'_, R, G, T> {
        self.raw.lock();
        // SAFETY: the lock is held, as required
        unsafe { self.guard() }
    }

    /// Attemps to acquire this lock.
    /// 
    /// If the lock could not be acquired at this time, then `None` is returned.
    /// Otherwise, an RAII guard is returned.
    /// The lock will be unlocked when the guard is dropped.
    /// 
    /// This function does not block
    #[inline]
    pub fn try_lock(&self) -> Option<ReentrantMutexGuard<'_, R, G, T>> {
        if self.raw.try_lock() {
            // SAFETY: The lock is held, as required
            Some(unsafe { self.guard() })
        } else {
            None
        }
    }

    /// returns a mutable reference to the underlying data
    /// 
    /// Since this call borrows the `Mutex` mutably, no actual locking needs to take place --- The mutable borrow statically guarantees no lock exists
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data.get() }
    }

    /// Checks whether the mutex is currently locked
    #[inline]
    pub fn is_locked(&self) -> bool {
        self.raw.is_locked()
    }

    /// Checks whether the mutex is currently held by teh current thread
    #[inline]
    pub fn is_owned_by_current_thread(&self) -> bool {
        self.raw.is_owned_by_current_thread()
    }

    /// Forcibly unlocks the mutex.
    /// 
    /// This is useful when combined with `mem::forget` to hold a lock without teh need to maintain a `MutexGuards` object alive, for example when dealing with FFI.
    /// 
    /// # Safety
    /// 
    /// This method must only be called if the current thread logically owns a `ReentrantMutexGuard`, but that guard has to be discarded using `mem::forget`.
    /// Behavior is undefined if a mutex is unlocked when it isn't locked.
    #[inline]
    pub unsafe fn force_unlock(&self) {
        self.raw.unlock();
    }

    /// returns the underlying raw mutex object.
    /// 
    /// Note taht you will most likely need to import the `RawMutex` trait from `lock_imp` to be able to call functions on the raw mutex.
    /// 
    /// # Safety
    /// 
    /// This method is unsafe because it allows unlocking a mutex while still holding a reference to a `ReentrantMutexGuard`.
    #[inline]
    pub unsafe fn raw(&self) -> &RawReentrantMutex<R, G> {
        &self.raw
    }

    /// Returns a raw pointer to the underlying data.
    /// 
    /// This is useful when combined with `meme::froget` to hold a lock withoug the need to maintain a `ReentrantMutexGuard` object alive, for example when dealing with FFI.
    /// 
    /// # Safety
    /// 
    /// YOu must ensure that there are no data races when dereferencing the returned pointer, 
    /// for example if the current thread logically own a 'ReentrantMutexGuard`, but that guard has been discarded using `mem::forget`.
    #[inline]
    pub fn data_ptr(&self) -> *mut T {
        self.data.get()
    }

    
}

impl<R: RawMutex, G: GetThreadId, T: ?Sized> Arc<ReentrantMutex<R, G, T>> {
    /// # Safety
    /// 
    /// The lock needs to be held for the behavior of this function to be defined.
    #[inline]
    unsafe fn guard_arc(&self) -> ArcReentrantMutexGuard<R, G, T> {
        ArcReentrantMutexGuard { remutex: self.clone(), marker: PhantomData }
    }

    /// Acquires a lock through an `Arc`.
    /// 
    /// This method is similar to the `lock` method; however, it requires the `Mutex` to be inside of an `Arc` and the resulting mutex guard has no lifetime requirements.
    #[inline]
    pub fn lock_arc(&self) -> ArcReentrantMutexGuard<R, G, T> {
        self.raw.lock();
        // SAFETY: the locking guarantee is upheld
        unsafe { self.guard_arc() }
    }

    /// Attemps to acquire a lock though an `Arc`
    /// 
    /// This method is similar to the `try_lock` method; however, it requires the `Mutex` to be inside of an `Arc` and the resulting mutex guard has no lifetime requirements.
    #[inline]
    pub fn try_lock_arc(&self) -> Option<ArcReentrantMutexGuard<R, G, T>> {
        if self.raw.try_lock() {
             // SAFETY: the locking guarantee is upheld
            Some(unsafe { self.guard_arc() })
        } else {
            None
        }
    }
}

impl<R: RawMutexFair, G: GetThreadId, T: ?Sized> ReentrantMutex<R, G, T> {
    /// Forcibly unlock the mutex using a fair unlock protocol.
    /// 
    /// This is useful when combined with `mem::forget` to hold a lock without the need to maintian a `ReentrantMutexGuard` object alive, for example when dealing with FFI.
    /// 
    /// # Safety
    /// 
    /// This method must only be called if the current thread logically owns a `ReentrantMutexGuard`, but that guard has been discarded using `mem::forget`.
    /// Behavior is undefined if a mutex is unlocked when it isn't locked.
    #[inline]
    pub unsafe fn force_unlock_fair(&self) {
        self.raw.unlock_fair();
    }
}

impl<R: RawMutexTimed, G: GetThreadId, T: ?Sized> ReentrantMutex<R, G, T> {
    /// Attempts to acquire thi lock until a timeout is reached.
    /// 
    /// If the lock could not be acquired before the timeout expired, then `None` is returned.
    /// Otherwise an RAII guard is returned.
    /// The lock will be unlcoked when the guard is dropped
    #[inline]
    pub fn try_lock_for(&self, timeout: Duration) -> Option<ReentrantMutexGuard<'_, R, G, T>> {
        if self.raw.try_lock_for(timeout) {
            // SAFETY: The lock is held, as required
            Some(unsafe { self.guard() })
        } else {
            None
        }
    }

    /// Attempts to acquire thi lock until a timeout is reached.
    /// 
    /// If the lock could not be acquired before the timeout expired, then `None` is returned.
    /// Otherwise an RAII guard is returned.
    /// The lock will be unlcoked when the guard is dropped
    #[inline]
    pub fn try_lock_until(&self, timeout: Instant) -> Option<ReentrantMutexGuard<'_, R, G, T>> {
        if self.raw.try_lock_until(timeout) {
            // SAFETY: The lock is held, as required
            Some(unsafe { self.guard() })
        } else {
            None
        }
    }
}

impl<R: RawMutexTimed, G: GetThreadId, T: ?Sized> Arc<ReentrantMutex<R, G, T>> {
    /// Attempts to acquire this lock thorugh an `Arc` until a timeout is reached.
    /// 
    /// This method is similar to the `try_lock_for` mthod; however, it requires the mutexto the inside of an `Arc` and the result mutex guard has no lifetime requirements.
    #[inline]
    pub fn try_lock_arc_for(&self, timeout: Duration) -> Option<ArcReentrantMutexGuard<R, G, T>> {
        if self.raw.try_lock_for(timeout) {
           // SAFETY: The lock is held, as required
           Some(unsafe { self.guard_arc() })
        } else {
            None
        }
    }

    /// Attempts to acquire this lock thorugh an `Arc` until a timeout is reached.
    /// 
    /// This method is similar to the `try_lock_for` mthod; however, it requires the mutexto the inside of an `Arc` and the result mutex guard has no lifetime requirements.
    #[inline]
    pub fn try_lock_arc_until(&self, timeout: Instant) -> Option<ArcReentrantMutexGuard<R, G, T>> {
        if self.raw.try_lock_until(timeout) {
           // SAFETY: The lock is held, as required
           Some(unsafe { self.guard_arc() })
        } else {
            None
        }
    }
}

impl<R: RawMutex, G: GetThreadId, T: ?Sized + Default> Default for ReentrantMutex<R, G, T> {
    #[inline]
    fn default() -> Self {
        ReentrantMutex::new(Default::default())
    }
}

impl<R: RawMutex, G: GetThreadId, T> From<T> for ReentrantMutex<R, G, T> {
    #[inline]
    fn from(t: T) -> Self {
        ReentrantMutex::new(t)
    }
}

impl<R: RawMutex, G: GetThreadId, T: ?Sized + fmt::Debug> fmt::Debug for ReentrantMutex<R, G, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.try_lock() {
            Some(guard) => f.debug_struct("Mutex").field("data", &&*guard).finish(),
            None => {
                struct LockedPlaceholder;
                impl fmt::Debug for LockedPlaceholder {
                    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        f.write_str("<locked>")
                    }
                }

                f.debug_struct("Mutex")
                    .field("data", &LockedPlaceholder)
                .finish()
            }
        }
    }
}

/// An RAII implementation of a "scoped lock" of a mutex.
/// When this structure is dropped (falls out of scoope), the lock will be unlocked.
/// 
/// The data protected by the mutex ca nbe accessed through this guard via its `Deref` implementation.
#[must_use = "if unused the mutex will immediataly unlock"]
pub struct ReentrantMutexGuard<'a, R: RawMutex, G: GetThreadId, T: ?Sized> {
    remutex: &'a ReentrantMutex<R, G, T>,
    marker: PhantomData<(&'a mut T, GuardNoSend)>,
}

unsafe impl<'a, R: RawMutex + Sync + 'a, G: GetThreadId + Sync + 'a, T: ?Sized + Sync + 'a> Sync for ReentrantMutexGuard<'a, R, G, T> {}

impl<'a, R: RawMutex + 'a, G: GetThreadId + 'a, T: ?Sized + 'a> ReentrantMutexGuard<'a, R, G, T> {
    /// Returns  a reference to the original `ReentrantMutex` object.
    pub fn remutex(s: &Self) -> &'a ReentrantMutex<R, G, T> {
        s.remutex
    }

    /// Makes a new `MappedReentrantMutexGuard` for a component of the locked data
    /// 
    /// This operation cannot fail as the `ReentrantMutexGuard` passed in already locked the mutex.
    /// 
    /// This is an associated function tha need to be used as `ReentrantMutexGuard::map(...)`.
    /// A method would interfere with methods of the same name on the contents of the locked data.
    #[inline]
    pub fn map<U: ?Sized, F>(s: Self, f: F) -> Result<MappedReentrantMutexGuard<'a, R, G, U>, Self>
    where 
        F : FnOnce(&mut T) -> Option<&mut U>,
    {
        let raw = &s.remutex.raw;
        let data = match f(unsafe { &mut *s.remutex.data.get() }) {
            Some(data) => data,
            None => return Err(s),
        };
        mem::forget(s);
        Ok(MappedReentrantMutexGuard {
            raw,
            data,
            marker: PhantomData
        })
    }

    /// Attempts to make a new `MappedReentrantMutexGuard` for a component of the locked data.
    /// The original guard is returned if the close returns `None`.
    /// 
    /// This operation cannot fail as the `ReentrantMutexGuard` passed in already locked the mutex.
    /// 
    /// This is an associated function that needs to be used as `ReentrantMutexGuard::try_map(...)`.
    /// A method would interfere with methods of the same name of th contents of the locked data.
    #[inline]
    pub fn try_map<U: ?Sized, F>(s: Self, f: F) -> Result<MappedReentrantMutexGuard<'a, R, G, U>, Self>
    where
        F : FnOnce(&mut T) -> Option<&mut U>,
    {
        let raw = &s.remutex.raw;
        let data = match f(unsafe { &mut *s.remutex.data.get() }) {
            Some(data) => data,
            None => return Err(s),
        };
        mem::forget(s);
        Ok(MappedReentrantMutexGuard {
            raw,
            data,
            marker: PhantomData
        })
    }

    /// Temporarily unlocks the mutex to execute the given function.
    /// 
    /// This is safe because `&mut` guarantees that there exists no other references to the data protected by this mutex.
    #[inline]
    pub fn unlocked<F, U>(s: &mut Self, f: F) -> U
    where
        F : FnOnce() -> U,
    {
        // SAFETY: A ReentrantMutexGuard always holds the lock
        unsafe {
            s.remutex.raw.unlock();
        }
        defer!(s.remutex.raw.lock());
        f()
    }

    /// Leaks the mutex guard and return a mutable reference to the dat protected by the mutex
    /// 
    /// THis will leave the `Mutex` in a locked state
    #[inline]
    pub fn leak(s: Self) -> &'a mut T {
        let r = unsafe { &mut *s.remutex.data.get() };
        mem::forget(s);
        r
    }
}

impl<'a, R: RawMutexFair + 'a, G: GetThreadId + 'a, T: ?Sized + 'a> ReentrantMutexGuard<'a, R, G, T> {
    /// Unlocks the mutex using a fair unlock protocol.
    /// 
    /// By default, mutexes are unfair and allow the current thread to re-lock the mutex before another has the chance to acquire the lock, even if that thread has been blocked on the mutex for a long time.
    /// This is the default, baseuce it allows much higher throughput as it avoids forcing a context switch on every mutex unlock.
    /// This can result in one thread acquiring a mutex many more times than other threads.
    /// 
    /// However in some cases it can be beneficial to wnsure fairness by forisng the lock topass on the a waiting thread if ther is one.
    /// This is done by using this method instad of dropping the `ReentrantMutexGuard` normally.
    #[inline]
    pub fn unlock_fair(s: Self) {
        // Safety: A ReentrantMutexGuard always holds the lock.
        unsafe {
            s.remutex.raw.unlock_fair();
        }
        mem::forget(s);
    }

    /// Temporarily unlocks to ececute the given function.
    /// 
    /// The mutex is unlocked using a fair unlock protocal.
    /// 
    /// This is safe because `&mut` gurarantees that there exists no other references to the dat protected by the mutex.
    #[inline]
    pub fn unlocked_fair<F, U>(s: &mut Self, f: F) -> U 
    where
        F : FnOnce() -> U,
    {
        // Safety: A ReentrantMutexGuard always hold the lock.
        unsafe {
            s.remutex.raw.unlock_fair();
        }
        defer!(s.remutex.raw.lock());
        f()
    }

    /// Temporarily yields the mutex to a waiting thread if there is one.
    /// 
    /// This method is functionally equivalent to calling `unlock_fair` followed by `lock`, however it can be much more efficient in the case ther are no waiting threads.
    #[inline]
    pub fn bump(s: &mut Self) {
        // Safety: A ReentrantMutexGuard always holds the lock.
        unsafe {
            s.remutex.raw.bump();
        }
    }
}

impl<'a, R: RawMutex + 'a, G: GetThreadId + 'a, T: ?Sized + 'a> Deref for ReentrantMutexGuard<'a, R, G, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.remutex.data.get() }
    }
}

impl<'a, R: RawMutex + 'a, G: GetThreadId + 'a, T: ?Sized + 'a> Drop for ReentrantMutexGuard<'a, R, G, T> {
    fn drop(&mut self) {
        // Safety: A ReentrantMutexGuard always holds the lock
        unsafe {
            self.remutex.raw.unlock();
        }
    }
}

impl<'a, R: RawMutex + 'a, G: GetThreadId + 'a, T: ?Sized + fmt::Debug + 'a> fmt::Debug for ReentrantMutexGuard<'a, R, G, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<'a, R: RawMutex + 'a, G: GetThreadId + 'a, T: ?Sized + fmt::Display + 'a> fmt::Display for ReentrantMutexGuard<'a, R, G, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

/// An RAII mutex guard returned by the `Arc` locking operations on `Mutex`.
/// 
/// This is similar to the `ReentrantMutexGuard` struct, except instead of using a reference to unlock the `Mutex`
#[must_use = "if unused the Mutex will immediately unlock"]
pub struct ArcReentrantMutexGuard<R: RawMutex, G: GetThreadId, T: ?Sized> {
    remutex : Arc<ReentrantMutex<R, G, T>>,
    marker  : PhantomData<GuardNoSend>
}

impl<R: RawMutex, G:GetThreadId, T: ?Sized> ArcReentrantMutexGuard<R, G, T> {
    /// Returns a reference and return the `Mutex` this is guarding, containing in its `Arc`.
    #[inline]
    pub fn remutex(s: &Self) -> &Arc<ReentrantMutex<R, G, T>> {
        &s.remutex
    }

    /// Unlocks the mutex and returns ths `Arc` tha was held by the [`ArcMutexGuard`].
    #[inline]
    pub fn into_arc(s: Self) -> Arc<ReentrantMutex<R, G, T>> {
        // SAFETY: skip our Dopt impl and manually unlock the mutex
        let arc = unsafe { ptr::read(&s.remutex) };
        mem::forget(s);
        unsafe {
            arc.raw.unlock();
        }
        arc
    }

    /// Temporarily unlocks the mutex to execute the given function.
    /// 
    /// This is safe because '&mut' guarantees that there exists no other references to the data protexted by the mutex
    #[inline]
    pub fn unlocked<F, U>(s: &mut Self, f: F) -> U
    where
        F : FnOnce() -> U
    {
        // SAFETY: An ArcMutexGuard always holds the lock
        unsafe {
            s.remutex.raw.unlock();
        }
        defer!(s.remutex.raw.lock());
        f()
    }
}

impl<R: RawMutexFair, G:GetThreadId, T: ?Sized> ArcReentrantMutexGuard<R, G, T> {
    /// Unlocks the mutex using a fair unlock protocol
    /// 
    /// This is functionally identical to the `unlock_fair` method on [`ReentrantMutexGuard`]
    #[inline]
    pub fn unlock_fair(s: Self) {
        // SAFETY: An ArcMutexGuard always holds the lock
        unsafe {
            s.remutex.raw.unlock_fair();
        }

        // SAFETY: make sure the Arc gets it reference dereferences
        let mut s = ManuallyDrop::new(s);
        unsafe { ptr::drop_in_place(&mut s.remutex) };
    }

    /// Temporarily unlocks the mutexs to execute the given function
    /// 
    /// This is functionally identical to the `unlocked_fair` method on [`ReentrantMutexGuard`]
    #[inline]
    pub fn unlocked_fair<F, U>(s: &mut Self, f: F) -> U 
    where
        F : FnOnce() -> U
    {
        // SAFETY: An ArcMutexGuard always holds the lock
        unsafe {
            s.remutex.raw.unlock_fair();
        }
        defer!(s.remutex.raw.lock());
        f()
    }

    /// Temporarily yields the mutex to the waiting thread if there is one.
    /// 
    /// This is functionally identical to the `bum` method on [`ReentrantMutexGuard`]
    #[inline]
    pub fn bump(s: &mut Self) {
        // SAFETY: A ReentrantMutexGuard always holds the lock
        unsafe {
            s.remutex.raw.bump()
        }
    }
}

impl<R: RawMutex, G:GetThreadId, T: ?Sized> Deref for ArcReentrantMutexGuard<R, G, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.remutex.data.get() }
    }
}

impl<R: RawMutex, G:GetThreadId, T: ?Sized> Drop for ArcReentrantMutexGuard<R, G, T> {
    fn drop(&mut self) {
        // SAFETY: A ReentrantMutexGuard always holds the lock
        unsafe {
            self.remutex.raw.unlock();
        }
    }
}

/// An RAII mutex guard returned by `ReentrantMutexGuard::map`, which can point to a subfield of the protected data.
/// 
/// The main difference between `MappedReentrantMutexGuard` and `ReentrantMutexGuard` is that the former doesn't support temporarily unlocking and re-locking,
/// since that could introduce soundness issues if the locked boject is modified by another thread.
#[must_use = "if unused the Mutex will immediately unlock"]
pub struct MappedReentrantMutexGuard<'a, R: RawMutex, G: GetThreadId, T: ?Sized> {
    raw    : &'a RawReentrantMutex<R, G>,
    data   : *mut T,
    marker : PhantomData<&'a T>,
}

unsafe impl<'a, R: RawMutex + Sync + 'a, G:GetThreadId + Sync + 'a, T: ?Sized + Sync + 'a> Sync for MappedReentrantMutexGuard<'a, R, G, T>
{}

impl<'a, R: RawMutex + 'a, G:GetThreadId + 'a, T: ?Sized + 'a> MappedReentrantMutexGuard<'a, R, G, T> {
    /// Makes a new `MappedReentrantMutexGuard` for a component of hte locked data.
    /// 
    /// This operation connot fail as the `MappedReentrantMutexGuard` passed is already locked the mutex.
    /// 
    /// This is an associated function that needs to be used as `MappedReentrantMutexGuard::map(...)`
    /// A method would interfere with methods of the same name on the contents to the locked data
    #[inline]
    pub fn map<U: ?Sized, F>(s: Self, f: F) -> MappedReentrantMutexGuard<'a, R, G, U>
    where 
        F : FnOnce(&mut T) -> &mut U,
    {
        let raw = s.raw;
        let data = f(unsafe { &mut *s.data });
        mem::forget(s);
        MappedReentrantMutexGuard { 
            raw, 
            data, 
            marker: PhantomData 
        }
    }

    /// Attempts to make a new `MappedReentrantMutexGuard` for a component of the locked data.
    /// THe original guard is returned if the closure returns `None`.
    /// 
    /// This operation cannot fail as the `MappedReentrantMutexGuard` passed in already locked the mutex.
    /// 
    /// This is an associated function that needs to be used as `MappedReentrantMutexGuard::try_map(...)`.
    /// A method would interfere with methods of the same name on the conetens of the locked data.
    #[inline]
    pub fn try_map<U: ?Sized, F>(s: Self, f: F) -> Result<MappedReentrantMutexGuard<'a, R, G, U>, Self> 
    where
        F : FnOnce(&mut T) -> Option<&mut U>
    {
        let raw = s.raw;
        let data = match f(unsafe { &mut *s.data }) {
            Some(data) => data,
            None => return Err(s),
        };
        mem::forget(s);
        Ok(MappedReentrantMutexGuard { 
            raw, 
            data, 
            marker: PhantomData 
        })
    }
}

impl<'a, R: RawMutexFair + 'a, G:GetThreadId + 'a, T: ?Sized + 'a> MappedReentrantMutexGuard<'a, R, G, T> {
    /// Unlocks the mutex using a fair unlock protocol.
    /// 
    /// By default, mutexes are unfair and allow the current thread to re-lock the mutex before another has the change to acquire the lock, even if that thread has been blocked on the mutex for a long time.
    /// This is the default because it allows much higher throughput as it avoids forcing a context switch on every mutex unlock.
    /// This can result in one htread acquiring a mutex many more times than other threads.
    /// 
    /// However is some cases it can be beneficial to ensure fairness by forcing a lock to pass on to a waiting thread if there is one.
    /// This is done by using this method instead of dropping the `ReentrantMutexGuard` normally.
    #[inline]
    pub fn unlock_fair(s: Self) {
        // SAFETY: A MappedReentrantMutexGuard always holds the lock
        unsafe {
            s.raw.unlock_fair();
        }
        mem::forget(s);
    }
}

impl<'a, R: RawMutex + 'a, G:GetThreadId + 'a, T: ?Sized + 'a> Deref for MappedReentrantMutexGuard<'a, R, G, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data }
    }
}

impl<'a, R: RawMutex + 'a, G:GetThreadId + 'a, T: ?Sized + 'a> Drop for MappedReentrantMutexGuard<'a, R, G, T> {
    fn drop(&mut self) {
        // SAFETY: A MappedReentrantMutexGuard always holds the lock
        unsafe {
            self.raw.unlock();
        }
    }
}

impl<'a, R: RawMutex + 'a, G:GetThreadId + 'a, T: ?Sized + fmt::Debug + 'a> fmt::Debug for MappedReentrantMutexGuard<'a, R, G, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<'a, R: RawMutex + 'a, G:GetThreadId + 'a, T: ?Sized + fmt::Display + 'a> fmt::Display for MappedReentrantMutexGuard<'a, R, G, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}