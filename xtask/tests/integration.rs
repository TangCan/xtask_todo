//! Integration tests that run the xtask binary to cover `main` and `run()`.

use std::process::Command;

fn xtask_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_xtask"))
}

#[test]
fn xtask_run_exits_success() {
    let out = xtask_bin().arg("run").output().unwrap();
    assert!(out.status.success(), "xtask run should succeed");
    assert!(String::from_utf8_lossy(&out.stdout).contains("xtask run"));
}
