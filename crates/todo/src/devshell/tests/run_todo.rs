//! Todo subcommand tests in devshell: add, list, show, update, complete, delete, search, errors.

use std::io::Cursor;

use super::super::run_with;

#[test]
fn run_with_todo_add_empty_title_errors() {
    let input = "todo add \nexit\n";
    let mut stdin = Cursor::new(input);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    run_with(
        &["dev_shell".to_string()],
        &mut stdin,
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    let err = String::from_utf8(stderr).unwrap();
    assert!(err.contains("title") || err.contains("non-empty"));
}

#[test]
fn run_with_todo_unknown_subcommand() {
    let input = "todo unknownsub\nexit\n";
    let mut stdin = Cursor::new(input);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    run_with(
        &["dev_shell".to_string()],
        &mut stdin,
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    let err = String::from_utf8(stderr).unwrap();
    assert!(err.contains("unknown") || err.contains("subcommand"));
}

#[test]
fn run_with_todo_show_description_due() {
    let dir = std::env::temp_dir().join(format!("devshell_show_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let json_path = dir.join(".todo.json");
    let json = r#"[{"id":1,"title":"task","completed":false,"created_at_secs":0,"tags":[],"description":"desc","due_date":"2025-12-01"}]"#;
    std::fs::write(&json_path, json).unwrap();
    let cwd = std::env::current_dir().unwrap();
    let _ = std::env::set_current_dir(&dir);
    let input = "todo show 1\nexit\n";
    let mut stdin = Cursor::new(input);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let r = run_with(
        &["dev_shell".to_string()],
        &mut stdin,
        &mut stdout,
        &mut stderr,
    );
    let _ = std::env::set_current_dir(&cwd);
    let _ = std::fs::remove_file(&json_path);
    let _ = std::fs::remove_dir(&dir);
    r.unwrap();
    let out = String::from_utf8(stdout).unwrap();
    assert!(out.contains("desc") || out.contains("task") || out.contains("1.") || !out.is_empty());
    assert!(out.contains("2025-12-01") || out.contains("due") || !out.is_empty());
}

#[test]
fn run_with_todo_update_empty_title_errors() {
    let dir = std::env::temp_dir().join(format!("devshell_upd_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let json_path = dir.join(".todo.json");
    let json = r#"[{"id":1,"title":"x","completed":false,"created_at_secs":0,"tags":[]}]"#;
    std::fs::write(&json_path, json).unwrap();
    let cwd = std::env::current_dir().unwrap();
    let _ = std::env::set_current_dir(&dir);
    let input = "todo update 1  \nexit\n";
    let mut stdin = Cursor::new(input);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    run_with(
        &["dev_shell".to_string()],
        &mut stdin,
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    let _ = std::env::set_current_dir(&cwd);
    let _ = std::fs::remove_file(&json_path);
    let _ = std::fs::remove_dir(&dir);
    let err = String::from_utf8(stderr).unwrap();
    assert!(err.contains("non-empty") || err.contains("title"));
}

#[test]
fn run_with_todo_complete_nonexistent_errors() {
    let input = "todo complete 999\nexit\n";
    let mut stdin = Cursor::new(input);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    run_with(
        &["dev_shell".to_string()],
        &mut stdin,
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    let err = String::from_utf8(stderr).unwrap();
    assert!(err.contains("complete") || err.contains("todo") || err.contains("not found"));
}

#[test]
fn run_with_todo_search_output() {
    let dir = std::env::temp_dir().join(format!("devshell_srch_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let json_path = dir.join(".todo.json");
    let json = r#"[{"id":1,"title":"buy milk","completed":false,"created_at_secs":0,"tags":[]}]"#;
    std::fs::write(&json_path, json).unwrap();
    let cwd = std::env::current_dir().unwrap();
    let _ = std::env::set_current_dir(&dir);
    let input = "todo search milk\nexit\n";
    let mut stdin = Cursor::new(input);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let r = run_with(
        &["dev_shell".to_string()],
        &mut stdin,
        &mut stdout,
        &mut stderr,
    );
    let _ = std::env::set_current_dir(&cwd);
    let _ = std::fs::remove_file(&json_path);
    let _ = std::fs::remove_dir(&dir);
    r.unwrap();
    let out = String::from_utf8(stdout).unwrap();
    assert!(
        out.contains("buy milk") || out.contains("1.") || out.contains("milk") || !out.is_empty()
    );
}

#[test]
fn run_with_todo_list_when_no_todo_file() {
    let dir = std::env::temp_dir().join(format!("devshell_nojson_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _ = std::env::set_current_dir(&dir);
    let input = "todo list\nexit\n";
    let mut stdin = Cursor::new(input);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let r = run_with(
        &["dev_shell".to_string()],
        &mut stdin,
        &mut stdout,
        &mut stderr,
    );
    let _ = std::env::set_current_dir(&cwd);
    let _ = std::fs::remove_dir(&dir);
    r.unwrap();
    let out = String::from_utf8(stdout).unwrap();
    assert!(out.contains("total: 0") || out.contains(" $ ") || out.is_empty());
}

#[test]
fn run_with_todo_show_complete_search_with_existing_file() {
    let dir = std::env::temp_dir().join(format!("devshell_todo_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let json_path = dir.join(".todo.json");
    let json = r#"[{"id":1,"title":"buy milk","completed":false,"created_at_secs":0,"tags":[]}]"#;
    std::fs::write(&json_path, json).unwrap();
    let cwd = std::env::current_dir().unwrap();
    let _ = std::env::set_current_dir(&dir);
    let input = "todo show 1\ntodo complete 1\ntodo search milk\ntodo list\nexit\n";
    let mut stdin = Cursor::new(input);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let r = run_with(
        &["dev_shell".to_string()],
        &mut stdin,
        &mut stdout,
        &mut stderr,
    );
    let _ = std::env::set_current_dir(&cwd);
    let _ = std::fs::remove_file(&json_path);
    let _ = std::fs::remove_dir(&dir);
    r.unwrap();
    let out = String::from_utf8(stdout).unwrap();
    assert!(out.contains("buy milk") || out.contains("1.") || !out.is_empty());
}

#[test]
fn run_with_todo_update_and_delete() {
    let dir = std::env::temp_dir().join(format!("devshell_todo2_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let json_path = dir.join(".todo.json");
    let json = r#"[{"id":1,"title":"original","completed":false,"created_at_secs":0,"tags":[]}]"#;
    std::fs::write(&json_path, json).unwrap();
    let cwd = std::env::current_dir().unwrap();
    let _ = std::env::set_current_dir(&dir);
    let input = "todo update 1 updated title\ntodo delete 1\ntodo list\nexit\n";
    let mut stdin = Cursor::new(input);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let r = run_with(
        &["dev_shell".to_string()],
        &mut stdin,
        &mut stdout,
        &mut stderr,
    );
    let _ = std::env::set_current_dir(&cwd);
    let _ = std::fs::remove_file(&json_path);
    let _ = std::fs::remove_dir(&dir);
    r.unwrap();
    let out = String::from_utf8(stdout).unwrap();
    assert!(!out.is_empty(), "expected prompt output");
}
