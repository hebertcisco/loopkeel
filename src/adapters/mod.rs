use crate::cli::{Agent, OutputFormat};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AgentMessage<'a> {
    pub agent: &'a str,
    pub iteration: u64,
    pub action: &'a str,
    pub goal: &'a str,
}

pub fn render(
    agent: &Agent,
    output: &OutputFormat,
    iteration: u64,
    action: &str,
    goal: &str,
) -> String {
    let name = match agent {
        Agent::Claude => "claude",
        Agent::Codex => "codex",
        Agent::Cursor => "cursor",
    };
    let message = AgentMessage {
        agent: name,
        iteration,
        action,
        goal,
    };
    match output {
        OutputFormat::Json => serde_json::to_string(&message).unwrap_or_default(),
        OutputFormat::Text => format!("[{}] iteration {}: {}", name, iteration, action),
    }
}
