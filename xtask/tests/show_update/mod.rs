//! Story 1.5: `show` / `update` end-to-end via `cargo xtask todo`.

use std::fs;

use crate::common::xtask_bin;

/// TC-T7-1: `--json show` success payload shape.
#[test]
fn xtask_todo_json_show_success() {
    let dir = std::env::temp_dir().join(format!("xtask_integ_todo_show_ok_{}", std::process::id()));
    let _ = fs::create_dir_all(&dir);
    let _ = fs::remove_file(dir.join(".todo.json"));

    assert!(xtask_bin()
        .arg("todo")
        .arg("add")
        .arg("detail task")
        .arg("--description")
        .arg("desc")
        .arg("--due-date")
        .arg("2026-05-01")
        .current_dir(&dir)
        .status()
        .unwrap()
        .success());

    let out = xtask_bin()
        .arg("todo")
        .arg("--json")
        .arg("show")
        .arg("1")
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out.stderr);
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).expect("json");
    assert_eq!(v["status"], "success");
    assert_eq!(v["data"]["id"], 1);
    assert_eq!(v["data"]["title"], "detail task");
    assert_eq!(v["data"]["description"], "desc");
    assert_eq!(v["data"]["due_date"], "2026-05-01");
    assert_eq!(v["data"]["completed"], false);

    let _ = fs::remove_file(dir.join(".todo.json"));
}

/// TC-A2-2: `show` id 0 exits 2 + error JSON.
#[test]
fn xtask_todo_json_show_id_zero_exit_2() {
    let dir = std::env::temp_dir().join(format!("xtask_integ_todo_sh0_{}", std::process::id()));
    let _ = fs::create_dir_all(&dir);
    let _ = fs::remove_file(dir.join(".todo.json"));

    let out = xtask_bin()
        .arg("todo")
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
}

/// TC-A2-3 / TC-T7-2: `show` nonexistent id exits 3.
#[test]
fn xtask_todo_json_show_nonexistent_exit_3() {
    let dir = std::env::temp_dir().join(format!("xtask_integ_todo_shnx_{}", std::process::id()));
    let _ = fs::create_dir_all(&dir);
    let _ = fs::remove_file(dir.join(".todo.json"));

    assert!(xtask_bin()
        .arg("todo")
        .arg("add")
        .arg("only")
        .current_dir(&dir)
        .status()
        .unwrap()
        .success());

    let out = xtask_bin()
        .arg("todo")
        .arg("--json")
        .arg("show")
        .arg("404")
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

    let _ = fs::remove_file(dir.join(".todo.json"));
}

/// TC-T8-1: `update` success with `--json`.
#[test]
fn xtask_todo_json_update_success() {
    let dir = std::env::temp_dir().join(format!("xtask_integ_todo_upd_ok_{}", std::process::id()));
    let _ = fs::create_dir_all(&dir);
    let _ = fs::remove_file(dir.join(".todo.json"));

    assert!(xtask_bin()
        .arg("todo")
        .arg("add")
        .arg("before")
        .current_dir(&dir)
        .status()
        .unwrap()
        .success());

    let out = xtask_bin()
        .arg("todo")
        .arg("--json")
        .arg("update")
        .arg("1")
        .arg("after")
        .arg("--due-date")
        .arg("2026-07-15")
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out.stderr);
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).expect("json");
    assert_eq!(v["status"], "success");
    assert_eq!(v["data"]["id"], 1);
    assert_eq!(v["data"]["title"], "after");

    let show = xtask_bin()
        .arg("todo")
        .arg("--json")
        .arg("show")
        .arg("1")
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(show.status.success(), "{:?}", show.stderr);
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&show.stdout).trim()).expect("json");
    assert_eq!(v["data"]["due_date"], "2026-07-15");

    let _ = fs::remove_file(dir.join(".todo.json"));
}

/// Story 1.1 pattern: invalid optional on `update` does not alter `.todo.json`.
#[test]
fn xtask_todo_json_update_invalid_optional_preserves_store() {
    let dir = std::env::temp_dir().join(format!("xtask_integ_todo_upd_bad_{}", std::process::id()));
    let _ = fs::create_dir_all(&dir);
    let _ = fs::remove_file(dir.join(".todo.json"));

    assert!(xtask_bin()
        .arg("todo")
        .arg("add")
        .arg("seed")
        .current_dir(&dir)
        .status()
        .unwrap()
        .success());
    let before = fs::read_to_string(dir.join(".todo.json")).unwrap();

    let out = xtask_bin()
        .arg("todo")
        .arg("update")
        .arg("1")
        .arg("next")
        .arg("--due-date")
        .arg("not-a-date")
        .current_dir(&dir)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2), "{:?}", out.stderr);
    let after = fs::read_to_string(dir.join(".todo.json")).unwrap();
    assert_eq!(after, before);

    let _ = fs::remove_file(dir.join(".todo.json"));
}

/// Story 1.5 AC4: `--dry-run update` with nonexistent id exits 3 (aligned with `complete`).
#[test]
fn xtask_todo_json_update_dry_run_nonexistent_exit_3() {
    let dir = std::env::temp_dir().join(format!(
        "xtask_integ_todo_upd_dry_nx_{}",
        std::process::id()
    ));
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
        .arg("--dry-run")
        .arg("update")
        .arg("99")
        .arg("ghost")
        .current_dir(&dir)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3), "{:?}", out.stderr);
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).expect("json");
    assert_eq!(v["status"], "error");
    assert_eq!(v["error"]["code"], 3);

    let _ = fs::remove_file(dir.join(".todo.json"));
}
