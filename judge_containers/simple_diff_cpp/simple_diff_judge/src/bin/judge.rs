use serde::{Deserialize, Serialize};
use simple_diff_judge::*;
use std::fs;
use std::io::Write;

#[derive(Deserialize, Debug)]
struct Test {
    input: String,
    output: String,
    time_limit: u64,
    memory_limit: u64,
}
#[derive(Deserialize, Debug)]
struct TaskData {
    code: String,
    tests: Vec<Test>,
}
#[derive(Deserialize, Debug)]
struct Submission {
    task_data: TaskData,
}

#[derive(Serialize, Debug)]
struct TestResult {
    err: String,
    message: String,
    score: u32,
    metrics: Metrics,
}
#[derive(Serialize, Debug)]
struct Feedback {
    test_results: Vec<TestResult>,
}

fn main() {
    let stdin = std::io::stdin().lock();
    let deserializer = serde_json::Deserializer::from_reader(stdin);
    let submission: Submission = deserializer.into_iter().next().unwrap().unwrap();

    fs::create_dir("work").unwrap();
    std::env::set_current_dir("work").unwrap();

    fs::write("submission.cpp", submission.task_data.code).unwrap();

    let compiler_output = std::process::Command::new("g++")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .arg("-static")
        .arg("-O3")
        .arg("-o")
        .arg("submission")
        .arg("submission.cpp")
        .output()
        .expect("failed to execute compiler");

    let mut test_results = Vec::new();

    if !compiler_output.status.success() {
        let compiler_error_msg = String::from_utf8(compiler_output.stderr).unwrap();
        for _ in 0..submission.task_data.tests.len() {
            let test_result = TestResult {
                err: "CE".to_owned(),
                message: compiler_error_msg.clone(),
                score: 0,
                metrics: Metrics { time: 0, memory: 0 },
            };
            test_results.push(test_result);
        }
        let feedback = Feedback {
            test_results: test_results,
        };
        let feedback = serde_json::to_string(&feedback).unwrap();
        println!("{}", feedback);
        return;
    }
    for test in submission.task_data.tests {
        let mut child = std::process::Command::new("/judge/bin/supervisor")
            .arg("/judge/work/submission")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        let mut child_stdin = child.stdin.take().unwrap();
        child_stdin.write_all(test.input.as_bytes()).unwrap();
        child_stdin.flush().unwrap();
        drop(child_stdin);

        let output = child.wait_with_output().unwrap();
        let output_stderr = String::from_utf8(output.stderr).unwrap();
        let output = String::from_utf8(output.stdout).unwrap();
        let test_result = {
            if let Ok(supervisor_return) = serde_json::from_str::<SupervisorReturn>(&output_stderr)
            {
                match supervisor_return {
                    SupervisorReturn::Ok(metrics) => {
                        if test.time_limit > 0 && metrics.time > test.time_limit {
                            TestResult {
                                err: "TLE".to_owned(),
                                message: "time limit exceeded".to_owned(),
                                score: 0,
                                metrics,
                            }
                        } else if test.memory_limit > 0 && metrics.memory > test.memory_limit {
                            TestResult {
                                err: "MLE".to_owned(),
                                message: "memory limit exceeded".to_owned(),
                                score: 0,
                                metrics,
                            }
                        } else if output == test.output {
                            TestResult {
                                err: "OK".to_owned(),
                                message: "accepted".to_owned(),
                                score: 1,
                                metrics,
                            }
                        } else {
                            TestResult {
                                err: "WA".to_owned(),
                                message: "wrong answer".to_owned(),
                                score: 0,
                                metrics,
                            }
                        }
                    }
                    SupervisorReturn::RuntimeErr => TestResult {
                        err: "RE".to_owned(),
                        message: "runtime error".to_owned(),
                        score: 0,
                        metrics: Metrics { time: 0, memory: 0 },
                    },
                    SupervisorReturn::SecurityViolation { syscall_num } => TestResult {
                        err: "RV".to_owned(),
                        message: format!("illegal syscall {} attempted", syscall_num),
                        score: 0,
                        metrics: Metrics { time: 0, memory: 0 },
                    },
                }
            } else {
                eprint!("{}", output_stderr);
                TestResult {
                    err: "WTF".to_owned(),
                    message: "judge returned incorrect output, you win".to_owned(),
                    score: 0,
                    metrics: Metrics { time: 0, memory: 0 },
                }
            }
        };
        test_results.push(test_result);
    }

    let feedback = Feedback {
        test_results: test_results,
    };
    let feedback = serde_json::to_string(&feedback).unwrap();
    println!("{}", feedback);
}
