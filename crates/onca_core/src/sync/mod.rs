// Portions of the project have been copied from parking_lot and is copyrighted by Amanieu d'Antra under the MIT license (located in: '3rd-party-licenses/parking_lot')

//! Useful synchronization primitives
//! 
//! ## The need for synchronization
//! 
//! Conceptually, a rust program is a seried of operation which will be executed on a computer.
//! The timeline of events happening in the program is consistent with the order of operations in code.
//! 
//! Consider the following code, operating on some global static variables
//! 
//! ```rust
//! static mut A: u32 = 0;
//! static mut B: u32 = 0;
//! static mut C: u32 = 0;
//! 
//! fn main() {
//!     unsafe {
//!         A = 3;
//!         B = 4;
//!         A = A + B;
//!         C = B;
//!         println!("{A} {B}, {C}");
//!         C = A;
//!     }
//! }
//! ```
//! 
//! It appears as if some variables stored in memory are changed, an addition is performed, result is stored in `A` and the variable `C` is modified twice.
//! 
//! When only a single thread is involved, the results are as expected: the line `7, 4, 4` gets printed.
//! 
//! As for what happens behind the scenes, when optimizations are enabled, the  final generated machine code migh look very different from this code.
//! 
//! - The first store to `C` might be moved before the before the store to `A` or `B`, _as if_ we had written `C = 4; A = 3; B = 4;`.
//! 
//! - Assignment of `A + B` to `A` might be removed, since the sum can be stored in a temporary location until it gets printed, with the global variable never getting updated.
//! 
//! - The final result could be determined just by looking at the code at compile time, so [constant folding] might turn the whole block into a simple `println!("7 4 4");
//! 
//! The compiler is allowed to perform any combination of theseoptimizations, as ling as the final optimized code, when executed, produces the same results as the one without optimizations
//! 
//! Due to the [concurrency] involved in modern computers, assumptions about the program's execution order are often wrong.
//! Access to global variables can lead to nodeterministic results, **even if** compiler optimizations are disabled, and it is **still possible** to introduce synchronization bugs.
//! 
//! Note that thnks to rust's safety guarentees, accessing global (static) variables requires `unsafe` code, assuming we don't use any of the synchronization primitives in this module
//! 
//! [constant folding]: https://en.wikipedia.org/wiki/Constant_folding
//! [concurrency]: https://en.wikipedia.org/wiki/Concurrency_(computer_science)
//! 
//! ## Out of order execution
//! 
//! Instructions can be executes in a different order than the one we defined, due to various reasons:
//! 
//! - The **compiler** reodering instructions:
//!   If the compiler can issue an instruction at an earlier point, it will try to do so. 
//!   For example, it might hoist memory loads at th top of a code block, so that the CPU can start [prefecthing] the values from memory.
//! 
//!   In single-threaded scenarios, this can cause issues when writing signal handlers or certain kinds of low-level code.
//!   Use [compiler fences] to prevent this reordering.
//! 
//! - A *single processor** executing instructions [out-of-order]:
//!   Modern CPUs are capable of [superscalar] execution, i.e. multiple instructions might be executing at he same time, even thoug the machine code describes a sequential process.
//! 
//!   This kind of reordering is handled transparently by the CPU.
//! 
//! - A **multiprocessor** system executing multiple hardware threads at the same time:
//!   In multi-threaded scenearios, you can use two kinds of primitives to deal with synchronization:
//!   - [memory fences] to ensure memory accesses to the same memory location doesn't lead to undefined behavior
//!   - [atomic operations] to ensure simultaniously access to the same memory location doesn't lead to undefined behavior
//! 
//! [prefetching]: https://en.wikipedia.org/wiki/Cache_prefetching
//! [compiler fences]: core::sync::atomic::compiler_fence
//! [out-of-order]: https://en.wikipedia.org/wiki/Out-of-order_execution
//! [superscalar]: https://en.wikipedia.org/wiki/Superscalar_processor
//! [memory fences]: core::sync::atomic::fence
//! [atomic operations]: core::sync::atomic
//! 
//! ## Higher-level synchronization primitives
//! 
//! Most of the low-level synchronization primitives are quite error-prone and inconvenient to use, which is why onca also exposes some higher level synchronization objects.
//! 
//! The abstractions can be built out of lower-level primitives.
//! For efficiency, teh sync object in onca are usually implemented with help from the operating system's kernel, which is able to rechedule the threads while they are eblocked on acquiring a lock.
//! 
//! The following is an overview of the available synchronization primitives:
//! 
//! - [`Barrier`]: Ensures multiple threads will wait for each other to reach a point in the program, before continuing execution all together.
//! 
//! - [`CondVar`]: Condition variable: providing the ability to block a thread while waiting for an even to occur.
//! 
//! ## Inspiration
//! 
//! A major part of the sync module was inspired by [Webkit's locks] (which is itself inspired by linux' futexes) and the subsequentally the [`parking_lot`] crate
//! 
//! ### Why not simply use [`parking_lot`], even just its core parts?
//! 
//! The main reason is [`smallvec`], as onca want control over all allocations, instead of rust's allocator system.
//! While allocations could be avoided, this would limit the amount of threads that can access a single primitive to 8, if [`parking_lot`]'s implementation details don't change
//! 
// TODO
//! [`Barrier`]:
//! 
//! [WebKit's locks]: https://webkit.org/blog/6161/locking-in-webkit/
//! [`parking_lot`]: https://github.com/Amanieu/parking_lot

pub mod thread_parker;
pub mod lock_imp;

mod condvar;
mod elision;
mod fair_mutex;
mod mutex;
mod once;
mod raw_fair_mutex;
mod raw_mutex;
mod raw_rwlock;
mod remutex;
mod rwlock;

pub use fair_mutex::{FairMutex, FairMutexGuard, FairMappedMutexGuard};
pub use mutex::{Mutex, MutexGuard, MappedMutexGuard};
pub use once::{Once, OnceState};
pub use raw_fair_mutex::RawFairMutex;
pub use raw_mutex::RawMutex;
pub use raw_rwlock::RawRwLock;
pub use remutex::{MappedReentrantMutexGuard, RawThreadId, ReentrantMutex, ReentrantMutexGuard};
pub use rwlock::{MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockUpgradableReadGuard, RwLockWriteGuard};


#[cfg(target_os = "windows")]
pub use crate::os::windows::sync::*;

// NOTE(jel): if we'd implement parking_lot's deadlock detection, we need to have GuardNoSend here if it would be enabled
type GuardMarker = lock_imp::GuardSend;