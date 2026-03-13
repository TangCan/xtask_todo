//! JSON output, dry-run, and init-ai tests for `cmd_todo`.

use crate::tests::{cwd_test_lock, RestoreCwd};
use crate::todo::{
    cmd_todo, load_todos, todo_args, TodoAddArgs, TodoArgs, TodoInitAiArgs, TodoListArgs,
    TodoShowArgs, TodoSub,
};

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
            repeat_until: None,
            repeat_count: None,
        }),
        json: true,
        dry_run: false,
    };
    cmd_todo(add_cmd).unwrap();

    let list_cmd = TodoArgs {
        sub: TodoSub::List(TodoListArgs::default()),
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
            repeat_until: None,
            repeat_count: None,
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
        repeat_until: None,
        repeat_count: None,
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

#[test]
fn cmd_todo_add_invalid_priority_returns_parameter_error() {
    let _guard = cwd_test_lock();
    let dir = std::env::temp_dir().join(format!("xtask_todo_bad_pri_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let _ = std::fs::remove_file(".todo.json");

    let err = cmd_todo(todo_args(TodoSub::Add(TodoAddArgs {
        title: "x".to_string(),
        description: None,
        due_date: None,
        priority: Some("invalid_priority".to_string()),
        tags: None,
        repeat_rule: None,
        repeat_until: None,
        repeat_count: None,
    })))
    .unwrap_err();
    assert_eq!(err.exit_code(), 2);
    assert!(err.to_string().contains("invalid priority"));
}

#[test]
fn cmd_todo_add_invalid_repeat_rule_returns_parameter_error() {
    let _guard = cwd_test_lock();
    let dir = std::env::temp_dir().join(format!("xtask_todo_bad_rep_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let _ = std::fs::remove_file(".todo.json");

    let err = cmd_todo(todo_args(TodoSub::Add(TodoAddArgs {
        title: "x".to_string(),
        description: None,
        due_date: None,
        priority: None,
        tags: None,
        repeat_rule: Some("bad_rule".to_string()),
        repeat_until: None,
        repeat_count: None,
    })))
    .unwrap_err();
    assert_eq!(err.exit_code(), 2);
    assert!(err.to_string().contains("invalid repeat_rule"));
}
