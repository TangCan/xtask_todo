//! Tests for `cmd_todo`: add, list, complete, delete, show, update.

use crate::tests::{RestoreCwd, CWD_TEST_MUTEX};
use crate::todo::{
    cmd_todo, load_todos, TodoAddArgs, TodoArgs, TodoCompleteArgs, TodoDeleteArgs, TodoDto,
    TodoListArgs, TodoShowArgs, TodoSub, TodoUpdateArgs,
};

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
        description: None,
        due_date: None,
        priority: None,
        tags: Vec::new(),
        repeat_rule: None,
    }];
    std::fs::write(&path, serde_json::to_string_pretty(&dtos).unwrap()).unwrap();

    cmd_todo(TodoArgs {
        sub: TodoSub::Complete(TodoCompleteArgs {
            id: 1,
            no_next: false,
        }),
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
        sub: TodoSub::Complete(TodoCompleteArgs {
            id: 0,
            no_next: false,
        }),
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

#[test]
fn cmd_todo_show_found_and_not_found() {
    let _guard = CWD_TEST_MUTEX.lock().unwrap();
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
    }];
    std::fs::write(".todo.json", serde_json::to_string_pretty(&dtos).unwrap()).unwrap();

    cmd_todo(TodoArgs {
        sub: TodoSub::Show(TodoShowArgs { id: 1 }),
    })
    .unwrap();

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
    }];
    std::fs::write(
        ".todo.json",
        serde_json::to_string_pretty(&dtos_done).unwrap(),
    )
    .unwrap();
    cmd_todo(TodoArgs {
        sub: TodoSub::Show(TodoShowArgs { id: 1 }),
    })
    .unwrap();

    let err = cmd_todo(TodoArgs {
        sub: TodoSub::Show(TodoShowArgs { id: 99 }),
    })
    .unwrap_err();
    assert!(err.to_string().contains("not found"));

    let err = cmd_todo(TodoArgs {
        sub: TodoSub::Show(TodoShowArgs { id: 0 }),
    })
    .unwrap_err();
    assert!(err.to_string().contains("invalid id 0"));
}

#[test]
fn cmd_todo_update_and_update_id_zero_errors() {
    let _guard = CWD_TEST_MUTEX.lock().unwrap();
    let dir = std::env::temp_dir().join(format!("xtask_todo_upd_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let _ = std::fs::remove_file(".todo.json");
    cmd_todo(TodoArgs {
        sub: TodoSub::Add(TodoAddArgs {
            title: "original".to_string(),
        }),
    })
    .unwrap();
    cmd_todo(TodoArgs {
        sub: TodoSub::Update(TodoUpdateArgs {
            id: 1,
            title: "updated".to_string(),
        }),
    })
    .unwrap();
    let todos = load_todos().unwrap();
    assert_eq!(todos.len(), 1);
    assert_eq!(todos[0].title, "updated");

    let err = cmd_todo(TodoArgs {
        sub: TodoSub::Update(TodoUpdateArgs {
            id: 0,
            title: "x".to_string(),
        }),
    })
    .unwrap_err();
    assert!(err.to_string().contains("invalid id 0"));
}
