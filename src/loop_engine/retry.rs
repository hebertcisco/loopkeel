use crate::error::{LoopError, Result};
use std::{path::Path, time::Duration};
use tokio::{process::Command, time};

pub async fn command(
    working_directory: &Path,
    command: &str,
    retries: u32,
    backoff_ms: u64,
    timeout: Option<u64>,
) -> Result<bool> {
    for attempt in 0..=retries {
        let future = Command::new("sh")
            .current_dir(working_directory)
            .arg("-c")
            .arg(command)
            .status();
        let result = if let Some(seconds) = timeout {
            time::timeout(Duration::from_secs(seconds), future)
                .await
                .ok()
                .and_then(|result| result.ok())
        } else {
            future.await.ok()
        };
        if result.map(|status| status.success()).unwrap_or(false) {
            return Ok(true);
        }
        if attempt < retries {
            time::sleep(Duration::from_millis(
                backoff_ms.saturating_mul(2u64.saturating_pow(attempt)),
            ))
            .await;
        }
    }
    Err(LoopError::CommandFailed(command.into()))
}

#[cfg(test)]
mod tests {
    use super::command;
    use std::fs;

    #[tokio::test]
    async fn successful_command_returns_true() {
        let dir = tempfile::tempdir().unwrap();
        assert!(command(dir.path(), "exit 0", 1, 1, None).await.unwrap());
    }

    #[tokio::test]
    async fn command_runs_in_requested_directory() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("marker"), b"ok").unwrap();
        assert!(command(dir.path(), "test -f marker", 0, 1, None)
            .await
            .unwrap());
    }
}
