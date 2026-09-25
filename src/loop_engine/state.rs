use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LoopStatus {
    Running,
    Paused,
    Completed,
    Stopped,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopState {
    pub goal: String,
    pub status: LoopStatus,
    pub iteration: u64,
    pub started_at: u64,
    pub updated_at: u64,
    pub last_action: String,
    pub last_error: Option<String>,
    pub successes: u64,
    pub errors: u64,
}

impl LoopState {
    pub fn new(goal: String) -> Self {
        let now = now();
        Self {
            goal,
            status: LoopStatus::Running,
            iteration: 0,
            started_at: now,
            updated_at: now,
            last_action: "initialized".into(),
            last_error: None,
            successes: 0,
            errors: 0,
        }
    }
}
pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
