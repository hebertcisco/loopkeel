use crate::{
    cli::{SkillInstallArgs, SkillPathArgs, SkillScope, SkillTarget},
    error::{LoopError, Result},
};
use std::{
    env,
    path::{Path, PathBuf},
};
use tokio::fs;

const SKILL_SOURCE: &str = include_str!("../skills/loop-orchestrator/SKILL.md");

pub async fn install(root: &Path, args: SkillInstallArgs) -> Result<()> {
    let paths = paths(root, &args.target, &args.scope, &args.name)?;
    for path in &paths {
        if fs::try_exists(path).await? && !args.force {
            return Err(LoopError::SkillExists(path.clone()));
        }
    }
    for path in &paths {
        fs::create_dir_all(path).await?;
        fs::write(path.join("SKILL.md"), SKILL_SOURCE).await?;
        println!("installed skill: {}", path.display());
    }
    Ok(())
}

pub fn print_paths(root: &Path, args: SkillPathArgs) -> Result<()> {
    for path in paths(root, &args.target, &args.scope, &args.name)? {
        println!("{}", path.display());
    }
    Ok(())
}

fn paths(
    root: &Path,
    target: &SkillTarget,
    scope: &SkillScope,
    name: &str,
) -> Result<Vec<PathBuf>> {
    let base = match scope {
        SkillScope::Project => root.to_path_buf(),
        SkillScope::User => env::var_os("HOME")
            .map(PathBuf::from)
            .ok_or(LoopError::HomeNotFound)?,
    };
    let targets = match target {
        SkillTarget::Claude => vec![".claude"],
        SkillTarget::Codex => vec![".codex"],
        SkillTarget::Both => vec![".claude", ".codex"],
    };
    Ok(targets
        .into_iter()
        .map(|dir| base.join(dir).join("skills").join(name))
        .collect())
}
