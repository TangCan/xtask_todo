//! Tests for clippy subcommand.

use crate::clippy::{status_to_result, ClippyArgs};
use crate::tests::{cwd_test_lock, dir_outside_cwd, RestoreCwd};
use crate::{run_with, XtaskCmd, XtaskSub};

#[test]
fn status_to_result_success() {
    let status = std::process::Command::new("true").status().unwrap();
    assert!(status_to_result(status, "test").is_ok());
}

#[test]
fn run_subcommand_clippy() {
    let _guard = cwd_test_lock();
    let cmd = XtaskCmd {
        sub: XtaskSub::Clippy(ClippyArgs {}),
    };
    let _ = run_with(cmd);
}

#[test]
fn cmd_clippy_fail_returns_err() {
    let _guard = cwd_test_lock();
    // Dir outside workspace so cargo clippy runs only in this project (CI temp_dir may be under workspace).
    let dir = dir_outside_cwd("xtask_clippy_fail");
    std::fs::create_dir_all(&dir).unwrap();
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    std::fs::create_dir_all("src").unwrap();
    std::fs::write(
        "Cargo.toml",
        r#"[package]
name = "fail"
version = "0.1.0"
edition = "2021"
[dependencies]
nonexistent_crate_xyz = "999"
"#,
    )
    .unwrap();
    std::fs::write("src/lib.rs", "pub fn f() {}").unwrap();
    std::env::set_var("XTASK_CLIPPY_QUIET", "1");
    let result = crate::clippy::cmd_clippy(ClippyArgs {});
    std::env::remove_var("XTASK_CLIPPY_QUIET");
    assert!(result.is_err());
}
