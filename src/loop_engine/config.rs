use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub max_iterations: Option<u64>,
    pub timeout: Option<u64>,
    pub success_command: Option<String>,
    pub retries: Option<u32>,
    pub backoff_ms: Option<u64>,
    pub buffer_limit: Option<usize>,
    pub agent: Option<String>,
}

pub async fn read(root: &Path) -> Result<Config> {
    match fs::read_to_string(root.join(".loop.toml")).await {
        Ok(value) => Ok(toml::from_str(&value)?),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Config::default()),
        Err(error) => Err(error.into()),
    }
}
