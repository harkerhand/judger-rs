use crate::error::ResultCode;
use crate::{Config, ErrorCode, LogLevel, Logger, child_process};
use nix::libc;
use nix::sys::signal::Signal;
use nix::unistd::{ForkResult, Uid, fork};
use serde::Serialize;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, SystemTime};

#[derive(Debug, Serialize, Default)]
pub struct RunResult {
    cpu_time: i32,
    real_time: i32,
    memory: i64,
    signal: i32,
    exit_code: i32,
    error: String,
    result: String,
}

/// Runs the judger with the given configuration.
/// Returns a `RunResult` containing the execution results.
/// # Arguments
/// * `config` - A reference to the `Config` struct containing the judger configuration
/// # Returns
/// * `Result<RunResult, String>` - On success, returns `Ok(RunResult)`. On failure, returns `Err(String)` with an error message.
pub fn run(config: &Config) -> Result<RunResult, String> {
    let mut logger = Logger::new(&config.log_path)
        .map_err(|e| format!("Failed to open log file {}: {:?}", &config.log_path, e))?;
    let mut result = RunResult::default();

    let uid = Uid::current();
    if !uid.is_root() {
        result.error = ErrorCode::RootRequired.to_string();
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
        result.error = ErrorCode::InvalidConfig.to_string();
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
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child, .. }) => {
            let cancel_flag = Arc::new(AtomicBool::new(false));
            if config.max_real_time != -1 {
                let cancel_flag_clone = Arc::clone(&cancel_flag);
                let max_real_time = config.max_real_time;
                thread::spawn(move || {
                    thread::sleep(Duration::from_millis(max_real_time as u64));
                    if !cancel_flag_clone.load(Ordering::SeqCst) {
                        let _ = nix::sys::signal::kill(child, Signal::SIGKILL);
                    }
                });
            }

            let mut status: i32 = 0;
            let mut rusage: libc::rusage = unsafe { std::mem::zeroed() };
            let wait_pid = unsafe { libc::wait4(child.as_raw(), &mut status, 0, &mut rusage) };
            if wait_pid == -1 {
                result.error = ErrorCode::WaitFailed.to_string();
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
                result.result = ResultCode::SystemError.to_string();
            } else {
                result.exit_code = libc::WEXITSTATUS(status);
                result.cpu_time = (rusage.ru_utime.tv_sec as i64 * 1000
                    + (rusage.ru_utime.tv_usec as i64 / 1000))
                    as i32;
                result.memory = (rusage.ru_maxrss as i64) * 1024;

                if result.exit_code != 0 {
                    result.result = ResultCode::RuntimeError.to_string();
                }
                if result.signal == Signal::SIGSEGV as i32 {
                    if config.max_memory != -1 && result.memory > config.max_memory {
                        result.result = ResultCode::MemoryLimitExceeded.to_string();
                    } else {
                        result.result = ResultCode::RuntimeError.to_string();
                    }
                } else {
                    if result.signal != 0 {
                        result.result = ResultCode::RuntimeError.to_string();
                    }
                    if config.max_memory != -1 && result.memory > config.max_memory {
                        result.result = ResultCode::MemoryLimitExceeded.to_string();
                    }
                    if config.max_real_time != -1 && result.real_time > config.max_real_time {
                        result.result = ResultCode::RealTimeLimitExceeded.to_string();
                    }
                    if config.max_cpu_time != -1 && result.cpu_time > config.max_cpu_time {
                        result.result = ResultCode::CpuTimeLimitExceeded.to_string();
                    }
                }
            }
            Ok(result)
        }
        Ok(ForkResult::Child) => match child_process(config, logger) {
            Ok(_) => std::process::exit(0),
            Err(e) => {
                eprintln!("Child process failed: {:?}", e);
                std::process::exit(e as i32);
            }
        },
        Err(_) => Ok(RunResult {
            error: ErrorCode::ForkFailed.to_string(),
            ..Default::default()
        }),
    }
}
