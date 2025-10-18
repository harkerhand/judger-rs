use serde::Serialize;

mod seccomp;
mod utils;
mod logger;

use nix::sys::resource::{Resource, setrlimit};
use nix::unistd::{ForkResult, Gid, Uid, execve, fork, setgid, setuid};
use std::ffi::CString;
use std::fs::File;
use std::os::fd::{AsRawFd};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use nix::libc;
use nix::sys::signal::Signal;
pub use crate::logger::Logger;
use crate::utils::{get_time_us};

/// Configuration for the judger.
#[derive(Debug)]
pub struct Config {
    pub max_cpu_time: i32,
    pub max_real_time: i32,
    pub max_memory: i64,
    pub max_stack: i64,
    pub max_process_number: i32,
    pub max_output_size: i64,
    pub memory_limit_check_only: bool,
    pub exe_path: String,
    pub input_path: String,
    pub output_path: String,
    pub error_path: String,
    pub args: Vec<String>,
    pub env: Vec<String>,
    pub log_path: String,
    pub seccomp_rule_name: Option<String>,
    pub uid: u32,
    pub gid: u32,
}

impl Config {
    pub(crate) fn check(&self) -> bool {
        !((self.max_cpu_time < 1 && self.max_cpu_time != -1) ||
            (self.max_real_time < 1 && self.max_real_time != -1) ||
            (self.max_stack < 1) ||
            (self.max_memory < 1 && self.max_memory != -1) ||
            (self.max_process_number < 1 && self.max_process_number != -1) ||
            (self.max_output_size < 1 && self.max_output_size != -1))
    }
}


#[derive(Debug, Serialize)]
pub struct RunResult {
    cpu_time: i32,
    real_time: i32,
    memory: i64,
    signal: i32,
    exit_code: i32,
    error: i32,
    result: i32,
}

impl RunResult {
    pub fn new() -> Self {
        RunResult {
            cpu_time: 0,
            real_time: 0,
            memory: 0,
            signal: 0,
            exit_code: 0,
            error: 0,
            result: 0,
        }
    }
}


#[allow(dead_code)]
#[derive(Debug)]
pub enum ErrorCode {
    Success = 0,
    InvalidConfig = -1,
    ForkFailed = -2,
    PthreadFailed = -3,
    WaitFailed = -4,
    RootRequired = -5,
    LoadSeccompFailed = -6,
    SetrlimitFailed = -7,
    Dup2Failed = -8,
    SetuidFailed = -9,
    ExecveFailed = -10,
    SpjError = -11,
}

#[allow(dead_code)]
enum ResultCode {
    WrongAnswer = -1,
    CpuTimeLimitExceeded = 1,
    RealTimeLimitExceeded = 2,
    MemoryLimitExceeded = 3,
    RuntimeError = 4,
    SystemError = 5,
}


/// Run the judger with the given configuration.
pub fn run(config: &Config) -> Result<RunResult, String> {
    let mut logger = Logger::new(&config.log_path).map_err(
        |e| format!("Failed to open log file {}: {:?}", &config.log_path, e)
    )?;
    let mut result = RunResult::new();

    let uid = Uid::current();
    if !uid.is_root() {
        result.error = ErrorCode::RootRequired as i32;
        logger.write(
            0,
            file!(),
            line!(),
            format_args!("Error: Root privileges are required to run the judger."),
        ).map_err(
            |e| format!("Failed to write to log file: {:?}", e))?;
        return Ok(result);
    }

    if !config.check() {
        result.error = ErrorCode::InvalidConfig as i32;
        logger.write(
            0,
            file!(),
            line!(),
            format_args!("Error: Invalid configuration provided."),
        ).map_err(
            |e| format!("Failed to write to log file: {:?}", e))?;
        return Ok(result);
    }

    let start_time = get_time_us();

    match unsafe { fork() } {
        Ok(ForkResult::Parent { child, .. }) => {
            let cancel_flag = Arc::new(AtomicBool::new(false));
            if config.max_real_time != -1 {
                let child_pid = child.clone();
                let cancel_flag_clone = Arc::clone(&cancel_flag);
                let max_real_time = config.max_real_time;
                thread::spawn(move || {
                    thread::sleep(Duration::from_millis(max_real_time as u64));
                    if !cancel_flag_clone.load(Ordering::SeqCst) {
                        let _ = nix::sys::signal::kill(child_pid, Signal::SIGKILL);
                    }
                });
            }

            let mut status: i32 = 0;
            let mut rusage: libc::rusage = unsafe { std::mem::zeroed() };
            let wait_pid = unsafe { libc::wait4(child.as_raw(), &mut status, 0, &mut rusage) };
            if wait_pid == -1 {
                result.error = ErrorCode::WaitFailed as i32;
                return Ok(result);
            }

            let real_time = (get_time_us() - start_time) / 1000;
            result.real_time = real_time as i32;
            cancel_flag.store(true, Ordering::SeqCst);

            if libc::WIFSIGNALED(status) {
                result.signal = libc::WTERMSIG(status);
            }

            if result.signal == Signal::SIGUSR1 as i32 {
                result.result = ResultCode::SystemError as i32;
            } else {
                result.exit_code = libc::WEXITSTATUS(status);
                result.cpu_time = (rusage.ru_utime.tv_sec as i64 * 1000 + (rusage.ru_utime.tv_usec as i64 / 1000)) as i32;
                result.memory = (rusage.ru_maxrss as i64) * 1024;

                if result.exit_code != 0 {
                    result.result = ResultCode::RuntimeError as i32;
                }
                if result.signal == Signal::SIGSEGV as i32 {
                    if config.max_memory != -1 && result.memory > config.max_memory {
                        result.result = ResultCode::MemoryLimitExceeded as i32;
                    } else {
                        result.result = ResultCode::RuntimeError as i32;
                    }
                } else {
                    if result.signal != 0 {
                        result.result = ResultCode::RuntimeError as i32;
                    }
                    if config.max_memory != -1 && result.memory > config.max_memory {
                        result.result = ResultCode::MemoryLimitExceeded as i32;
                    }
                    if config.max_real_time != -1 && result.real_time > config.max_real_time {
                        result.result = ResultCode::RealTimeLimitExceeded as i32;
                    }
                    if config.max_cpu_time != -1 && result.cpu_time > config.max_cpu_time {
                        result.result = ResultCode::CpuTimeLimitExceeded as i32;
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
        Err(_) => {
            let mut result = RunResult {
                cpu_time: 0,
                real_time: 0,
                memory: 0,
                signal: 0,
                exit_code: 0,
                error: 0,
                result: 0,
            };
            result.error = ErrorCode::ForkFailed as i32;
            Ok(result)
        }
    }
}

pub fn child_process(config: &Config, mut logger: Logger) -> Result<(), ErrorCode> {
    if config.max_stack != -1 {
        setrlimit(
            Resource::RLIMIT_STACK,
            config.max_stack as u64,
            config.max_stack as u64,
        )
            .map_err(|_| ErrorCode::SetrlimitFailed)?;
    }
    if !config.memory_limit_check_only && config.max_memory != -1 {
        setrlimit(
            Resource::RLIMIT_AS,
            (config.max_memory * 2) as u64,
            (config.max_memory * 2) as u64,
        )
            .map_err(|_| ErrorCode::SetrlimitFailed)?;
    }
    if config.max_cpu_time != -1 {
        setrlimit(
            Resource::RLIMIT_CPU,
            (config.max_cpu_time / 1000) as u64,
            (config.max_cpu_time / 1000) as u64,
        )
            .map_err(|_| ErrorCode::SetrlimitFailed)?;
    }
    if config.max_process_number != -1 {
        setrlimit(
            Resource::RLIMIT_NPROC,
            config.max_process_number as u64,
            config.max_process_number as u64,
        )
            .map_err(|_| ErrorCode::SetrlimitFailed)?;
    }
    if config.max_output_size != -1 {
        setrlimit(
            Resource::RLIMIT_FSIZE,
            config.max_output_size as u64,
            config.max_output_size as u64,
        )
            .map_err(|_| ErrorCode::SetrlimitFailed)?;
    }


    let input_file = File::open(&config.input_path).map_err(|_| ErrorCode::Dup2Failed)?;
    if unsafe { libc::dup2(input_file.as_raw_fd(), 0) } == -1 {
        logger.write(
            0,
            file!(),
            line!(),
            format_args!("Error: Failed to redirect standard input."),
        ).map_err(|_| ErrorCode::Dup2Failed)?;
        return Err(ErrorCode::Dup2Failed);
    }

    let output_file = File::create(&config.output_path).map_err(|_| ErrorCode::Dup2Failed)?;
    if unsafe { libc::dup2(output_file.as_raw_fd(), 1) } == -1 {
        logger.write(
            0,
            file!(),
            line!(),
            format_args!("Error: Failed to redirect standard output."),
        ).map_err(|_| ErrorCode::Dup2Failed)?;
        return Err(ErrorCode::Dup2Failed);
    }

    let error_file = File::create(&config.error_path).map_err(|_| ErrorCode::Dup2Failed)?;
    if unsafe { libc::dup2(error_file.as_raw_fd(), 2) } == -1 {
        logger.write(
            0,
            file!(),
            line!(),
            format_args!("Error: Failed to redirect standard error."),
        ).map_err(|_| ErrorCode::Dup2Failed)?;
        return Err(ErrorCode::Dup2Failed);
    }

    setgid(Gid::from_raw(config.gid)).map_err(|_| ErrorCode::SetuidFailed)?;
    setuid(Uid::from_raw(config.uid)).map_err(|_| ErrorCode::SetuidFailed)?;

    if let Some(rule_name) = &config.seccomp_rule_name {
        seccomp::load_seccomp_rules(rule_name).map_err(|_| ErrorCode::LoadSeccompFailed)?;
    }

    let exe_path = CString::new(config.exe_path.clone()).unwrap();
    let args: Vec<CString> = config
        .args
        .iter()
        .map(|arg| CString::new(arg.as_str()).unwrap())
        .collect();
    let env: Vec<CString> = config
        .env
        .iter()
        .map(|e| CString::new(e.as_str()).unwrap())
        .collect();
    execve(&exe_path, &args, &env).map_err(|_| ErrorCode::ExecveFailed)?;

    Ok(())
}
