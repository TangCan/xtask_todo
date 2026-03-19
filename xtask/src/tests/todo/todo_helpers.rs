//! Tests for `todo_file`, `load_todos`, `format_*`, `is_old_open`, `print_todo_list_items`.

use std::time::{Duration, SystemTime};

use crate::todo::format::{
    format_duration, format_time_ago, is_old_open, print_todo_list_items, AGE_THRESHOLD_DAYS,
};
use crate::todo::io::{load_todos, todo_file};
use xtask_todo_lib::{Todo, TodoId};

#[test]
fn todo_file_returns_path() {
    let path = todo_file().unwrap();
    assert!(path.ends_with(".todo.json"));
}

#[test]
fn load_todos_empty_when_no_file() {
    let path = todo_file().unwrap();
    if !path.exists() {
        let todos = load_todos().unwrap();
        assert!(todos.is_empty());
        return;
    }
    let backup = path.with_extension("json.bak");
    let _ = std::fs::rename(&path, &backup);
    let todos = load_todos().unwrap();
    let _ = std::fs::rename(&backup, &path);
    assert!(todos.is_empty());
}

#[test]
fn format_time_ago_just_now() {
    let now = SystemTime::now();
    assert_eq!(format_time_ago(now), "just now");
}

#[test]
fn format_time_ago_minutes() {
    let when = SystemTime::now() - Duration::from_secs(90);
    let s = format_time_ago(when);
    assert!(s.ends_with("m ago"));
}

#[test]
fn format_time_ago_hours() {
    let when = SystemTime::now() - Duration::from_hours(2);
    let s = format_time_ago(when);
    assert!(s.ends_with("h ago"));
}

#[test]
fn format_time_ago_days() {
    let when = SystemTime::now() - Duration::from_hours(48);
    let s = format_time_ago(when);
    assert!(s.ends_with("d ago"));
}

#[test]
fn format_duration_seconds() {
    assert_eq!(format_duration(Duration::from_secs(30)), "30s");
}

#[test]
fn format_duration_minutes() {
    assert_eq!(format_duration(Duration::from_secs(90)), "1m");
}

#[test]
fn format_duration_hours() {
    assert_eq!(format_duration(Duration::from_hours(2)), "2h");
}

#[test]
fn format_duration_days() {
    assert_eq!(format_duration(Duration::from_hours(72)), "3d");
}

#[test]
fn is_old_open_completed_false() {
    let created = SystemTime::now() - Duration::from_secs(AGE_THRESHOLD_DAYS * 86400 + 86400);
    let t = Todo {
        id: TodoId::from_raw(1).unwrap(),
        title: "x".into(),
        completed: true,
        created_at: created,
        completed_at: Some(SystemTime::now()),
        description: None,
        due_date: None,
        priority: None,
        tags: Vec::new(),
        repeat_rule: None,
        repeat_until: None,
        repeat_count: None,
    };
    assert!(!is_old_open(&t, SystemTime::now()));
}

#[test]
fn is_old_open_recent_false() {
    let created = SystemTime::now();
    let t = Todo {
        id: TodoId::from_raw(1).unwrap(),
        title: "x".into(),
        completed: false,
        created_at: created,
        completed_at: None,
        description: None,
        due_date: None,
        priority: None,
        tags: Vec::new(),
        repeat_rule: None,
        repeat_until: None,
        repeat_count: None,
    };
    assert!(!is_old_open(&t, SystemTime::now()));
}

#[test]
fn is_old_open_old_true() {
    let created = SystemTime::now() - Duration::from_secs(86400 * (AGE_THRESHOLD_DAYS + 1));
    let t = Todo {
        id: TodoId::from_raw(1).unwrap(),
        title: "x".into(),
        completed: false,
        created_at: created,
        completed_at: None,
        description: None,
        due_date: None,
        priority: None,
        tags: Vec::new(),
        repeat_rule: None,
        repeat_until: None,
        repeat_count: None,
    };
    assert!(is_old_open(&t, SystemTime::now()));
}

#[test]
fn load_todos_from_invalid_json_defaults_empty() {
    let _guard = crate::tests::cwd_test_lock();
    let dir = std::env::temp_dir().join("xtask_todo_test_invalid");
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = crate::tests::RestoreCwd::new(&dir, &cwd);
    let path = dir.join(".todo.json");
    std::fs::write(&path, "not json").unwrap();

    let todos = load_todos().unwrap();
    assert!(todos.is_empty());

    let _ = std::fs::remove_file(&path);
}

#[test]
fn print_todo_list_items_empty() {
    print_todo_list_items(&[], false);
    print_todo_list_items(&[], true);
}

#[test]
fn print_todo_list_items_old_open_with_color() {
    let created = SystemTime::now() - Duration::from_secs(AGE_THRESHOLD_DAYS * 86400 + 86400);
    let items = vec![Todo {
        id: TodoId::from_raw(1).unwrap(),
        title: "old open".to_string(),
        completed: false,
        created_at: created,
        completed_at: None,
        description: None,
        due_date: None,
        priority: None,
        tags: Vec::new(),
        repeat_rule: None,
        repeat_until: None,
        repeat_count: None,
    }];
    print_todo_list_items(&items, true);
}
