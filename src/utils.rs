use nix::libc;
use nix::libc::timeval;

/// Error codes for the judger.
#[derive(Debug)]
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

pub(crate) enum ResultCode {
    #[allow(dead_code)]
    WrongAnswer = -1,
    CpuTimeLimitExceeded = 1,
    RealTimeLimitExceeded = 2,
    MemoryLimitExceeded = 3,
    RuntimeError = 4,
    SystemError = 5,
}

pub(crate) fn get_time_us() -> u64 {
    unsafe {
        let mut tv: timeval = std::mem::zeroed();
        libc::gettimeofday(&mut tv, std::ptr::null_mut());
        (tv.tv_sec * 1000 * 1000 + tv.tv_usec) as u64
    }
}
