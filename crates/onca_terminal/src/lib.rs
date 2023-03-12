use onca_core::{
    prelude::*,
    io::{self, prelude::*},
};

mod os;
use onca_core_macros::flags;
use os::os_imp;

#[derive(Clone, Copy)]
pub enum TerminalColor {
    Black,
    DarkRed,
    DarkGreen,
    DarkYellow,
    DarkBlue,
    DarkMagenta,
    DarkCyan,
    DarkGray,
    Gray,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Custom(u8, u8, u8)
}

/// Cursor move direction
pub enum CursorMove {
    /// Move the cursor up by 'n' characters
    Up(u32),
    /// Move the cursor down by 'n' characters
    Down(u32),
    /// Move the cursor forward by 'n' characters
    Forward(u32),
    /// Move the cursor backward by 'n' characters
    Backward(u32),
    /// Move the cursor to an absolute position
    Position(u32, u32),
}

/// Terminal cursor shape
pub enum CursorShape {
    /// User configured cursor shape
    User,
    /// Blinking block
    BlinkingBlock,
    /// Steady block
    SteadyBlock,
    /// Blinking underline
    BlinkingUnderline,
    /// Steady underline
    SteadyUnderline,
    /// Blinking bar
    BlinkingBar,
    /// Steady bar
    SteadyBar,
}

/// Terminal text modification
pub enum TextMod {
    /// Insert `n` spaces at the current cursor position
    Insert(u32),
    /// Delete `n` characters
    Delete(u32),
    /// Erase `n` characters by overwritting them with a space
    Erase(u32),
    /// Insert `n` lines
    InsertLine(u32),
    /// Delete `n` lines
    DeleteLine(u32),
}

/// Text formatting flags
#[flags]
pub enum TextFormatting {
    /// Text is bold
    Bold,
    /// Text is underlined
    Underline,
    /// Foreground and background colors are inverted
    Negative
}

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
        Self::write_foreground_color(fore);
        Self::write_background_color(back);
        Self::write_formatting(formatting);
        let res = Self::write(text);
        Self::reset_color_and_formatting();
        res
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
            let _ = match cur_move {
                CursorMove::Up(n) => write!(buf, "\0x1B[{n}A"),
                CursorMove::Down(n) => write!(buf, "\0x1B[{n}B"),
                CursorMove::Forward(n) => write!(buf, "\0x1B[{n}C"),
                CursorMove::Backward(n) => write!(buf, "\0x1B[{n}D"),
                CursorMove::Position(x, y) => write!(buf, "\0x1B[{y};{x}H"),
            };
        })
    }

    /// Set if the cursor is blinking
    pub fn set_cursor_blinking(blinking: bool) {
        Self::exec_terminal_sequence(|buf| {
            if blinking {
                let _ = write!(buf, "\x1B[?12h");
            } else {
                let _ = write!(buf, "\x1B[?12l");
            };
        })
    }

    /// Set if the cursor is shown
    pub fn set_cursor_shown(show: bool) {
        Self::exec_terminal_sequence(|buf| {
            if show {
                let _ = write!(buf, "\x1B[?25h");
            } else {
                let _ = write!(buf, "\x1B[?25l");
            };
        })
    }

    /// Set the cursor shape
    pub fn set_cursor_shape(shape: CursorShape) {
        Self::exec_terminal_sequence(|buf| {
            let _ = match shape {
                CursorShape::User              => write!(buf, "\x1B[0 q"),
                CursorShape::BlinkingBlock     => write!(buf, "\x1B[1 q"),
                CursorShape::SteadyBlock       => write!(buf, "\x1B[2 q"),
                CursorShape::BlinkingUnderline => write!(buf, "\x1B[3 q"),
                CursorShape::SteadyUnderline   => write!(buf, "\x1B[4 q"),
                CursorShape::BlinkingBar       => write!(buf, "\x1B[5 q"),
                CursorShape::SteadyBar         => write!(buf, "\x1B[6 q"),
            };
        })
    }

    /// Scroll the terminal by `n` rows (negative values scroll up)
    pub fn scroll(n: i32) {
        Self::exec_terminal_sequence(|buf| {
            if n >= 0 {
                let _ = write!(buf, "\x1B{n}S");
            } else {
                let m = -n;
                let _ = write!(buf, "\x1B{m}T"); 
            }
        })
    }

    /// Modify the text in the terminal
    pub fn text_mod(tmod: TextMod) {
        Self::exec_terminal_sequence(|buf| {
            let _ = match tmod {
                TextMod::Insert(n)     => write!(buf, "\x1B[{n}@"),
                TextMod::Delete(n)     => write!(buf, "\x1B[{n}P"),
                TextMod::Erase(n)      => write!(buf, "\x1B[{n}X"),
                TextMod::InsertLine(n) => write!(buf, "\x1B[{n}L"),
                TextMod::DeleteLine(n) => write!(buf, "\x1B[{n}M"),
            };
        })
    }

    /// Set the current text formatting
    pub fn set_formatting(formatting: TextFormatting) {
        Self::write_formatting(formatting)
    }

    fn write_formatting(formatting: TextFormatting) {
        Self::exec_terminal_sequence(|buf| {
            let _ = if formatting.is_set(TextFormatting::Bold) {
                write!(buf, "\x1B[1m")
            } else {
                write!(buf, "\x1B[22m")
            };
            let _ = if formatting.is_set(TextFormatting::Underline) {
                write!(buf, "\x1B[4m")
            } else {
                write!(buf, "\x1B[24m")
            };
            let _ = if formatting.is_set(TextFormatting::Negative) {
                write!(buf, "\x1B[7m")
            } else {
                write!(buf, "\x1B[27m")
            };
        })
    }

    /// Set the foreground color
    pub fn set_foreground_color(color: TerminalColor) {
        Self::write_foreground_color(color)
    }

    fn write_foreground_color(color: TerminalColor) {
        Self::exec_terminal_sequence(|buf| {
            let _ = match color {
                TerminalColor::Black           => write!(buf, "\x1B[30m"),
                TerminalColor::DarkRed         => write!(buf, "\x1B[31m"),
                TerminalColor::DarkGreen       => write!(buf, "\x1B[32m"),
                TerminalColor::DarkYellow      => write!(buf, "\x1B[33m"),
                TerminalColor::DarkBlue        => write!(buf, "\x1B[34m"),
                TerminalColor::DarkMagenta     => write!(buf, "\x1B[35m"),
                TerminalColor::DarkCyan        => write!(buf, "\x1B[36m"),
                TerminalColor::DarkGray        => write!(buf, "\x1B[37m"),
                TerminalColor::Gray            => write!(buf, "\x1B[90m"),
                TerminalColor::Red             => write!(buf, "\x1B[91m"),
                TerminalColor::Green           => write!(buf, "\x1B[92m"),
                TerminalColor::Yellow          => write!(buf, "\x1B[93m"),
                TerminalColor::Blue            => write!(buf, "\x1B[94m"),
                TerminalColor::Magenta         => write!(buf, "\x1B[95m"),
                TerminalColor::Cyan            => write!(buf, "\x1B[96m"),
                TerminalColor::White           => write!(buf, "\x1B[97m"),
                TerminalColor::Custom(r, g, b) => write!(buf, "\x1B[38;2;{r};{g};{b}m"),
            };
        })
    }

    /// Set the background color
    pub fn set_background_color(color: TerminalColor) {
        Self::write_background_color(color)
    }

    fn write_background_color(color: TerminalColor) {
        Self::exec_terminal_sequence(|buf| {
            let _ = match color {
                TerminalColor::Black           => write!(buf, "\x1B[40m"),
                TerminalColor::DarkRed         => write!(buf, "\x1B[41m"),
                TerminalColor::DarkGreen       => write!(buf, "\x1B[42m"),
                TerminalColor::DarkYellow      => write!(buf, "\x1B[43m"),
                TerminalColor::DarkBlue        => write!(buf, "\x1B[44m"),
                TerminalColor::DarkMagenta     => write!(buf, "\x1B[45m"),
                TerminalColor::DarkCyan        => write!(buf, "\x1B[46m"),
                TerminalColor::DarkGray        => write!(buf, "\x1B[47m"),
                TerminalColor::Gray            => write!(buf, "\x1B[100m"),
                TerminalColor::Red             => write!(buf, "\x1B[101m"),
                TerminalColor::Green           => write!(buf, "\x1B[102m"),
                TerminalColor::Yellow          => write!(buf, "\x1B[103m"),
                TerminalColor::Blue            => write!(buf, "\x1B[104m"),
                TerminalColor::Magenta         => write!(buf, "\x1B[105m"),
                TerminalColor::Cyan            => write!(buf, "\x1B[106m"),
                TerminalColor::White           => write!(buf, "\x1B[107m"),
                TerminalColor::Custom(r, g, b) => write!(buf, "\x1B[48;2;{r};{g};{b}m"),
            };
        })
    }

    /// Write a terminal sequence to the terminal, the sequence is written via the supplied `write_sequence` function
    pub fn exec_terminal_sequence<F>(write_sequence: F)
        where F : FnOnce(&mut DynArray<u8>)
    {
        let _scoped_alloc = ScopedAlloc::new(UseAlloc::TlsTemp);

        let mut buffer = DynArray::with_capacity(32);
        write_sequence(&mut buffer);
        let _ = unsafe { os_imp::Terminal::write(core::str::from_utf8_unchecked(&buffer)) };
    }

    /// Get the output handle
    pub fn get_output_handle(&self) -> TerminalIOHandle {
        os_imp::Terminal::get_output_handle()
    }
}