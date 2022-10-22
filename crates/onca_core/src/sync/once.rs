// Portions of the project have been copied from parking_lot and is copyrighted by Amanieu d'Antra under the MIT license (located in: '3rd-party-licenses/parking_lot')

use core::{
    fmt,
    mem,
    sync::atomic::{fence, AtomicU8, Ordering},
};
use super::thread_parker::{self, SpinWait, DEFAULT_PARK_TOKEN, DEFAULT_UNPARK_TOKEN};

const DONE_BIT   : u8 = 0b0001;
const POISON_BIT : u8 = 0b0010;
const LOCKED_BIT : u8 = 0b0100;
const PARKED_BIT : u8 = 0b1000;

/// Current state of a `Once`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OnceState {
    /// A closure has not been executed yet.
    New,

    /// A closure was executed but panicked.
    Poisoned,

    /// A thread is currently executing a closure.
    InProgress,

    /// A closue has completed successfully.
    Done,
}

impl OnceState {
    /// Returns whether the associated `Once` has been poisoned
    /// 
    /// Once an initialization routine for a `Once` has panicked it will forever indicate to future forced initialization routines that it is poisoned
    #[inline]
    pub fn poisoned(self) -> bool {
        match self {
            OnceState::Poisoned => true,
            _ => false,
        }
    }

    /// Returns whether teh associated `Once` has successfully executed a closure.
    #[inline]
    pub fn done(self) -> bool {
        match self {
            OnceState::Done => true,
            _ => false
        }
    }
}

/// A synchronization primitive which can be used to run a one-time initialization.
/// Useful for one-time initialization for globals, FFI or related functionality.
/// 
/// # Differences from the standard library `Once`
/// 
/// - Only requires 1 byte of space, instead of 1 word.
/// - Not required to be `'static`.
/// - Relaxed memory barriers in the fast path, which can significantly improve performance on some architectures.
/// - Efficient handling of micro-contention using adaptive spinning.
pub struct Once(AtomicU8);

impl Once {
    /// Creates a new `Once` value
    #[inline]
    pub const fn new() -> Self {
        Self(AtomicU8::new(0))
    }

    /// Returns the current state of this `Once`
    #[inline]
    pub fn state(&self) -> OnceState {
        let state = self.0.load(Ordering::Acquire);
        if state & DONE_BIT != 0 {
            OnceState::Done
        } else if state & LOCKED_BIT != 0 {
            OnceState::InProgress
        } else if state & POISON_BIT != 0 {
            OnceState::Poisoned
        } else {
            OnceState::New
        }
    }

    /// Performs an initialization routine once and only once.
    /// The given closure will be executed if this is the first time `call_once` has been called, and otherwise the routine will *not* be invoked.
    /// 
    /// This method will block the calling thread if another initialization routine is currently running.
    /// 
    /// When this function returns, it is guaranteed that some initialization has run and completed (it may not be the closure specified).
    /// It is also guaranteeed that wny memeory writes performed by the executed closure can be reliably observed by other threads at this point (there is a happens-before relation betwwn the closure and code executing after the return).
    #[inline]
    pub fn call_once<F>(&self, f: F)
    where 
        F : FnOnce(),
    {
        if self.0.load(Ordering::Acquire) == DONE_BIT {
            return;
        }

        let mut f = Some(f);
        self.call_once_slow(false, &mut |_| unsafe { f.take().unwrap_unchecked()() });
    }

    /// Performs the same function as `call_once` except ignores poisoning
    /// 
    /// If this `Once` has been poisoned (some intiailization panicked), then this function will continue to attempt to call initialization functions until one of them doesn't panic.
    /// 
    /// The closure `f` is yielded a structure which can be used to query the state of this `Once` (whether initialization has previously panicked or not)
    #[inline]
    pub fn call_once_force<F>(&self, f: F)
    where
        F : FnOnce(OnceState),
    {
        if self.0.load(Ordering::Acquire) == DONE_BIT {
            return;
        }

        let mut f = Some(f);
        self.call_once_slow(true, &mut |state| unsafe { f.take().unwrap_unchecked()(state) });
    }

    /// This is a non-generic functon to reduce the monomorphization cost of unsing `call_once` (this isn't exactly a trivial or small iplementation).
    /// 
    /// Additionally, this is tagged with `#[cold]` as it should indeed be cold and it helps let LLVM know that calls to this function should be off the fast path.
    /// Essentially, this should help generate more straight line code in LLVM.
    /// 
    /// Finally, this takes an `FnMut` instead of a `FnOnce` because there's currently no way to take an `FnOnce` and call it via virtual dispatch without some allocation overhead.
    #[cold]
    fn call_once_slow(&self, ignore_poison: bool, f: &mut dyn FnMut(OnceState)) {
        let mut spinwait = SpinWait::new();
        let mut state = self.0.load(Ordering::Relaxed);
        loop {
            // If another thread called the closure, we're done
            if state & DONE_BIT != 0 {
                // An acquire fence is needed here since we didn't load the state with Ordering::Acquire.
                fence(Ordering::Acquire);
                return;
            }

            // If the state has been poisoned and we aren't forcing, then panic
            if state & POISON_BIT != 0 && !ignore_poison {
                // Need the fnece here as well for the same reason
                fence(Ordering::Acquire);
                panic!("Once instance has previously been poisoned");
            }

            // Grab teh lock if it isn't locked, even if there is a queue on it.
            // We also clear the poison bit since we are going to try running the closure again.
            if state & LOCKED_BIT == 0 {
                match self.0.compare_exchange_weak(
                    state,
                    (state | LOCKED_BIT) & !PARKED_BIT,
                    Ordering::Acquire, 
                    Ordering::Relaxed
                ) {
                    Ok(_) => break,
                    Err(x) => state = x,
                }
                continue;
            }

            // If there is no queue, try spinning a few times
            if state & PARKED_BIT == 0 && spinwait.spin() {
                state = self.0.load(Ordering::Relaxed);
                continue;
            }

            // Set the parked bit.
            if state & PARKED_BIT == 0 {
                if let Err(x) = self.0.compare_exchange_weak(
                    state, 
                    state | PARKED_BIT, 
                    Ordering::Relaxed,
                    Ordering::Relaxed
                ) {
                    state = x;
                    continue;
                }
            }

            // Park our thread until we are woken up by the thead that own the lock.
            let addr = self as *const _ as usize;
            let validate = || self.0.load(Ordering::Relaxed) == LOCKED_BIT | PARKED_BIT;
            let before_sleep = || {};
            let timed_out = |_, _| unreachable!();
            unsafe {
                thread_parker::park(
                    addr,
                    validate,
                    before_sleep,
                    timed_out,
                    DEFAULT_PARK_TOKEN,
                    None
                );
            }

            // Loop back and check if the done bit was set.
            spinwait.reset();
            state = self.0.load(Ordering::Relaxed);
        }

        struct PanicGuard<'a>(&'a Once);
        impl<'a> Drop for PanicGuard<'a> {
            fn drop(&mut self) {
                // Mark the state as poisoned, unlcok if and unpark all threads
                let once = self.0;
                let state = once.0.swap(POISON_BIT, Ordering::Release);
                if state & PARKED_BIT != 0 {
                    let addr = once as *const _ as usize;
                    unsafe {
                        thread_parker::unpark_all(addr, DEFAULT_UNPARK_TOKEN);
                    }
                }
            }
        }

        // At this point we have the lock, so run the closure.
        // Make sure we properly clean up if the closure panicks.
        let guard = PanicGuard(self);
        let once_state = if state & POISON_BIT != 0 {
            OnceState::Poisoned
        } else {
            OnceState::New
        };
        f(once_state);
        mem::forget(guard);

        // Now unlocks the state, set the done bit and unpark all threads
        let state = self.0.swap(DONE_BIT, Ordering::Release);
        if state & PARKED_BIT != 0 {
            let addr = self as *const _ as usize;
            unsafe {
                thread_parker::unpark_all(addr, DEFAULT_UNPARK_TOKEN);
            }
        }
    }
}

impl Default for Once {
    fn default() -> Self {
        Once::new()
    }
}

impl fmt::Debug for Once {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Once")
            .field("state", &self.state())
        .finish()
    }
}









