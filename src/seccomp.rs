use libseccomp::{ScmpAction, ScmpArgCompare, ScmpCompareOp, ScmpFilterContext, ScmpSyscall};
use nix::libc;
use std::os::unix::ffi::OsStrExt;

pub fn load_seccomp_rules(rule_name: &str) -> Result<(), ()> {
    match rule_name {
        "c_cpp" => c_cpp_seccomp_rules(false),
        "c_cpp_file_io" => c_cpp_seccomp_rules(true),
        "golang" => golang_seccomp_rules(),
        "node" => node_seccomp_rules(),
        _ => Err(()),
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

    for syscall_name in syscalls_whitelist.iter() {
        let syscall = ScmpSyscall::from_name(syscall_name).map_err(|_| ())?;
        filter
            .add_rule(ScmpAction::Allow, syscall)
            .map_err(|_| ())?;
    }

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

    for syscall_name in syscalls_blacklist.iter() {
        let syscall = ScmpSyscall::from_name(syscall_name).map_err(|_| ())?;
        filter
            .add_rule(ScmpAction::KillProcess, syscall)
            .map_err(|_| ())?;
    }

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
