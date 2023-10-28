use std::fmt::Write;
use onca_common::{
    prelude::*,
    io
};
use onca_common_macros::flags;



/// Terminal color (can be both the front and back color)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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

impl TerminalColor {
    /// Get an escape code representing the terminal foreground color
    pub fn fore_to_escape_code(&self) -> String {
        match self {
            TerminalColor::Black           => "\x1B[30m".to_string(),
            TerminalColor::DarkRed         => "\x1B[31m".to_string(),
            TerminalColor::DarkGreen       => "\x1B[32m".to_string(),
            TerminalColor::DarkYellow      => "\x1B[33m".to_string(),
            TerminalColor::DarkBlue        => "\x1B[34m".to_string(),
            TerminalColor::DarkMagenta     => "\x1B[35m".to_string(),
            TerminalColor::DarkCyan        => "\x1B[36m".to_string(),
            TerminalColor::DarkGray        => "\x1B[37m".to_string(),
            TerminalColor::Gray            => "\x1B[90m".to_string(),
            TerminalColor::Red             => "\x1B[91m".to_string(),
            TerminalColor::Green           => "\x1B[92m".to_string(),
            TerminalColor::Yellow          => "\x1B[93m".to_string(),
            TerminalColor::Blue            => "\x1B[94m".to_string(),
            TerminalColor::Magenta         => "\x1B[95m".to_string(),
            TerminalColor::Cyan            => "\x1B[96m".to_string(),
            TerminalColor::White           => "\x1B[97m".to_string(),
            TerminalColor::Custom(r, g, b) => format!("\x1B[38;2;{r};{g};{b}m"),
        }
    }

    /// Write the terminal foreground color escape code to an `io::Write`
    pub fn write_fore_escape_code(&self, writer: &mut dyn io::Write) -> io::Result<()> {
        match self {
            TerminalColor::Black           => write!(writer, "\x1B[30m"),
            TerminalColor::DarkRed         => write!(writer, "\x1B[31m"),
            TerminalColor::DarkGreen       => write!(writer, "\x1B[32m"),
            TerminalColor::DarkYellow      => write!(writer, "\x1B[33m"),
            TerminalColor::DarkBlue        => write!(writer, "\x1B[34m"),
            TerminalColor::DarkMagenta     => write!(writer, "\x1B[35m"),
            TerminalColor::DarkCyan        => write!(writer, "\x1B[36m"),
            TerminalColor::DarkGray        => write!(writer, "\x1B[37m"),
            TerminalColor::Gray            => write!(writer, "\x1B[90m"),
            TerminalColor::Red             => write!(writer, "\x1B[91m"),
            TerminalColor::Green           => write!(writer, "\x1B[92m"),
            TerminalColor::Yellow          => write!(writer, "\x1B[93m"),
            TerminalColor::Blue            => write!(writer, "\x1B[94m"),
            TerminalColor::Magenta         => write!(writer, "\x1B[95m"),
            TerminalColor::Cyan            => write!(writer, "\x1B[96m"),
            TerminalColor::White           => write!(writer, "\x1B[97m"),
            TerminalColor::Custom(r, g, b) => write!(writer, "\x1B[38;2;{r};{g};{b}m"),
        }
    }
    /// Get an escape code representing the terminal foreground color
    pub fn back_to_escape_code(&self) -> String {
        match self {
            TerminalColor::Black           => "\x1B[40m".to_string(),
            TerminalColor::DarkRed         => "\x1B[41m".to_string(),
            TerminalColor::DarkGreen       => "\x1B[42m".to_string(),
            TerminalColor::DarkYellow      => "\x1B[43m".to_string(),
            TerminalColor::DarkBlue        => "\x1B[44m".to_string(),
            TerminalColor::DarkMagenta     => "\x1B[45m".to_string(),
            TerminalColor::DarkCyan        => "\x1B[46m".to_string(),
            TerminalColor::DarkGray        => "\x1B[47m".to_string(),
            TerminalColor::Gray            => "\x1B[100m".to_string(),
            TerminalColor::Red             => "\x1B[101m".to_string(),
            TerminalColor::Green           => "\x1B[102m".to_string(),
            TerminalColor::Yellow          => "\x1B[103m".to_string(),
            TerminalColor::Blue            => "\x1B[104m".to_string(),
            TerminalColor::Magenta         => "\x1B[105m".to_string(),
            TerminalColor::Cyan            => "\x1B[106m".to_string(),
            TerminalColor::White           => "\x1B[107m".to_string(),
            TerminalColor::Custom(r, g, b) => format!("\x1B[48;2;{r};{g};{b}m"),
        }
    }

    /// Write the terminal foreground color escape code to an `io::Write`
    pub fn write_back_escape_code(&self, writer: &mut dyn io::Write) -> io::Result<()> {
        match self {
            TerminalColor::Black           => write!(writer, "\x1B[40m"),
            TerminalColor::DarkRed         => write!(writer, "\x1B[41m"),
            TerminalColor::DarkGreen       => write!(writer, "\x1B[42m"),
            TerminalColor::DarkYellow      => write!(writer, "\x1B[43m"),
            TerminalColor::DarkBlue        => write!(writer, "\x1B[44m"),
            TerminalColor::DarkMagenta     => write!(writer, "\x1B[45m"),
            TerminalColor::DarkCyan        => write!(writer, "\x1B[46m"),
            TerminalColor::DarkGray        => write!(writer, "\x1B[47m"),
            TerminalColor::Gray            => write!(writer, "\x1B[100m"),
            TerminalColor::Red             => write!(writer, "\x1B[101m"),
            TerminalColor::Green           => write!(writer, "\x1B[102m"),
            TerminalColor::Yellow          => write!(writer, "\x1B[103m"),
            TerminalColor::Blue            => write!(writer, "\x1B[104m"),
            TerminalColor::Magenta         => write!(writer, "\x1B[105m"),
            TerminalColor::Cyan            => write!(writer, "\x1B[106m"),
            TerminalColor::White           => write!(writer, "\x1B[107m"),
            TerminalColor::Custom(r, g, b) => write!(writer, "\x1B[48;2;{r};{g};{b}m"),
        }
    }
}


/// Cursor move direction
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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

impl CursorMove {
    /// Get an escape code representing the terminal cursor move
    pub fn to_escape_code(&self) -> String {
        match self {
            CursorMove::Up(n)          => format!("\0x1B[{n}A"),
            CursorMove::Down(n)        => format!("\0x1B[{n}B"),
            CursorMove::Forward(n)     => format!("\0x1B[{n}C"),
            CursorMove::Backward(n)    => format!("\0x1B[{n}D"),
            CursorMove::Position(x, y) => format!("\0x1B[{y};{x}H"),
        }
    }

    /// Write the terminal cursor move escape code to an `io::Write`
    pub fn write_escape_code(&self, writer: &mut dyn io::Write) -> io::Result<()> {
        match self {
            CursorMove::Up(n)          => write!(writer, "\0x1B[{n}A"),
            CursorMove::Down(n)        => write!(writer, "\0x1B[{n}B"),
            CursorMove::Forward(n)     => write!(writer, "\0x1B[{n}C"),
            CursorMove::Backward(n)    => write!(writer, "\0x1B[{n}D"),
            CursorMove::Position(x, y) => write!(writer, "\0x1B[{y};{x}H"),
        }
    }
}

/// Terminal cursor shape
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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

impl CursorShape {
    /// Get an escape code representing the terminal cursor move
    pub fn to_escape_code(&self) -> String {
        match self {
            CursorShape::User              => format!("\x1B[0 q"),
            CursorShape::BlinkingBlock     => format!("\x1B[1 q"),
            CursorShape::SteadyBlock       => format!("\x1B[2 q"),
            CursorShape::BlinkingUnderline => format!("\x1B[3 q"),
            CursorShape::SteadyUnderline   => format!("\x1B[4 q"),
            CursorShape::BlinkingBar       => format!("\x1B[5 q"),
            CursorShape::SteadyBar         => format!("\x1B[6 q"),
        }
    }

    /// Write the terminal cursor move escape code to an `io::Write`
    pub fn write_escape_code(&self, writer: &mut dyn io::Write) -> io::Result<()> {
        match self {
            CursorShape::User              => write!(writer, "\x1B[0 q"),
            CursorShape::BlinkingBlock     => write!(writer, "\x1B[1 q"),
            CursorShape::SteadyBlock       => write!(writer, "\x1B[2 q"),
            CursorShape::BlinkingUnderline => write!(writer, "\x1B[3 q"),
            CursorShape::SteadyUnderline   => write!(writer, "\x1B[4 q"),
            CursorShape::BlinkingBar       => write!(writer, "\x1B[5 q"),
            CursorShape::SteadyBar         => write!(writer, "\x1B[6 q"),
        }
    }
}

/// Cursor action
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CursorAction {
    /// Is the cursor shown
    Show(bool),
    /// Does the cursor blink
    Blink(bool),
}

impl CursorAction {
    /// Get an escape code representing the terminal cursor blink
    pub fn to_escape_code(self) -> &'static str {
        match self {
            CursorAction::Show(show) => if show {
                "\x1B[?25h"
            } else {
                "\x1B[?25l"
            },
            CursorAction::Blink(blink) => if blink {
                "\x1B[?12h"
            } else {
                "\x1B[?12l"
            },
        }
    }

    /// Write the terminal cursor blink escape code to an `io::Write`
    pub fn write_escape_code(self, writer: &mut dyn io::Write) -> io::Result<()> {
        match self {
            CursorAction::Show(show) => if show {
                write!(writer, "\x1B[?25h")
            } else {
                write!(writer, "\x1B[?25l")
            },
            CursorAction::Blink(blink) => if blink {
                write!(writer, "\x1B[?12h")
            } else {
                write!(writer, "\x1B[?12l")
            },
        }
    }
}

/// Terminal text modification
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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


impl TextMod {
    /// Get an escape code representing the terminal text  mod
    pub fn to_escape_code(&self) -> String {
        match self {
            TextMod::Insert(n)     => format!("\x1B[{n}@"),
            TextMod::Delete(n)     => format!("\x1B[{n}P"),
            TextMod::Erase(n)      => format!("\x1B[{n}X"),
            TextMod::InsertLine(n) => format!("\x1B[{n}L"),
            TextMod::DeleteLine(n) => format!("\x1B[{n}M"),
        }
    }

    /// Write the terminal text mod escape code to an `io::Write`
    pub fn write_escape_code(&self, writer: &mut dyn io::Write) -> io::Result<()> {
        match self {
            TextMod::Insert(n)     => write!(writer, "\x1B[{n}@"),
            TextMod::Delete(n)     => write!(writer, "\x1B[{n}P"),
            TextMod::Erase(n)      => write!(writer, "\x1B[{n}X"),
            TextMod::InsertLine(n) => write!(writer, "\x1B[{n}L"),
            TextMod::DeleteLine(n) => write!(writer, "\x1B[{n}M"),
        }
    }
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

impl TextFormatting {
    /// Get an escape code representing the terminal text formatting
    pub fn to_escape_code(&self) -> String {
        let mut string = String::new();
        if self.contains(TextFormatting::Bold) {
            _ = write!(string, "\x1B[1m")
        } else {
            _ = write!(string, "\x1B[22m")
        };
        if self.contains(TextFormatting::Underline) {
            _ = write!(string, "\x1B[4m")
        } else {
            _ = write!(string, "\x1B[24m")
        };
        if self.contains(TextFormatting::Negative) {
            _ = write!(string, "\x1B[7m")
        } else {
            _ = write!(string, "\x1B[27m")
        };
        string
    }

    /// Write the terminal text formatting escape code to an `io::Write`
    pub fn write_escape_code(&self, writer: &mut dyn io::Write) -> io::Result<()> {
        if self.contains(TextFormatting::Bold) {
            write!(writer, "\x1B[1m")
        } else {
            write!(writer, "\x1B[22m")
        }?;
        if self.contains(TextFormatting::Underline) {
            write!(writer, "\x1B[4m")
        } else {
            write!(writer, "\x1B[24m")
        }?;
        if self.contains(TextFormatting::Negative) {
            write!(writer, "\x1B[7m")
        } else {
            write!(writer, "\x1B[27m")
        }
    }
}

/// Cursor blinking
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CursorBlink {
    /// Cursor blinks
    On,
    /// Cursor doesn't blink
    Off
}

impl CursorBlink {
    /// Get an escape code representing the terminal cursor blink
    pub fn to_escape_code(&self) -> &str {
        match self {
            CursorBlink::On  => "\x1B[?12h",
            CursorBlink::Off => "\x1B[?12l",
        }
    }

    /// Write the terminal cursor blink escape code to an `io::Write`
    pub fn write_escape_code(&self, writer: &mut dyn io::Write) -> io::Result<()> {
        match self {
            CursorBlink::On  => write!(writer, "\x1B[?12h"),
            CursorBlink::Off => write!(writer, "\x1B[?12l"),
        }
    }
}
