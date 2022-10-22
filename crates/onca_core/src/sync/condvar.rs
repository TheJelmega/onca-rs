// Portions of the project have been copied from parking_lot and is copyrighted by Amanieu d'Antra under the MIT license (located in: '3rd-party-licenses/parking_lot')

use core::{
    fmt,
    ptr,
    sync::atomic::{AtomicPtr, Ordering},
    ops::Deref,
};
use std::ops::DerefMut;
use super::mutex::MutexGuard;
use super::raw_mutex::{RawMutex, TOKEN_HANDOFF, TOKEN_NORMAL};
use super::lock_imp::RawMutex as RawMutexT;
use super::thread_parker::{self, ParkResult, RequeueOp, UnparkResult, DEFAULT_PARK_TOKEN};
use crate::time::{Duration, Instant};

/// A type indicationg whether a timed wait on a condition variable returned due to a time out or not.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct WaitTimeoutResult(bool);

impl WaitTimeoutResult {
    /// Returns whether the wiat was known to have timed out.
    #[inline]
    pub fn timed_out(self) -> bool {
        self.0
    }
}

/// A condition variable
/// 
/// Condition variables represent the ability to block a thread such that it consumes no GPU time while waiting for an even to occur.
/// Condition variablles are typically associated with a boolean predicate (a condition) and a mutex.
/// The predicate is always verified inside of the mutex before determining that a thread must block.
/// 
/// Note that this module places one additional restriction over the system condition variables: each condvar can be used with only one mutex at a time.
/// Any attempt to use mutliple mutexes on the same condition variable simultaneously will result in a runtime panic.
/// However it is possible to switch to a different mutec, if there are not threads currently waiting on the condition variable.
/// 
/// # Differences from the standard library `Condvar`
/// 
/// - No spurious wakeups: A wait will only return a non-timeout result if it was woken up by `notify_one` or `notify_all`.
/// - `Condvar::notify_all` will only wake up a single thread, teh rest are requeued to wait for the `Mutex` to be unlocked by the thread that was woken up.
/// - Only requires 1 word of space, whereas the standard library boxes the `Condvar` due to platform limitations.
/// - Can be statically constructed.
/// - Does not erquire any drop glue when dropped
/// - Inline fast path for the uncontended case. 
pub struct Condvar {
    state : AtomicPtr<RawMutex>,
}

impl Condvar {
    /// Creates a new condition variable which is ready to be waited on and notified.
    #[inline]
    pub const fn new() -> Self {
        Self { state: AtomicPtr::new(ptr::null_mut()) }
    }

    /// Wakes up one blocked thread on this condvar.
    /// 
    /// Returns whtehr a thread was woken up.
    /// 
    /// If there is a blocked thread on this condition variable, then it will be woken up from its call to `wait` or `wait_times`.
    /// Calls to `notify_one` are not buffered in any way.
    /// 
    /// To wake up all threads, see `notify_all`
    #[inline]
    pub fn notify_one(&self) -> bool {
        // Nothing to do if there are no waiting threads
        let state = self.state.load(Ordering::Relaxed);
        if state.is_null() {
            return false;
        }
        self.notify_one_slow(state)
    }

    #[cold]
    fn notify_one_slow(&self, mutex: *mut RawMutex) -> bool {
        // Unpark one thread and requeue the rest onto the mutex
        let from = self as *const _ as usize;
        let to = mutex as usize;
        let validate = || {
            // Make sure that our atomic state still point to the same mutex.
            // If not then it means that all threads on the current mutex were woken up and a new waiting thread switched to a different mutex.
            // In that case we can get away with doing nothing.
            if self.state.load(Ordering::Relaxed) != mutex {
                return RequeueOp::Abort;
            }

            // Unpark one thread if the mutex is unlocked, otherwise just requeue everything to the mutex.
            // This is safe to do here since unlcoking the mutex when the parked bit is set requires locking the queue.
            // There is the possibility of a race if the mutex gets locked after we check, but that doesn't matter in this case.
            if unsafe { (*mutex).mark_parked_if_locked() } {
                RequeueOp::RequeueOne
            } else {
                RequeueOp::UnparkOne
            }
        };
        let callback = |_op, result: UnparkResult| {
            // Clear our stat if there are no more waiting threads
            if !result.have_more_threads {
                self.state.store(ptr::null_mut(), Ordering::Relaxed);
            }
            TOKEN_NORMAL
        };
        let res = unsafe { thread_parker::unpark_requeue(from, to, validate, callback) };

        res.unparked_threads + res.requeued_threads != 0
    }

    /// Wakes up all blocked threads on this condvar.
    /// 
    /// Return th number of threads woken up.
    /// 
    /// This method will ensure that any current waiters on the condition wariable are awoken.
    /// Calls to `notify_all()` are not buffered in any way.
    /// 
    /// To wake up only one thread, see `notify_one()`
    #[inline]
    pub fn notify_all(&self) -> usize {
        // Nothing to do if there are no waiting threads
        let state = self.state.load(Ordering::Relaxed);
        if state.is_null() {
            return 0;
        }
        self.notify_all_slow(state)
    }

    #[cold]
    fn notify_all_slow(&self, mutex: *mut RawMutex) -> usize {
        // Unpark one thread and requeue the result onto the mutex.
        let from = self as *const _ as usize;
        let to = mutex as usize;
        let validate = || {
            // Make sure that our atomic state still point to the same mutex.
            // If not then it means that all threads on the current mutex were woken up and a new waiting thread switched to a different mutex.
            // In that case we can get away with doing nothing.
            if self.state.load(Ordering::Relaxed) != mutex {
                return RequeueOp::Abort;
            }

            // Clear our state since we are going to unpark to requeue all threads.
            self.state.store(ptr::null_mut(), Ordering::Relaxed);

            // Unpark one thread if the mutex is unlocked, otherwise just requeue everything to the mutex.
            // This is safe to do here since unlcoking the mutex when the parked bit is set requires locking the queue.
            // There is the possibility of a race if the mutex gets locked after we check, but that doesn't matter in this case.
            if unsafe { (*mutex).mark_parked_if_locked() } {
                RequeueOp::RequeueAll
            } else {
                RequeueOp::UnparkOnRequestRest
            }
        };
        let callback = |op, result: UnparkResult| {
            // If we requeued to the mutex, mark is as having parked threads.
            // The RequeueAll case is already handled above
            if op == RequeueOp::UnparkOnRequestRest && result.requeued_threads != 0 {
                unsafe { (*mutex).mark_parked() };
            }
            TOKEN_NORMAL
        };
        let res = unsafe { thread_parker::unpark_requeue(from, to, validate, callback) };

        res.unparked_threads + res.requeued_threads
    }

    /// Blocks the current thread until this condition variable receives a notification.
    /// 
    /// This function will atomically unlock the mutex specified (represented by `mutex_guard`) and block the current thread.
    /// This means that any calls to `notify_*()` which happen logically after the mutex is unlocked are candidates to wake this thread up.
    /// When this function call returns, the lock specificed will have been re-acquired.
    /// 
    /// # Panics
    /// 
    /// This function will pnaic if another thread is waiting on the `Condvar` with a different `Mutex` object.
    #[inline]
    pub fn wait<T: ?Sized>(&self, mutex_guard: &mut MutexGuard<'_, T>) {
        self.wait_until_internal(unsafe { MutexGuard::mutex(mutex_guard).raw() }, None);
    }

    /// Waits on this condition variable for a notification, timing out after the specified time instant.
    /// 
    /// The semantics of this function are equivalend to `wait()` except that the thread will be blocked roughly until `timeout` is reached.
    /// This method should not be used for precise timing due to anomalies such as preemption or platform differences that may not cause the maximum amount of time waited to be precisely `timeout`.
    /// 
    /// Note that the best effort is made to ensure that the time waited is measured with a monotonic clock, and not affected by the changes made to the system time.
    /// 
    /// The returned `WaitTimeoutResult` value indicates if the timeout is known to have elapsed.
    /// 
    /// Like `wait`, the lock specified will be re-acquired when this function returns, regardless of whether the timeout elapsed or not.
    /// 
    /// # Panics
    /// 
    /// This function will panic if another thread is waiting on the `Condvar` with a different `Mutex` object
    #[inline]
    pub fn wait_until<T: ?Sized>(&self, mutex_guard: &mut MutexGuard<'_, T>, timeout: Instant) -> WaitTimeoutResult {
        self.wait_until_internal(unsafe { MutexGuard::mutex(mutex_guard).raw() }, Some(timeout))
    }

    /// Waits on this condition variable for a notification, timing out after the specified time instant.
    /// 
    /// The semantics of this function are equivalend to `wait()` except that the thread will be blocked roughly until `timeout` is reached.
    /// This method should not be used for precise timing due to anomalies such as preemption or platform differences that may not cause the maximum amount of time waited to be precisely `timeout`.
    /// 
    /// Note that the best effort is made to ensure that the time waited is measured with a monotonic clock, and not affected by the changes made to the system time.
    /// 
    /// The returned `WaitTimeoutResult` value indicates if the timeout is known to have elapsed.
    /// 
    /// Like `wait`, the lock specified will be re-acquired when this function returns, regardless of whether the timeout elapsed or not.
    #[inline]
    pub fn wait_for<T: ?Sized>(&self, mutex_guard: &mut MutexGuard<'_, T>, timeout: Duration) -> WaitTimeoutResult {
        let deadline = Instant::now().checked_add(timeout);
        self.wait_until_internal(unsafe { MutexGuard::mutex(mutex_guard).raw() }, deadline)
    }

    // This is a non-generic function to reducethe monomorphization cost of using `Wait_until`
    fn wait_until_internal(&self, mutex: &RawMutex, timeout: Option<Instant>) -> WaitTimeoutResult {
        let result;
        let mut bad_mutex = false;
        let mut requeued = false;
        {
            let addr = self as *const _ as usize;
            let lock_addr = mutex as *const _ as *mut _;
            let validate = || {
                // Ensure we don't use two different mutexes with teh same Condvar at the same time.
                // This is done while locked to avoid races with notify_one
                let state = self.state.load(Ordering::Relaxed);
                if state.is_null() {
                    self.state.store(lock_addr, Ordering::Relaxed);
                } else if state != lock_addr {
                    bad_mutex = true;
                    return false;
                }
                true
            };
            let before_sleep = || {
                // unlock the mutex before sleeping...
                unsafe { mutex.unlock() };
            };
            let timed_out = |k, was_last_thread| {
                // If we were requeued to a mutex, then we did not time out.
                // We'll just park ourselves on the mutex again when we try to lock it later.
                requeued = k != addr;

                // If we were the last thread on the queue, then we need to clear our state.
                // This is normally done by the notify_{one,all} functions when not timing out.
                if !requeued && was_last_thread {
                    self.state.store(ptr::null_mut(), Ordering::Relaxed);
                }
            };
            result = unsafe {
                thread_parker::park(
                    addr,
                    validate,
                    before_sleep,
                    timed_out,
                    DEFAULT_PARK_TOKEN,
                    timeout
                )
            };
        }

        // Panic if we tried to use multiple mutexes with a Condvar.
        // Note that at this point the MutexGuard is still locked. 
        // It will be unlocked by the unwinding logic.
        if bad_mutex {
            panic!("attempted to use a condition variable with more than one mutex");
        }

        // ... and re-lock it once we are done sleeping
        if result == ParkResult::Unparked(TOKEN_HANDOFF) {
            // IMPL: deadlock detection has call here
        } else {
            mutex.lock()
        }

        WaitTimeoutResult(!(result.is_unparked() || requeued))
    }

    #[inline]
    fn wait_while_until_internal<T, F>(
        &self,
        mutex_guard: &mut MutexGuard<'_, T>,
        mut condition : F,
        timeout: Option<Instant>
    ) -> WaitTimeoutResult
    where
        T : ?Sized,
        F : FnMut(&mut T) -> bool,
    {
        let mut result = WaitTimeoutResult(false);

        while !result.timed_out() && condition(mutex_guard.deref_mut()) {
            result = self.wait_until_internal(unsafe { MutexGuard::mutex(mutex_guard).raw() }, timeout);
        }

        result
    }

    /// Block the current thread until this condition variable receives a notification.
    /// If the provided condition evaluates to `false`, then the thread is no longer blocked and the operation is completed.
    /// If the condition evaluates to `true`, then the thread is blocked again and waits for another notification before repeating this process.
    /// 
    /// This function will atomically unlock the mutex specified (represented by `mutex_guard`) and block the current thread.
    /// This means that any calls to `notify_*()` which happen logically after the mutex is unlocked are candidates to wake this thread up.
    /// When this function call returns, the lock specificed will have been re-acquired.
    /// 
    /// # Panics
    /// 
    /// This function will pnaic if another thread is waiting on the `Condvar` with a different `Mutex` object.
    #[inline]
    pub fn wait_while<T, F>(&self, mutex_guard: &mut MutexGuard<'_, T>, condition : F)
    where
        T : ?Sized,
        F : FnMut(&mut T) -> bool,
    {
        self.wait_while_until_internal(mutex_guard, condition, None);
    }

    /// Waits on this condition variable for a notification, timing out oafter the specified time instant.
    /// 
    /// If the provided condition evaluates to `false`, then the thread is no longer blocked and the operation is completed.
    /// If the condition evaluates to `true`, then the thread is blocked again and waits for another notification before repeating this process.
    /// 
    /// The semantics of this function are equivalend to `wait()` except that the thread will be blocked roughly until `timeout` is reached.
    /// This method should not be used for precise timing due to anomalies such as preemption or platform differences that may not cause the maximum amount of time waited to be precisely `timeout`.
    /// 
    /// Note that the best effort is made to ensure that the time waited is measured with a monotonic clock, and not affected by the changes made to the system time.
    /// 
    /// The returned `WaitTimeoutResult` value indicates if the timeout is known to have elapsed.
    /// 
    /// Like `wait`, the lock specified will be re-acquired when this function returns, regardless of whether the timeout elapsed or not.
    /// 
    /// # Panics
    /// 
    /// This function will panic if another thread is waiting on the `Condvar` with a different `Mutex` object
    #[inline]
    pub fn wait_while_until<T, F>(
        &self,
        mutex_guard: &mut MutexGuard<'_, T>,
        condition : F,
        timeout: Instant
    ) -> WaitTimeoutResult
    where
        T : ?Sized,
        F : FnMut(&mut T) -> bool
    {
        self.wait_while_until_internal(mutex_guard, condition, Some(timeout))
    }

    /// Waits on this condition variable for a notification, timing out oafter the specified time instant.
    /// 
    /// If the provided condition evaluates to `false`, then the thread is no longer blocked and the operation is completed.
    /// If the condition evaluates to `true`, then the thread is blocked again and waits for another notification before repeating this process.
    /// 
    /// The semantics of this function are equivalend to `wait()` except that the thread will be blocked roughly until `timeout` is reached.
    /// This method should not be used for precise timing due to anomalies such as preemption or platform differences that may not cause the maximum amount of time waited to be precisely `timeout`.
    /// 
    /// Note that the best effort is made to ensure that the time waited is measured with a monotonic clock, and not affected by the changes made to the system time.
    /// 
    /// The returned `WaitTimeoutResult` value indicates if the timeout is known to have elapsed.
    /// 
    /// Like `wait`, the lock specified will be re-acquired when this function returns, regardless of whether the timeout elapsed or not.
    /// 
    /// # Panics
    /// 
    /// This function will panic if another thread is waiting on the `Condvar` with a different `Mutex` object
    #[inline]
    pub fn wait_while_for<T, F>(
        &self,
        mutex_guard: &mut MutexGuard<'_, T>,
        condition : F,
        timeout: Duration
    ) -> WaitTimeoutResult
    where
        T : ?Sized,
        F : FnMut(&mut T) -> bool
    {
        let deadline = Instant::now().checked_add(timeout);
        self.wait_while_until_internal(mutex_guard, condition, deadline)
    }
}

impl Default for Condvar {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Condvar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("Condvar { .. }")
    }
}
  
