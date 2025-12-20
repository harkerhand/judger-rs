use serde::Serialize;
use std::fmt::Display;

/// Error codes for the judger.
#[derive(Debug, Clone, Serialize, Default, PartialEq, Eq)]
pub enum ErrorCode {
    /// Operation completed successfully.
    #[default]
    Success,
    /// Configuration is invalid.
    InvalidConfig,
    /// Forking a new process failed.
    ForkFailed,
    /// Compiling the source code failed.
    CompileError,
    /// Waiting for a process failed.
    WaitFailed,
    /// Root privileges are required.
    RootRequired,
    /// Loading seccomp rules failed.
    LoadSeccompFailed,
    /// Setting resource limits failed.
    SetrlimitFailed,
    /// Duplicating file descriptors failed.
    Dup2Failed,
    /// Setting user ID failed.
    SetuidFailed,
    /// Executing the target program failed.
    ExecveFailed,
    /// Special judge program error.
    SpjError,
    /// System error
    SystemError,
    /// Cpu time limit exceeded
    CpuTimeLimitExceeded,
    /// Real time limit exceeded
    RealTimeLimitExceeded,
    /// Memory limit exceeded
    MemoryLimitExceeded,
    /// Runtime error
    RuntimeError,
    /// Interactor produced wrong answer
    WrongAnswer(String),
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCode::WrongAnswer(msg) => write!(f, "Wrong Answer: {}", msg),
            _ => write!(f, "{:?}", self),
        }
    }
}

impl ErrorCode {
    /// Convert the ErrorCode to its corresponding i32 value.
    pub fn to_i32(&self) -> i32 {
        match self {
            ErrorCode::Success => 0,
            ErrorCode::InvalidConfig => -1,
            ErrorCode::ForkFailed => -2,
            ErrorCode::CompileError => -3,
            ErrorCode::WaitFailed => -4,
            ErrorCode::RootRequired => -5,
            ErrorCode::LoadSeccompFailed => -6,
            ErrorCode::SetrlimitFailed => -7,
            ErrorCode::Dup2Failed => -8,
            ErrorCode::SetuidFailed => -9,
            ErrorCode::ExecveFailed => -10,
            ErrorCode::SpjError => -11,
            ErrorCode::SystemError => -12,
            ErrorCode::CpuTimeLimitExceeded => 1,
            ErrorCode::RealTimeLimitExceeded => 2,
            ErrorCode::MemoryLimitExceeded => 3,
            ErrorCode::RuntimeError => 4,
            ErrorCode::WrongAnswer(_) => 5,
        }
    }
}
