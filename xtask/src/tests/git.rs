//! Tests for git subcommand.

use std::process::Stdio;

use crate::git::{cmd_git, GitAddArgs, GitArgs, GitCommitArgs, GitSub};
use crate::tests::{RestoreCwd, CWD_TEST_MUTEX};
use crate::{run_with, XtaskCmd, XtaskSub};

#[test]
fn run_subcommand_git_add() {
    let _guard = CWD_TEST_MUTEX.lock().unwrap();
    let dir = std::env::temp_dir().join(format!("xtask_git_add_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _restore = RestoreCwd::new(&dir, &cwd);
    let _ = std::process::Command::new("git")
        .args(["init", "-b", "main"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let cmd = XtaskCmd {
        sub: XtaskSub::Git(GitArgs {
            sub: GitSub::Add(GitAddArgs {}),
        }),
    };
    let _ = run_with(cmd);
}

#[test]
fn run_subcommand_git_commit() {
    let _guard = CWD_TEST_MUTEX.lock().unwrap();
    let dir = std::env::temp_dir().join(format!("xtask_git_commit_run_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _restore = RestoreCwd::new(&dir, &cwd);
    let _ = std::process::Command::new("git")
        .args(["init", "-b", "main"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let cmd = XtaskCmd {
        sub: XtaskSub::Git(GitArgs {
            sub: GitSub::Commit(GitCommitArgs {}),
        }),
    };
    let _ = run_with(cmd);
}

#[test]
fn cmd_git_add_in_nongit_dir_returns_err() {
    let _guard = CWD_TEST_MUTEX.lock().unwrap();
    let dir = std::env::temp_dir().join(format!("xtask_nongit_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let cmd = GitArgs {
        sub: GitSub::Add(GitAddArgs {}),
    };
    let result = cmd_git(&cmd);
    assert!(result.is_err());
}

#[test]
fn cmd_git_commit_with_nothing_to_commit_returns_err() {
    let _guard = CWD_TEST_MUTEX.lock().unwrap();
    let dir = std::env::temp_dir().join(format!("xtask_git_commit_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let _ = std::process::Command::new("git")
        .args(["init", "-b", "main"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let cmd = GitArgs {
        sub: GitSub::Commit(GitCommitArgs {}),
    };
    let result = cmd_git(&cmd);
    assert!(result.is_err());
}
