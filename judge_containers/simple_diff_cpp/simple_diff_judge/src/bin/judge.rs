use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;

#[derive(Deserialize, Debug)]
struct Test {
    input: String,
    output: String,
    time_limit: u32,
    memory_limit: u32,
}
#[derive(Deserialize, Debug)]
struct TaskData {
    code: String,
    tests: Vec<Test>,
}
#[derive(Deserialize, Debug)]
struct Submission {
    // _id: u32,
    // _task_type: String,
    // _lang: String,
    task_data: TaskData,
}

#[derive(Serialize, Debug)]
struct Metrics {
    time: u32,
    memory: u32,
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
        eprint!("{}", output_stderr);
        let output = String::from_utf8(output.stdout).unwrap();
        let test_result = if output_stderr == "SEC\n" {
            TestResult {
                err: "SEC".to_owned(),
                message: "illegal syscall attempted".to_owned(),
                score: 0,
                metrics: Metrics { time: 0, memory: 0 },
            }
        } else if output == test.output {
            TestResult {
                err: "OK".to_owned(),
                message: "accepted".to_owned(),
                score: 100,
                metrics: Metrics { time: 0, memory: 0 },
            }
        } else {
            TestResult {
                err: "WA".to_owned(),
                message: "wrong answer".to_owned(),
                score: 0,
                metrics: Metrics { time: 0, memory: 0 },
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
