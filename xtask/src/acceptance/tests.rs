use std::fs;
use std::path::PathBuf;

use super::checks::{check_cargo_xtask_alias, check_pre_commit_has_msvc, check_workspace_members};
use super::report::{build_report, manual_skip_rows};
use super::{workspace_root, AutomatedCheck, CheckStatus};

#[test]
fn workspace_root_finds_repo() {
    let root = workspace_root().expect("root");
    assert!(root.join("Cargo.toml").is_file());
    let text = fs::read_to_string(root.join("Cargo.toml")).unwrap();
    assert!(text.contains("[workspace]"));
}

#[test]
fn nf1_nf2_nf6_pass_on_repo() {
    let root = workspace_root().unwrap();
    assert!(matches!(
        check_workspace_members(&root).status,
        CheckStatus::Pass
    ));
    assert!(matches!(
        check_cargo_xtask_alias(&root).status,
        CheckStatus::Pass
    ));
    assert!(matches!(
        check_pre_commit_has_msvc(&root).status,
        CheckStatus::Pass
    ));
}

#[test]
fn build_report_all_pass_shows_success_section() {
    let root = PathBuf::from("/tmp/acceptance-test-root");
    let checks = vec![AutomatedCheck {
        id: "X",
        description: "ok",
        command: "echo".into(),
        status: CheckStatus::Pass,
    }];
    let r = build_report(&root, &checks, &[("M1", "manual")]);
    assert!(r.contains("# Acceptance report"));
    assert!(r.contains("✅ PASS"));
    assert!(r.contains("**状态**: ✅ **全部自动化检查通过**"));
    assert!(r.contains("| M1 |"));
}

#[test]
fn build_report_fail_includes_detail_block() {
    let root = PathBuf::from("/tmp");
    let checks = vec![AutomatedCheck {
        id: "F1",
        description: "bad",
        command: "cargo test".into(),
        status: CheckStatus::Fail("compile error\nline2".into()),
    }];
    let r = build_report(&root, &checks, &[]);
    assert!(r.contains("❌ FAIL"));
    assert!(r.contains("### ❌ 失败详情: F1"));
    assert!(r.contains("compile error"));
    assert!(r.contains("**状态**: ❌ **存在失败的自动化检查**"));
}

#[test]
fn build_report_truncates_very_long_command_in_table() {
    let root = PathBuf::from("/tmp");
    let long = "c".repeat(120);
    let checks = vec![AutomatedCheck {
        id: "L",
        description: "long cmd",
        command: long.clone(),
        status: CheckStatus::Pass,
    }];
    let r = build_report(&root, &checks, &[]);
    assert!(r.contains('…'));
    assert!(!r.contains(&long));
}

#[test]
fn build_report_skip_shows_reason() {
    let root = PathBuf::from("/tmp");
    let checks = vec![AutomatedCheck {
        id: "S",
        description: "skipped",
        command: "x".into(),
        status: CheckStatus::Skip("no tool".into()),
    }];
    let r = build_report(&root, &checks, &[]);
    assert!(r.contains("⏸ SKIP"));
    assert!(r.contains("no tool"));
}

#[test]
fn manual_skip_rows_non_empty() {
    assert!(!manual_skip_rows().is_empty());
}
