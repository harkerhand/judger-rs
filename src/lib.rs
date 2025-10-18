#![deny(missing_docs)]
//! A Rust library for sandboxed code execution and resource limitation.

mod child;
mod logger;
mod runner;
mod seccomp;
mod utils;

pub use crate::child::child_process;
pub use crate::logger::LogLevel;
pub use crate::logger::Logger;
pub use crate::runner::run;
pub use crate::seccomp::SeccompRuleName;
pub use crate::utils::ErrorCode;

/// Configuration for the judger.
#[derive(Debug)]
pub struct Config {
    /// Maximum CPU time in milliseconds (-1 for unlimited).
    pub max_cpu_time: i32,
    /// Maximum real time in milliseconds (-1 for unlimited).
    pub max_real_time: i32,
    /// Maximum memory in bytes (-1 for unlimited).
    pub max_memory: i64,
    /// Maximum stack size in bytes.
    pub max_stack: i64,
    /// Maximum number of processes (-1 for unlimited).
    pub max_process_number: i32,
    /// Maximum output size in bytes (-1 for unlimited).
    pub max_output_size: i64,
    /// If true, only check memory limit without enforcing it.
    pub memory_limit_check_only: bool,
    /// Path to the executable.
    pub exe_path: String,
    /// Path to the input file.
    pub input_path: String,
    /// Path to the output file.
    pub output_path: String,
    /// Path to the error file.
    pub error_path: String,
    /// Arguments to pass to the executable.
    pub args: Vec<String>,
    /// Environment variables for the executable.
    pub env: Vec<String>,
    /// Path to the log file.
    pub log_path: String,
    /// Name of the seccomp rule to apply.
    pub seccomp_rule_name: Option<SeccompRuleName>,
    /// User ID to run the process as.
    pub uid: u32,
    /// Group ID to run the process as.
    pub gid: u32,
}

impl Config {
    pub(crate) fn check(&self) -> bool {
        !((self.max_cpu_time < 1 && self.max_cpu_time != -1)
            || (self.max_real_time < 1 && self.max_real_time != -1)
            || (self.max_stack < 1)
            || (self.max_memory < 1 && self.max_memory != -1)
            || (self.max_process_number < 1 && self.max_process_number != -1)
            || (self.max_output_size < 1 && self.max_output_size != -1))
    }
}
