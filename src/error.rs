use clap::ValueEnum;
use std::fmt::Display;

/// Error codes for the judger.
#[derive(Debug, ValueEnum, Clone)]
pub enum ErrorCode {
    /// Operation completed successfully.
    Success = 0,
    /// Configuration is invalid.
    InvalidConfig = -1,
    /// Forking a new process failed.
    ForkFailed = -2,
    /// Creating a new pthread failed.
    PthreadFailed = -3,
    /// Waiting for a process failed.
    WaitFailed = -4,
    /// Root privileges are required.
    RootRequired = -5,
    /// Loading seccomp rules failed.
    LoadSeccompFailed = -6,
    /// Setting resource limits failed.
    SetrlimitFailed = -7,
    /// Duplicating file descriptors failed.
    Dup2Failed = -8,
    /// Setting user ID failed.
    SetuidFailed = -9,
    /// Executing the target program failed.
    ExecveFailed = -10,
    /// Special judge program error.
    SpjError = -11,
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let description = self
            .to_possible_value()
            .and_then(|v| v.get_help().map(|help| help.to_string()))
            .unwrap_or("Unknown error".to_string());
        write!(f, "{}", description)
    }
}

#[derive(ValueEnum, Clone)]
pub(crate) enum ResultCode {
    #[allow(dead_code)]
    /// Wrong answer
    WrongAnswer = -1,
    /// Cpu time limit exceeded
    CpuTimeLimitExceeded = 1,
    /// Real time limit exceeded
    RealTimeLimitExceeded = 2,
    /// Memory limit exceeded
    MemoryLimitExceeded = 3,
    /// Runtime error
    RuntimeError = 4,
    /// System error
    SystemError = 5,
}

impl Display for ResultCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let description = self
            .to_possible_value()
            .and_then(|v| v.get_help().map(|h| h.to_string()))
            .unwrap_or("Unknown result".to_string());
        write!(f, "{}", description)
    }
}
