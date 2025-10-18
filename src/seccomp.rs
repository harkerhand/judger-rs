use clap::ValueEnum;
use libseccomp::{ScmpAction, ScmpArgCompare, ScmpCompareOp, ScmpFilterContext, ScmpSyscall};
use nix::libc;

/// Seccomp rule names for different programming languages and general use.
#[derive(ValueEnum, Clone, Debug)]
pub enum SeccompRuleName {
    /// C/C++ seccomp rules.
    CCpp,
    /// C/C++ seccomp rules with file I/O allowed.
    CCppFileIO,
    /// Golang seccomp rules.
    Golang,
    /// Node.js seccomp rules.
    Node,
    /// General seccomp rules.
    General,
}

pub fn load_seccomp_rules(rule_name: &SeccompRuleName) -> Result<(), ()> {
    match rule_name {
        SeccompRuleName::CCpp => c_cpp_seccomp_rules(false),
        SeccompRuleName::CCppFileIO => c_cpp_seccomp_rules(true),
        SeccompRuleName::Golang => golang_seccomp_rules(),
        SeccompRuleName::Node => node_seccomp_rules(),
        SeccompRuleName::General => general_seccomp_rules(),
    }
}

fn c_cpp_seccomp_rules(allow_write_file: bool) -> Result<(), ()> {
    let syscalls_whitelist = [
        "access",
        "arch_prctl",
        "brk",
        "clock_gettime",
        "close",
        "exit_group",
        "faccessat",
        "fstat",
        "futex",
        "getrandom",
        "lseek",
        "mmap",
        "mprotect",
        "munmap",
        "newfstatat",
        "pread64",
        "prlimit64",
        "read",
        "readlink",
        "readv",
        "rseq",
        "set_robust_list",
        "set_tid_address",
        "write",
        "writev",
        "execve",
    ];

    let mut filter = ScmpFilterContext::new(ScmpAction::KillProcess).map_err(|_| ())?;

    apply_seccomp_filter(&mut filter, &syscalls_whitelist, ScmpAction::Allow)?;

    if allow_write_file {
        for name in ["open", "openat", "dup", "dup2", "dup3"].iter() {
            let syscall = ScmpSyscall::from_name(name).map_err(|_| ())?;
            filter
                .add_rule(ScmpAction::Allow, syscall)
                .map_err(|_| ())?;
        }
    } else {
        // 不允许写文件，只允许 read-only 打开
        let open_sys = ScmpSyscall::from_name("open").map_err(|_| ())?;
        // 对参数 1（flags），执行 MaskedEq 比较：
        //   (flags & (O_WRONLY | O_RDWR)) == 0
        let cmp_open = ScmpArgCompare::new(
            1,
            ScmpCompareOp::MaskedEqual((libc::O_WRONLY | libc::O_RDWR) as u64),
            0,
        );
        filter
            .add_rule_conditional(ScmpAction::Allow, open_sys, &[cmp_open])
            .map_err(|_| ())?;

        // openat 系统调用
        let openat_sys = ScmpSyscall::from_name("openat").map_err(|_| ())?;
        // 对参数 2（flags），执行 MaskedEq 比较：
        //   (flags & (O_WRONLY | O_RDWR)) == 0
        let cmp_openat = ScmpArgCompare::new(
            2,
            ScmpCompareOp::MaskedEqual((libc::O_WRONLY | libc::O_RDWR) as u64),
            0,
        );
        filter
            .add_rule_conditional(ScmpAction::Allow, openat_sys, &[cmp_openat])
            .map_err(|_| ())?;
    }

    filter.load().map_err(|_| ())?;
    Ok(())
}

fn golang_seccomp_rules() -> Result<(), ()> {
    let syscalls_blacklist = ["socket", "fork", "vfork", "kill", "execveat"];

    let mut filter = ScmpFilterContext::new(ScmpAction::Allow).map_err(|_| ())?;

    apply_seccomp_filter(&mut filter, &syscalls_blacklist, ScmpAction::KillProcess)?;

    filter.load().map_err(|_| ())?;
    Ok(())
}

fn node_seccomp_rules() -> Result<(), ()> {
    let syscalls_blacklist = ["socket", "fork", "vfork", "kill", "execveat"];

    let mut filter = ScmpFilterContext::new(ScmpAction::Allow).map_err(|_| ())?;

    for syscall_name in syscalls_blacklist.iter() {
        let syscall = ScmpSyscall::from_name(syscall_name).map_err(|_| ())?;
        filter
            .add_rule(ScmpAction::KillProcess, syscall)
            .map_err(|_| ())?;
    }

    filter.load().map_err(|_| ())?;
    Ok(())
}

fn general_seccomp_rules() -> Result<(), ()> {
    let syscalls_blacklist = ["clone", "fork", "vfork", "kill", "execveat"];

    let mut filter = ScmpFilterContext::new(ScmpAction::Allow).map_err(|_| ())?;

    apply_seccomp_filter(&mut filter, &syscalls_blacklist, ScmpAction::KillProcess)?;

    // 对 socket 使用 KillProcess（与 C 实现保持一致的严格策略）
    let socket_sys = ScmpSyscall::from_name("socket").map_err(|_| ())?;
    filter
        .add_rule(ScmpAction::KillProcess, socket_sys)
        .map_err(|_| ())?;

    // 不允许通过 open/openat 以写方式打开（kill when flags indicate write）
    let open_sys = ScmpSyscall::from_name("open").map_err(|_| ())?;
    let cmp_open_w = ScmpArgCompare::new(
        1,
        ScmpCompareOp::MaskedEqual(libc::O_WRONLY as u64),
        libc::O_WRONLY as u64,
    );
    filter
        .add_rule_conditional(ScmpAction::KillProcess, open_sys, &[cmp_open_w])
        .map_err(|_| ())?;
    let cmp_open_rw = ScmpArgCompare::new(
        1,
        ScmpCompareOp::MaskedEqual(libc::O_RDWR as u64),
        libc::O_RDWR as u64,
    );
    filter
        .add_rule_conditional(ScmpAction::KillProcess, open_sys, &[cmp_open_rw])
        .map_err(|_| ())?;

    let openat_sys = ScmpSyscall::from_name("openat").map_err(|_| ())?;
    let cmp_openat_w = ScmpArgCompare::new(
        2,
        ScmpCompareOp::MaskedEqual(libc::O_WRONLY as u64),
        libc::O_WRONLY as u64,
    );
    filter
        .add_rule_conditional(ScmpAction::KillProcess, openat_sys, &[cmp_openat_w])
        .map_err(|_| ())?;
    let cmp_openat_rw = ScmpArgCompare::new(
        2,
        ScmpCompareOp::MaskedEqual(libc::O_RDWR as u64),
        libc::O_RDWR as u64,
    );
    filter
        .add_rule_conditional(ScmpAction::KillProcess, openat_sys, &[cmp_openat_rw])
        .map_err(|_| ())?;

    filter.load().map_err(|_| ())?;
    Ok(())
}

fn apply_seccomp_filter(
    filter: &mut ScmpFilterContext,
    sys_calls: &[&str],
    action: ScmpAction,
) -> Result<(), ()> {
    Ok(for syscall_name in sys_calls.iter() {
        let syscall = ScmpSyscall::from_name(syscall_name).map_err(|_| ())?;
        filter.add_rule(action, syscall).map_err(|_| ())?;
    })
}
