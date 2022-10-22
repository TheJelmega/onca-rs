// Portions of the project have been copied from parking_lot and is copyrighted by Amanieu d'Antra under the MIT license (located in: '3rd-party-licenses/parking_lot')
use super::imp::thread_yield;
use core::hint::spin_loop;

// Wastes some CPU time for the given number or iterations, using a hint to indicate to the CPU that we are spinning
#[inline]
fn cpu_relax(iterations: u32) {
    for _ in 0..iterations {
        spin_loop()
    }
}

// A counter used to perform exponential backoff in spin loops

#[derive(Default)]
pub struct SpinWait {
    counter: u32,
}

impl SpinWait {
    /// Creates a new `SpinWait`
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Resets a `SpinWait` to its initial state
    #[inline]
    pub fn reset(&mut self) {
        self.counter = 0;
    }

    /// Spins until the sleep threshold has been reached.
    /// 
    /// This function returns wether the sleep threshold has been reached, at which point further spinning has diminishing returns and the thread should be parked instead.
    /// 
    /// The spin strategy will initially use a CPU-bound loop, but will fall back to yielding he CPU to the OS after a few iterations.
    #[inline]
    pub fn spin(&mut self) -> bool {
        if self.counter >= 10 {
            return false;
        }

        self.counter += 1;
        if self.counter <= 3 {
            cpu_relax(1 << self.counter);
        } else {
            thread_yield();
        }
        true
    }

    /// Spins wihtout yielding the thread to the OS.
    /// 
    /// Instead, the backoff is simply capped at a maximum value.
    /// This can be used to improve throughput in `compare_exchange` loops that have a high contention
    #[inline]
    pub fn spin_no_yield(&mut self) {
        self.counter += 1;
        if self.counter > 10 {
            self.counter = 10;
        }
        cpu_relax(1 << self.counter);
    }
}