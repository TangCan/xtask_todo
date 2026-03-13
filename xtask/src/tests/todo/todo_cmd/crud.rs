//! Basic CRUD and id-zero error tests for `cmd_todo`.

use crate::tests::{cwd_test_lock, RestoreCwd};
use crate::todo::{
    cmd_todo, load_todos, todo_args, TodoAddArgs, TodoCompleteArgs, TodoDeleteArgs, TodoDto,
    TodoListArgs, TodoShowArgs, TodoSub, TodoUpdateArgs,
};

#[test]
fn cmd_todo_add_list_save_load() {
    let _guard = cwd_test_lock();
    let dir = std::env::temp_dir().join(format!("xtask_todo_test_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let _ = std::fs::remove_file(".todo.json");

    let add_cmd = todo_args(TodoSub::Add(TodoAddArgs {
        title: "test task".to_string(),
        description: None,
        due_date: None,
        priority: None,
        tags: None,
        repeat_rule: None,
        repeat_until: None,
        repeat_count: None,
    }));
    cmd_todo(add_cmd).unwrap();
    let add_with_opts = todo_args(TodoSub::Add(TodoAddArgs {
        title: "full task".to_string(),
        description: Some("a desc".to_string()),
        due_date: Some("2026-01-15".to_string()),
        priority: Some("high".to_string()),
        tags: Some("work,urgent".to_string()),
        repeat_rule: Some("daily".to_string()),
        repeat_until: None,
        repeat_count: None,
    }));
    cmd_todo(add_with_opts).unwrap();
    let todos = load_todos().unwrap();
    assert_eq!(todos.len(), 2);
    assert_eq!(todos[0].title, "test task");
    assert_eq!(todos[1].title, "full task");
    assert_eq!(todos[1].description.as_deref(), Some("a desc"));
    assert_eq!(todos[1].due_date.as_deref(), Some("2026-01-15"));
    assert!(todos[1].tags.contains(&"work".to_string()));

    let list_cmd = todo_args(TodoSub::List(TodoListArgs::default()));
    cmd_todo(list_cmd).unwrap();
}

#[test]
fn cmd_todo_complete_and_delete() {
    let _guard = cwd_test_lock();
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
        description: None,
        due_date: None,
        priority: None,
        tags: Vec::new(),
        repeat_rule: None,
        repeat_until: None,
        repeat_count: None,
    }];
    std::fs::write(&path, serde_json::to_string_pretty(&dtos).unwrap()).unwrap();

    cmd_todo(todo_args(TodoSub::Complete(TodoCompleteArgs {
        id: 1,
        no_next: false,
    })))
    .unwrap();

    cmd_todo(todo_args(TodoSub::Delete(TodoDeleteArgs { id: 1 }))).unwrap();

    let todos = load_todos().unwrap();
    assert!(todos.is_empty());

    let _ = std::fs::remove_file(&path);
}

#[test]
fn cmd_todo_list_empty_prints_no_tasks() {
    let _guard = cwd_test_lock();
    let dir = std::env::temp_dir().join("xtask_todo_list_empty");
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let _ = std::fs::remove_file(".todo.json");

    let list_cmd = todo_args(TodoSub::List(TodoListArgs::default()));
    cmd_todo(list_cmd).unwrap();
}

#[test]
fn cmd_todo_list_with_completed_todo() {
    let _guard = cwd_test_lock();
    let dir = std::env::temp_dir().join("xtask_todo_list_completed");
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let _ = std::fs::remove_file(".todo.json");
    let dtos = vec![TodoDto {
        id: 1,
        title: "done".to_string(),
        completed: true,
        created_at_secs: 0,
        completed_at_secs: Some(60),
        description: None,
        due_date: None,
        priority: None,
        tags: Vec::new(),
        repeat_rule: None,
        repeat_until: None,
        repeat_count: None,
    }];
    std::fs::write(".todo.json", serde_json::to_string_pretty(&dtos).unwrap()).unwrap();

    let list_cmd = todo_args(TodoSub::List(TodoListArgs::default()));
    cmd_todo(list_cmd).unwrap();
}

#[test]
fn cmd_todo_complete_id_zero_errors() {
    let _guard = cwd_test_lock();
    let dir = std::env::temp_dir().join("xtask_todo_id0");
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let _ = std::fs::remove_file(".todo.json");

    let err = cmd_todo(todo_args(TodoSub::Complete(TodoCompleteArgs {
        id: 0,
        no_next: false,
    })))
    .unwrap_err();
    assert_eq!(err.exit_code(), 2, "id 0 is parameter error per §3.1");
    assert!(err.to_string().contains("invalid id 0"));
}

#[test]
fn cmd_todo_complete_nonexistent_id_returns_exit_code_3() {
    let _guard = cwd_test_lock();
    let dir = std::env::temp_dir().join(format!("xtask_todo_comp_nx_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let _ = std::fs::remove_file(".todo.json");
    cmd_todo(todo_args(TodoSub::Add(TodoAddArgs {
        title: "only".to_string(),
        description: None,
        due_date: None,
        priority: None,
        tags: None,
        repeat_rule: None,
        repeat_until: None,
        repeat_count: None,
    })))
    .unwrap();
    let err = cmd_todo(todo_args(TodoSub::Complete(TodoCompleteArgs {
        id: 999,
        no_next: false,
    })))
    .unwrap_err();
    assert_eq!(err.exit_code(), 3, "nonexistent id is data error per §3.1");
    assert!(err.to_string().contains("not found"));
}

#[test]
fn cmd_todo_delete_id_zero_errors() {
    let _guard = cwd_test_lock();
    let dir = std::env::temp_dir().join("xtask_todo_del_id0");
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let _ = std::fs::remove_file(".todo.json");

    let err = cmd_todo(todo_args(TodoSub::Delete(TodoDeleteArgs { id: 0 }))).unwrap_err();
    assert_eq!(err.exit_code(), 2, "id 0 is parameter error per §3.1");
    assert!(err.to_string().contains("invalid id 0"));
}

#[test]
fn cmd_todo_delete_nonexistent_id_returns_exit_code_3() {
    let _guard = cwd_test_lock();
    let dir = std::env::temp_dir().join(format!("xtask_todo_del_nx_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let _ = std::fs::remove_file(".todo.json");
    cmd_todo(todo_args(TodoSub::Add(TodoAddArgs {
        title: "only".to_string(),
        description: None,
        due_date: None,
        priority: None,
        tags: None,
        repeat_rule: None,
        repeat_until: None,
        repeat_count: None,
    })))
    .unwrap();
    let err = cmd_todo(todo_args(TodoSub::Delete(TodoDeleteArgs { id: 999 }))).unwrap_err();
    assert_eq!(err.exit_code(), 3, "nonexistent id is data error per §3.1");
    assert!(err.to_string().contains("not found"));
}

#[test]
fn cmd_todo_show_found_and_not_found() {
    let _guard = cwd_test_lock();
    let dir = std::env::temp_dir().join(format!("xtask_todo_show_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let _ = std::fs::remove_file(".todo.json");
    let dtos = vec![TodoDto {
        id: 1,
        title: "show me".to_string(),
        completed: false,
        created_at_secs: 0,
        completed_at_secs: None,
        description: None,
        due_date: None,
        priority: None,
        tags: Vec::new(),
        repeat_rule: None,
        repeat_until: None,
        repeat_count: None,
    }];
    std::fs::write(".todo.json", serde_json::to_string_pretty(&dtos).unwrap()).unwrap();

    cmd_todo(todo_args(TodoSub::Show(TodoShowArgs { id: 1 }))).unwrap();

    let dtos_done = vec![TodoDto {
        id: 1,
        title: "done".to_string(),
        completed: true,
        created_at_secs: 0,
        completed_at_secs: Some(120),
        description: None,
        due_date: None,
        priority: None,
        tags: Vec::new(),
        repeat_rule: None,
        repeat_until: None,
        repeat_count: None,
    }];
    std::fs::write(
        ".todo.json",
        serde_json::to_string_pretty(&dtos_done).unwrap(),
    )
    .unwrap();
    cmd_todo(todo_args(TodoSub::Show(TodoShowArgs { id: 1 }))).unwrap();

    let err = cmd_todo(todo_args(TodoSub::Show(TodoShowArgs { id: 99 }))).unwrap_err();
    assert_eq!(err.exit_code(), 3, "nonexistent id is data error per §3.1");
    assert!(err.to_string().contains("not found"));

    let err = cmd_todo(todo_args(TodoSub::Show(TodoShowArgs { id: 0 }))).unwrap_err();
    assert_eq!(err.exit_code(), 2, "id 0 is parameter error per §3.1");
    assert!(err.to_string().contains("invalid id 0"));
}

#[test]
fn cmd_todo_update_and_update_id_zero_errors() {
    let _guard = cwd_test_lock();
    let dir = std::env::temp_dir().join(format!("xtask_todo_upd_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let _ = std::fs::remove_file(".todo.json");
    cmd_todo(todo_args(TodoSub::Add(TodoAddArgs {
        title: "original".to_string(),
        description: None,
        due_date: None,
        priority: None,
        tags: None,
        repeat_rule: None,
        repeat_until: None,
        repeat_count: None,
    })))
    .unwrap();
    cmd_todo(todo_args(TodoSub::Update(TodoUpdateArgs {
        id: 1,
        title: "updated".to_string(),
        description: None,
        due_date: None,
        priority: None,
        tags: None,
        repeat_rule: None,
        repeat_until: None,
        repeat_count: None,
        clear_repeat_rule: false,
    })))
    .unwrap();
    let todos = load_todos().unwrap();
    assert_eq!(todos.len(), 1);
    assert_eq!(todos[0].title, "updated");

    let err = cmd_todo(todo_args(TodoSub::Update(TodoUpdateArgs {
        id: 0,
        title: "x".to_string(),
        description: None,
        due_date: None,
        priority: None,
        tags: None,
        repeat_rule: None,
        repeat_until: None,
        repeat_count: None,
        clear_repeat_rule: false,
    })))
    .unwrap_err();
    assert_eq!(err.exit_code(), 2, "id 0 is parameter error per §3.1");
    assert!(err.to_string().contains("invalid id 0"));

    let err = cmd_todo(todo_args(TodoSub::Update(TodoUpdateArgs {
        id: 999,
        title: "x".to_string(),
        description: None,
        due_date: None,
        priority: None,
        tags: None,
        repeat_rule: None,
        repeat_until: None,
        repeat_count: None,
        clear_repeat_rule: false,
    })))
    .unwrap_err();
    assert_eq!(err.exit_code(), 3, "nonexistent id is data error per §3.1");
    assert!(err.to_string().contains("not found"));
}

#[test]
fn cmd_todo_add_empty_title_returns_parameter_error() {
    let _guard = cwd_test_lock();
    let dir = std::env::temp_dir().join(format!("xtask_todo_empty_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let _ = std::fs::remove_file(".todo.json");

    let err = cmd_todo(todo_args(TodoSub::Add(TodoAddArgs {
        title: String::new(),
        description: None,
        due_date: None,
        priority: None,
        tags: None,
        repeat_rule: None,
        repeat_until: None,
        repeat_count: None,
    })))
    .unwrap_err();
    assert_eq!(err.exit_code(), 2);
    assert!(err.to_string().contains("non-empty"));
}
