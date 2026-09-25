use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Clone, ValueEnum)]
pub enum Agent {
    Claude,
    Codex,
    Cursor,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}

#[derive(Parser, Debug)]
#[command(
    name = "loop",
    visible_alias = "$loop",
    version,
    about = "Orchestrate resilient agentic work loops"
)]
pub struct Cli {
    #[arg(long, default_value = ".", global = true, env = "LOOP_PROJECT_DIR")]
    pub project_dir: PathBuf,
    #[arg(long, global = true, env = "LOOP_AGENT", default_value = "codex")]
    pub agent: Agent,
    #[arg(long, value_enum, global = true, default_value = "text")]
    pub output: OutputFormat,
    #[arg(long, global = true)]
    pub log_level: Option<String>,
    #[arg(long, value_enum, global = true, default_value = "pretty")]
    pub log_format: LogFormat,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum LogFormat {
    Pretty,
    Json,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Start(StartArgs),
    Step,
    Status,
    Pause,
    Resume,
    Stop,
    Skill {
        #[command(subcommand)]
        command: SkillCommand,
    },
}

#[derive(Subcommand, Debug)]
pub enum SkillCommand {
    Install(SkillInstallArgs),
    Path(SkillPathArgs),
}

#[derive(Debug, Clone, ValueEnum)]
pub enum SkillTarget {
    Claude,
    Codex,
    Both,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum SkillScope {
    Project,
    User,
}

#[derive(Args, Debug, Clone)]
pub struct SkillInstallArgs {
    #[arg(long, value_enum, default_value = "both")]
    pub target: SkillTarget,
    #[arg(long, value_enum, default_value = "project")]
    pub scope: SkillScope,
    #[arg(long, default_value = "loop-orchestrator")]
    pub name: String,
    #[arg(long)]
    pub force: bool,
}

#[derive(Args, Debug, Clone)]
pub struct SkillPathArgs {
    #[arg(long, value_enum, default_value = "both")]
    pub target: SkillTarget,
    #[arg(long, value_enum, default_value = "project")]
    pub scope: SkillScope,
    #[arg(long, default_value = "loop-orchestrator")]
    pub name: String,
}

#[derive(Args, Debug, Clone)]
pub struct StartArgs {
    #[arg(short, long)]
    pub goal: Option<String>,
    #[arg(long)]
    pub plan: Option<PathBuf>,
    #[arg(long)]
    pub max_iterations: Option<u64>,
    #[arg(long)]
    pub timeout: Option<u64>,
    #[arg(long)]
    pub success_command: Option<String>,
    #[arg(long)]
    pub retries: Option<u32>,
    #[arg(long)]
    pub backoff_ms: Option<u64>,
    #[arg(long)]
    pub buffer_limit: Option<usize>,
}
