//! Process-level checks for `cargo xtask ghcr` (CLI wiring + error path).

use crate::common::xtask_bin;

#[test]
fn ghcr_invalid_source_exits_failure_with_message() {
    let out = xtask_bin()
        .args(["ghcr", "--source", "nope"])
        .output()
        .expect("spawn xtask ghcr");
    assert!(
        !out.status.success(),
        "expected non-zero exit for invalid --source: {:?}",
        out.stderr
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(
        combined.contains("invalid --source") || combined.contains("invalid"),
        "expected invalid --source hint, got: {combined}"
    );
}

#[test]
fn ghcr_help_succeeds() {
    let out = xtask_bin()
        .args(["ghcr", "--help"])
        .output()
        .expect("spawn");
    assert!(out.status.success(), "{:?}", out.stderr);
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("ghcr") || s.contains("source"), "help: {s}");
}
