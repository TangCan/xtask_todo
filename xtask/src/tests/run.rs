//! Tests for run, coverage, fmt subcommands and todo crate API.

use crate::coverage::{cmd_coverage, parse_coverage_percentage, CoverageArgs};
use crate::fmt::{cmd_fmt, FmtArgs};
use crate::run::RunArgs;
use crate::tests::{RestoreCwd, CWD_TEST_MUTEX};
use crate::{run_with, XtaskCmd, XtaskSub};
use todo::{InMemoryStore, TodoId, TodoList};

#[test]
fn todo_crate_api_coverage() {
    let list = TodoList::default();
    assert!(list.list().is_empty());
    let mut list2 = TodoList::with_store(InMemoryStore::new());
    let id = list2.create("ok").unwrap();
    assert_eq!(list2.list().len(), 1);
    assert!(list2
        .create("")
        .unwrap_err()
        .to_string()
        .contains("invalid input"));
    let bad_id = TodoId::from_raw(999).unwrap();
    assert!(list2
        .complete(bad_id)
        .unwrap_err()
        .to_string()
        .contains("not found"));
    assert!(list2
        .delete(bad_id)
        .unwrap_err()
        .to_string()
        .contains("not found"));
    list2.delete(id).unwrap();
    assert!(list2.list().is_empty());
    let store = InMemoryStore::from_todos(vec![]);
    let mut list3 = TodoList::with_store(store);
    let _ = list3.create("first").unwrap();
    assert_eq!(list3.list().len(), 1);
}

#[test]
fn parse_coverage_percentage_extracts_pct() {
    assert_eq!(
        parse_coverage_percentage("|| 100.00% coverage, 61/61 lines covered"),
        Some(100.0)
    );
    assert_eq!(
        parse_coverage_percentage("72.33% coverage, 183/253 lines covered"),
        Some(72.33)
    );
    assert!(parse_coverage_percentage("no number here").is_none());
}

#[test]
fn run_subcommand_run() {
    let cmd = XtaskCmd {
        sub: XtaskSub::Run(RunArgs {}),
    };
    let out = run_with(cmd);
    assert!(out.is_ok());
}

#[test]
fn run_subcommand_coverage() {
    std::env::set_var("XTASK_COVERAGE_TEST_FAKE", "1");
    let cmd = XtaskCmd {
        sub: XtaskSub::Coverage(CoverageArgs {}),
    };
    let out = run_with(cmd);
    std::env::remove_var("XTASK_COVERAGE_TEST_FAKE");
    assert!(out.is_ok());
}

#[test]
fn cmd_coverage_fake_success() {
    let _guard = CWD_TEST_MUTEX.lock().unwrap();
    std::env::set_var("XTASK_COVERAGE_TEST_FAKE", "1");
    let out = cmd_coverage(CoverageArgs {});
    std::env::remove_var("XTASK_COVERAGE_TEST_FAKE");
    assert!(out.is_ok());
}

#[test]
fn cmd_coverage_fake_fail_returns_err() {
    let _guard = CWD_TEST_MUTEX.lock().unwrap();
    std::env::remove_var("XTASK_COVERAGE_TEST_FAKE");
    std::env::set_var("XTASK_COVERAGE_TEST_FAKE_FAIL", "1");
    let out = cmd_coverage(CoverageArgs {});
    std::env::remove_var("XTASK_COVERAGE_TEST_FAKE_FAIL");
    assert!(out.is_err());
}

#[test]
fn run_subcommand_fmt() {
    let _guard = CWD_TEST_MUTEX.lock().unwrap();
    let cmd = XtaskCmd {
        sub: XtaskSub::Fmt(FmtArgs {}),
    };
    let _ = run_with(cmd);
}

#[test]
fn cmd_fmt_in_nocargo_dir_returns_err() {
    let _guard = CWD_TEST_MUTEX.lock().unwrap();
    let dir = std::env::temp_dir().join(format!("xtask_fmt_fail_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _restore = RestoreCwd::new(&dir, &cwd);
    std::env::set_var("XTASK_FMT_QUIET", "1");
    let result = cmd_fmt(FmtArgs {});
    std::env::remove_var("XTASK_FMT_QUIET");
    assert!(result.is_err());
}
