use std::fs;

use crate::common::xtask_bin;

/// TC-T10-1 / Story 1.6: `todo --json search <kw>` returns items matching domain `TodoList::search`.
#[test]
fn xtask_todo_search_json_hit_lists_matching_items() {
    let dir = std::env::temp_dir().join(format!(
        "xtask_integ_todo_search_hit_{}",
        std::process::id()
    ));
    let _ = fs::create_dir_all(&dir);
    let _ = fs::remove_file(dir.join(".todo.json"));

    assert!(xtask_bin()
        .arg("todo")
        .arg("add")
        .arg("Buy MILK")
        .arg("--description")
        .arg("store run")
        .current_dir(&dir)
        .status()
        .unwrap()
        .success());
    assert!(xtask_bin()
        .arg("todo")
        .arg("add")
        .arg("other")
        .current_dir(&dir)
        .status()
        .unwrap()
        .success());

    let out = xtask_bin()
        .arg("todo")
        .arg("--json")
        .arg("search")
        .arg("milk")
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out.stderr);
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).expect("json");
    assert_eq!(v["status"], "success");
    assert_eq!(v["data"]["empty"], false);
    let items = v["data"]["items"].as_array().expect("items");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["title"], "Buy MILK");

    let _ = fs::remove_file(dir.join(".todo.json"));
}

/// TC-T10-2 / Story 1.6: zero-hit `search` JSON matches empty `list` payload (Story 1.2).
#[test]
fn xtask_todo_search_json_no_match_empty_payload() {
    let dir = std::env::temp_dir().join(format!(
        "xtask_integ_todo_search_empty_{}",
        std::process::id()
    ));
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
        .arg("search")
        .arg("nomatch")
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out.stderr);
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).expect("json");
    assert_eq!(v["data"]["empty"], true);
    assert_eq!(v["data"]["message"], "No tasks.");
    assert_eq!(v["data"]["items"], serde_json::json!([]));

    let _ = fs::remove_file(dir.join(".todo.json"));
}

/// TC-T10-2: empty store + search still uses empty list JSON shape.
#[test]
fn xtask_todo_search_json_empty_store_empty_payload() {
    let dir = std::env::temp_dir().join(format!(
        "xtask_integ_todo_search_empty_store_{}",
        std::process::id()
    ));
    let _ = fs::create_dir_all(&dir);
    let _ = fs::remove_file(dir.join(".todo.json"));

    let out = xtask_bin()
        .arg("todo")
        .arg("--json")
        .arg("search")
        .arg("x")
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out.stderr);
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).expect("json");
    assert_eq!(v["data"]["empty"], true);
    assert_eq!(v["data"]["message"], "No tasks.");

    let _ = fs::remove_file(dir.join(".todo.json"));
}

/// TC-T11-1 / Story 1.6: `todo --json stats` exposes total / incomplete / complete.
#[test]
fn xtask_todo_stats_json_counts_match_store() {
    let dir = std::env::temp_dir().join(format!(
        "xtask_integ_todo_stats_json_{}",
        std::process::id()
    ));
    let _ = fs::create_dir_all(&dir);
    let _ = fs::remove_file(dir.join(".todo.json"));

    assert!(xtask_bin()
        .arg("todo")
        .arg("add")
        .arg("a")
        .current_dir(&dir)
        .status()
        .unwrap()
        .success());
    assert!(xtask_bin()
        .arg("todo")
        .arg("add")
        .arg("b")
        .current_dir(&dir)
        .status()
        .unwrap()
        .success());
    assert!(xtask_bin()
        .arg("todo")
        .arg("complete")
        .arg("1")
        .current_dir(&dir)
        .status()
        .unwrap()
        .success());

    let out = xtask_bin()
        .arg("todo")
        .arg("--json")
        .arg("stats")
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out.stderr);
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).expect("json");
    assert_eq!(v["status"], "success");
    assert_eq!(v["data"]["total"], 2);
    assert_eq!(v["data"]["incomplete"], 1);
    assert_eq!(v["data"]["complete"], 1);

    let _ = fs::remove_file(dir.join(".todo.json"));
}

/// Story 1.6 AC4: trim-empty keyword returns all todos (integration with `todo search ""`).
#[test]
fn xtask_todo_search_json_blank_keyword_returns_all() {
    let dir = std::env::temp_dir().join(format!(
        "xtask_integ_todo_search_blank_{}",
        std::process::id()
    ));
    let _ = fs::create_dir_all(&dir);
    let _ = fs::remove_file(dir.join(".todo.json"));

    assert!(xtask_bin()
        .arg("todo")
        .arg("add")
        .arg("one")
        .current_dir(&dir)
        .status()
        .unwrap()
        .success());
    assert!(xtask_bin()
        .arg("todo")
        .arg("add")
        .arg("two")
        .current_dir(&dir)
        .status()
        .unwrap()
        .success());

    let out = xtask_bin()
        .arg("todo")
        .arg("--json")
        .arg("search")
        .arg("")
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out.stderr);
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).expect("json");
    assert_eq!(v["data"]["empty"], false);
    let items = v["data"]["items"].as_array().expect("items");
    assert_eq!(items.len(), 2);

    let _ = fs::remove_file(dir.join(".todo.json"));
}
