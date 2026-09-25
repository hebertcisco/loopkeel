use thiserror::Error;

pub type Result<T> = std::result::Result<T, LoopError>;

#[derive(Debug, Error)]
pub enum LoopError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("no active loop checkpoint found")]
    NoCheckpoint,
    #[error("goal is required (use --goal or --plan)")]
    MissingGoal,
    #[error("success command failed after retries: {0}")]
    CommandFailed(String),
    #[error("skill already exists: {0} (use --force to replace it)")]
    SkillExists(std::path::PathBuf),
    #[error("cannot determine the user home directory")]
    HomeNotFound,
}
