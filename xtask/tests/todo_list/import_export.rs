//! TC-T12 / Story 1.7: `export` / `import` JSON payloads via real `cargo xtask todo`.

use std::fs;

use crate::common::xtask_bin;

/// TC-T12-1: `--json export` success `data` has export count and file path.
#[test]
fn xtask_todo_export_json_payload_tc_t12_1() {
    let dir = std::env::temp_dir().join(format!("xtask_integ_export_json_{}", std::process::id()));
    let _ = fs::create_dir_all(&dir);
    let _ = fs::remove_file(dir.join(".todo.json"));

    assert!(xtask_bin()
        .arg("todo")
        .arg("add")
        .arg("export-me")
        .current_dir(&dir)
        .status()
        .unwrap()
        .success());

    let export_path = dir.join("tasks.json");
    let out = xtask_bin()
        .arg("todo")
        .arg("--json")
        .arg("export")
        .arg(&export_path)
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out.stderr);
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).expect("json");
    assert_eq!(v["status"], "success");
    assert_eq!(v["data"]["exported"], 1);
    assert_eq!(
        v["data"]["file"].as_str().unwrap(),
        export_path.display().to_string()
    );
    assert!(export_path.exists());

    let _ = fs::remove_file(dir.join(".todo.json"));
}

/// TC-T12-2: merge import `--json` has `merged`, `count`; store gains imported rows.
#[test]
fn xtask_todo_import_merge_json_payload_tc_t12_2() {
    let dir = std::env::temp_dir().join(format!("xtask_integ_import_merge_{}", std::process::id()));
    let _ = fs::create_dir_all(&dir);
    let _ = fs::remove_file(dir.join(".todo.json"));

    assert!(xtask_bin()
        .arg("todo")
        .arg("add")
        .arg("local")
        .current_dir(&dir)
        .status()
        .unwrap()
        .success());

    let import_path = dir.join("incoming.json");
    fs::write(
        &import_path,
        r#"[{"id":99,"title":"from-file","completed":false,"created_at_secs":0,"tags":[]}]"#,
    )
    .unwrap();

    let out = xtask_bin()
        .arg("todo")
        .arg("--json")
        .arg("import")
        .arg(&import_path)
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out.stderr);
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).expect("json");
    assert_eq!(v["data"]["merged"], true);
    assert_eq!(v["data"]["count"], 1);
    assert_eq!(
        v["data"]["file"].as_str().unwrap(),
        import_path.display().to_string()
    );

    let list = xtask_bin()
        .arg("todo")
        .arg("--json")
        .arg("list")
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(list.status.success(), "{:?}", list.stderr);
    let lv: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&list.stdout).trim()).expect("json");
    assert_eq!(lv["data"]["items"].as_array().unwrap().len(), 2);

    let _ = fs::remove_file(dir.join(".todo.json"));
}

/// TC-T12-3: `--replace` import `--json` has `replaced`, `count`; store matches file only.
#[test]
fn xtask_todo_import_replace_json_payload_tc_t12_3() {
    let dir =
        std::env::temp_dir().join(format!("xtask_integ_import_replace_{}", std::process::id()));
    let _ = fs::create_dir_all(&dir);
    let _ = fs::remove_file(dir.join(".todo.json"));

    assert!(xtask_bin()
        .arg("todo")
        .arg("add")
        .arg("drop-a")
        .current_dir(&dir)
        .status()
        .unwrap()
        .success());
    assert!(xtask_bin()
        .arg("todo")
        .arg("add")
        .arg("drop-b")
        .current_dir(&dir)
        .status()
        .unwrap()
        .success());

    let import_path = dir.join("only.json");
    fs::write(
        &import_path,
        r#"[{"id":1,"title":"sole","completed":false,"created_at_secs":0,"tags":[]}]"#,
    )
    .unwrap();

    let out = xtask_bin()
        .arg("todo")
        .arg("--json")
        .arg("import")
        .arg("--replace")
        .arg(&import_path)
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out.stderr);
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).expect("json");
    assert_eq!(v["data"]["replaced"], true);
    assert_eq!(v["data"]["count"], 1);

    let list = xtask_bin()
        .arg("todo")
        .arg("--json")
        .arg("list")
        .current_dir(&dir)
        .output()
        .unwrap();
    let lv: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&list.stdout).trim()).expect("json");
    let items = lv["data"]["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["title"], "sole");

    let _ = fs::remove_file(dir.join(".todo.json"));
}

/// TC-A4-1 / AC4: `import --dry-run` does not persist merge to `.todo.json`; JSON still previews.
#[test]
fn xtask_todo_import_dry_run_merge_preserves_store_tc_a4() {
    let dir = std::env::temp_dir().join(format!("xtask_integ_import_dry_{}", std::process::id()));
    let _ = fs::create_dir_all(&dir);
    let _ = fs::remove_file(dir.join(".todo.json"));

    assert!(xtask_bin()
        .arg("todo")
        .arg("add")
        .arg("only-local")
        .current_dir(&dir)
        .status()
        .unwrap()
        .success());

    let store_path = dir.join(".todo.json");
    let before = fs::read_to_string(&store_path).unwrap();

    let import_path = dir.join("extra.json");
    fs::write(
        &import_path,
        r#"[{"id":1,"title":"would-merge","completed":false,"created_at_secs":0,"tags":[]}]"#,
    )
    .unwrap();

    let out = xtask_bin()
        .arg("todo")
        .arg("--json")
        .arg("--dry-run")
        .arg("import")
        .arg(&import_path)
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out.stderr);
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).expect("json");
    assert_eq!(v["data"]["merged"], true);
    assert_eq!(v["data"]["count"], 1);

    let after = fs::read_to_string(&store_path).unwrap();
    assert_eq!(
        before, after,
        "dry-run import must not write merged todos to .todo.json"
    );

    let list = xtask_bin()
        .arg("todo")
        .arg("--json")
        .arg("list")
        .current_dir(&dir)
        .output()
        .unwrap();
    let lv: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&list.stdout).trim()).expect("json");
    assert_eq!(lv["data"]["items"].as_array().unwrap().len(), 1);

    let _ = fs::remove_file(&store_path);
}

/// AC2: `import --replace --dry-run` previews replace but must not persist store changes.
#[test]
fn xtask_todo_import_dry_run_replace_preserves_store() {
    let dir = std::env::temp_dir().join(format!(
        "xtask_integ_import_dry_replace_{}",
        std::process::id()
    ));
    let _ = fs::create_dir_all(&dir);
    let _ = fs::remove_file(dir.join(".todo.json"));

    assert!(xtask_bin()
        .arg("todo")
        .arg("add")
        .arg("keep-local")
        .current_dir(&dir)
        .status()
        .unwrap()
        .success());

    let store_path = dir.join(".todo.json");
    let before = fs::read_to_string(&store_path).unwrap();

    let import_path = dir.join("replace-only.json");
    fs::write(
        &import_path,
        r#"[{"id":1,"title":"would-replace","completed":false,"created_at_secs":0,"tags":[]}]"#,
    )
    .unwrap();

    let out = xtask_bin()
        .arg("todo")
        .arg("--json")
        .arg("--dry-run")
        .arg("import")
        .arg("--replace")
        .arg(&import_path)
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out.stderr);
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).expect("json");
    assert_eq!(v["data"]["replaced"], true);
    assert_eq!(v["data"]["count"], 1);

    let after = fs::read_to_string(&store_path).unwrap();
    assert_eq!(
        before, after,
        "dry-run import --replace must not overwrite .todo.json"
    );

    let list = xtask_bin()
        .arg("todo")
        .arg("--json")
        .arg("list")
        .current_dir(&dir)
        .output()
        .unwrap();
    let lv: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&list.stdout).trim()).expect("json");
    assert_eq!(lv["data"]["items"].as_array().unwrap().len(), 1);
    assert_eq!(lv["data"]["items"][0]["title"], "keep-local");

    let _ = fs::remove_file(&store_path);
}

/// AC5: import of missing file exits 1; existing store unchanged.
#[test]
fn xtask_todo_import_missing_file_exits_general_preserves_store() {
    let dir =
        std::env::temp_dir().join(format!("xtask_integ_import_missing_{}", std::process::id()));
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

    let store_path = dir.join(".todo.json");
    let before = fs::read_to_string(&store_path).unwrap();
    let missing = dir.join("nope.json");

    let out = xtask_bin()
        .arg("todo")
        .arg("--json")
        .arg("import")
        .arg(&missing)
        .current_dir(&dir)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1), "{:?}", out.stderr);
    let err_line = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let ev: serde_json::Value = serde_json::from_str(&err_line).expect("error json");
    assert_eq!(ev["status"], "error");
    assert_eq!(ev["error"]["code"], 1);

    assert_eq!(fs::read_to_string(&store_path).unwrap(), before);

    let _ = fs::remove_file(&store_path);
}

/// AC5: syntactically invalid JSON import exits 1; store unchanged.
#[test]
fn xtask_todo_import_invalid_json_exits_general_preserves_store() {
    let dir =
        std::env::temp_dir().join(format!("xtask_integ_import_badjson_{}", std::process::id()));
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

    let store_path = dir.join(".todo.json");
    let before = fs::read_to_string(&store_path).unwrap();

    let bad_path = dir.join("bad.json");
    fs::write(&bad_path, "not a json array").unwrap();

    let out = xtask_bin()
        .arg("todo")
        .arg("--json")
        .arg("import")
        .arg(&bad_path)
        .current_dir(&dir)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1), "{:?}", out.stderr);
    let err_line = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let ev: serde_json::Value = serde_json::from_str(&err_line).expect("error json");
    assert_eq!(ev["status"], "error");
    assert_eq!(ev["error"]["code"], 1);

    assert_eq!(fs::read_to_string(&store_path).unwrap(), before);

    let _ = fs::remove_file(&store_path);
}
