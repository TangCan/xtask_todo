use std::fs;

use crate::common::{todo_bin, xtask_bin};

/// Covers `src/bin/todo.rs` (standalone todo CLI).
#[test]
fn todo_bin_help_exits_success() {
    let out = todo_bin().arg("--help").output().unwrap();
    assert!(
        out.status.success(),
        "todo --help: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(
        s.contains("todo") || s.contains("Todo"),
        "help should mention todo: {s}"
    );
}

#[test]
fn xtask_run_exits_success() {
    let out = xtask_bin().arg("run").output().unwrap();
    assert!(out.status.success(), "xtask run should succeed");
    assert!(String::from_utf8_lossy(&out.stdout).contains("xtask run"));
}

#[test]
fn xtask_todo_list_empty_json_matches_empty_semantics() {
    let dir = std::env::temp_dir().join(format!(
        "xtask_integ_todo_list_empty_{}",
        std::process::id()
    ));
    let _ = fs::create_dir_all(&dir);
    let _ = fs::remove_file(dir.join(".todo.json"));

    let list = xtask_bin()
        .arg("todo")
        .arg("--json")
        .arg("list")
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(
        list.status.success(),
        "xtask todo --json list should succeed on empty store: {:?}",
        list.stderr
    );
    let stdout = String::from_utf8_lossy(&list.stdout);
    let line = stdout.trim();
    let v: serde_json::Value = serde_json::from_str(line).expect("valid JSON line");
    assert_eq!(v["status"], "success");
    assert_eq!(v["data"]["empty"], true);
    assert_eq!(v["data"]["message"], "No tasks.");
    assert_eq!(v["data"]["items"], serde_json::json!([]));
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
fn xtask_todo_init_ai_json_success_has_generated_true() {
    let dir = std::env::temp_dir().join(format!(
        "xtask_integ_todo_init_ai_json_{}",
        std::process::id()
    ));
    let _ = fs::create_dir_all(&dir);
    let out_dir = dir.join("commands");

    let out = xtask_bin()
        .arg("todo")
        .arg("--json")
        .arg("init-ai")
        .arg("--output")
        .arg(&out_dir)
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out.stderr);
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).expect("json");
    assert_eq!(v["status"], "success");
    assert_eq!(v["data"]["generated"], true);
}

#[test]
fn todo_bin_json_list_empty_matches_contract() {
    let dir = std::env::temp_dir().join(format!(
        "todo_bin_integ_list_empty_json_{}",
        std::process::id()
    ));
    let _ = fs::create_dir_all(&dir);
    let _ = fs::remove_file(dir.join(".todo.json"));

    let out = todo_bin()
        .arg("--json")
        .arg("list")
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out.stderr);
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).expect("json");
    assert_eq!(v["status"], "success");
    assert_eq!(v["data"]["empty"], true);
    assert_eq!(v["data"]["message"], "No tasks.");
    assert_eq!(v["data"]["items"], serde_json::json!([]));
}

#[test]
fn todo_bin_json_show_invalid_id_outputs_error_contract() {
    let dir = std::env::temp_dir().join(format!("todo_bin_integ_show_id0_{}", std::process::id()));
    let _ = fs::create_dir_all(&dir);
    let _ = fs::remove_file(dir.join(".todo.json"));

    let out = todo_bin()
        .arg("--json")
        .arg("show")
        .arg("0")
        .current_dir(&dir)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2), "{:?}", out.stderr);
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).expect("json");
    assert_eq!(v["status"], "error");
    assert_eq!(v["error"]["code"], 2);
    assert!(v["error"]["message"]
        .as_str()
        .unwrap()
        .contains("invalid id 0"));
}
