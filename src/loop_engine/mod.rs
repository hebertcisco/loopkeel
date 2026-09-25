pub mod state;
pub mod stop_criteria;

mod config;
mod persistence;
mod retry;

use crate::{
    adapters,
    cli::{Agent, OutputFormat, StartArgs},
    error::{LoopError, Result},
};
use state::{now, LoopState, LoopStatus};
use std::{path::PathBuf, time::Duration};
use tokio::{fs, sync::mpsc, time};
use tracing::{info, warn, Instrument};

#[derive(Debug)]
pub struct Engine {
    root: PathBuf,
    agent: Agent,
    output: OutputFormat,
}

impl Engine {
    pub fn root_dir(&self) -> PathBuf {
        self.root.clone()
    }
}

#[derive(Debug)]
pub struct EngineOptions {
    pub goal: String,
    pub plan: Option<PathBuf>,
    pub max_iterations: Option<u64>,
    pub timeout: Option<u64>,
    pub success_command: Option<String>,
    pub retries: Option<u32>,
    pub backoff_ms: Option<u64>,
    pub buffer_limit: Option<usize>,
}

impl From<StartArgs> for EngineOptions {
    fn from(a: StartArgs) -> Self {
        Self {
            goal: a.goal.unwrap_or_default(),
            plan: a.plan,
            max_iterations: a.max_iterations,
            timeout: a.timeout,
            success_command: a.success_command,
            retries: a.retries,
            backoff_ms: a.backoff_ms,
            buffer_limit: a.buffer_limit,
        }
    }
}

impl Engine {
    pub fn new(root: PathBuf, agent: Agent, output: OutputFormat) -> Result<Self> {
        Ok(Self {
            root,
            agent,
            output,
        })
    }
    fn dir(&self) -> PathBuf {
        self.root.join(".loop")
    }
    pub async fn start(&self, mut opts: EngineOptions) -> Result<()> {
        if opts.goal.is_empty() {
            if let Some(plan) = &opts.plan {
                opts.goal = fs::read_to_string(plan).await?;
            }
        }
        if opts.goal.is_empty() {
            return Err(LoopError::MissingGoal);
        }
        let config = config::read(&self.root).await?;
        let max_iterations = opts.max_iterations.or(config.max_iterations).unwrap_or(10);
        let timeout = opts.timeout.or(config.timeout);
        let success_command = opts.success_command.or(config.success_command);
        let retries = opts.retries.or(config.retries).unwrap_or(3);
        let backoff_ms = opts.backoff_ms.or(config.backoff_ms).unwrap_or(500);
        let buffer_limit = opts.buffer_limit.or(config.buffer_limit).unwrap_or(64);
        let mut state = match persistence::load(&self.root).await {
            Ok(existing) if !matches!(existing.status, LoopStatus::Completed) => existing,
            _ => LoopState::new(opts.goal),
        };
        persistence::save(&self.root, &state).await?;
        persistence::append_event(&self.root, &state, "started").await?;
        let (tx, mut rx) = mpsc::channel::<String>(8);
        let control = persistence::control_path(&self.root);
        let watcher = tokio::spawn(async move {
            let mut tick = time::interval(Duration::from_millis(250));
            loop {
                tick.tick().await;
                if let Ok(cmd) = fs::read_to_string(&control).await {
                    let _ = tx.send(cmd.trim().to_string()).await;
                    let _ = fs::remove_file(&control).await;
                }
            }
        });
        let mut ticks = time::interval(Duration::from_millis(10));
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
        loop {
            tokio::select! {
                signal = tokio::signal::ctrl_c() => { signal?; state.status = LoopStatus::Stopped; state.updated_at = now(); state.last_action = "interrupted; checkpoint saved".into(); persistence::save(&self.root, &state).await?; persistence::append_event(&self.root, &state, "stopped").await?; break; }
                signal = terminate.recv() => { signal.ok_or(LoopError::NoCheckpoint)?; state.status = LoopStatus::Stopped; state.updated_at = now(); state.last_action = "terminated; checkpoint saved".into(); persistence::save(&self.root, &state).await?; persistence::append_event(&self.root, &state, "stopped").await?; break; }
                Some(command) = rx.recv() => { match command.as_str() { "pause" => state.status = LoopStatus::Paused, "resume" => state.status = LoopStatus::Running, "stop" => state.status = LoopStatus::Stopped, "complete" => state.status = LoopStatus::Completed, _ => warn!(command = %command, "unknown control command") } state.updated_at = now(); persistence::save(&self.root, &state).await?; persistence::append_event(&self.root, &state, &command).await?; if !matches!(state.status, LoopStatus::Running | LoopStatus::Paused) { break; } }
                _ = ticks.tick(), if state.status == LoopStatus::Running => { let passed = self.perform_step(&mut state, timeout, success_command.as_deref(), retries, backoff_ms, buffer_limit).await?; if stop_criteria::reached(&state, max_iterations, passed) { state.status = LoopStatus::Completed; persistence::save(&self.root, &state).await?; break; } }
            }
        }
        watcher.abort();
        info!(iteration = state.iteration, status = ?state.status, "loop finished");
        Ok(())
    }

    async fn perform_step(
        &self,
        state: &mut LoopState,
        timeout: Option<u64>,
        success_command: Option<&str>,
        retries: u32,
        backoff_ms: u64,
        _buffer_limit: usize,
    ) -> Result<bool> {
        let span = tracing::info_span!("iteration", number = state.iteration + 1);
        async {
            state.iteration += 1;
            state.updated_at = now();
            state.last_action = format!("agent planning/executing goal: {}", state.goal);
            persistence::append_event(&self.root, state, &state.last_action).await?;
            let rendered = adapters::render(
                &self.agent,
                &self.output,
                state.iteration,
                &state.last_action,
                &state.goal,
            );
            info!(message = %rendered, "iteration action");
            let passed = if let Some(command) = success_command {
                match retry::command(command, retries, backoff_ms, timeout).await {
                    Ok(true) => {
                        state.successes += 1;
                        state.last_action = "success condition passed".into();
                        true
                    }
                    Ok(false) => false,
                    Err(error) => {
                        state.errors += 1;
                        state.last_error = Some(error.to_string());
                        state.status = LoopStatus::Failed;
                        persistence::save(&self.root, state).await?;
                        return Err(error);
                    }
                }
            } else {
                state.successes += 1;
                false
            };
            persistence::save(&self.root, state).await?;
            Ok::<bool, LoopError>(passed)
        }
        .instrument(span)
        .await
    }
    pub async fn step(&self) -> Result<()> {
        let mut state = persistence::load(&self.root).await?;
        if state.status == LoopStatus::Paused
            || matches!(state.status, LoopStatus::Completed | LoopStatus::Stopped)
        {
            return Ok(());
        }
        let goal = state.goal.clone();
        self.perform_step(&mut state, None, None, 0, 0, 64).await?;
        let _ = goal;
        Ok(())
    }
    pub async fn status(&self) -> Result<()> {
        let state = persistence::load(&self.root).await?;
        match self.output {
            OutputFormat::Json => println!("{}", serde_json::to_string(&state)?),
            OutputFormat::Text => println!(
                "status={:?} iteration={} successes={} last_action={}",
                state.status, state.iteration, state.successes, state.last_action
            ),
        }
        Ok(())
    }
    pub async fn control(&self, command: &str) -> Result<()> {
        fs::create_dir_all(self.dir()).await?;
        fs::write(persistence::control_path(&self.root), command).await?;
        println!("sent {command}");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn checkpoint_is_written_each_step() {
        let dir = tempfile::tempdir().unwrap();
        let e = Engine::new(dir.path().into(), Agent::Codex, OutputFormat::Json).unwrap();
        e.start(EngineOptions {
            goal: "test".into(),
            plan: None,
            max_iterations: Some(1),
            timeout: None,
            success_command: None,
            retries: None,
            backoff_ms: None,
            buffer_limit: Some(4),
        })
        .await
        .unwrap();
        let s = persistence::load(dir.path()).await.unwrap();
        assert_eq!(s.iteration, 1);
        assert_eq!(s.status, LoopStatus::Completed);
    }
}
