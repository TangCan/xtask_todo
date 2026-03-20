//! List filters/sort, dry-run update/complete/delete, and show-with-fields tests.

use crate::tests::{cwd_test_lock, RestoreCwd};
use crate::todo::args::{
    todo_args, TodoAddArgs, TodoArgs, TodoCompleteArgs, TodoDeleteArgs, TodoListArgs, TodoShowArgs,
    TodoSub, TodoUpdateArgs,
};
use crate::todo::cmd_todo;
use crate::todo::io::load_todos;

#[test]
fn cmd_todo_list_with_filters_and_sort() {
    let _cwd_lock = cwd_test_lock();
    let dir = std::env::temp_dir().join(format!("xtask_todo_list_filt_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _restore = RestoreCwd::new(&dir, &cwd);
    let _ = std::fs::remove_file(".todo.json");
    cmd_todo(todo_args(TodoSub::Add(TodoAddArgs {
        title: "a".to_string(),
        description: None,
        due_date: Some("2026-02-01".to_string()),
        priority: Some("low".to_string()),
        tags: Some("t1".to_string()),
        repeat_rule: None,
        repeat_until: None,
        repeat_count: None,
    })))
    .unwrap();
    cmd_todo(todo_args(TodoSub::Add(TodoAddArgs {
        title: "b".to_string(),
        description: None,
        due_date: Some("2026-01-01".to_string()),
        priority: Some("high".to_string()),
        tags: Some("t2".to_string()),
        repeat_rule: None,
        repeat_until: None,
        repeat_count: None,
    })))
    .unwrap();

    let list_cmd = todo_args(TodoSub::List(TodoListArgs {
        status: Some("incomplete".to_string()),
        priority: None,
        tags: None,
        due_before: None,
        due_after: None,
        sort: Some("due-date".to_string()),
    }));
    cmd_todo(list_cmd).unwrap();

    let list_priority = todo_args(TodoSub::List(TodoListArgs {
        status: None,
        priority: Some("high".to_string()),
        tags: None,
        due_before: None,
        due_after: None,
        sort: None,
    }));
    cmd_todo(list_priority).unwrap();

    let list_tags = todo_args(TodoSub::List(TodoListArgs {
        status: None,
        priority: None,
        tags: Some("t1".to_string()),
        due_before: None,
        due_after: None,
        sort: Some("title".to_string()),
    }));
    cmd_todo(list_tags).unwrap();

    let list_due = todo_args(TodoSub::List(TodoListArgs {
        status: None,
        priority: None,
        tags: None,
        due_before: Some("2026-06-01".to_string()),
        due_after: Some("2025-12-01".to_string()),
        sort: Some("priority".to_string()),
    }));
    cmd_todo(list_due).unwrap();
}

#[test]
fn cmd_todo_list_invalid_status_returns_parameter_error() {
    let _cwd_lock = cwd_test_lock();
    let dir = std::env::temp_dir().join(format!("xtask_todo_bad_st_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _restore = RestoreCwd::new(&dir, &cwd);
    let _ = std::fs::remove_file(".todo.json");

    let err = cmd_todo(todo_args(TodoSub::List(TodoListArgs {
        status: Some("invalid".to_string()),
        priority: None,
        tags: None,
        due_before: None,
        due_after: None,
        sort: None,
    })))
    .unwrap_err();
    assert_eq!(err.exit_code(), 2);
    assert!(err.to_string().contains("invalid status"));
}

#[test]
fn cmd_todo_list_invalid_due_before_returns_parameter_error() {
    let _cwd_lock = cwd_test_lock();
    let dir = std::env::temp_dir().join(format!("xtask_todo_bad_due_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _restore = RestoreCwd::new(&dir, &cwd);
    let _ = std::fs::remove_file(".todo.json");

    let err = cmd_todo(todo_args(TodoSub::List(TodoListArgs {
        status: None,
        priority: None,
        tags: None,
        due_before: Some("2026/01/01".to_string()),
        due_after: None,
        sort: None,
    })))
    .unwrap_err();
    assert_eq!(err.exit_code(), 2);
    assert!(err.to_string().contains("invalid due_before"));
}

#[test]
fn cmd_todo_dry_run_update_and_complete_and_delete() {
    let _cwd_lock = cwd_test_lock();
    let dir = std::env::temp_dir().join(format!("xtask_todo_dry_ucd_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _restore = RestoreCwd::new(&dir, &cwd);
    let _ = std::fs::remove_file(".todo.json");
    cmd_todo(todo_args(TodoSub::Add(TodoAddArgs {
        title: "task".to_string(),
        description: None,
        due_date: None,
        priority: None,
        tags: None,
        repeat_rule: None,
        repeat_until: None,
        repeat_count: None,
    })))
    .unwrap();

    let dry_update_json = TodoArgs {
        sub: TodoSub::Update(TodoUpdateArgs {
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
        }),
        json: true,
        dry_run: true,
    };
    cmd_todo(dry_update_json).unwrap();

    let dry_update_plain = TodoArgs {
        sub: TodoSub::Update(TodoUpdateArgs {
            id: 1,
            title: "would update".to_string(),
            description: None,
            due_date: None,
            priority: None,
            tags: None,
            repeat_rule: None,
            repeat_until: None,
            repeat_count: None,
            clear_repeat_rule: false,
        }),
        json: false,
        dry_run: true,
    };
    cmd_todo(dry_update_plain).unwrap();

    let dry_complete = TodoArgs {
        sub: TodoSub::Complete(TodoCompleteArgs {
            id: 1,
            no_next: false,
        }),
        json: true,
        dry_run: true,
    };
    cmd_todo(dry_complete).unwrap();

    let dry_complete_plain = TodoArgs {
        sub: TodoSub::Complete(TodoCompleteArgs {
            id: 1,
            no_next: false,
        }),
        json: false,
        dry_run: true,
    };
    cmd_todo(dry_complete_plain).unwrap();

    let dry_delete = TodoArgs {
        sub: TodoSub::Delete(TodoDeleteArgs { id: 1 }),
        json: true,
        dry_run: true,
    };
    cmd_todo(dry_delete).unwrap();

    let dry_delete_plain = TodoArgs {
        sub: TodoSub::Delete(TodoDeleteArgs { id: 1 }),
        json: false,
        dry_run: true,
    };
    cmd_todo(dry_delete_plain).unwrap();

    let todos = load_todos().unwrap();
    assert_eq!(todos.len(), 1);
}

#[test]
fn cmd_todo_show_with_all_fields() {
    let _cwd_lock = cwd_test_lock();
    let dir = std::env::temp_dir().join(format!("xtask_todo_show_full_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _restore = RestoreCwd::new(&dir, &cwd);
    let _ = std::fs::remove_file(".todo.json");
    cmd_todo(todo_args(TodoSub::Add(TodoAddArgs {
        title: "full".to_string(),
        description: Some("desc".to_string()),
        due_date: Some("2026-06-01".to_string()),
        priority: Some("medium".to_string()),
        tags: Some("a,b".to_string()),
        repeat_rule: Some("weekly".to_string()),
        repeat_until: None,
        repeat_count: None,
    })))
    .unwrap();

    cmd_todo(todo_args(TodoSub::Show(TodoShowArgs { id: 1 }))).unwrap();
}

#[test]
fn cmd_todo_update_clear_repeat_rule_clears_repeat_rule() {
    let _cwd_lock = cwd_test_lock();
    let dir = std::env::temp_dir().join(format!("xtask_todo_clear_rep_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _restore = RestoreCwd::new(&dir, &cwd);
    let _ = std::fs::remove_file(".todo.json");

    cmd_todo(todo_args(TodoSub::Add(TodoAddArgs {
        title: "recurring".to_string(),
        description: None,
        due_date: None,
        priority: None,
        tags: None,
        repeat_rule: Some("daily".to_string()),
        repeat_until: None,
        repeat_count: None,
    })))
    .unwrap();
    let todos = load_todos().unwrap();
    assert_eq!(todos.len(), 1);
    assert!(todos[0].repeat_rule.is_some());

    cmd_todo(todo_args(TodoSub::Update(TodoUpdateArgs {
        id: 1,
        title: "recurring".to_string(),
        description: None,
        due_date: None,
        priority: None,
        tags: None,
        repeat_rule: None,
        repeat_until: None,
        repeat_count: None,
        clear_repeat_rule: true,
    })))
    .unwrap();
    let todos = load_todos().unwrap();
    assert_eq!(todos.len(), 1);
    assert!(todos[0].repeat_rule.is_none());
}

#[test]
fn cmd_todo_add_with_repeat_until_and_repeat_count() {
    let _cwd_lock = cwd_test_lock();
    let dir = std::env::temp_dir().join(format!("xtask_todo_repeat_opts_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _restore = RestoreCwd::new(&dir, &cwd);
    let _ = std::fs::remove_file(".todo.json");

    cmd_todo(todo_args(TodoSub::Add(TodoAddArgs {
        title: "limited repeat".to_string(),
        description: None,
        due_date: Some("2026-06-01".to_string()),
        priority: None,
        tags: None,
        repeat_rule: Some("weekly".to_string()),
        repeat_until: Some("2026-12-31".to_string()),
        repeat_count: Some("3".to_string()),
    })))
    .unwrap();
    let todos = load_todos().unwrap();
    assert_eq!(todos.len(), 1);
    assert_eq!(todos[0].repeat_until.as_deref(), Some("2026-12-31"));
    assert_eq!(todos[0].repeat_count, Some(3));
}
