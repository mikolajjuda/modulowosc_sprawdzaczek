use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Metrics {
    pub time: u64,
    pub memory: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SupervisorReturn {
    Ok(Metrics),
    RuntimeErr,
    SecurityViolation{syscall_num: u64},
}
