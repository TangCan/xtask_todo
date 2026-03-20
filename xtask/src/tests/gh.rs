//! Tests for gh subcommand.

use crate::gh::{cmd_gh, GhArgs, GhLogArgs, GhSub};
use crate::tests::path_test_lock;
use crate::{run_with, XtaskCmd, XtaskSub};

/// Run `cmd_gh` (gh log). When `gh` is not in PATH we get Err and cover error branches.
#[test]
fn cmd_gh_log_errors_when_gh_not_found() {
    let _path = path_test_lock();
    let path = std::env::var_os("PATH");
    std::env::set_var("PATH", "");
    let args = GhArgs {
        sub: GhSub::Log(GhLogArgs {}),
    };
    let r = cmd_gh(&args);
    if let Some(p) = path {
        std::env::set_var("PATH", p);
    } else {
        std::env::remove_var("PATH");
    }
    assert!(r.is_err(), "expected Err when gh not in PATH: {r:?}");
}

#[test]
fn run_with_gh_returns_run_failure_when_gh_fails() {
    let _path = path_test_lock();
    let path = std::env::var_os("PATH");
    std::env::set_var("PATH", "");
    let cmd = XtaskCmd {
        sub: XtaskSub::Gh(GhArgs {
            sub: GhSub::Log(GhLogArgs {}),
        }),
    };
    let r = run_with(cmd);
    if let Some(p) = path {
        std::env::set_var("PATH", p);
    } else {
        std::env::remove_var("PATH");
    }
    assert!(r.is_err(), "expected RunFailure when gh not in PATH: {r:?}");
}

/// When `gh` exists but `gh run list` fails (e.g. exit 1), we cover the "list failed" branch.
#[cfg(unix)]
#[test]
fn cmd_gh_log_errors_when_gh_run_list_fails() {
    use std::os::unix::fs::PermissionsExt;
    let _path = path_test_lock();
    let path = std::env::var_os("PATH");
    let dir = std::env::temp_dir().join(format!("xtask_gh_fake_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let gh_script = dir.join("gh");
    std::fs::write(&gh_script, "#!/bin/sh\nexit 1\n").unwrap();
    let mut perms = std::fs::metadata(&gh_script).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&gh_script, perms).unwrap();
    std::env::set_var("PATH", &dir);
    let args = GhArgs {
        sub: GhSub::Log(GhLogArgs {}),
    };
    let r = cmd_gh(&args);
    if let Some(p) = path {
        std::env::set_var("PATH", p);
    } else {
        std::env::remove_var("PATH");
    }
    let _ = std::fs::remove_dir_all(&dir);
    assert!(r.is_err(), "expected Err when gh run list fails: {r:?}");
}
