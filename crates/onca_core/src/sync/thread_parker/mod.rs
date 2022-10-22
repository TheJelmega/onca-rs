// Portions of the project have been copied from parking_lot and is copyrighted by Amanieu d'Antra under the MIT license (located in: '3rd-party-licenses/parking_lot')
//! This module exposes a low-level API for creating efficient synchronization primitives
//! 
//! # The parking lot
//! 
//! To keep synchronization primitives small, all thread queuing and suspending functionality is offloadedto the *parking lot*.
//! The idea dehind this is based on the Webkit [`WTF::ParkingLot`](https://webkit.org/blog/6161/locking-in-webkit/) class, 
//! which essentially consists of a hash table mapping of lock addresses to queues of parked (sleeping) threads.
//! The Webkit parking lot was itself inspired by Linux [futexes](http://man7.org/linux/man-pages/man2/futex.2.html),
//! but it is more powerful since it allows invoking callbacks while holding a queue lock
//! 
//! Thre are two main operation that can be performed on the parking lot:
//! 
//! - *Parking* refers to suspending the thread while simultaneously enqueing it on a queue keyed by some address.
//! - *Unparking* refers to dequeueing a thread from a queue keyed by some address and resuming it.
//! 
//! See the documentation of the individual functions for more details
//! 
//! # Building custom synchonization primitives
//! 
//! Building custom synchronization primitives is very simple since theparking lot takes care of all the hard parts for you.
//! A simple example for a custom primitive would be to integratea `Mutex` inseide another data type.
//! Since a mutex only requires 2 bits, it can share space with other data.
//! For example, one could crate an `ArcMutex` type that combines the atomic reference count and the two mutex bitgs in the same word

mod spin_wait;
mod word_lock;
mod parking_lot;

cfg_if::cfg_if!{
    if #[cfg(windows)] {
        use crate::os::windows::sync as imp;
    }
}

pub use self::parking_lot::{park, unpark_all, unpark_filter, unpark_one, unpark_requeue};
pub use self::parking_lot::{FilterOp, ParkResult, ParkToken, RequeueOp, UnparkToken, UnparkResult};
pub use self::parking_lot::{DEFAULT_PARK_TOKEN, DEFAULT_UNPARK_TOKEN};
pub use self::spin_wait::SpinWait;