//! `git` subcommand - stage common paths, commit with message.

use argh::FromArgs;
use std::process::Command;

use crate::clippy::status_to_result;

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "git")]
/// Git helpers (e.g. stage common paths)
pub struct GitArgs {
    #[argh(subcommand)]
    pub sub: GitSub,
}

#[derive(FromArgs, Clone)]
#[argh(subcommand)]
pub enum GitSub {
    Add(GitAddArgs),
    Commit(GitCommitArgs),
}

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "add")]
/// Stage .github, .specstory, xtask, docs (equiv: git add .github .specstory xtask docs)
pub struct GitAddArgs {}

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "commit")]
/// Commit with message "Sync" (equiv: git commit -m "Sync")
pub struct GitCommitArgs {}

/// Run git subcommand (add or commit).
///
/// # Errors
/// Returns an error if the git command exits with a non-zero status.
pub fn cmd_git(args: &GitArgs) -> Result<(), Box<dyn std::error::Error>> {
    match &args.sub {
        GitSub::Add(_) => {
            let status = Command::new("git")
                .args([
                    "add",
                    ".github",
                    ".specstory",
                    "xtask",
                    "docs",
                    "crates",
                    ".githooks",
                    "openspec",
                    "README.md",
                ])
                .status()?;
            status_to_result(status, "git add")
        }
        GitSub::Commit(_) => {
            let status = Command::new("git")
                .args(["commit", "-m", "Sync"])
                .status()?;
            status_to_result(status, "git commit")
        }
    }
}
