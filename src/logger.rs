// src/logger.rs
use std::fmt::Arguments;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};

/// Log levels supported by the logger.
#[derive(Debug)]
pub enum LogLevel {
    /// Fatal log level, indicating a critical error.
    Fatal,
    /// Warning log level, indicating a potential issue.
    Warning,
    /// Info log level, indicating general information.
    Info,
    /// Debug log level, indicating detailed debugging information.
    Debug,
}

/// A simple logger that writes log entries to a specified file.
/// Each log entry includes a log level, timestamp, source filename, line number, and message.
/// The logger supports four log levels: FATAL, WARNING, INFO, and DEBUG.
/// # Example
/// ```rust
///  use judger::Logger;
///  use std::fmt::Arguments;
///  let mut logger = Logger::new("app.log").expect("Failed to create logger");
///  logger.write(2, file!(), line!(), format_args!("This is an info message")).expect("Failed to write log");
/// ```
/// # Errors
/// The `new` method returns an `io::Error` if the log file cannot be created or opened.
/// The `write` method returns an `io::Error` if writing to the log file fails or if an invalid log level is provided.
pub struct Logger {
    log_fp: File,
}

impl Logger {
    /// Creates a new logger that writes to the specified file.
    /// If the file does not exist, it will be created. If it exists, logs will be appended.
    /// # Errors
    /// Returns an `io::Error` if the file cannot be created or opened.
    /// # Example
    /// ```rust
    ///  use judger::Logger;
    ///  let logger = Logger::new("app.log").expect("Failed to create logger");
    /// ```
    /// # Arguments
    /// * `filename` - The path to the log file.
    /// # Returns
    /// A `Result` containing the `Logger` or an `io::Error`.
    pub fn new(filename: &str) -> io::Result<Logger> {
        let log_fp = OpenOptions::new()
            .create(true)
            .append(true)
            .open(filename)?;
        Ok(Logger { log_fp })
    }

    /// Writes a log entry to the log file with the specified level, source filename, line number, and message.
    /// # Errors
    /// Returns an `io::Error` if writing to the log file fails or if an invalid log level is provided.
    /// # Example
    /// ```rust
    ///  use judger::Logger;
    ///  use std::fmt::Arguments;
    ///  use judger::LogLevel;
    ///  let mut logger = Logger::new("app.log").expect("Failed to create logger");
    ///  logger.write(LogLevel::Info, file!(), line!(), format_args!("This is an info message")).expect("Failed to write log");
    /// ```
    /// # Arguments
    /// * `level` - The log level of the entry.
    /// * `source_filename` - The source filename where the log entry is generated.
    /// * `line` - The line number in the source file.
    /// * `args` - The formatted message to log.
    /// # Returns
    /// A `Result` indicating success or containing an `io::Error`.
    pub fn write(
        &mut self,
        level: LogLevel,
        source_filename: &str,
        line: u32,
        args: Arguments,
    ) -> io::Result<()> {
        log_write_fmt(&mut self.log_fp, level, source_filename, line, args)
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        let _ = self.log_fp.flush();
    }
}

fn log_write_fmt(
    log_fp: &mut File,
    level: LogLevel,
    source_filename: &str,
    line: u32,
    args: Arguments,
) -> io::Result<()> {
    // Timestamp as seconds since epoch (simple cross-platform fallback)
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let datetime = format!("{}", now);

    let mut msg_buf = String::new();
    std::fmt::write(&mut msg_buf, args).ok();

    let entry = format!(
        "{:?} [{}] [{}:{}] {}\n",
        level, datetime, source_filename, line, msg_buf
    );

    // Write atomically to the file (append)
    log_fp.write_all(entry.as_bytes())
}
