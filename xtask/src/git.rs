//! `git` subcommand - stage common paths, pre-commit checks, commit with message.

use argh::FromArgs;
use std::path::PathBuf;
use std::process::Command;

use crate::clippy::status_to_result;

/// Runs the same checks as `.githooks/pre-commit` (fmt, clippy, rustdoc, .rs line limit, test,
/// `xtask-todo-lib` Windows MSVC cross-compile) without committing.
fn run_pre_commit_checks() -> Result<(), Box<dyn std::error::Error>> {
    let root = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()?
        .stdout;
    let root = String::from_utf8(root)
        .map_err(|_| "git rev-parse produced invalid UTF-8")?
        .trim_end()
        .to_string();
    if root.is_empty() {
        return Err("not a git repository".into());
    }
    let hook = PathBuf::from(&root).join(".githooks").join("pre-commit");
    if !hook.exists() {
        return Err(format!("pre-commit hook not found: {}", hook.display()).into());
    }
    let status = Command::new("sh")
        .arg(hook.as_os_str())
        .current_dir(&root)
        .status()?;
    status_to_result(status, "pre-commit")
}

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
    PreCommit(GitPreCommitArgs),
    Commit(GitCommitArgs),
}

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "pre-commit")]
/// Run the same checks as the pre-commit hook (fmt, clippy, rustdoc, .rs line limit, test,
/// `cargo check -p xtask-todo-lib --target x86_64-pc-windows-msvc`) without committing
pub struct GitPreCommitArgs {}

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "add")]
/// Stage common project paths (docs, xtask, crates, hooks, and related metadata files)
pub struct GitAddArgs {}

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "commit")]
/// Commit with message "Sync" by default, or use -m/--message to set the message
pub struct GitCommitArgs {
    /// commit message (default: "Sync")
    #[argh(option, short = 'm')]
    pub message: Option<String>,
}

/// Run git subcommand (add, pre-commit, or commit).
///
/// # Errors
/// Returns an error if the git command or pre-commit checks exit with a non-zero status.
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
                    ".cursor",
                    "Cargo.toml",
                    "_bmad-output",
                ])
                .status()?;
            status_to_result(status, "git add")
        }
        GitSub::PreCommit(_) => run_pre_commit_checks(),
        GitSub::Commit(a) => {
            let msg = a.message.as_deref().unwrap_or("Sync");
            let status = Command::new("git").args(["commit", "-m", msg]).status()?;
            status_to_result(status, "git commit")
        }
    }
}
