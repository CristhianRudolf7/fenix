//! Linux-enforced read-only helper. Invoked in a fresh process before runtime startup.
use anyhow::{Context, Result, bail};
use landlock::{
    ABI, Access, AccessFs, PathBeneath, PathFd, Ruleset, RulesetAttr, RulesetCreatedAttr,
    RulesetStatus,
};
use nix::libc;
use seccompiler::{
    BpfProgram, SeccompAction, SeccompCmpArgLen, SeccompCmpOp, SeccompCondition, SeccompFilter,
    SeccompRule,
};
use std::{collections::BTreeMap, os::unix::process::CommandExt, path::Path, process::Command};

/// Fails closed when the kernel cannot enforce every requested filesystem right.
/// This changes only the helper process; callers must never invoke it in the server.
pub fn exec(command: &str, cwd: &Path, inputs: &[String]) -> Result<()> {
    if inputs.is_empty() {
        bail!("Read-only execution requires explicit input paths");
    }
    let mut rules = Ruleset::default()
        .handle_access(AccessFs::from_all(ABI::V3))?
        .create()?;
    // Runtime libraries and read-only configuration are the only implicit inputs.
    for path in [
        "/usr",
        "/bin",
        "/sbin",
        "/lib",
        "/lib64",
        "/etc",
        "/dev/null",
        "/dev/urandom",
    ]
    .into_iter()
    .map(str::to_owned)
    .chain(inputs.iter().cloned())
    {
        if Path::new(&path).exists() {
            let access = if Path::new(&path).is_dir() {
                AccessFs::from_read(ABI::V3)
            } else {
                AccessFs::ReadFile | AccessFs::Execute
            };
            rules = rules.add_rule(PathBeneath::new(PathFd::new(path)?, access))?;
        }
    }
    let status = rules.restrict_self()?;
    if status.ruleset != RulesetStatus::FullyEnforced || !status.no_new_privs {
        bail!("Read-only execution unavailable: Linux Landlock ABI 3 must be fully enforced");
    }
    // An allowlist prevents network/IPC, metadata mutation, ptrace, mounts, signals
    // to unrelated processes, io_uring and other paths around filesystem rules.
    let calls = [
        libc::SYS_read,
        libc::SYS_readv,
        libc::SYS_pread64,
        libc::SYS_close,
        libc::SYS_openat,
        libc::SYS_newfstatat,
        libc::SYS_fstat,
        libc::SYS_statx,
        libc::SYS_lseek,
        libc::SYS_mmap,
        libc::SYS_mprotect,
        libc::SYS_munmap,
        libc::SYS_brk,
        libc::SYS_mremap,
        libc::SYS_madvise,
        libc::SYS_futex,
        libc::SYS_rt_sigaction,
        libc::SYS_rt_sigprocmask,
        libc::SYS_rt_sigreturn,
        libc::SYS_sigaltstack,
        libc::SYS_exit,
        libc::SYS_exit_group,
        libc::SYS_execve,
        libc::SYS_clone,
        libc::SYS_wait4,
        libc::SYS_waitid,
        libc::SYS_getpid,
        libc::SYS_getppid,
        libc::SYS_gettid,
        libc::SYS_getuid,
        libc::SYS_geteuid,
        libc::SYS_getgid,
        libc::SYS_getegid,
        libc::SYS_getgroups,
        libc::SYS_uname,
        libc::SYS_getcwd,
        libc::SYS_chdir,
        libc::SYS_fchdir,
        libc::SYS_getdents64,
        libc::SYS_readlinkat,
        libc::SYS_faccessat,
        libc::SYS_faccessat2,
        libc::SYS_clock_gettime,
        libc::SYS_clock_nanosleep,
        libc::SYS_nanosleep,
        libc::SYS_gettimeofday,
        libc::SYS_getrandom,
        libc::SYS_prlimit64,
        libc::SYS_getrlimit,
        libc::SYS_getrusage,
        libc::SYS_sysinfo,
        libc::SYS_set_tid_address,
        libc::SYS_set_robust_list,
        libc::SYS_rseq,
        libc::SYS_sched_getaffinity,
        libc::SYS_sched_yield,
        libc::SYS_dup,
        libc::SYS_dup3,
        libc::SYS_pipe2,
        libc::SYS_poll,
        libc::SYS_ppoll,
        libc::SYS_pselect6,
        libc::SYS_getpgrp,
        libc::SYS_getpgid,
        libc::SYS_setpgid,
    ];
    let mut allowed: BTreeMap<i64, Vec<SeccompRule>> =
        calls.into_iter().map(|n| (n, vec![])).collect();
    #[cfg(target_arch = "x86_64")]
    for n in [
        libc::SYS_open,
        libc::SYS_stat,
        libc::SYS_lstat,
        libc::SYS_access,
        libc::SYS_readlink,
        libc::SYS_arch_prctl,
        libc::SYS_dup2,
        libc::SYS_pipe,
        libc::SYS_select,
        libc::SYS_vfork,
        libc::SYS_fork,
    ] {
        allowed.insert(n, vec![]);
    }
    for n in [libc::SYS_write, libc::SYS_writev] {
        allowed.insert(
            n,
            [1, 2]
                .into_iter()
                .map(|fd| {
                    SeccompRule::new(vec![
                        SeccompCondition::new(0, SeccompCmpArgLen::Dword, SeccompCmpOp::Eq, fd)
                            .unwrap(),
                    ])
                    .unwrap()
                })
                .collect(),
        );
    }
    // Descriptor management and advisory database locks; no leases or ownership changes.
    allowed.insert(
        libc::SYS_fcntl,
        [
            libc::F_SETLK,
            libc::F_SETLKW,
            libc::F_GETLK,
            libc::F_GETFD,
            libc::F_SETFD,
            libc::F_GETFL,
            libc::F_SETFL,
            libc::F_DUPFD,
            libc::F_DUPFD_CLOEXEC,
        ]
        .into_iter()
        .map(|op| {
            SeccompRule::new(vec![
                SeccompCondition::new(1, SeccompCmpArgLen::Dword, SeccompCmpOp::Eq, op as u64)
                    .unwrap(),
            ])
            .unwrap()
        })
        .collect(),
    );
    let filter: BpfProgram = SeccompFilter::new(
        allowed,
        SeccompAction::Errno(libc::EPERM as u32),
        SeccompAction::Allow,
        std::env::consts::ARCH
            .try_into()
            .context("Unsupported read-only architecture")?,
    )?
    .try_into()?;
    seccompiler::apply_filter(&filter)?;
    Err(Command::new("/bin/bash")
        .args(["--noprofile", "--norc", "-c", command])
        .current_dir(cwd)
        .env_remove("BASH_ENV")
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .exec()
        .into())
}
