use crate::{Config, ErrorCode, LogLevel, Logger, seccomp};
use nix::libc;
use nix::sys::resource::{Resource, setrlimit};
use nix::unistd::{Gid, Uid, execve, setgid, setuid};
use std::ffi::CString;
use std::fs::File;
use std::os::fd::{AsRawFd, RawFd};

/// Function to be executed in the child process.
/// Sets resource limits, redirects standard I/O,
/// changes user and group IDs, loads seccomp rules, and executes the target program.
/// # Arguments
/// * `config` - Reference to the configuration struct.
/// * `logger` - Logger instance for logging errors.
/// # Returns
/// * `Result<(), ErrorCode>` - Ok on success, Err with ErrorCode on failure.
pub fn child_process(
    config: &Config,
    mut logger: Logger,
    fds: Option<(RawFd, RawFd)>,
) -> Result<(), ErrorCode> {
    if config.max_stack != -1 {
        setrlimit(
            Resource::RLIMIT_STACK,
            config.max_stack as u64,
            config.max_stack as u64,
        )
        .map_err(|_| ErrorCode::SetrlimitFailed)?;
    }
    if config.max_memory != -1 {
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
            (config.max_cpu_time / 1000 + 1) as u64,
            (config.max_cpu_time / 1000 + 1) as u64,
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

    let (input_fd, output_fd, _input_file, _output_file) = match fds {
        Some((inf, outf)) => (inf, outf, None, None),
        None => {
            let input_file = File::open(&config.input_path).map_err(|_| ErrorCode::Dup2Failed)?;
            let output_file =
                File::create(&config.output_path).map_err(|_| ErrorCode::Dup2Failed)?;
            (
                input_file.as_raw_fd(),
                output_file.as_raw_fd(),
                Some(input_file),
                Some(output_file),
            )
        }
    };

    if unsafe { libc::dup2(input_fd, 0) } == -1 {
        logger
            .write(
                LogLevel::Fatal,
                file!(),
                line!(),
                format_args!("Error: Failed to redirect standard input."),
            )
            .map_err(|_| ErrorCode::Dup2Failed)?;
        return Err(ErrorCode::Dup2Failed);
    }

    if unsafe { libc::dup2(output_fd, 1) } == -1 {
        logger
            .write(
                LogLevel::Fatal,
                file!(),
                line!(),
                format_args!("Error: Failed to redirect standard output."),
            )
            .map_err(|_| ErrorCode::Dup2Failed)?;
        return Err(ErrorCode::Dup2Failed);
    }

    let error_file = File::create(&config.error_path).map_err(|_| ErrorCode::Dup2Failed)?;
    if unsafe { libc::dup2(error_file.as_raw_fd(), 2) } == -1 {
        logger
            .write(
                LogLevel::Fatal,
                file!(),
                line!(),
                format_args!("Error: Failed to redirect standard error."),
            )
            .map_err(|_| ErrorCode::Dup2Failed)?;
        return Err(ErrorCode::Dup2Failed);
    }

    setgid(Gid::from_raw(config.gid)).map_err(|_| ErrorCode::SetuidFailed)?;
    setuid(Uid::from_raw(config.uid)).map_err(|_| ErrorCode::SetuidFailed)?;

    if let Some(rule_name) = &config.seccomp_rule_name {
        seccomp::load_seccomp_rules(rule_name).map_err(|_| ErrorCode::LoadSeccompFailed)?;
    }

    if let Ok(exe_path) = CString::new(config.exe_path.clone()) {
        let args: Vec<CString> = config
            .args
            .iter()
            .map(|arg| CString::new(arg.as_str()).unwrap_or_default())
            .collect();
        let env: Vec<CString> = config
            .env
            .iter()
            .map(|e| CString::new(e.as_str()).unwrap_or_default())
            .collect();
        execve(&exe_path, &args, &env).map_err(|_| ErrorCode::ExecveFailed)?;
    } else {
        logger
            .write(
                LogLevel::Fatal,
                file!(),
                line!(),
                format_args!("Error: Invalid executable path."),
            )
            .map_err(|_| ErrorCode::ExecveFailed)?;
        return Err(ErrorCode::ExecveFailed);
    }
    Ok(())
}
