// src/logger.rs
use std::fs::OpenOptions;
use std::fs::File;
use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use std::fmt::Arguments;

const LOG_LEVEL_NOTE: [&str; 4] = ["FATAL", "WARNING", "INFO", "DEBUG"];

pub struct Logger {
    log_fp: File,
}

impl Logger {
    pub fn new(filename: &str) -> io::Result<Logger> {
        let log_fp = OpenOptions::new().create(true).append(true).open(filename)?;
        Ok(Logger { log_fp })
    }

    pub fn write(&mut self, level: usize, source_filename: &str, line: u32, args: Arguments) -> io::Result<()> {
        log_write_fmt(&mut self.log_fp, level, source_filename, line, args)
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        let _ = self.log_fp.flush();
    }
}

fn log_write_fmt(log_fp: &mut File, level: usize, source_filename: &str, line: u32, args: Arguments) -> io::Result<()> {
    if level >= LOG_LEVEL_NOTE.len() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid log level"));
    }

    // Timestamp as seconds since epoch (simple cross-platform fallback)
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let datetime = format!("{}", now);

    let mut msg_buf = String::new();
    std::fmt::write(&mut msg_buf, args).ok();

    let entry = format!(
        "{} [{}] [{}:{}]{}\n",
        LOG_LEVEL_NOTE[level],
        datetime,
        source_filename,
        line,
        msg_buf
    );

    // Write atomically to the file (append)
    log_fp.write_all(entry.as_bytes())
}

