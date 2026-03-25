use std::fs;

use crate::common::xtask_bin;

/// Story 1.4 / TC-T3-1, TC-T4-1: `todo --json complete` then `delete` success payloads.
#[test]
fn xtask_todo_json_complete_then_delete_success() {
    let dir =
        std::env::temp_dir().join(format!("xtask_integ_todo_comp_del_{}", std::process::id()));
    let _ = fs::create_dir_all(&dir);
    let _ = fs::remove_file(dir.join(".todo.json"));

    assert!(xtask_bin()
        .arg("todo")
        .arg("add")
        .arg("one task")
        .current_dir(&dir)
        .status()
        .unwrap()
        .success());

    let comp = xtask_bin()
        .arg("todo")
        .arg("--json")
        .arg("complete")
        .arg("1")
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(comp.status.success(), "{:?}", comp.stderr);
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&comp.stdout).trim()).expect("json");
    assert_eq!(v["status"], "success");
    assert_eq!(v["data"]["id"], 1);
    assert_eq!(v["data"]["completed"], true);

    let del = xtask_bin()
        .arg("todo")
        .arg("--json")
        .arg("delete")
        .arg("1")
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(del.status.success(), "{:?}", del.stderr);
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&del.stdout).trim()).expect("json");
    assert_eq!(v["status"], "success");
    assert_eq!(v["data"]["id"], 1);
    assert_eq!(v["data"]["deleted"], true);

    let list = xtask_bin()
        .arg("todo")
        .arg("--json")
        .arg("list")
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(list.status.success(), "{:?}", list.stderr);
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&list.stdout).trim()).expect("json");
    assert_eq!(v["data"]["empty"], true);

    let _ = fs::remove_file(dir.join(".todo.json"));
}

/// Story 1.4: `complete --no-next` with `--json` succeeds (flag accepted end-to-end).
#[test]
fn xtask_todo_json_complete_no_next_success() {
    let dir = std::env::temp_dir().join(format!("xtask_integ_todo_comp_nn_{}", std::process::id()));
    let _ = fs::create_dir_all(&dir);
    let _ = fs::remove_file(dir.join(".todo.json"));

    assert!(xtask_bin()
        .arg("todo")
        .arg("add")
        .arg("plain")
        .current_dir(&dir)
        .status()
        .unwrap()
        .success());

    let out = xtask_bin()
        .arg("todo")
        .arg("--json")
        .arg("complete")
        .arg("1")
        .arg("--no-next")
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out.stderr);
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).expect("json");
    assert_eq!(v["status"], "success");
    assert_eq!(v["data"]["completed"], true);

    let _ = fs::remove_file(dir.join(".todo.json"));
}

/// TC-A2-2: `complete` id 0 exits 2 from real xtask + `--json`.
#[test]
fn xtask_todo_json_complete_id_zero_exit_2() {
    let dir = std::env::temp_dir().join(format!("xtask_integ_todo_c0_{}", std::process::id()));
    let _ = fs::create_dir_all(&dir);
    let _ = fs::remove_file(dir.join(".todo.json"));

    let out = xtask_bin()
        .arg("todo")
        .arg("--json")
        .arg("complete")
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
    assert!(
        !dir.join(".todo.json").exists(),
        "parameter error must not write .todo.json"
    );
}

/// TC-A2-2: `delete` id 0 exits 2 from real xtask + `--json`.
#[test]
fn xtask_todo_json_delete_id_zero_exit_2() {
    let dir = std::env::temp_dir().join(format!("xtask_integ_todo_d0_{}", std::process::id()));
    let _ = fs::create_dir_all(&dir);
    let _ = fs::remove_file(dir.join(".todo.json"));

    let out = xtask_bin()
        .arg("todo")
        .arg("--json")
        .arg("delete")
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
    assert!(!dir.join(".todo.json").exists());
}

/// TC-A2-3: `complete` nonexistent id exits 3 from real xtask + `--json`.
#[test]
fn xtask_todo_json_complete_nonexistent_exit_3() {
    let dir = std::env::temp_dir().join(format!("xtask_integ_todo_cnx_{}", std::process::id()));
    let _ = fs::create_dir_all(&dir);
    let _ = fs::remove_file(dir.join(".todo.json"));

    assert!(xtask_bin()
        .arg("todo")
        .arg("add")
        .arg("solo")
        .current_dir(&dir)
        .status()
        .unwrap()
        .success());

    let out = xtask_bin()
        .arg("todo")
        .arg("--json")
        .arg("complete")
        .arg("999")
        .current_dir(&dir)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3), "{:?}", out.stderr);
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).expect("json");
    assert_eq!(v["status"], "error");
    assert_eq!(v["error"]["code"], 3);
    assert!(v["error"]["message"]
        .as_str()
        .unwrap()
        .contains("not found"));
}

/// TC-A2-3: `delete` nonexistent id exits 3 from real xtask + `--json`.
#[test]
fn xtask_todo_json_delete_nonexistent_exit_3() {
    let dir = std::env::temp_dir().join(format!("xtask_integ_todo_dnx_{}", std::process::id()));
    let _ = fs::create_dir_all(&dir);
    let _ = fs::remove_file(dir.join(".todo.json"));

    assert!(xtask_bin()
        .arg("todo")
        .arg("add")
        .arg("solo")
        .current_dir(&dir)
        .status()
        .unwrap()
        .success());

    let out = xtask_bin()
        .arg("todo")
        .arg("--json")
        .arg("delete")
        .arg("999")
        .current_dir(&dir)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3), "{:?}", out.stderr);
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).expect("json");
    assert_eq!(v["status"], "error");
    assert_eq!(v["error"]["code"], 3);
    assert!(v["error"]["message"]
        .as_str()
        .unwrap()
        .contains("not found"));
}

/// Story 1.4 AC4: failed `complete` (not found) must not mutate `.todo.json`.
#[test]
fn xtask_todo_json_complete_nonexistent_preserves_store() {
    let dir = std::env::temp_dir().join(format!(
        "xtask_integ_todo_c_preserve_{}",
        std::process::id()
    ));
    let _ = fs::create_dir_all(&dir);
    let _ = fs::remove_file(dir.join(".todo.json"));

    assert!(xtask_bin()
        .arg("todo")
        .arg("add")
        .arg("keep")
        .current_dir(&dir)
        .status()
        .unwrap()
        .success());

    let path = dir.join(".todo.json");
    let before = fs::read_to_string(&path).expect("store after add");

    let out = xtask_bin()
        .arg("todo")
        .arg("--json")
        .arg("complete")
        .arg("42")
        .current_dir(&dir)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3), "{:?}", out.stderr);

    let after = fs::read_to_string(&path).expect("store after failed complete");
    assert_eq!(before, after, "failed complete must not corrupt store");

    let _ = fs::remove_file(&path);
}

/// Story 1.4 AC4: failed `delete` (not found) must not mutate `.todo.json`.
#[test]
fn xtask_todo_json_delete_nonexistent_preserves_store() {
    let dir = std::env::temp_dir().join(format!(
        "xtask_integ_todo_d_preserve_{}",
        std::process::id()
    ));
    let _ = fs::create_dir_all(&dir);
    let _ = fs::remove_file(dir.join(".todo.json"));

    assert!(xtask_bin()
        .arg("todo")
        .arg("add")
        .arg("keep")
        .current_dir(&dir)
        .status()
        .unwrap()
        .success());

    let path = dir.join(".todo.json");
    let before = fs::read_to_string(&path).expect("store after add");

    let out = xtask_bin()
        .arg("todo")
        .arg("--json")
        .arg("delete")
        .arg("42")
        .current_dir(&dir)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3), "{:?}", out.stderr);

    let after = fs::read_to_string(&path).expect("store after failed delete");
    assert_eq!(before, after, "failed delete must not corrupt store");

    let _ = fs::remove_file(&path);
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
