//! Integration tests that run the xtask binary to cover `main` and `run()`.

use std::fs;
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

#[test]
fn xtask_todo_add_then_list_shows_task() {
    let dir = std::env::temp_dir().join("xtask_integ_todo");
    let _ = fs::create_dir_all(&dir);
    let _ = fs::remove_file(dir.join(".todo.json"));

    let add = xtask_bin()
        .arg("todo")
        .arg("add")
        .arg("integration test task")
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(
        add.status.success(),
        "xtask todo add should succeed: {:?}",
        add.stderr
    );
    assert!(
        String::from_utf8_lossy(&add.stdout).contains("integration test task"),
        "stdout should contain task title"
    );

    let list = xtask_bin()
        .arg("todo")
        .arg("list")
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(list.status.success(), "xtask todo list should succeed");
    let out = String::from_utf8_lossy(&list.stdout);
    assert!(
        out.contains("integration test task"),
        "list should show the task: {out}"
    );
    assert!(
        out.contains("[1]") || out.contains("[2]") || out.contains("[3]"),
        "list should show an id"
    );

    let _ = fs::remove_file(dir.join(".todo.json"));
}

#[test]
fn xtask_todo_add_with_repeat_options_then_list() {
    let dir = std::env::temp_dir().join("xtask_integ_todo_repeat");
    let _ = fs::create_dir_all(&dir);
    let _ = fs::remove_file(dir.join(".todo.json"));

    let add = xtask_bin()
        .arg("todo")
        .arg("add")
        .arg("weekly review")
        .arg("--repeat-rule")
        .arg("weekly")
        .arg("--repeat-until")
        .arg("2026-12-31")
        .arg("--repeat-count")
        .arg("3")
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(
        add.status.success(),
        "add with repeat options should succeed: {}",
        String::from_utf8_lossy(&add.stderr)
    );

    let list = xtask_bin()
        .arg("todo")
        .arg("list")
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(list.status.success(), "list should succeed");
    let out = String::from_utf8_lossy(&list.stdout);
    assert!(
        out.contains("weekly review"),
        "list should show the task: {out}"
    );

    let _ = fs::remove_file(dir.join(".todo.json"));
}
