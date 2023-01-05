use windows::Win32::System::Threading::GetCurrentThreadId;

use crate::sys::ThreadId;

pub(crate) fn get_thread_id() -> ThreadId {
    ThreadId(unsafe { GetCurrentThreadId() })
}

pub(crate) fn get_main_thread_id() -> ThreadId {
    unsafe {

        static mut MAIN_THREAD_ID : u32 = 0;
        
        
        /// Function pointer used in CRT initialization section to set the cached main thread id 
        // Make sure this function isn't removed
        #[used]
        // Place the poitner inside of CRT initialization section so it is loaded before the main entry point
        //
        // See: https://doc.rust-lang.org/stable/reference/abi.html#the-link_section-attribute
        // and: https://learn.microsoft.com/en-us/cpp/c-runtime-library/crt-initialization
         #[link_section = ".CRT$XCU"]
        static INIT_MAIN_THREAD_ID : unsafe fn() = {
            unsafe fn initer() {
                MAIN_THREAD_ID = GetCurrentThreadId();
            }
            initer
        };
        
        ThreadId(MAIN_THREAD_ID)
    }
}