//! Tests for todo subcommand and helpers.

use std::time::{Duration, SystemTime};

use crate::tests::{RestoreCwd, CWD_TEST_MUTEX};
use crate::todo::{
    cmd_todo, format_duration, format_time_ago, is_old_open, load_todos, print_todo_list_items,
    todo_file, TodoAddArgs, TodoArgs, TodoCompleteArgs, TodoDeleteArgs, TodoDto, TodoListArgs,
    TodoSub, AGE_THRESHOLD_DAYS,
};
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
    };
    assert!(is_old_open(&t, SystemTime::now()));
}

#[test]
fn cmd_todo_add_list_save_load() {
    let _guard = CWD_TEST_MUTEX.lock().unwrap();
    let dir = std::env::temp_dir().join(format!("xtask_todo_test_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let _ = std::fs::remove_file(".todo.json");

    let add_cmd = TodoArgs {
        sub: TodoSub::Add(TodoAddArgs {
            title: "test task".to_string(),
        }),
    };
    cmd_todo(add_cmd).unwrap();
    let todos = load_todos().unwrap();
    assert_eq!(todos.len(), 1, "expected exactly one todo after add");
    assert_eq!(todos[0].title, "test task");

    let list_cmd = TodoArgs {
        sub: TodoSub::List(TodoListArgs {}),
    };
    cmd_todo(list_cmd).unwrap();
}

#[test]
fn cmd_todo_complete_and_delete() {
    let _guard = CWD_TEST_MUTEX.lock().unwrap();
    let dir = std::env::temp_dir().join("xtask_todo_test2");
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let path = dir.join(".todo.json");

    let dtos = vec![TodoDto {
        id: 1,
        title: "complete me".to_string(),
        completed: false,
        created_at_secs: 0,
        completed_at_secs: None,
    }];
    std::fs::write(&path, serde_json::to_string_pretty(&dtos).unwrap()).unwrap();

    cmd_todo(TodoArgs {
        sub: TodoSub::Complete(TodoCompleteArgs { id: 1 }),
    })
    .unwrap();

    cmd_todo(TodoArgs {
        sub: TodoSub::Delete(TodoDeleteArgs { id: 1 }),
    })
    .unwrap();

    let todos = load_todos().unwrap();
    assert!(todos.is_empty());

    let _ = std::fs::remove_file(&path);
}

#[test]
fn load_todos_from_invalid_json_defaults_empty() {
    let _guard = CWD_TEST_MUTEX.lock().unwrap();
    let dir = std::env::temp_dir().join("xtask_todo_test_invalid");
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let path = dir.join(".todo.json");
    std::fs::write(&path, "not json").unwrap();

    let todos = load_todos().unwrap();
    assert!(todos.is_empty());

    let _ = std::fs::remove_file(&path);
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
    }];
    print_todo_list_items(&items, true);
}

#[test]
fn cmd_todo_list_empty_prints_no_tasks() {
    let _guard = CWD_TEST_MUTEX.lock().unwrap();
    let dir = std::env::temp_dir().join("xtask_todo_list_empty");
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let _ = std::fs::remove_file(".todo.json");

    let list_cmd = TodoArgs {
        sub: TodoSub::List(TodoListArgs {}),
    };
    cmd_todo(list_cmd).unwrap();
}

#[test]
fn cmd_todo_list_with_completed_todo() {
    let _guard = CWD_TEST_MUTEX.lock().unwrap();
    let dir = std::env::temp_dir().join("xtask_todo_list_completed");
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let _ = std::fs::remove_file(".todo.json");
    // One completed todo with completed_at set so list hits the "用时" branch
    let dtos = vec![TodoDto {
        id: 1,
        title: "done".to_string(),
        completed: true,
        created_at_secs: 0,
        completed_at_secs: Some(60),
    }];
    std::fs::write(".todo.json", serde_json::to_string_pretty(&dtos).unwrap()).unwrap();

    let list_cmd = TodoArgs {
        sub: TodoSub::List(TodoListArgs {}),
    };
    cmd_todo(list_cmd).unwrap();
}

#[test]
fn cmd_todo_complete_id_zero_errors() {
    let _guard = CWD_TEST_MUTEX.lock().unwrap();
    let dir = std::env::temp_dir().join("xtask_todo_id0");
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let _ = std::fs::remove_file(".todo.json");

    let err = cmd_todo(TodoArgs {
        sub: TodoSub::Complete(TodoCompleteArgs { id: 0 }),
    })
    .unwrap_err();
    assert!(err.to_string().contains("invalid id 0"));
}

#[test]
fn cmd_todo_delete_id_zero_errors() {
    let _guard = CWD_TEST_MUTEX.lock().unwrap();
    let dir = std::env::temp_dir().join("xtask_todo_del_id0");
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let _ = std::fs::remove_file(".todo.json");

    let err = cmd_todo(TodoArgs {
        sub: TodoSub::Delete(TodoDeleteArgs { id: 0 }),
    })
    .unwrap_err();
    assert!(err.to_string().contains("invalid id 0"));
}
