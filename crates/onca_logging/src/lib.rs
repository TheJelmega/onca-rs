use core::fmt::{Display, Arguments};

use onca_core::{
    prelude::*,
    io::{self, prelude::*},
    sync::Mutex,
    mem::HeapPtr,
    time::TimeStamp,
};
use onca_terminal::{Terminal, TerminalColor, TextFormatting};

pub const LOGGER : Mutex<Logger> = Mutex::new(Logger::const_new());

/// Logging level
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum LogLevel {
    /// Severe error: will probably result in a crash
    Severe,
    /// Error: may not result in a crash
    Error,
    /// Warning: While not as bad as an error, it may result to something like a performance regression
    Warning,
    /// General info
    Info,
    /// Verbose info
    Verbose,
    /// Debug info (includes verbose info)
    Debug,
}

impl LogLevel {
    fn get_terminal_color_and_fromatting(&self) -> (TerminalColor, TerminalColor, TextFormatting) {
        match self {
            LogLevel::Severe  => (TerminalColor::Black , TerminalColor::DarkRed, TextFormatting::Bold),
            LogLevel::Error   => (TerminalColor::Red   , TerminalColor::Black  , TextFormatting::None),
            LogLevel::Warning => (TerminalColor::Yellow, TerminalColor::Black  , TextFormatting::None),
            LogLevel::Info    => (TerminalColor::Gray  , TerminalColor::Black  , TextFormatting::None),
            LogLevel::Verbose => (TerminalColor::Gray  , TerminalColor::Black  , TextFormatting::None),
            LogLevel::Debug   => (TerminalColor::Blue  , TerminalColor::Black  , TextFormatting::None),
        }
    }
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Severe => f.write_str("SEVERE"),
            LogLevel::Error => f.write_str("ERROR"),
            LogLevel::Warning => f.write_str("WARNING"),
            LogLevel::Info => f.write_str("INFO"),
            LogLevel::Verbose => f.write_str("VERBOSE"),
            LogLevel::Debug => f.write_str("DEBUG"),
        }
    }
}

/// Log category
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LogCategory {
    category     : &'static str,
    sub_category : Option<&'static str>
}

impl LogCategory {
    pub const fn new(name: &'static str) -> Self {
        Self { category: name, sub_category: None }
    }

    pub const fn new_with_sub(name: &'static str, sub_name: &'static str) -> Self {
        Self { category: name, sub_category: Some(sub_name) }
    }
}

impl Display for LogCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.sub_category {
            Some(sub) => f.write_fmt(format_args!("{}({sub})", self.category)),
            None => f.write_fmt(format_args!("{}", self.category)),
        }
    }
}

/// Additional info about where the log occured
pub struct LogLocation {
    file : &'static str,
    line : u32,
    func : &'static str,
    time : TimeStamp,
}

impl LogLocation {
    /// Creates a new log location
    pub const fn new(file: &'static str, line: u32, func: &'static str, time: TimeStamp) -> Self {
        Self { file, line, func, time }
    }

    /// Get the file name where the log occured
    pub const fn file(&self) -> &str {
        self.file
    }

    /// Get the line where the log occurred
    pub const fn line(&self) -> u32 {
        self.line
    }

    /// Get the function where the log occurred
    pub const fn function(&self) -> &str {
        self.func
    }

    /// Get the timestamp when the log occurred
    pub const fn timestamp(&self) -> TimeStamp {
        self.time
    }
}

struct LogLocationFormatter<'a> {
    loc   : &'a LogLocation,
    level : LogLevel
}

impl<'a> LogLocationFormatter<'a> {
    fn new(loc: &'a LogLocation, level: LogLevel) -> Self {
        Self { loc, level }
    }
}

impl<'a> Display for LogLocationFormatter<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.level {
            LogLevel::Severe => f.write_fmt(format_args!("({}:{}: {})", self.loc.file(), self.loc.line(), self.loc.function())),
            LogLevel::Error => f.write_fmt(format_args!("({}:{}: {})", self.loc.file(), self.loc.line(), self.loc.function())),
            LogLevel::Warning => Ok(()),
            LogLevel::Info => Ok(()),
            LogLevel::Verbose => Ok(()),
            LogLevel::Debug => f.write_fmt(format_args!("({}:{}: {})", self.loc.file(), self.loc.line(), self.loc.function())),
        }
    }
}

pub fn get_func_name<F>(_: F) -> &'static str {
    core::any::type_name::<F>()
}

// TODO: Find a better way to get the function name
#[macro_export]
macro_rules! log_location {
    () => {
        $crate::LogLocation::new(file!(), line!(), "", onca_core::time::get_timestamp())
    };
    ($func: expr) => {
        $crate::LogLocation::new(file!(), line!(), $crate::get_func_name($func), onca_core::time::get_timestamp())
    };
}

/// Logger
/// 
/// Supports up to 8 writers, e.g. terminal, file, in-game console, external tool, etc
pub struct Logger {
    max_level : LogLevel,
    writers   : [Option<HeapPtr<dyn io::Write>>; 8],
}

impl Logger {
    pub const fn const_new() -> Logger {
        /// Workaround, as you can only initialize an array of `None`s, if it fulfills `Option<T> where T: Copy`
        const NONE: Option<HeapPtr<dyn io::Write>> = None;
        Self { 
            max_level: LogLevel::Debug,
            writers: [NONE; 8],
        }
    }

    /// Set the maximum log level (severe == lowest, debug == highest)
    pub fn set_max_level(&mut self, level: LogLevel) {
        self.max_level = level;
    }

    /// Add a writer. 
    /// 
    /// Returns `Ok(index)` if space was available. This index can be used to remove the writer later on.
    /// 
    /// Otherwise returns an `Err` with the provided writer
    pub fn add_writer(&mut self, writer: HeapPtr<dyn io::Write>) -> Result<usize, HeapPtr<dyn io::Write>> {
        let mut index = None;
        for i in 0..8 {
            if let None = self.writers[i] {
                index = Some(i);
                break;
            }
        }

        match index {
            Some(idx) => {
                self.writers[idx] = Some(writer);
                Ok(idx)
            },
            None => Err(writer),
        }
    }

    /// Remove a writer from the logger
    pub fn remove_writer(&mut self, index: usize) -> Option<HeapPtr<dyn io::Write>> {
        core::mem::replace(&mut self.writers[index], None)
    }

    /// Log a message to the console
    pub fn log(&mut self, category: LogCategory, level: LogLevel, loc: LogLocation, text: &str) {
        if level as u8 <= self.max_level as u8 {
            let _scoped_alloc = ScopedAlloc::new(UseAlloc::TlsTemp);

            let mut formatted = String::new();

            let loc_formatter = LogLocationFormatter::new(&loc, level);
            let timestamp = loc.timestamp();
            let _ = write!(&mut formatted, "{timestamp} [{category}] {level}{loc_formatter}: {text}");

            self.write_message(&formatted, level);
        }
    }

    pub fn log_fmt(&mut self, category: LogCategory, level: LogLevel, loc: LogLocation, format: Arguments) {
        if level as u8 <= self.max_level as u8 {
            let _scoped_alloc = ScopedAlloc::new(UseAlloc::TlsTemp);

            let mut formatted = String::new();

            let loc_formatter = LogLocationFormatter::new(&loc, level);
            let timestamp = loc.timestamp();
            let _ = write!(&mut formatted, "{timestamp} [{category}] {level}{loc_formatter}: ");
            formatted = formatted.replace('\\', "/");
            let _ = write!(&mut formatted, "{format}\n");

            self.write_message(&formatted, level);
        }
    }

    fn write_message(&mut self, message: &str, level: LogLevel) {
        let (fore, back, formatting) = level.get_terminal_color_and_fromatting();
        let _ = Terminal::write_with(message, fore, back, formatting);

        for writer in &mut self.writers {
            if let Some(writer) = writer {
                let _ = writer.write(message.as_bytes());
            }
        }
    }
}

#[macro_export]
macro_rules! log {
    ($category:expr, $level:expr, $func:expr, $text:expr) => {
        $crate::LOGGER.lock().log_fmt($category, $level, $crate::log_location!($func), format_args!($text));
    };
    ($category:expr, $level:expr, $func:expr, $format:expr, $($arg:expr),*) => {
        $crate::LOGGER.lock().log_fmt($category, $level, $crate::log_location!($func), format_args!($format, $($arg),*));
    };
}

#[macro_export]
macro_rules! log_severe {
    ($category:expr, $func:expr, $text:expr) => {
        $crate::LOGGER.lock().log_fmt($category, $crate::LogLevel::Severe, $crate::log_location!($func), format_args!($text));
    };
    ($category:expr, $func:expr, $format:expr, $($arg:expr),*) => {
        $crate::LOGGER.lock().log_fmt($category, $crate::LogLevel::Severe, $crate::log_location!($func), format_args!($format, $($arg),*));
    };
}

#[macro_export]
macro_rules! log_error {
    ($category:expr, $func:expr, $text:expr) => {
        $crate::LOGGER.lock().log_fmt($category, $crate::LogLevel::Error, $crate::log_location!($func), format_args!($text));
    };
    ($category:expr, $func:expr, $format:expr, $($arg:expr),*) => {
        $crate::LOGGER.lock().log_fmt($category, $crate::LogLevel::Error, $crate::log_location!($func), format_args!($format, $($arg),*));
    };
}

#[macro_export]
macro_rules! log_warning {
    ($category:expr, $text:expr) => {
        $crate::LOGGER.lock().log_fmt($category, $crate::LogLevel::Warning, $crate::log_location!(), format_args!($text));
    };
    ($category:expr, $format:expr, $($arg:expr),*) => {
        $crate::LOGGER.lock().log_fmt($category, $crate::LogLevel::Warning, $crate::log_location!(), format_args!($format, $($arg),*));
    };
}

#[macro_export]
macro_rules! log_info {
    ($category:expr, $text:expr) => {
        $crate::LOGGER.lock().log_fmt($category, $crate::LogLevel::Info, $crate::log_location!(), format_args!($text));
    };
    ($category:expr, $format:expr, $($arg:expr),*) => {
        $crate::LOGGER.lock().log_fmt($category, $crate::LogLevel::Info, $crate::log_location!(), format_args!($format, $($arg),*));
    };
}

#[macro_export]
macro_rules! log_verbose {
    ($category:expr, $text:expr) => {
        $crate::LOGGER.lock().log_fmt($category, $crate::LogLevel::Verbose, $crate::log_location!(), format_args!($text));
    };
    ($category:expr, $format:expr, $($arg:expr),*) => {
        $crate::LOGGER.lock().log_fmt($category, $crate::LogLevel::Verbose, $crate::log_location!(), format_args!($format, $($arg),*));
    };
}

#[macro_export]
macro_rules! log_debug {
    ($category:expr, $func:expr, $text:expr) => {
        $crate::LOGGER.lock().log_fmt($category, $crate::LogLevel::Debug, $crate::log_location!($func), format_args!($text));
    };
    ($category:expr, $func:expr, $format:expr, $($arg:expr),*) => {
        $crate::LOGGER.lock().log_fmt($category, $crate::LogLevel::Debug, $crate::log_location!($func), format_args!($format, $($arg),*));
    };
}

pub fn test() {
    let category = LogCategory::new("cat");
    let other = 1u32;

    log_severe!(category, test, "Something happened");
    log_severe!(category, test, "Something happened in {other}");
}

