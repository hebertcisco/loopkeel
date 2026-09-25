mod adapters;
mod cli;
mod error;
mod loop_engine;
mod observability;
mod skills;

use clap::Parser;
use cli::{Cli, Command, SkillCommand};
use error::Result;
use loop_engine::{Engine, EngineOptions};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    observability::init(cli.log_format, cli.log_level.as_deref());
    let root = cli
        .project_dir
        .canonicalize()
        .unwrap_or(cli.project_dir.clone());
    let engine = Engine::new(root, cli.agent, cli.output)?;
    match cli.command {
        Command::Start(args) => engine.start(EngineOptions::from(args)).await,
        Command::Step => engine.step().await,
        Command::Status => engine.status().await,
        Command::Pause => engine.control("pause").await,
        Command::Resume => engine.control("resume").await,
        Command::Stop => engine.control("stop").await,
        Command::Skill { command } => match command {
            SkillCommand::Install(args) => skills::install(&engine.root_dir(), args).await,
            SkillCommand::Path(args) => skills::print_paths(&engine.root_dir(), args),
        },
    }
}
