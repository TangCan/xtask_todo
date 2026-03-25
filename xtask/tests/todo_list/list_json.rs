use std::fs;

use crate::common::xtask_bin;

/// TC-T9-2 / Story 1.3: `todo --json list --sort due-date` returns items in due-date order.
#[test]
fn xtask_todo_list_json_sort_due_date_orders_items() {
    let dir = std::env::temp_dir().join(format!(
        "xtask_integ_todo_list_json_sort_{}",
        std::process::id()
    ));
    let _ = fs::create_dir_all(&dir);
    let _ = fs::remove_file(dir.join(".todo.json"));

    assert!(xtask_bin()
        .arg("todo")
        .arg("add")
        .arg("later")
        .arg("--due-date")
        .arg("2026-02-01")
        .current_dir(&dir)
        .status()
        .unwrap()
        .success());
    assert!(xtask_bin()
        .arg("todo")
        .arg("add")
        .arg("earlier")
        .arg("--due-date")
        .arg("2026-01-01")
        .current_dir(&dir)
        .status()
        .unwrap()
        .success());

    let list = xtask_bin()
        .arg("todo")
        .arg("--json")
        .arg("list")
        .arg("--sort")
        .arg("due-date")
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(
        list.status.success(),
        "xtask todo --json list --sort due-date: {:?}",
        list.stderr
    );
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&list.stdout).trim()).expect("json");
    assert_eq!(v["status"], "success");
    assert_eq!(v["data"]["empty"], false);
    let items = v["data"]["items"].as_array().expect("items array");
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["title"], "earlier");
    assert_eq!(items[0]["due_date"], "2026-01-01");
    assert_eq!(items[1]["title"], "later");
    assert_eq!(items[1]["due_date"], "2026-02-01");

    let _ = fs::remove_file(dir.join(".todo.json"));
}

/// Story 1.3: filtered list with no matches uses same empty JSON shape as Story 1.2.
#[test]
fn xtask_todo_list_json_filter_no_match_empty_payload() {
    let dir = std::env::temp_dir().join(format!(
        "xtask_integ_todo_list_json_empty_f_{}",
        std::process::id()
    ));
    let _ = fs::create_dir_all(&dir);
    let _ = fs::remove_file(dir.join(".todo.json"));

    assert!(xtask_bin()
        .arg("todo")
        .arg("add")
        .arg("only")
        .arg("--tags")
        .arg("alpha")
        .current_dir(&dir)
        .status()
        .unwrap()
        .success());

    let list = xtask_bin()
        .arg("todo")
        .arg("--json")
        .arg("list")
        .arg("--tags")
        .arg("beta")
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(list.status.success(), "{:?}", list.stderr);
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&list.stdout).trim()).expect("json");
    assert_eq!(v["data"]["empty"], true);
    assert_eq!(v["data"]["message"], "No tasks.");
    assert_eq!(v["data"]["items"], serde_json::json!([]));

    let _ = fs::remove_file(dir.join(".todo.json"));
}

/// TC-T9-3: invalid `--status` on list exits 2 from real xtask invocation.
#[test]
fn xtask_todo_list_invalid_status_exit_code_2() {
    let dir = std::env::temp_dir().join(format!(
        "xtask_integ_todo_bad_list_st_{}",
        std::process::id()
    ));
    let _ = fs::create_dir_all(&dir);
    let _ = fs::remove_file(dir.join(".todo.json"));

    let out = xtask_bin()
        .arg("todo")
        .arg("--json")
        .arg("list")
        .arg("--status")
        .arg("bogus")
        .current_dir(&dir)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2), "{:?}", out.stderr);
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).expect("json");
    assert_eq!(v["status"], "error");
    assert_eq!(v["error"]["code"], 2);
    assert!(
        !dir.join(".todo.json").exists(),
        "invalid list must not create or write .todo.json"
    );
}

/// TC-DATE-2: invalid `--due-before` on `list` exits 2 from real xtask invocation.
#[test]
fn xtask_todo_list_invalid_due_before_exit_code_2() {
    let dir =
        std::env::temp_dir().join(format!("xtask_integ_todo_bad_due_b_{}", std::process::id()));
    let _ = fs::create_dir_all(&dir);
    let _ = fs::remove_file(dir.join(".todo.json"));

    let out = xtask_bin()
        .arg("todo")
        .arg("--json")
        .arg("list")
        .arg("--due-before")
        .arg("2026/01/01")
        .current_dir(&dir)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2), "{:?}", out.stderr);
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).expect("json");
    assert_eq!(v["status"], "error");
    assert_eq!(v["error"]["code"], 2);
    assert!(
        !dir.join(".todo.json").exists(),
        "invalid list must not create or write .todo.json"
    );
}

/// TC-DATE-2: invalid `--due-after` on `list` exits 2 from real xtask invocation.
#[test]
fn xtask_todo_list_invalid_due_after_exit_code_2() {
    let dir =
        std::env::temp_dir().join(format!("xtask_integ_todo_bad_due_a_{}", std::process::id()));
    let _ = fs::create_dir_all(&dir);
    let _ = fs::remove_file(dir.join(".todo.json"));

    let out = xtask_bin()
        .arg("todo")
        .arg("--json")
        .arg("list")
        .arg("--due-after")
        .arg("not-a-date")
        .current_dir(&dir)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2), "{:?}", out.stderr);
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).expect("json");
    assert_eq!(v["status"], "error");
    assert_eq!(v["error"]["code"], 2);
    assert!(
        !dir.join(".todo.json").exists(),
        "invalid list must not create or write .todo.json"
    );
}

/// Invalid `list` parameters must not mutate an existing `.todo.json` (AC2).
#[test]
fn xtask_todo_list_invalid_status_preserves_existing_store() {
    let dir = std::env::temp_dir().join(format!(
        "xtask_integ_todo_bad_list_preserve_{}",
        std::process::id()
    ));
    let _ = fs::create_dir_all(&dir);
    let _ = fs::remove_file(dir.join(".todo.json"));

    assert!(xtask_bin()
        .arg("todo")
        .arg("add")
        .arg("keep me")
        .current_dir(&dir)
        .status()
        .unwrap()
        .success());

    let path = dir.join(".todo.json");
    let before = fs::read_to_string(&path).expect("store after add");

    let out = xtask_bin()
        .arg("todo")
        .arg("--json")
        .arg("list")
        .arg("--status")
        .arg("bogus")
        .current_dir(&dir)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2), "{:?}", out.stderr);

    let after = fs::read_to_string(&path).expect("store after failed list");
    assert_eq!(
        before, after,
        "invalid list must not write or corrupt .todo.json"
    );

    let _ = fs::remove_file(&path);
}
