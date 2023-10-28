#![feature(local_key_cell_methods)]

use std::{
    cell::RefCell,
    io::Write,
};
use onca_common::{
    prelude::*,
    io,
};

mod escape_codes;
pub use escape_codes::*;

mod os;
use os::os_imp;

/// Terminal I/O (currently only write is supported)
pub struct Terminal;
pub type TerminalIOHandle = os_imp::IOHandle;

impl Terminal {
    /// Initialize the terminal, if it isn't initialized yet
    pub fn init() -> io::Result<()> {
        os_imp::Terminal::init()
    }

    /// Write a string to the terminal
    pub fn write(text: &str) -> io::Result<usize> {
        os_imp::Terminal::write(text)
    }

    /// Write a string to the terminal, with the given colors and formatting
    pub fn write_with(text: &str, fore: TerminalColor, back: TerminalColor, formatting: TextFormatting) -> io::Result<usize> {
        scoped_alloc!(AllocId::Malloc);
        thread_local! {
            static CACHE_LINE: RefCell<Option<String>> = RefCell::new(None);
        }

        // Longest line of escape codes is: 24-bit fore color, 24-bit back color, disable bold, underline and negative, and reset at the end
        const LONGEST_ESCAPE_CODES: &str = "\x1B[38;2;255;255;255m\x1B[48;2;255;255;255m\0x1B[22m\0x1B[24m\0x1B[27m\x1B[0m";
        const MIN_ADDITONAL_SIZE: usize = LONGEST_ESCAPE_CODES.len();

        CACHE_LINE.with_borrow_mut(|opt| {
            if opt.is_none() {
                *opt = Some(String::new());
            }
            let mut buf = opt.as_mut().unwrap();
            buf.clear();
            let needed_size = text.len() + MIN_ADDITONAL_SIZE;
            if needed_size > buf.capacity() {
                buf.reserve(needed_size - buf.capacity());
            }


            // SAFETY: We only write valid UTF-8, so we can safely write to the buffer as if it was a string
            _ = fore.write_fore_escape_code(unsafe { buf.as_mut_vec() });
            _ = back.write_back_escape_code(unsafe { buf.as_mut_vec() });
            _ = formatting.write_escape_code(unsafe { buf.as_mut_vec() });
            buf.push_str(text);
            buf.push_str("\x1B[0m");
            Self::write(&buf)
        })
    }

    /// Write bytes to the terminal
    pub fn write_bytes(bytes: &[u8]) -> io::Result<usize> {
        os_imp::Terminal::write_bytes(bytes)
    }

    pub fn reset_color_and_formatting() {
        Self::exec_terminal_sequence(|buf| {
            let _ = write!(buf, "\x1B[0m");
        })
    }

    /// Move the cursor
    pub fn move_cursor(cur_move: CursorMove) {
        Self::exec_terminal_sequence(|buf| {
            let _ = cur_move.write_escape_code(buf);
        })
    }

    /// Apply an action to the cursor
    pub fn cursor_action(action: CursorAction) {
        Self::exec_terminal_sequence(|buf| {
            _ = action.write_escape_code(buf);
        })
    }

    /// Set the cursor shape
    pub fn set_cursor_shape(shape: CursorShape) {
        Self::exec_terminal_sequence(|buf| {
            _ = shape.write_escape_code(buf);
        })
    }

    /// Scroll the terminal by `n` rows (negative values scroll up)
    pub fn scroll(n: i32) {
        Self::exec_terminal_sequence(|buf| {
            if n >= 0 {
                _ = write!(buf, "\x1B{n}S");
            } else {
                let m = -n;
                _ = write!(buf, "\x1B{m}T"); 
            }
        })
    }

    /// Modify the text in the terminal
    pub fn text_mod(tmod: TextMod) {
        Self::exec_terminal_sequence(|buf| {
            _ = tmod.write_escape_code(buf);
        })
    }

    /// Set the current text formatting
    pub fn set_formatting(formatting: TextFormatting) {
        Self::write_formatting(formatting)
    }

    fn write_formatting(formatting: TextFormatting) {
        Self::exec_terminal_sequence(|buf| {
            _ = formatting.write_escape_code(buf);
        })
    }

    /// Set the foreground color
    pub fn set_foreground_color(color: TerminalColor) {
        Self::write_foreground_color(color)
    }

    fn write_foreground_color(color: TerminalColor) {
        Self::exec_terminal_sequence(|buf| {
            _ = color.write_fore_escape_code(buf);
        })
    }

    /// Set the background color
    pub fn set_background_color(color: TerminalColor) {
        Self::write_background_color(color)
    }

    fn write_background_color(color: TerminalColor) {
        Self::exec_terminal_sequence(|buf| {
            let _ = color.write_back_escape_code(buf);
        })
    }

    /// Write a terminal sequence to the terminal, the sequence is written via the supplied `write_sequence` function
    pub fn exec_terminal_sequence<F>(write_sequence: F)
        where F : FnOnce(&mut Vec<u8>)
    {
        let _scoped_alloc = ScopedAlloc::new(AllocId::TlsTemp);

        let mut buffer = Vec::with_capacity(32);
        write_sequence(&mut buffer);
        let _ = unsafe { os_imp::Terminal::write(core::str::from_utf8_unchecked(&buffer)) };
    }

    /// Get the output handle
    pub fn get_output_handle(&self) -> TerminalIOHandle {
        os_imp::Terminal::get_output_handle()
    }
}