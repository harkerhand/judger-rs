#![deny(missing_docs)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
//! A Rust library for sandboxed code execution and resource limitation.
//! This library provides functionalities to run untrusted code with specified resource limits,
//! such as CPU time, memory usage, and process count. It also supports seccomp filtering for enhanced security.
//! # Features
//! - Configurable resource limits
//! - Seccomp filtering
//! - Detailed logging
//! - Error handling with specific error codes
//! # Example
//! ```rust
//!  use judger::{Config, SeccompRuleName, run};
//!  let config = Config {
//!     max_cpu_time: 1000,
//!     max_real_time: 2000,
//!     max_memory: 128 * 1024 * 1024,
//!     max_stack: 32 * 1024 * 1024,
//!     max_process_number: 1,
//!     max_output_size: 10000,
//!     exe_path: "hello_world".to_string(),
//!     input_path: "1.in".to_string(),
//!     output_path: "1.out".to_string(),
//!     error_path: "1.err".to_string(),
//!     args: vec![],
//!     env: vec![],
//!     log_path: "judger.log".to_string(),
//!     seccomp_rule_name: Some(SeccompRuleName::CCpp),
//!     uid: 0,
//!     gid: 0,
//!  };
//!  let result = run(&config);
//!  println!("{:?}", result);
//! ```
//! # Modules
//! - `child`: Handles the child process execution and resource limiting.
//! - `logger`: Provides logging functionalities.
//! - `runner`: Manages the overall execution flow.
//! - `seccomp`: Implements seccomp filtering.
//! - `utils`: Contains utility functions and error codes.
//! # Error Handling
//! The library defines a set of error codes in the `utils` module to represent various failure scenarios.
//! Users can handle these errors appropriately based on their needs.
//! # Logging
//! The `logger` module provides a simple logging mechanism with different log levels.
//! Users can log messages to a specified log file for debugging and monitoring purposes.
//! # Security
//! The library supports seccomp filtering to restrict the system calls that the executed code can make,
//! enhancing the security of the sandboxed environment.
//! # License
//! This library is open-source and available under the MIT License.
//! Feel free to use and modify it according to your needs.
//! # Contributions
//! Contributions are welcome! Please submit issues and pull requests on the [GitHub repository](https://github.com/harkerhand/judger-rs).
//! # Author
//! Developed by [harkerhand](https://github.com/harkerhand).

mod child;
mod error;
mod logger;
mod runner;
mod seccomp;

pub use child::child_process;
pub use error::ErrorCode;
pub use logger::LogLevel;
pub use logger::Logger;
pub use runner::run;
pub use seccomp::SeccompRuleName;

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

impl Default for Config {
    fn default() -> Self {
        Config {
            max_cpu_time: 1000,
            max_real_time: 2000,
            max_memory: 128 * 1024 * 1024,
            max_stack: 32 * 1024 * 1024,
            max_process_number: 1,
            max_output_size: 10000,
            exe_path: Default::default(),
            input_path: Default::default(),
            output_path: Default::default(),
            error_path: Default::default(),
            args: Default::default(),
            env: Default::default(),
            log_path: Default::default(),
            seccomp_rule_name: Some(SeccompRuleName::General),
            uid: 0,
            gid: 0,
        }
    }
}
