use nix::sys::wait::waitpid;
use nix::sys::{ptrace, resource, signal, wait};
use nix::unistd::{execv, fork, ForkResult};
use seccompiler::{BpfProgram, SeccompAction, SeccompFilter, SeccompRule};
use simple_diff_judge::*;
use std::env;
use std::ffi::{c_long, CString};

fn empty_rules(syscalls: &[c_long]) -> Vec<(c_long, Vec<SeccompRule>)> {
    syscalls.iter().map(|syscall| (*syscall, vec![])).collect()
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
                            let regs = ptrace::getregs(child).unwrap();
                            eprintln!(
                                "{}",
                                serde_json::to_string(&SupervisorReturn::SecurityViolation {
                                    syscall_num: regs.orig_rax as u64
                                })
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
            let rules = empty_rules(&[
                libc::SYS_exit,
                libc::SYS_read,
                libc::SYS_write,
                libc::SYS_execve,
                libc::SYS_arch_prctl,
                libc::SYS_brk,
                libc::SYS_set_tid_address,
                libc::SYS_set_robust_list,
                libc::SYS_rseq,
                libc::SYS_prlimit64,
                libc::SYS_readlinkat,
                libc::SYS_getrandom,
                libc::SYS_mprotect,
                libc::SYS_futex,
                libc::SYS_newfstatat,
                libc::SYS_exit_group,
                libc::SYS_uname,
                libc::SYS_readlink,
                libc::SYS_fstat,
            ]);
            let filter: BpfProgram = SeccompFilter::new(
                rules.into_iter().collect(),
                SeccompAction::Trace(123),
                SeccompAction::Allow,
                std::env::consts::ARCH.try_into().unwrap(),
            )
            .unwrap()
            .try_into()
            .unwrap();
            ptrace::traceme().unwrap();
            signal::raise(signal::Signal::SIGSTOP).unwrap();
            seccompiler::apply_filter(&filter).unwrap();
            execv(&args[0], &args[1..]).unwrap();
        }
        Err(_) => {
            panic!("fork failed");
        }
    }
}
