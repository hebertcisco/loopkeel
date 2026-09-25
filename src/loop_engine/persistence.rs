use crate::{
    error::{LoopError, Result},
    loop_engine::state::{now, LoopState},
};
use std::path::{Path, PathBuf};
use tokio::{fs, io::AsyncWriteExt};

pub fn directory(root: &Path) -> PathBuf {
    root.join(".loop")
}
pub fn control_path(root: &Path) -> PathBuf {
    directory(root).join("control")
}

pub async fn load(root: &Path) -> Result<LoopState> {
    let bytes = fs::read(directory(root).join("state.json"))
        .await
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                LoopError::NoCheckpoint
            } else {
                error.into()
            }
        })?;
    Ok(serde_json::from_slice(&bytes)?)
}

pub async fn save(root: &Path, state: &LoopState) -> Result<()> {
    fs::create_dir_all(directory(root)).await?;
    let path = directory(root).join("state.json");
    let temporary = path.with_extension("json.tmp");
    fs::write(&temporary, serde_json::to_vec_pretty(state)?).await?;
    fs::rename(temporary, path).await?;
    Ok(())
}

pub async fn append_event(root: &Path, state: &LoopState, action: &str) -> Result<()> {
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(directory(root).join("events.jsonl"))
        .await?;
    let line = serde_json::json!({"at": now(), "iteration": state.iteration, "status": state.status, "action": action});
    file.write_all(format!("{line}\n").as_bytes()).await?;
    file.flush().await?;
    Ok(())
}
