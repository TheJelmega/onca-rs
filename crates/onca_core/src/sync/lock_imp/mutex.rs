// Portions of the project have been copied from parking_lot and is copyrighted by Amanieu d'Antra under the MIT license (located in: '3rd-party-licenses/parking_lot')
use core::{
    fmt,
    ptr,
    mem::{self, ManuallyDrop},
    cell::UnsafeCell,
    marker::PhantomData,
    ops::{Deref, DerefMut}
};
use crate::{
    time::{Duration, Instant},
    mem::Arc
};

/// Basic operations for a mutex
/// 
/// Types implementing this trait can be used by `Mutex` to form a safe and fully-functioning mutex type.
/// 
/// # Safety
/// 
/// Implementations of this trait mus ensure that the mutex is actually exclusive, a lock can't be acquired while the mutex is already locked.
pub unsafe trait RawMutex {
    /// Initial value for an unlocked trait.
    /// A "non-constant" const item is a legacy way to supply an initialized value to downstream static items.
    /// Can hopefully be replaced with `const fn new() -> Self` at some point.
    const INIT : Self;

    /// Marker type which determines whether a lock guard should be `Send`.
    /// Use one of the `GuardSend` or `GuardNoSend` helper types here
    type GuardMarker;

    /// Acquires this mutex, blocking the current thread until it is able to do so.
    fn lock(&self);

    /// Attempts to acquire this mutex without blocking.
    /// Returns 'true' if the lock was successfully acquired and `false` otherwise.
    fn try_lock(&self) -> bool;

    /// Unlocks the mutex.
    /// 
    /// # Safety
    /// 
    /// This method may only be called if the mutex is held in the current context, i.e. it must be paired with a successful call to [`lock`], [`try_lock`], [`try_lock_for`] or [try_lock_until`].
    /// 
    /// [`lock`]: RawMutex::lock
    /// [`try_lock`]: RawMutex::try_lock
    /// [`try_lock_for`]: RawMutexTimed::try_lock_for
    /// [`try_lock_until`]: RawMutexTimed::try_lock_until
    unsafe fn unlock(&self);

    /// Checks whtehr the mutex is currently locked.
    #[inline]
    fn is_locked(&self) -> bool {
        let acquired_lock = self.try_lock();
        if acquired_lock {
            // Safety: The lock has been successfully acquired above.
            unsafe {
                self.unlock();
            }
        }
        !acquired_lock
    }
}

/// Additional methods for mutexes which support fair unlocking.
/// 
/// Fair unlocking means that the lock is handed directly over to the next waiting thread if there is one, without giving other threads the opportunity to "steal" the lock in the meantime.
/// This is typically slower than unfair unlocking, but may be necessary in certain circumstances.
pub unsafe trait RawMutexFair : RawMutex {
    /// Unlocks this mutex using a fair unlock protocol.
    /// 
    /// # Safety
    /// 
    /// This method may only be called if the mutex is held in the current context, see the documentation of [`unlock`]
    /// 
    /// [`unlock`]: RawMutex::unlock
    unsafe fn unlock_fair(&self);

    /// Temporarily yields the mutex to a waiting thread if there is one.
    /// 
    /// This method is functionally equivalent to calling `unlock_fair` followed by `lock`, however it can be much more efficient in the case where there are no waiting threads.
    /// 
    /// # Safety
    /// 
    /// This method may only be calledif the mutex is held in the current context, see the documentation of [`unlock`]
    /// 
    /// [`unlock`]: RawMutex::unlock
    unsafe fn bump(&self) {
        self.unlock_fair();
        self.lock();
    }
}

/// Additional methods for mutexes which support locking with timeouts.
// We don't have `Duration` and `Instant` as associated types, cause we are using the types we specified for onca
pub unsafe trait RawMutexTimed : RawMutex {
    /// Attempts to acquire this lock until a timout is reached.
    fn try_lock_for(&self, timeout: Duration) -> bool;

    /// Attempts to acquire this lock until a timeout is reached
    fn try_lock_until(&self, timeout: Instant) -> bool;
}

/// A mutual exclusion primitive useful for protecting shared data.
/// 
/// This mutex will block threads for the lock to become available.
pub struct Mutex<R, T: ?Sized> {
    raw  : R,
    data : UnsafeCell<T>,
}

unsafe impl<R: RawMutex + Send, T: ?Sized + Send> Send for Mutex<R, T> {}
unsafe impl<R: RawMutex + Sync, T: ?Sized + Sync> Sync for Mutex<R, T> {}

impl<R: RawMutex, T> Mutex<R, T> {
    /// Creates a new mutex in an unlocked state ready for use
    #[inline]
    pub const fn new(val: T) -> Mutex<R, T> {
        Mutex { 
            raw: R::INIT,
            data: UnsafeCell::new(val)
        }
    }

    /// Consume this mutex, returning the underlying data
    #[inline]
    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }
}

impl<R, T> Mutex<R, T> {
    /// Creates a new mutex based on a pre-existing raw mutex
    /// 
    /// This allows creating a mutex in a constant context
    pub const fn const_new(raw_mutex: R, val: T) -> Mutex<R, T> {
        Mutex { raw: raw_mutex, data: UnsafeCell::new(val) }
    }
}

impl<R: RawMutex, T: ?Sized> Mutex<R, T> {
    /// # Safety
    /// 
    /// THe lock must be held when calling this method
    unsafe fn guard(&self) -> MutexGuard<'_, R, T> {
        MutexGuard { mutex: self, marker: PhantomData }
    }

    /// Acquires a mutex, blocking the current thread until it is able to do so
    /// 
    /// This function will block the local thread until it is available to acquire the mutex.
    /// Upon returning th thread is the only thread with the mutex held.
    /// An RAII guard is returned to allow scoped unlock of the lock.
    /// When the guard goes out of scope, the mutex will be unlocked.
    /// 
    /// Attempts to lock a mutex in the thread which already hold the lock will result in a deadlock
    #[inline]
    pub fn lock(&self) -> MutexGuard<'_, R, T> {
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
    pub fn try_lock(&self) -> Option<MutexGuard<'_, R, T>> {
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

    /// Forcibly unlocks the mutex.
    /// 
    /// This is useful when combined with `mem::forget` to hold a lock without teh need to maintain a `MutexGuards` object alive, for example when dealing with FFI.
    /// 
    /// # Safety
    /// 
    /// This method must only be called if the current thread logically owns a `MutexGuard`, but that guard has to be discarded using `mem::forget`.
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
    /// This method is unsafe because it allows unlocking a mutex while still holding a reference to a `MutexGuard`.
    #[inline]
    pub unsafe fn raw(&self) -> &R {
        &self.raw
    }

    /// Returns a raw pointer to the underlying data.
    /// 
    /// This is useful when combined with `meme::froget` to hold a lock withoug the need to maintain a `MutexGuard` object alive, for example when dealing with FFI.
    /// 
    /// # Safety
    /// 
    /// YOu must ensure that there are no data races when dereferencing the returned pointer, 
    /// for example if the current thread logically own a 'MutexGuard`, but that guard has been discarded using `mem::forget`.
    #[inline]
    pub fn data_ptr(&self) -> *mut T {
        self.data.get()
    }

    
}

impl<R: RawMutex, T: ?Sized> Arc<Mutex<R, T>> {
    /// # Safety
    /// 
    /// The lock needs to be held for the behavior of this function to be defined.
    #[inline]
    unsafe fn guard_arc(&self) -> ArcMutexGuard<R, T> {
        ArcMutexGuard { mutex: self.clone(), marker: PhantomData }
    }

    /// Acquires a lock through an `Arc`.
    /// 
    /// This method is similar to the `lock` method; however, it requires the `Mutex` to be inside of an `Arc` and the resulting mutex guard has no lifetime requirements.
    #[inline]
    pub fn lock_arc(&self) -> ArcMutexGuard<R, T> {
        self.raw.lock();
        // SAFETY: the locking guarantee is upheld
        unsafe { self.guard_arc() }
    }

    /// Attemps to acquire a lock though an `Arc`
    /// 
    /// This method is similar to the `try_lock` method; however, it requires the `Mutex` to be inside of an `Arc` and the resulting mutex guard has no lifetime requirements.
    #[inline]
    pub fn try_lock_arc(&self) -> Option<ArcMutexGuard<R, T>> {
        if self.raw.try_lock() {
             // SAFETY: the locking guarantee is upheld
            Some(unsafe { self.guard_arc() })
        } else {
            None
        }
    }
}

impl<R: RawMutexFair, T: ?Sized> Mutex<R, T> {
    /// Forcibly unlock the mutex using a fair unlock protocol.
    /// 
    /// This is useful when combined with `mem::forget` to hold a lock without the need to maintian a `MutexGuard` object alive, for example when dealing with FFI.
    /// 
    /// # Safety
    /// 
    /// This method must only be called if the current thread logically owns a `MutexGuard`, but that guard has been discarded using `mem::forget`.
    /// Behavior is undefined if a mutex is unlocked when it isn't locked.
    #[inline]
    pub unsafe fn force_unlock_fair(&self) {
        self.raw.unlock_fair();
    }
}

impl<R: RawMutexTimed, T: ?Sized> Mutex<R, T> {
    /// Attempts to acquire thi lock until a timeout is reached.
    /// 
    /// If the lock could not be acquired before the timeout expired, then `None` is returned.
    /// Otherwise an RAII guard is returned.
    /// The lock will be unlcoked when the guard is dropped
    #[inline]
    pub fn try_lock_for(&self, timeout: Duration) -> Option<MutexGuard<'_, R, T>> {
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
    pub fn try_lock_until(&self, timeout: Instant) -> Option<MutexGuard<'_, R, T>> {
        if self.raw.try_lock_until(timeout) {
            // SAFETY: The lock is held, as required
            Some(unsafe { self.guard() })
        } else {
            None
        }
    }
}

impl<R: RawMutexTimed, T: ?Sized> Arc<Mutex<R, T>> {
    /// Attempts to acquire this lock thorugh an `Arc` until a timeout is reached.
    /// 
    /// This method is similar to the `try_lock_for` mthod; however, it requires the mutexto the inside of an `Arc` and the result mutex guard has no lifetime requirements.
    #[inline]
    pub fn try_lock_arc_for(&self, timeout: Duration) -> Option<ArcMutexGuard<R, T>> {
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
    pub fn try_lock_arc_until(&self, timeout: Instant) -> Option<ArcMutexGuard<R, T>> {
        if self.raw.try_lock_until(timeout) {
           // SAFETY: The lock is held, as required
           Some(unsafe { self.guard_arc() })
        } else {
            None
        }
    }
}

impl<R: RawMutex, T: ?Sized + Default> Default for Mutex<R, T> {
    #[inline]
    fn default() -> Self {
        Mutex::new(Default::default())
    }
}

impl<R: RawMutex, T> From<T> for Mutex<R, T> {
    #[inline]
    fn from(t: T) -> Self {
        Mutex::new(t)
    }
}

impl<R: RawMutex, T: ?Sized + fmt::Debug> fmt::Debug for Mutex<R, T> {
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
/// The data protected by the mutex ca nbe accessed through this guard via its `Deref` and `DerefMut` implementation.
#[must_use = "if unused the mutex will immediataly unlock"]
pub struct MutexGuard<'a, R: RawMutex, T: ?Sized> {
    mutex: &'a Mutex<R, T>,
    marker: PhantomData<(&'a mut T, R::GuardMarker)>,
}

unsafe impl<'a, R: RawMutex + Sync + 'a, T: ?Sized + Sync + 'a> Sync for MutexGuard<'a, R, T> {}

impl<'a, R: RawMutex + 'a, T: ?Sized + 'a> MutexGuard<'a, R, T> {
    /// Returns  a reference to the original `Mutex` object.
    pub fn mutex(s: &Self) -> &'a Mutex<R, T> {
        s.mutex
    }

    /// Makes a new `MappedMutexGuard` for a component of the locked data
    /// 
    /// This operation cannot fail as the `MutexGuard` passed in already locked the mutex.
    /// 
    /// This is an associated function tha need to be used as `MutexGuard::map(...)`.
    /// A method would interfere with methods of the same name on the contents of the locked data.
    #[inline]
    pub fn map<U: ?Sized, F>(s: Self, f: F) -> Result<MappedMutexGuard<'a, R, U>, Self>
    where 
        F : FnOnce(&mut T) -> Option<&mut U>,
    {
        let raw = &s.mutex.raw;
        let data = match f(unsafe { &mut *s.mutex.data.get() }) {
            Some(data) => data,
            None => return Err(s),
        };
        mem::forget(s);
        Ok(MappedMutexGuard {
            raw,
            data,
            marker: PhantomData
        })
    }

    /// Attempts to make a new `MappedMutexGuard` for a component of the locked data.
    /// The original guard is returned if the close returns `None`.
    /// 
    /// This operation cannot fail as the `MutexGuard` passed in already locked the mutex.
    /// 
    /// This is an associated function that needs to be used as `MutexGuard::try_map(...)`.
    /// A method would interfere with methods of the same name of th contents of the locked data.
    #[inline]
    pub fn try_map<U: ?Sized, F>(s: Self, f: F) -> Result<MappedMutexGuard<'a, R, U>, Self>
    where
        F : FnOnce(&mut T) -> Option<&mut U>,
    {
        let raw = &s.mutex.raw;
        let data = match f(unsafe { &mut *s.mutex.data.get() }) {
            Some(data) => data,
            None => return Err(s),
        };
        mem::forget(s);
        Ok(MappedMutexGuard {
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
        // SAFETY: A MutexGuard always holds the lock
        unsafe {
            s.mutex.raw.unlock();
        }
        defer!(s.mutex.raw.lock());
        f()
    }

    /// Leaks the mutex guard and return a mutable reference to the dat protected by the mutex
    /// 
    /// THis will leave the `Mutex` in a locked state
    #[inline]
    pub fn leak(s: Self) -> &'a mut T {
        let r = unsafe { &mut *s.mutex.data.get() };
        mem::forget(s);
        r
    }
}

impl<'a, R: RawMutexFair + 'a, T: ?Sized + 'a> MutexGuard<'a, R, T> {
    /// Unlocks the mutex using a fair unlock protocol.
    /// 
    /// By default, mutexes are unfair and allow the current thread to re-lock the mutex before another has the chance to acquire the lock, even if that thread has been blocked on the mutex for a long time.
    /// This is the default, baseuce it allows much higher throughput as it avoids forcing a context switch on every mutex unlock.
    /// This can result in one thread acquiring a mutex many more times than other threads.
    /// 
    /// However in some cases it can be beneficial to wnsure fairness by forisng the lock topass on the a waiting thread if ther is one.
    /// This is done by using this method instad of dropping the `MutexGuard` normally.
    #[inline]
    pub fn unlock_fair(s: Self) {
        // Safety: A MutexGuard always holds the lock.
        unsafe {
            s.mutex.raw.unlock_fair();
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
        // Safety: A MutexGuard always hold the lock.
        unsafe {
            s.mutex.raw.unlock_fair();
        }
        defer!(s.mutex.raw.lock());
        f()
    }

    /// Temporarily yields the mutex to a waiting thread if there is one.
    /// 
    /// This method is functionally equivalent to calling `unlock_fair` followed by `lock`, however it can be much more efficient in the case ther are no waiting threads.
    #[inline]
    pub fn bump(s: &mut Self) {
        // Safety: A MutexGuard always holds the lock.
        unsafe {
            s.mutex.raw.bump();
        }
    }
}

impl<'a, R: RawMutex + 'a, T: ?Sized + 'a> Deref for MutexGuard<'a, R, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<'a, R: RawMutex + 'a, T: ?Sized + 'a> DerefMut for MutexGuard<'a, R, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<'a, R: RawMutex + 'a, T: ?Sized + 'a> Drop for MutexGuard<'a, R, T> {
    fn drop(&mut self) {
        // Safety: A MutexGuard always holds the lock
        unsafe {
            self.mutex.raw.unlock();
        }
    }
}

impl<'a, R: RawMutex + 'a, T: ?Sized + fmt::Debug + 'a> fmt::Debug for MutexGuard<'a, R, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<'a, R: RawMutex + 'a, T: ?Sized + fmt::Display + 'a> fmt::Display for MutexGuard<'a, R, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

/// An RAII mutex guard returned by the `Arc` locking operations on `Mutex`.
/// 
/// This is similar to the `MutexGuard` struct, except instead of using a reference to unlock the `Mutex`
#[must_use = "if unused the Mutex will immediately unlock"]
pub struct ArcMutexGuard<R: RawMutex, T: ?Sized> {
    mutex  : Arc<Mutex<R, T>>,
    marker : PhantomData<*const ()>
}

unsafe impl<R: RawMutex + Send + Sync, T: ?Sized + Send> Send for ArcMutexGuard<R, T> 
where
    R::GuardMarker : Send
{}

unsafe impl<R: RawMutex + Send + Sync, T: ?Sized + Sync> Sync for ArcMutexGuard<R, T> 
where
    R::GuardMarker : Sync
{}

impl<R: RawMutex, T: ?Sized> ArcMutexGuard<R, T> {
    /// Returns a reference and return the `Mutex` this is guarding, containing in its `Arc`.
    #[inline]
    pub fn mutex(s: &Self) -> &Arc<Mutex<R, T>> {
        &s.mutex
    }

    /// Unlocks the mutex and returns ths `Arc` tha was held by the [`ArcMutexGuard`].
    #[inline]
    pub fn into_arc(s: Self) -> Arc<Mutex<R, T>> {
        // SAFETY: skip our Dopt impl and manually unlock the mutex
        let arc = unsafe { ptr::read(&s.mutex) };
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
            s.mutex.raw.unlock();
        }
        defer!(s.mutex.raw.lock());
        f()
    }
}

impl<R: RawMutexFair, T: ?Sized> ArcMutexGuard<R, T> {
    /// Unlocks the mutex using a fair unlock protocol
    /// 
    /// This is functionally identical to the `unlock_fair` method on [`MutexGuard`]
    #[inline]
    pub fn unlock_fair(s: Self) {
        // SAFETY: An ArcMutexGuard always holds the lock
        unsafe {
            s.mutex.raw.unlock_fair();
        }

        // SAFETY: make sure the Arc gets it reference dereferences
        let mut s = ManuallyDrop::new(s);
        unsafe { ptr::drop_in_place(&mut s.mutex) };
    }

    /// Temporarily unlocks the mutexs to execute the given function
    /// 
    /// This is functionally identical to the `unlocked_fair` method on [`MutexGuard`]
    #[inline]
    pub fn unlocked_fair<F, U>(s: &mut Self, f: F) -> U 
    where
        F : FnOnce() -> U
    {
        // SAFETY: An ArcMutexGuard always holds the lock
        unsafe {
            s.mutex.raw.unlock_fair();
        }
        defer!(s.mutex.raw.lock());
        f()
    }

    /// Temporarily yields the mutex to the waiting thread if there is one.
    /// 
    /// This is functionally identical to the `bum` method on [`MutexGuard`]
    #[inline]
    pub fn bump(s: &mut Self) {
        // SAFETY: A MutexGuard always holds the lock
        unsafe {
            s.mutex.raw.bump()
        }
    }
}

impl<R: RawMutex, T: ?Sized> Deref for ArcMutexGuard<R, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.data.get() }
    }
}
impl<R: RawMutex, T: ?Sized> DerefMut for ArcMutexGuard<R, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<R: RawMutex, T: ?Sized> Drop for ArcMutexGuard<R, T> {
    fn drop(&mut self) {
        // SAFETY: A MutexGuard always holds the lock
        unsafe {
            self.mutex.raw.unlock();
        }
    }
}

/// An RAII mutex guard returned by `MutexGuard::map`, which can point to a subfield of the protected data.
/// 
/// The main difference between `MappedMutexGuard` and `MutexGuard` is that the former doesn't support temporarily unlocking and re-locking,
/// since that could introduce soundness issues if the locked boject is modified by another thread.
#[must_use = "if unused the Mutex will immediately unlock"]
pub struct MappedMutexGuard<'a, R: RawMutex, T: ?Sized> {
    raw    : &'a R,
    data   : *mut T,
    marker : PhantomData<&'a mut T>,
}

unsafe impl<'a, R: RawMutex + Sync + 'a, T: ?Sized + Sync + 'a> Sync for MappedMutexGuard<'a, R, T>
where
    R::GuardMarker : Sync
{}

unsafe impl<'a, R: RawMutex + Send + 'a, T: ?Sized + Send + 'a> Send for MappedMutexGuard<'a, R, T> 
where
    R::GuardMarker : Send
{}

impl<'a, R: RawMutex + 'a, T: ?Sized + 'a> MappedMutexGuard<'a, R, T> {
    /// Makes a new `MappedMutexGuard` for a component of hte locked data.
    /// 
    /// This operation connot fail as the `MappedMutexGuard` passed is already locked the mutex.
    /// 
    /// This is an associated function that needs to be used as `MappedMutexGuard::map(...)`
    /// A method would interfere with methods of the same name on the contents to the locked data
    #[inline]
    pub fn map<U: ?Sized, F>(s: Self, f: F) -> MappedMutexGuard<'a, R, U>
    where 
        F : FnOnce(&mut T) -> &mut U,
    {
        let raw = s.raw;
        let data = f(unsafe { &mut *s.data });
        mem::forget(s);
        MappedMutexGuard { 
            raw, 
            data, 
            marker: PhantomData 
        }
    }

    /// Attempts to make a new `MappedMutexGuard` for a component of the locked data.
    /// THe original guard is returned if the closure returns `None`.
    /// 
    /// This operation cannot fail as the `MappedMutexGuard` passed in already locked the mutex.
    /// 
    /// This is an associated function that needs to be used as `MappedMutexGuard::try_map(...)`.
    /// A method would interfere with methods of the same name on the conetens of the locked data.
    #[inline]
    pub fn try_map<U: ?Sized, F>(s: Self, f: F) -> Result<MappedMutexGuard<'a, R, U>, Self> 
    where
        F : FnOnce(&mut T) -> Option<&mut U>
    {
        let raw = s.raw;
        let data = match f(unsafe { &mut *s.data }) {
            Some(data) => data,
            None => return Err(s),
        };
        mem::forget(s);
        Ok(MappedMutexGuard { 
            raw, 
            data, 
            marker: PhantomData 
        })
    }
}

impl<'a, R: RawMutexFair + 'a, T: ?Sized + 'a> MappedMutexGuard<'a, R, T> {
    /// Unlocks the mutex using a fair unlock protocol.
    /// 
    /// By default, mutexes are unfair and allow the current thread to re-lock the mutex before another has the change to acquire the lock, even if that thread has been blocked on the mutex for a long time.
    /// This is the default because it allows much higher throughput as it avoids forcing a context switch on every mutex unlock.
    /// This can result in one htread acquiring a mutex many more times than other threads.
    /// 
    /// However is some cases it can be beneficial to ensure fairness by forcing a lock to pass on to a waiting thread if there is one.
    /// This is done by using this method instead of dropping the `MutexGuard` normally.
    #[inline]
    pub fn unlock_fair(s: Self) {
        // SAFETY: A MappedMutexGuard always holds the lock
        unsafe {
            s.raw.unlock_fair();
        }
        mem::forget(s);
    }
}

impl<'a, R: RawMutex + 'a, T: ?Sized + 'a> Deref for MappedMutexGuard<'a, R, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data }
    }
}

impl<'a, R: RawMutex + 'a, T: ?Sized + 'a> DerefMut for MappedMutexGuard<'a, R, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.data }
    }
}

impl<'a, R: RawMutex + 'a, T: ?Sized + 'a> Drop for MappedMutexGuard<'a, R, T> {
    fn drop(&mut self) {
        // SAFETY: A MappedMutexGuard always holds the lock
        unsafe {
            self.raw.unlock();
        }
    }
}

impl<'a, R: RawMutex + 'a, T: ?Sized + fmt::Debug + 'a> fmt::Debug for MappedMutexGuard<'a, R, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<'a, R: RawMutex + 'a, T: ?Sized + fmt::Display + 'a> fmt::Display for MappedMutexGuard<'a, R, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}