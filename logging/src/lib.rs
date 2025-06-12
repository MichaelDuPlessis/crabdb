use std::io::Write;
use std::{
    fmt::Display,
    sync::atomic::{AtomicU8, Ordering},
};

use chrono::Local;

/// The available log levels
/// A higher level will log itself an all log levels below it
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum LogLevel {
    None,
    Error,
    Warn,
    Debug,
    Info,
    Trace,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                LogLevel::None => "None",
                LogLevel::Error => "Error",
                LogLevel::Warn => "Warn",
                LogLevel::Debug => "Debug",
                LogLevel::Info => "Info",
                LogLevel::Trace => "Trace",
            }
        )
    }
}

/// The log level maintained internally
static LOG_LEVEL: AtomicU8 = AtomicU8::new(LogLevel::None as u8);

/// Intializes the logger
pub fn init_logger(log_level: LogLevel) {
    LOG_LEVEL.store(log_level as u8, Ordering::Relaxed);
}

/// logs a message
fn log(log_level: LogLevel, msg: std::fmt::Arguments) {
    // checking if the log level is correct
    if log_level as u8 <= LOG_LEVEL.load(Ordering::Relaxed) {
        // getting the file the error occured
        let file = file!();
        // getting the line number
        let line_num = line!();
        // getting the current time
        let now = Local::now();
        // getting the timestamp
        let timestamp = now.format("%Y-%m-%d %H:%M:%S%.3f");

        // writing log output to stderr
        writeln!(
            std::io::stderr(),
            "{} [{}] {}:{}: {}",
            timestamp,
            log_level,
            file,
            line_num,
            msg
        )
        .unwrap(); //  If the logger fails something really bad is probably happening
    }
}

/// Log an error
macro_rules! error {
    ($($args:tt)*) => {
        $crate::log($crate::LogLevel::Error, std::format_args!($($args)*));
    };
}

/// Log a warning
macro_rules! warn {
    ($($args:tt)*) => {
        $crate::log($crate::LogLevel::Error, std::format_args!($($args)*));
    };
}

/// Log a debug
macro_rules! debug {
    ($($args:tt)*) => {
        $crate::log($crate::LogLevel::Error, std::format_args!($($args)*));
    };
}

/// Log a info
macro_rules! info {
    ($($args:tt)*) => {
        $crate::log($crate::LogLevel::Error, std::format_args!($($args)*));
    };
}

/// Log a trace
macro_rules! trace {
    ($($args:tt)*) => {
        $crate::log($crate::LogLevel::Error, std::format_args!($($args)*));
    };
}
