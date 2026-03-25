//! Tests for git subcommand.

use std::process::Stdio;

use crate::git::{cmd_git, GitAddArgs, GitArgs, GitCommitArgs, GitSub};
use crate::tests::{
    cwd_test_lock, dir_outside_cwd, git_available, path_test_lock, sh_available, RestoreCwd,
};
use crate::{run_with, XtaskCmd, XtaskSub};

/// Sets `GIT_DIR` and `GIT_WORK_TREE` so git uses only the given repo; restores both on drop.
struct GitRepoGuard {
    prev_dir: Option<std::ffi::OsString>,
    prev_work_tree: Option<std::ffi::OsString>,
}
impl GitRepoGuard {
    fn new(git_dir: &std::path::Path, work_tree: &std::path::Path) -> Self {
        let prev_dir = std::env::var_os("GIT_DIR");
        let prev_work_tree = std::env::var_os("GIT_WORK_TREE");
        std::env::set_var("GIT_DIR", git_dir);
        std::env::set_var("GIT_WORK_TREE", work_tree);
        Self {
            prev_dir,
            prev_work_tree,
        }
    }
}
impl Drop for GitRepoGuard {
    fn drop(&mut self) {
        match &self.prev_dir {
            Some(v) => std::env::set_var("GIT_DIR", v),
            None => std::env::remove_var("GIT_DIR"),
        }
        match &self.prev_work_tree {
            Some(v) => std::env::set_var("GIT_WORK_TREE", v),
            None => std::env::remove_var("GIT_WORK_TREE"),
        }
    }
}

#[test]
fn run_subcommand_git_add() {
    let _cwd_lock = cwd_test_lock();
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
    let _cwd_lock = cwd_test_lock();
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
    if !git_available() {
        return;
    }
    let _cwd_lock = cwd_test_lock();
    // Dir outside workspace; pin GIT_DIR and GIT_WORK_TREE so git cannot use main repo (CI).
    let dir = dir_outside_cwd("xtask_nongit");
    std::fs::create_dir_all(&dir).unwrap();
    let dir = std::fs::canonicalize(&dir).unwrap_or(dir);
    let cwd = std::env::current_dir().unwrap();
    let _restore = RestoreCwd::new(&dir, &cwd);
    let bad_git_dir = dir.join(".git_absent");
    let _env_guard = GitRepoGuard::new(&bad_git_dir, &dir);
    let cmd = GitArgs {
        sub: GitSub::Add(GitAddArgs {}),
    };
    let result = cmd_git(&cmd);
    assert!(result.is_err());
}

#[test]
fn cmd_git_pre_commit_in_nongit_dir_returns_err() {
    if !git_available() || !sh_available() {
        return;
    }
    let _cwd_lock = cwd_test_lock();
    let dir = std::env::temp_dir().join(format!("xtask_git_precommit_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _restore = RestoreCwd::new(&dir, &cwd);
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
    if !git_available() || !sh_available() {
        return;
    }
    let _cwd_lock = cwd_test_lock();
    let dir = std::env::temp_dir().join(format!("xtask_git_nohook_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _restore = RestoreCwd::new(&dir, &cwd);
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
    if !git_available() || !sh_available() {
        return;
    }
    let _cwd_lock = cwd_test_lock();
    let dir = std::env::temp_dir().join(format!("xtask_git_precommit_ok_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _restore = RestoreCwd::new(&dir, &cwd);
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
    if !git_available() {
        return;
    }
    let _cwd_lock = cwd_test_lock();
    // Dir outside workspace so git only sees this empty repo (CI temp_dir may be under workspace).
    let dir = dir_outside_cwd("xtask_git_commit");
    std::fs::create_dir_all(&dir).unwrap();
    let dir = std::fs::canonicalize(&dir).unwrap_or(dir);
    let cwd = std::env::current_dir().unwrap();
    let _restore = RestoreCwd::new(&dir, &cwd);
    let _ = std::process::Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(&dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let git_dir = dir.join(".git");
    let _env_guard = GitRepoGuard::new(&git_dir, &dir);
    let cmd = GitArgs {
        sub: GitSub::Commit(GitCommitArgs { message: None }),
    };
    let result = cmd_git(&cmd);
    assert!(result.is_err());
}

#[test]
fn cmd_git_add_when_git_missing_from_path_returns_err() {
    let _path_lock = path_test_lock();
    let old = std::env::var_os("PATH");
    std::env::set_var("PATH", "");
    let cmd = GitArgs {
        sub: GitSub::Add(GitAddArgs {}),
    };
    let r = cmd_git(&cmd);
    match old {
        Some(v) => std::env::set_var("PATH", v),
        None => std::env::remove_var("PATH"),
    }
    assert!(r.is_err(), "expected error when git is missing from PATH");
}
