//! Tests for git subcommand.

use std::process::Stdio;

use crate::git::{cmd_git, GitAddArgs, GitArgs, GitCommitArgs, GitSub};
use crate::tests::{cwd_test_lock, RestoreCwd};
use crate::{run_with, XtaskCmd, XtaskSub};

/// Sets `GIT_DIR` to the given path for the duration of the guard; restores the previous value on drop.
struct GitDirGuard(Option<std::ffi::OsString>);
impl GitDirGuard {
    fn new(git_dir: &std::path::Path) -> Self {
        let prev = std::env::var_os("GIT_DIR");
        std::env::set_var("GIT_DIR", git_dir);
        Self(prev)
    }
}
impl Drop for GitDirGuard {
    fn drop(&mut self) {
        match &self.0 {
            Some(v) => std::env::set_var("GIT_DIR", v),
            None => std::env::remove_var("GIT_DIR"),
        }
    }
}

#[test]
fn run_subcommand_git_add() {
    let _guard = cwd_test_lock();
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
    let _guard = cwd_test_lock();
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
            sub: GitSub::Commit(GitCommitArgs { message: None }),
        }),
    };
    let _ = run_with(cmd);
}

#[test]
fn cmd_git_add_in_nongit_dir_returns_err() {
    let _guard = cwd_test_lock();
    // Use system temp dir so we're definitely outside the workspace (CI may run from xtask/ or target/).
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
fn cmd_git_pre_commit_in_nongit_dir_returns_err() {
    let _guard = cwd_test_lock();
    let dir = std::env::temp_dir().join(format!("xtask_git_precommit_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let cmd = GitArgs {
        sub: GitSub::PreCommit(crate::git::GitPreCommitArgs {}),
    };
    let result = cmd_git(&cmd);
    assert!(result.is_err());
    let s = result.unwrap_err().to_string();
    assert!(
        s.contains("not a git repository") || s.contains("pre-commit hook not found"),
        "got: {s}"
    );
}

#[test]
fn cmd_git_pre_commit_in_repo_without_hook_returns_err() {
    let _guard = cwd_test_lock();
    let dir = std::env::temp_dir().join(format!("xtask_git_nohook_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let _ = std::process::Command::new("git")
        .args(["init", "-b", "main"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let cmd = GitArgs {
        sub: GitSub::PreCommit(crate::git::GitPreCommitArgs {}),
    };
    let result = cmd_git(&cmd);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("pre-commit hook not found"));
}

#[test]
fn cmd_git_pre_commit_in_repo_with_hook_succeeds() {
    let _guard = cwd_test_lock();
    let dir = std::env::temp_dir().join(format!("xtask_git_precommit_ok_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let _ = std::process::Command::new("git")
        .args(["init", "-b", "main"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let githooks = dir.join(".githooks");
    let _ = std::fs::create_dir_all(&githooks);
    let hook = githooks.join("pre-commit");
    std::fs::write(&hook, "#!/bin/sh\nexit 0\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&hook).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&hook, perms).unwrap();
    }
    let cmd = GitArgs {
        sub: GitSub::PreCommit(crate::git::GitPreCommitArgs {}),
    };
    let result = cmd_git(&cmd);
    assert!(result.is_ok(), "pre-commit with hook: {result:?}");
}

#[test]
fn cmd_git_commit_with_nothing_to_commit_returns_err() {
    let _guard = cwd_test_lock();
    let dir = std::env::temp_dir().join(format!("xtask_git_commit_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let _ = std::process::Command::new("git")
        .args(["init", "-b", "main"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let git_dir = dir.join(".git");
    // Pin git to this repo so CI (where cwd or TMPDIR may be under workspace) still uses our empty repo.
    let _env_guard = GitDirGuard::new(&git_dir);
    let cmd = GitArgs {
        sub: GitSub::Commit(GitCommitArgs { message: None }),
    };
    let result = cmd_git(&cmd);
    assert!(result.is_err());
}
