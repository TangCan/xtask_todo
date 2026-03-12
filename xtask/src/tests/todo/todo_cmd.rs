//! Tests for `cmd_todo`: add, list, complete, delete, show, update, json, init-ai.

use crate::tests::{cwd_test_lock, RestoreCwd};
use crate::todo::{
    cmd_todo, load_todos, todo_args, TodoAddArgs, TodoArgs, TodoCompleteArgs, TodoDeleteArgs,
    TodoDto, TodoInitAiArgs, TodoListArgs, TodoShowArgs, TodoSub, TodoUpdateArgs,
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
    }));
    cmd_todo(add_cmd).unwrap();
    let todos = load_todos().unwrap();
    assert_eq!(todos.len(), 1, "expected exactly one todo after add");
    assert_eq!(todos[0].title, "test task");

    let list_cmd = todo_args(TodoSub::List(TodoListArgs {}));
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

    let list_cmd = todo_args(TodoSub::List(TodoListArgs {}));
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

    let list_cmd = todo_args(TodoSub::List(TodoListArgs {}));
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
    assert!(err.to_string().contains("invalid id 0"));
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
    assert!(err.to_string().contains("invalid id 0"));
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
    assert!(err.to_string().contains("not found"));

    let err = cmd_todo(todo_args(TodoSub::Show(TodoShowArgs { id: 0 }))).unwrap_err();
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
    })))
    .unwrap_err();
    assert!(err.to_string().contains("invalid id 0"));
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
    })))
    .unwrap_err();
    assert_eq!(err.exit_code(), 2);
    assert!(err.to_string().contains("non-empty"));
}

#[test]
fn cmd_todo_add_and_list_with_json() {
    let _guard = cwd_test_lock();
    let dir = std::env::temp_dir().join(format!("xtask_todo_json_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let _ = std::fs::remove_file(".todo.json");

    let add_cmd = TodoArgs {
        sub: TodoSub::Add(TodoAddArgs {
            title: "json task".to_string(),
            description: None,
            due_date: None,
            priority: None,
            tags: None,
            repeat_rule: None,
        }),
        json: true,
        dry_run: false,
    };
    cmd_todo(add_cmd).unwrap();

    let list_cmd = TodoArgs {
        sub: TodoSub::List(TodoListArgs {}),
        json: true,
        dry_run: false,
    };
    cmd_todo(list_cmd).unwrap();

    let todos = load_todos().unwrap();
    assert_eq!(todos.len(), 1);
    assert_eq!(todos[0].title, "json task");
}

#[test]
fn cmd_todo_dry_run_add_does_not_persist() {
    let _guard = cwd_test_lock();
    let dir = std::env::temp_dir().join(format!("xtask_todo_dry_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let _ = std::fs::remove_file(".todo.json");

    let add_cmd = TodoArgs {
        sub: TodoSub::Add(TodoAddArgs {
            title: "dry run".to_string(),
            description: None,
            due_date: None,
            priority: None,
            tags: None,
            repeat_rule: None,
        }),
        json: false,
        dry_run: true,
    };
    cmd_todo(add_cmd).unwrap();

    let todos = load_todos().unwrap();
    assert!(todos.is_empty());
}

#[test]
fn cmd_todo_init_ai_generates_file() {
    let _guard = cwd_test_lock();
    let dir = std::env::temp_dir().join(format!("xtask_todo_initai_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let out_dir = dir.join("out_commands");
    let _ = std::fs::create_dir_all(&out_dir);

    let init_cmd = TodoArgs {
        sub: TodoSub::InitAi(TodoInitAiArgs {
            for_tool: Some("cursor".to_string()),
            output: Some(out_dir.clone()),
        }),
        json: false,
        dry_run: false,
    };
    cmd_todo(init_cmd).unwrap();

    let path = out_dir.join("todo.md");
    assert!(path.exists());
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("Todo list commands"));
    assert!(content.contains("init-ai"));
}

#[test]
fn cmd_todo_show_and_stats_with_json() {
    let _guard = cwd_test_lock();
    let dir = std::env::temp_dir().join(format!("xtask_todo_show_json_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let _ = std::fs::remove_file(".todo.json");
    cmd_todo(todo_args(TodoSub::Add(TodoAddArgs {
        title: "item".to_string(),
        description: None,
        due_date: None,
        priority: None,
        tags: None,
        repeat_rule: None,
    })))
    .unwrap();

    let show_cmd = TodoArgs {
        sub: TodoSub::Show(TodoShowArgs { id: 1 }),
        json: true,
        dry_run: false,
    };
    cmd_todo(show_cmd).unwrap();

    let stats_cmd = TodoArgs {
        sub: TodoSub::Stats(crate::todo::TodoStatsArgs {}),
        json: true,
        dry_run: false,
    };
    cmd_todo(stats_cmd).unwrap();
}

#[test]
fn cmd_todo_init_ai_default_output_dir() {
    let _guard = cwd_test_lock();
    let dir = std::env::temp_dir().join(format!("xtask_todo_initai_def_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);

    let init_cmd = TodoArgs {
        sub: TodoSub::InitAi(TodoInitAiArgs {
            for_tool: None,
            output: None,
        }),
        json: false,
        dry_run: false,
    };
    cmd_todo(init_cmd).unwrap();

    let path = dir.join(".cursor").join("commands").join("todo.md");
    assert!(path.exists());
}

#[test]
fn cmd_todo_init_ai_with_json() {
    let _guard = cwd_test_lock();
    let dir = std::env::temp_dir().join(format!("xtask_todo_initai_json_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let out_dir = dir.join("out_json");
    let _ = std::fs::create_dir_all(&out_dir);

    let init_cmd = TodoArgs {
        sub: TodoSub::InitAi(TodoInitAiArgs {
            for_tool: None,
            output: Some(out_dir),
        }),
        json: true,
        dry_run: false,
    };
    cmd_todo(init_cmd).unwrap();
}
