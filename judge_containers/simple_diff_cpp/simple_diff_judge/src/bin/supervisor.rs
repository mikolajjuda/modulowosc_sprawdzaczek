use libseccomp::*;
use nix::sys::wait::waitpid;
use nix::sys::{ptrace, resource, signal, wait};
use nix::unistd::{execv, fork, ForkResult};
use simple_diff_judge::*;
use std::env;
use std::ffi::{CStr, CString};

fn apply_allow_rules(filter: &mut ScmpFilterContext, syscalls: &[&str]) {
    for syscall in syscalls {
        filter
            .add_rule(ScmpAction::Allow, ScmpSyscall::from_name(syscall).unwrap())
            .unwrap();
    }
}

fn timeval_to_ms(timeval: nix::sys::time::TimeVal) -> u64 {
    timeval.tv_sec() as u64 * 1000 + timeval.tv_usec() as u64 / 1000
}

fn main() {
    let args: Vec<CString> = env::args()
        .skip(1)
        .map(|arg| CString::new(arg).unwrap())
        .collect();
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => {
            waitpid(Some(child.into()), None).unwrap();
            ptrace::setoptions(child, ptrace::Options::PTRACE_O_TRACESECCOMP).unwrap();
            ptrace::cont(child, None).unwrap();
            loop {
                let status = waitpid(Some(child.into()), None).unwrap();
                match status {
                    wait::WaitStatus::Exited(_, code) => {
                        if code != 0 {
                            eprintln!(
                                "{}",
                                serde_json::to_string(&SupervisorReturn::RuntimeErr).unwrap()
                            );
                        } else {
                            let resources =
                                resource::getrusage(resource::UsageWho::RUSAGE_CHILDREN).unwrap();
                            let time = timeval_to_ms(resources.user_time())
                                + timeval_to_ms(resources.system_time());
                            let memory = resources.max_rss() as u64 * 1024;
                            let metrics = Metrics { time, memory };
                            eprintln!(
                                "{}",
                                serde_json::to_string(&SupervisorReturn::Ok(metrics)).unwrap()
                            );
                        }
                        break;
                    }
                    wait::WaitStatus::PtraceEvent(_, _, event) => {
                        if event == ptrace::Event::PTRACE_EVENT_SECCOMP as i32 {
                            eprintln!(
                                "{}",
                                serde_json::to_string(&SupervisorReturn::SecurityViolation)
                                    .unwrap()
                            );
                            ptrace::kill(child).unwrap();
                            std::process::exit(1);
                        }
                    }
                    wait::WaitStatus::Signaled(_, _, _) => {
                        break;
                    }
                    _ => {
                        ptrace::cont(child, None).unwrap();
                    }
                }
            }
        }
        Ok(ForkResult::Child) => {
            caps::clear(None, caps::CapSet::Inheritable).unwrap();
            let mut filter = ScmpFilterContext::new_filter(ScmpAction::Trace(123)).unwrap();
            filter.add_arch(ScmpArch::native()).unwrap();
            apply_allow_rules(
                &mut filter,
                &[
                    "exit",
                    "exit_group",
                    "read",
                    "write",
                    "brk",
                    "mmap",
                    "kill",
                    "arch_prctl",
                    "access",
                    "openat",
                    "newfstatat",
                    "close",
                    "pread64",
                    "set_tid_address",
                    "set_robust_list",
                    "rseq",
                    "mprotect",
                    "prlimit64",
                    "munmap",
                    "getrandom",
                    "gettid",
                    "getpid",
                    "tgkill",
                    "execve",
                    "futex",
                ],
            );
            ptrace::traceme().unwrap();
            signal::raise(signal::Signal::SIGSTOP).unwrap();
            filter.load().unwrap();
            execv(&args.last().unwrap(), &[] as &[&CStr; 0]).unwrap();
        }
        Err(_) => {
            panic!("fork failed");
        }
    }
}
