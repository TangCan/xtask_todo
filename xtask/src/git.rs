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
/// Commit with message "Sync" by default, or use -m/--message to set the message
pub struct GitCommitArgs {
    /// commit message (default: "Sync")
    #[argh(option, short = 'm')]
    pub message: Option<String>,
}

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
        GitSub::Commit(a) => {
            let msg = a.message.as_deref().unwrap_or("Sync");
            let status = Command::new("git").args(["commit", "-m", msg]).status()?;
            status_to_result(status, "git commit")
        }
    }
}
