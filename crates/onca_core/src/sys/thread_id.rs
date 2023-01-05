use core::fmt;
use crate::os;

/// Thread ID
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct ThreadId(pub(crate) u32);

impl ThreadId {
    /// Get the thread id as a u32
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

impl fmt::Display for ThreadId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("'Thread {}'", self.0))
    }
}

/// Get the thread ID of the current thread
pub fn get_thread_id() -> ThreadId {
    os::thread::get_thread_id()
}

/// Get the main thread ID
pub fn get_main_thread_id() -> ThreadId {
    os::thread::get_main_thread_id()
}

/// Check if the current code is being run on the main thread
pub fn is_on_main_thread() -> bool {
    get_thread_id() == get_main_thread_id() 
}
