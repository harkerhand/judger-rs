use crate::{Config, ErrorCode, LogLevel, Logger, child_process};
use nix::libc;
use nix::sys::signal::Signal;
use nix::unistd::{ForkResult, Uid, fork};
use serde::Serialize;
use std::os::fd::{AsRawFd, FromRawFd};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, SystemTime};

#[derive(Debug, Serialize, Default)]
pub struct RunResult {
    /// CPU time used in milliseconds.
    pub cpu_time: i32,
    /// Real time used in milliseconds.
    pub real_time: i32,
    /// Memory used in bytes.
    pub memory: i64,
    /// Signal that terminated the process.
    pub signal: i32,
    /// Exit code of the process.
    pub exit_code: i32,
    /// Error code if any error occurred during execution.
    pub result: ErrorCode,
}

/// Runs the judger with the given configuration.
/// Returns a `RunResult` containing the execution results.
/// # Arguments
/// * `config` - A reference to the `Config` struct containing the judger configuration
/// * `interactor` - An optional `PathBuf` for the interactor program
/// # Returns
/// * `Result<RunResult, String>` - On success, returns `Ok(RunResult)`. On failure, returns `Err(String)` with an error message.
pub fn run(config: &Config, interactor: Option<PathBuf>) -> Result<RunResult, String> {
    let mut logger = Logger::new(&config.log_path)
        .map_err(|e| format!("Failed to open log file {}: {:?}", &config.log_path, e))?;
    let mut result = RunResult::default();

    let uid = Uid::current();
    if !uid.is_root() {
        result.result = ErrorCode::RootRequired;
        logger
            .write(
                LogLevel::Fatal,
                file!(),
                line!(),
                format_args!("Error: Root privileges are required to run the judger."),
            )
            .map_err(|e| format!("Failed to write to log file: {:?}", e))?;
        return Ok(result);
    }

    if !config.check() {
        result.result = ErrorCode::InvalidConfig;
        logger
            .write(
                LogLevel::Fatal,
                file!(),
                line!(),
                format_args!("Error: Invalid configuration provided."),
            )
            .map_err(|e| format!("Failed to write to log file: {:?}", e))?;
        return Ok(result);
    }

    let start_time = SystemTime::now();
    let (interactor_stdin, interactor_stdout) = nix::unistd::pipe()
        .map_err(|e| format!("Failed to create pipe for interactor: {:?}", e))?;
    let (user_stdin, user_stdout) = nix::unistd::pipe()
        .map_err(|e| format!("Failed to create pipe for user program: {:?}", e))?;
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child, .. }) => {
            let inter_child = interactor.and_then(|path| {
                std::process::Command::new(path)
                    .args(vec![&config.input_path, &config.output_path])
                    .stdin(unsafe { std::process::Stdio::from_raw_fd(user_stdout.as_raw_fd()) })
                    .stdout(unsafe { std::process::Stdio::from_raw_fd(user_stdin.as_raw_fd()) })
                    .spawn()
                    .ok()
            });
            let cancel_flag = Arc::new(AtomicBool::new(false));
            if config.max_real_time != -1 {
                let cancel_flag_clone = Arc::clone(&cancel_flag);
                let max_real_time = config.max_real_time;
                thread::spawn(move || {
                    thread::sleep(Duration::from_millis(max_real_time as u64));
                    if !cancel_flag_clone.load(Ordering::SeqCst) {
                        let _ = nix::sys::signal::kill(child, Signal::SIGKILL);
                        if let Some(mut inter) = inter_child {
                            let _ = inter.kill();
                        }
                    }
                });
            }

            let mut status: i32 = 0;
            let mut rusage: libc::rusage = unsafe { std::mem::zeroed() };
            let wait_pid = unsafe { libc::wait4(child.as_raw(), &mut status, 0, &mut rusage) };
            if wait_pid == -1 {
                result.result = ErrorCode::WaitFailed;
                return Ok(result);
            }

            let duration = SystemTime::now()
                .duration_since(start_time)
                .map(|d| d.as_millis())
                .map_err(|e| format!("SystemTime error: {:?}", e))?;
            result.real_time = duration as i32;
            cancel_flag.store(true, Ordering::SeqCst);

            if libc::WIFSIGNALED(status) {
                result.signal = libc::WTERMSIG(status);
            }

            if result.signal == Signal::SIGUSR1 as i32 {
                result.result = ErrorCode::SystemError;
            } else {
                result.exit_code = libc::WEXITSTATUS(status);
                result.cpu_time = (rusage.ru_utime.tv_sec as i64 * 1000
                    + (rusage.ru_utime.tv_usec as i64 / 1000))
                    as i32;
                result.memory = (rusage.ru_maxrss as i64) * 1024;

                if result.exit_code != 0 {
                    result.result = ErrorCode::RuntimeError;
                }
                if result.signal == Signal::SIGSEGV as i32 {
                    if config.max_memory != -1 && result.memory > config.max_memory {
                        result.result = ErrorCode::MemoryLimitExceeded;
                    } else {
                        result.result = ErrorCode::RuntimeError;
                    }
                } else {
                    if result.signal != 0 {
                        result.result = ErrorCode::RuntimeError;
                    }
                    if config.max_memory != -1 && result.memory > config.max_memory {
                        result.result = ErrorCode::MemoryLimitExceeded;
                    }
                    if config.max_real_time != -1 && result.real_time > config.max_real_time {
                        result.result = ErrorCode::RealTimeLimitExceeded;
                    }
                    if config.max_cpu_time != -1 && result.cpu_time > config.max_cpu_time {
                        result.result = ErrorCode::CpuTimeLimitExceeded;
                    }
                }
            }
            Ok(result)
        }
        Ok(ForkResult::Child) => match child_process(
            config,
            logger,
            interactor.map(|_| (interactor_stdout.as_raw_fd(), interactor_stdin.as_raw_fd())),
        ) {
            Ok(_) => std::process::exit(0),
            Err(e) => {
                eprintln!("Child process failed: {:?}", e);
                std::process::exit(e as i32);
            }
        },
        Err(_) => Ok(RunResult {
            result: ErrorCode::ForkFailed,
            ..Default::default()
        }),
    }
}
