use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
struct Test {
    input: String,
    output: String,
}
#[derive(Deserialize, Debug)]
struct TaskData {
    code: String,
    tests: Vec<Test>,
    time_limit: u32,
    memory_limit: u32,
}
#[derive(Deserialize, Debug)]
struct Submission {
    id: u32,
    task_type: String,
    lang: String,
    task_data: TaskData,
}

#[derive(Serialize, Debug)]
struct Metrics {
    time: u32,
    memory: u32,
}
#[derive(Serialize, Debug)]
struct TestResult {
    score: u32,
    metrics: Metrics,
}
#[derive(Serialize, Debug)]
struct Feedback {
    err: String,
    message: String,
    test_results: Vec<TestResult>,
}

fn main() {
    let stdin = std::io::stdin().lock();
    let deserializer = serde_json::Deserializer::from_reader(stdin);
    let submission: Submission = deserializer.into_iter().next().unwrap().unwrap();
    let feedback = Feedback {
        err: "OK".to_owned(),
        message: "Accepted".to_owned(),
        test_results: vec![TestResult {
            score: 100,
            metrics: Metrics {
                time: 100,
                memory: 100,
            },
        }],
    };
    let feedback = serde_json::to_string(&feedback).unwrap();
    println!("{}", feedback);
}
