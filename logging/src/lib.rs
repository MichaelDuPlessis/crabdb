use chrono::Local;
use std::io::Write;
use std::{
    fmt::Display,
    sync::atomic::{AtomicU8, Ordering},
};

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
pub fn log(log_level: LogLevel, file: &str, line_num: u32, msg: std::fmt::Arguments) {
    // checking if the log level is correct
    if log_level as u8 <= LOG_LEVEL.load(Ordering::Relaxed) {
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
#[macro_export]
macro_rules! error {
    ($($args:tt)*) => {
        {
            // getting the file the error occured
            let file = file!();
            // getting the line number
            let line_num = line!();
            $crate::log($crate::LogLevel::Error, file, line_num, std::format_args!($($args)*));
        }
    };
}

/// Log a warning
#[macro_export]
macro_rules! warn {
    ($($args:tt)*) => {
        {
            // getting the file the error occured
            let file = file!();
            // getting the line number
            let line_num = line!();
            $crate::log($crate::LogLevel::Warn, file, line_num, std::format_args!($($args)*));
        }
    };
}

/// Log a debug
#[macro_export]
macro_rules! debug {
    ($($args:tt)*) => {
        {
            // getting the file the error occured
            let file = file!();
            // getting the line number
            let line_num = line!();
            $crate::log($crate::LogLevel::Debug, file, line_num, std::format_args!($($args)*));
        }
    };
}

/// Log a info
#[macro_export]
macro_rules! info {
    ($($args:tt)*) => {
        {
            // getting the file the error occured
            let file = file!();
            // getting the line number
            let line_num = line!();
            $crate::log($crate::LogLevel::Info, file, line_num, std::format_args!($($args)*));
        }
    };
}

/// Log a trace
#[macro_export]
macro_rules! trace {
    ($($args:tt)*) => {
        {
            // getting the file the error occured
            let file = file!();
            // getting the line number
            let line_num = line!();
            $crate::log($crate::LogLevel::Trace, file, line_num, std::format_args!($($args)*));
        }
    };
}
