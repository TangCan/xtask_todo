//! Tests for run, coverage, fmt subcommands and todo crate API.

use crate::coverage::{cmd_coverage, parse_coverage_percentage, CoverageArgs};
use crate::fmt::{cmd_fmt, FmtArgs};
use crate::publish::PublishArgs;
use crate::run::RunArgs;
use crate::tests::{cwd_test_lock, RestoreCwd};
use crate::todo::{todo_args, TodoArgs, TodoCompleteArgs, TodoListArgs, TodoSub};
use crate::{run_with, XtaskCmd, XtaskSub};
use xtask_todo_lib::{InMemoryStore, TodoId, TodoList};

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
        .complete(bad_id, false)
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
fn run_with_todo_parameter_error_returns_run_failure_exit_code_2() {
    let _guard = cwd_test_lock();
    let dir = std::env::temp_dir().join(format!("xtask_run_todo_err_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let _ = std::fs::remove_file(".todo.json");
    let cmd = XtaskCmd {
        sub: XtaskSub::Todo(TodoArgs {
            sub: TodoSub::Complete(TodoCompleteArgs {
                id: 0,
                no_next: false,
            }),
            json: false,
            dry_run: false,
        }),
    };
    let out = run_with(cmd);
    let err = out.unwrap_err();
    assert_eq!(err.code, 2);
    assert!(err.message.contains("invalid id 0"));
}

#[test]
fn run_with_todo_data_error_returns_run_failure_exit_code_3() {
    let _guard = cwd_test_lock();
    let dir = std::env::temp_dir().join(format!("xtask_run_todo_data_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let _ = std::fs::remove_file(".todo.json");
    let cmd = XtaskCmd {
        sub: XtaskSub::Todo(TodoArgs {
            sub: TodoSub::Show(crate::todo::TodoShowArgs { id: 99 }),
            json: false,
            dry_run: false,
        }),
    };
    let out = run_with(cmd);
    let err = out.unwrap_err();
    assert_eq!(err.code, 3);
    assert!(err.message.contains("not found"));
}

#[test]
fn run_with_todo_parameter_error_with_json_returns_run_failure_and_prints_json() {
    let _guard = cwd_test_lock();
    let dir = std::env::temp_dir().join(format!("xtask_run_todo_json_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let cwd = std::env::current_dir().unwrap();
    let _restore = RestoreCwd::new(&dir, &cwd);
    let _ = std::fs::remove_file(".todo.json");
    let cmd = XtaskCmd {
        sub: XtaskSub::Todo(TodoArgs {
            sub: TodoSub::Complete(TodoCompleteArgs {
                id: 0,
                no_next: false,
            }),
            json: true,
            dry_run: false,
        }),
    };
    let out = run_with(cmd);
    let err = out.unwrap_err();
    assert_eq!(err.code, 2);
}

#[test]
fn run_subcommand_publish_returns_err_outside_workspace() {
    let _guard = cwd_test_lock();
    let dir = std::env::temp_dir().join(format!("xtask_publish_run_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _restore = RestoreCwd::new(&dir, &cwd);
    let cmd = XtaskCmd {
        sub: XtaskSub::Publish(PublishArgs {}),
    };
    let out = run_with(cmd);
    assert!(out.is_err());
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
    let _guard = cwd_test_lock();
    std::env::set_var("XTASK_COVERAGE_TEST_FAKE", "1");
    let out = cmd_coverage(CoverageArgs {});
    std::env::remove_var("XTASK_COVERAGE_TEST_FAKE");
    assert!(out.is_ok());
}

#[test]
fn cmd_coverage_fake_fail_returns_err() {
    let _guard = cwd_test_lock();
    std::env::remove_var("XTASK_COVERAGE_TEST_FAKE");
    std::env::set_var("XTASK_COVERAGE_TEST_FAKE_FAIL", "1");
    let out = cmd_coverage(CoverageArgs {});
    std::env::remove_var("XTASK_COVERAGE_TEST_FAKE_FAIL");
    assert!(out.is_err());
}

#[test]
fn run_subcommand_fmt() {
    let _guard = cwd_test_lock();
    let cmd = XtaskCmd {
        sub: XtaskSub::Fmt(FmtArgs {}),
    };
    let _ = run_with(cmd);
}

#[test]
fn run_subcommand_todo_list() {
    let _guard = cwd_test_lock();
    let dir = std::env::temp_dir().join(format!("xtask_todo_run_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _restore = RestoreCwd::new(&dir, &cwd);
    let _ = std::fs::remove_file(".todo.json");
    let cmd = XtaskCmd {
        sub: XtaskSub::Todo(todo_args(TodoSub::List(TodoListArgs::default()))),
    };
    let out = run_with(cmd);
    assert!(out.is_ok());
}

#[test]
fn cmd_fmt_in_nocargo_dir_returns_err() {
    let _guard = cwd_test_lock();
    let dir = std::env::temp_dir().join(format!("xtask_fmt_fail_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _restore = RestoreCwd::new(&dir, &cwd);
    std::env::set_var("XTASK_FMT_QUIET", "1");
    let result = cmd_fmt(FmtArgs {});
    std::env::remove_var("XTASK_FMT_QUIET");
    assert!(result.is_err());
}
