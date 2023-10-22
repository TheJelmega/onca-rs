#![feature(local_key_cell_methods)]

use core::{
    fmt::{Display, Arguments},
    ptr::null,
    sync::atomic::{AtomicU8, self},
    cell::RefCell
};
use std::fmt::Write;
use onca_core::{
    prelude::*,
    io::{self, prelude::*},
    sync::{RwLock, Mutex},
    time::TimeStamp,
};
use onca_terminal::Terminal;

struct LoggerPtr(*const Logger);

unsafe impl Send for LoggerPtr {}
unsafe impl Sync for LoggerPtr {}

// The RwLock does not actually guard the value, but is used to have a global set-able state that is `Sync`
static LOGGER : RwLock<LoggerPtr> = RwLock::new(LoggerPtr(null()));

pub fn set_logger(logger: &Logger) {
    *LOGGER.write() = LoggerPtr(logger as *const _);
}

pub fn get_logger() -> &'static Logger {
    let ptr = LOGGER.read().0;
    assert!(ptr != null(), "Logger was not set");
    unsafe { &*ptr }
}

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

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Severe  => f.write_str("\x1B[1m\x1B[41m\x1B[30m[SEVERE ]\x1B[0m"),
            LogLevel::Error   => f.write_str(               "\x1B[91m[ERROR  ]\x1B[0m"),
            LogLevel::Warning => f.write_str(               "\x1B[93m[WARNING]\x1B[0m"),
            LogLevel::Info    => f.write_str(               "\x1B[37m[INFO   ]\x1B[0m"),
            LogLevel::Verbose => f.write_str(               "\x1B[90m[VERBOSE]\x1B[0m"),
            LogLevel::Debug   => f.write_str(               "\x1B[94m[DEBUG  ]\x1B[0m"),
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

pub struct LoggerState {
    writers:        [Option<Box<dyn io::Write>>; Self::MAX_WRITERS],
    cache:          Option<String>,
    writer_idx:     usize,
    always_flush:   bool,
    log_to_console: bool,
}

impl LoggerState {
    const MAX_WRITERS: usize = 8;
    const CACHE_FLUSH_LIMIT: usize = KiB(4);

    pub const fn new() -> Self {
        const NONE: Option<Box<dyn io::Write>> = None;

        // Cause the `Option` contians a `Box<T>`, the option is not Clone, so we need to manually build the array
        let writers = [
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ];

        Self {
            writers,
            cache: None,
            writer_idx: 0,
            always_flush: false,
            log_to_console: true,
        }
    }

    fn write_message(&mut self, message: &str) {
        scoped_alloc!(AllocId::Malloc);
        
        if self.cache.is_none() {
            self.cache = Some(String::with_capacity(Self::CACHE_FLUSH_LIMIT));
        }
        
        let cache = self.cache.as_mut().unwrap();
        cache.push_str(&message);
        self.flush_when_needed();
    }

    fn format_message(&mut self, fmt_args: Arguments) {
        scoped_alloc!(AllocId::Malloc);
        
        if self.cache.is_none() {
            self.cache = Some(String::with_capacity(Self::CACHE_FLUSH_LIMIT));
        }
        
        let cache = self.cache.as_mut().unwrap();
        _ = cache.write_fmt(fmt_args);

        self.flush_when_needed();
    }

    fn flush_when_needed(&mut self) {
        if self.always_flush || self.cache.as_ref().map_or(0, |cache| cache.len()) > Self::CACHE_FLUSH_LIMIT {
            self.flush();
        }
    }

    fn flush(&mut self) {
        if let Some(cache) = &mut self.cache {
            if self.log_to_console {
                _ = Terminal::write(&cache);
            }

            for writer in &mut self.writers {
                if let Some(writer) = writer {
                    _ = writer.write(cache.as_bytes());
                }
            }
            cache.clear();
        }
    }
}

/// Logger
/// 
/// Supports up to 8 writers, e.g. terminal, file, in-game console, external tool, etc
pub struct Logger {
    state: Mutex<LoggerState>,
    max_log_level: AtomicU8,
}

impl Logger {
    thread_local! {
        static FORMAT_CACHE: RefCell<Option<String>> = RefCell::new(None);
    }

    pub const fn new() -> Self {
        Self { 
            state: Mutex::new(LoggerState::new()),
            max_log_level: AtomicU8::new(LogLevel::Debug as u8),
        }
    }

    /// Set the maximum log level (severe == lowest, debug == highest)
    pub fn set_max_level(&self, level: LogLevel) {
        self.max_log_level.store(level as u8, atomic::Ordering::Relaxed)
    }

    /// Set whether the logger should flush after each write
    pub fn set_always_flush(&self, always_flush: bool) {
        self.state.lock().always_flush = always_flush;
    }

    /// Set whether the logger should log it's output to console
    pub fn set_log_to_console(&self, log_to_console: bool) {
        let mut state = self.state.lock();

        // Make sure to flush first, cause all messages before wanted/didn't want to be log to be written to console
        state.flush();
        state.log_to_console = log_to_console;
    }

    /// Add a writer. 
    /// 
    /// Returns `Ok(index)` if space was available. This index can be used to remove the writer later on.
    /// 
    /// Otherwise returns an `Err` with the provided writer
    pub fn add_writer(&self, writer: Box<dyn io::Write>) -> Result<usize, Box<dyn io::Write>> {
        let mut state = self.state.lock();

        let empty = state.writers.iter_mut().enumerate().find(|val| val.1.is_none());
        match empty {
            Some((id, slot)) => {
                *slot = Some(writer);
                Ok(id)
            },
            None => Err(writer),
        }
    }

    /// Remove a writer from the logger
    pub fn remove_writer(&self, index: usize) -> Option<Box<dyn io::Write>> {
        let mut state = self.state.lock();
        std::mem::replace(&mut state.writers[index], None)
    }

    /// Log a message to the console
    pub fn log(&self, category: LogCategory, level: LogLevel, loc: LogLocation, text: &str) {
        if level as u8 <= self.max_log_level.load(atomic::Ordering::Relaxed) {
            let loc_formatter = LogLocationFormatter::new(&loc, level);
            let timestamp = loc.timestamp();
            self.state.lock().format_message(format_args!("\x1B[38m{timestamp}\x1B[0m {level} [{category}] {loc_formatter}: {text}/n"));
        }
    }

    pub fn log_fmt(&self, category: LogCategory, level: LogLevel, loc: LogLocation, format: Arguments) {
        if level as u8 <= self.max_log_level.load(atomic::Ordering::Relaxed) as u8 {
            let loc_formatter = LogLocationFormatter::new(&loc, level);
            let timestamp = loc.timestamp();
            let mut state = self.state.lock();
            state.format_message(format_args!("\x1B[38m{timestamp}\x1B[0m {level} [{category}] {loc_formatter}: "));
            state.format_message(format);
            state.write_message("\n");
        }
    }

    pub fn flush(&self) {
        self.state.lock().flush()
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        self.flush();
    }
}

#[macro_export]
macro_rules! log {
    ($category:expr, $level:expr, $func:expr, $text:expr) => {
        $crate::get_logger().log_fmt($category, $level, $crate::log_location!($func), format_args!($text));
    };
    ($category:expr, $level:expr, $func:expr, $format:expr, $($arg:expr),*) => {
        $crate::get_logger().log_fmt($category, $level, $crate::log_location!($func), format_args!($format, $($arg),*));
    };
}

#[macro_export]
macro_rules! log_severe {
    ($category:expr, $func:expr, $text:expr) => {
        $crate::get_logger().log_fmt($category, $crate::LogLevel::Severe, $crate::log_location!($func), format_args!($text));
    };
    ($category:expr, $func:expr, $format:expr, $($arg:expr),*) => {
        $crate::get_logger().log_fmt($category, $crate::LogLevel::Severe, $crate::log_location!($func), format_args!($format, $($arg),*));
    };
}

#[macro_export]
macro_rules! log_error {
    ($category:expr, $func:expr, $text:expr) => {
        $crate::get_logger().log_fmt($category, $crate::LogLevel::Error, $crate::log_location!($func), format_args!($text));
    };
    ($category:expr, $func:expr, $format:expr, $($arg:expr),*) => {
        $crate::get_logger().log_fmt($category, $crate::LogLevel::Error, $crate::log_location!($func), format_args!($format, $($arg),*));
    };
}

#[macro_export]
macro_rules! log_warning {
    ($category:expr, $text:expr) => {
        $crate::get_logger().log_fmt($category, $crate::LogLevel::Warning, $crate::log_location!(), format_args!($text));
    };
    ($category:expr, $format:expr, $($arg:expr),*) => {
        $crate::get_logger().log_fmt($category, $crate::LogLevel::Warning, $crate::log_location!(), format_args!($format, $($arg),*));
    };
}

#[macro_export]
macro_rules! log_info {
    ($category:expr, $text:expr) => {
        $crate::get_logger().log_fmt($category, $crate::LogLevel::Info, $crate::log_location!(), format_args!($text));
    };
    ($category:expr, $format:expr, $($arg:expr),*) => {
        $crate::get_logger().log_fmt($category, $crate::LogLevel::Info, $crate::log_location!(), format_args!($format, $($arg),*));
    };
}

#[macro_export]
macro_rules! log_verbose {
    ($category:expr, $text:expr) => {
        $crate::get_logger().log_fmt($category, $crate::LogLevel::Verbose, $crate::log_location!(), format_args!($text));
    };
    ($category:expr, $format:expr, $($arg:expr),*) => {
        $crate::get_logger().log_fmt($category, $crate::LogLevel::Verbose, $crate::log_location!(), format_args!($format, $($arg),*));
    };
}

#[macro_export]
macro_rules! log_debug {
    ($category:expr, $func:expr, $text:expr) => {
        $crate::get_logger().log_fmt($category, $crate::LogLevel::Debug, $crate::log_location!($func), format_args!($text));
    };
    ($category:expr, $func:expr, $format:expr, $($arg:expr),*) => {
        $crate::get_logger().log_fmt($category, $crate::LogLevel::Debug, $crate::log_location!($func), format_args!($format, $($arg),*));
    };
}

pub fn test() {
    let category = LogCategory::new("cat");
    let other = 1u32;

    log_severe!(category, test, "Something happened");
    log_severe!(category, test, "Something happened in {other}");
}

