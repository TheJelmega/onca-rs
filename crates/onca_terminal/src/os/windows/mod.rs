use core::ffi::c_void;
use onca_core::io;
use windows::Win32::{
    Foundation::HANDLE, 
    System::Console::{
        WriteConsoleA, AllocConsole, GetStdHandle, SetConsoleMode, GetConsoleMode,
        STD_OUTPUT_HANDLE, STD_HANDLE,
        ENABLE_WRAP_AT_EOL_OUTPUT, ENABLE_VIRTUAL_TERMINAL_PROCESSING,
    }, Storage::FileSystem::WriteFile,
};

unsafe fn get_std_handle(handle: STD_HANDLE) -> io::Result<HANDLE> {
    GetStdHandle(handle).map_err(|err| io::Error::from_raw_os_error(err.code().0))
}

// The terminal code expects codepage 65001 (UTF-8) to be set, otherwise it may cause some weird glitches
pub struct Terminal;

pub type IOHandle = HANDLE;

impl Terminal {
    pub(crate) fn init() -> io::Result<()> {
        unsafe {
            // Try to create a new terminal. This will fail if one already exists, in case that happens, just reuse it
            let res = AllocConsole().as_bool();
            if res {
                let output = get_std_handle(STD_OUTPUT_HANDLE)?;
                let res = SetConsoleMode(output, ENABLE_WRAP_AT_EOL_OUTPUT | ENABLE_VIRTUAL_TERMINAL_PROCESSING).as_bool();
                if !res {
                    return Err(io::Error::last_os_error());
                }
            }
            Ok(())
        } 
    }

    pub(crate) fn write(text: &str) -> io::Result<usize> {
        Terminal::write_bytes(text.as_bytes())
    }

    pub(crate) fn write_bytes(bytes: &[u8]) -> io::Result<usize> {
        unsafe {
            let output = get_std_handle(STD_OUTPUT_HANDLE)?;
            if is_terminal(output) {
                Self::write_terminal(output, bytes)
            } else {
                Self::write_non_terminal(output, bytes)
            }
        }
    }

    unsafe fn write_terminal(handle: HANDLE, bytes: &[u8]) -> io::Result<usize> {
        let mut chars_written = 0;
        let res  = WriteConsoleA(
            handle,
            bytes,
            Some(&mut chars_written),
            None
        );
        if res.as_bool() {
            Ok(chars_written as usize)
        } else {
            Err(io::Error::last_os_error())
        }
    }

    unsafe fn write_non_terminal(handle: HANDLE, utf8: &[u8]) -> io::Result<usize> {
        let mut chars_written = 0;

        let len = utf8.len();
        let ptr = utf8.as_ptr();

        let res = WriteFile(
            handle,
            Some(ptr as *const c_void),
            len as u32,
            Some(&mut chars_written),
            None
        );

        if res.as_bool() {
            Ok(chars_written as usize)
        } else {
            Err(io::Error::last_os_error())
        }
    }

    pub(crate) fn get_output_handle() -> IOHandle {
        unsafe { get_std_handle(STD_OUTPUT_HANDLE).unwrap_or_default() }
    }
}

fn is_terminal(handle: HANDLE) -> bool {
    let mut mode = Default::default();
    unsafe { GetConsoleMode(handle, &mut mode).as_bool() }
}