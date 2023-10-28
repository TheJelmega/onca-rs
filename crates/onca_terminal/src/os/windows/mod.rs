use onca_core::io;
use windows::Win32::{
    Foundation::HANDLE, 
    System::Console::{
        WriteConsoleA, AllocConsole, GetStdHandle, SetConsoleMode, GetConsoleMode,
        STD_OUTPUT_HANDLE, STD_HANDLE,
        ENABLE_WRAP_AT_EOL_OUTPUT, ENABLE_VIRTUAL_TERMINAL_PROCESSING,
    }, Storage::FileSystem::WriteFile,
};

fn get_std_handle(handle: STD_HANDLE) -> io::Result<HANDLE> {
    unsafe { GetStdHandle(handle) }.map_err(|err| io::Error::from_raw_os_error(err.code().0))
}

// The terminal code expects codepage 65001 (UTF-8) to be set, otherwise it may cause some weird glitches
pub struct Terminal;

pub type IOHandle = HANDLE;

impl Terminal {
    pub(crate) fn init() -> io::Result<()> {
        // Try to create a new terminal. This will fail if one already exists, in case that happens, just reuse it
        match unsafe { AllocConsole() } {
            Ok(_) => Ok(()),
            Err(_) => {
                let output = get_std_handle(STD_OUTPUT_HANDLE)?;
                unsafe { SetConsoleMode(output, ENABLE_WRAP_AT_EOL_OUTPUT | ENABLE_VIRTUAL_TERMINAL_PROCESSING) }
                    .map_err(|err| io::Error::from_raw_os_error(err.code().0))
            },
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
        unsafe { WriteConsoleA(
            handle,
            bytes,
            Some(&mut chars_written),
            None
        ) }.map_or_else(|err| Err(io::Error::from_raw_os_error(err.code().0)), |_| Ok(chars_written as usize))
    }

    unsafe fn write_non_terminal(handle: HANDLE, utf8: &[u8]) -> io::Result<usize> {
        let mut chars_written = 0;
        WriteFile(
            handle,
            Some(utf8),
            Some(&mut chars_written),
            None
        ).map_or_else(|err| Err(io::Error::from_raw_os_error(err.code().0)), |_| Ok(chars_written as usize))
    }

    pub(crate) fn get_output_handle() -> IOHandle {
        get_std_handle(STD_OUTPUT_HANDLE).unwrap_or_default()
    }
}

fn is_terminal(handle: HANDLE) -> bool {
    let mut mode = Default::default();
    unsafe { GetConsoleMode(handle, &mut mode) }.is_ok()
}