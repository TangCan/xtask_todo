//! Tests for `cmd_todo` search, stats, export/import, and save/load path.

use crate::tests::{cwd_test_lock, RestoreCwd};
use crate::todo::{
    cmd_todo, load_todos, load_todos_from_path, save_todos_to_path, save_todos_to_path_with_format,
    todo_args, TodoAddArgs, TodoArgs, TodoDto, TodoExportArgs, TodoImportArgs, TodoSearchArgs,
    TodoStatsArgs, TodoSub,
};

#[test]
fn cmd_todo_search_and_stats() {
    let _guard = cwd_test_lock();
    let dir = std::env::temp_dir().join(format!("xtask_todo_srch_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let _ = std::fs::remove_file(".todo.json");
    cmd_todo(todo_args(TodoSub::Add(TodoAddArgs {
        title: "alpha".to_string(),
        description: None,
        due_date: None,
        priority: None,
        tags: None,
        repeat_rule: None,
        repeat_until: None,
        repeat_count: None,
    })))
    .unwrap();
    cmd_todo(todo_args(TodoSub::Add(TodoAddArgs {
        title: "beta".to_string(),
        description: None,
        due_date: None,
        priority: None,
        tags: None,
        repeat_rule: None,
        repeat_until: None,
        repeat_count: None,
    })))
    .unwrap();
    cmd_todo(todo_args(TodoSub::Search(TodoSearchArgs {
        keyword: "alpha".to_string(),
    })))
    .unwrap();
    cmd_todo(todo_args(TodoSub::Stats(TodoStatsArgs {}))).unwrap();
}

#[test]
fn cmd_todo_export_and_import_merge_replace() {
    let _guard = cwd_test_lock();
    let dir = std::env::temp_dir().join(format!("xtask_todo_io_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let _ = std::fs::remove_file(".todo.json");
    let export_path = dir.join("export.json");
    cmd_todo(todo_args(TodoSub::Add(TodoAddArgs {
        title: "one".to_string(),
        description: None,
        due_date: None,
        priority: None,
        tags: None,
        repeat_rule: None,
        repeat_until: None,
        repeat_count: None,
    })))
    .unwrap();
    cmd_todo(todo_args(TodoSub::Export(TodoExportArgs {
        file: export_path.clone(),
        format: None,
    })))
    .unwrap();
    assert!(export_path.exists());
    let from_export = load_todos_from_path(&export_path).unwrap();
    assert_eq!(from_export.len(), 1);
    assert_eq!(from_export[0].title, "one");

    let import_path = dir.join("import.json");
    let import_dtos = vec![TodoDto {
        id: 1,
        title: "imported".to_string(),
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
    std::fs::write(
        &import_path,
        serde_json::to_string_pretty(&import_dtos).unwrap(),
    )
    .unwrap();
    cmd_todo(todo_args(TodoSub::Import(TodoImportArgs {
        file: import_path.clone(),
        replace: false,
    })))
    .unwrap();
    let todos = load_todos().unwrap();
    assert_eq!(todos.len(), 2);

    cmd_todo(todo_args(TodoSub::Import(TodoImportArgs {
        file: import_path,
        replace: true,
    })))
    .unwrap();
    let todos = load_todos().unwrap();
    assert_eq!(todos.len(), 1);
    assert_eq!(todos[0].title, "imported");
}

#[test]
fn cmd_todo_export_and_import_with_json() {
    let _guard = cwd_test_lock();
    let dir = std::env::temp_dir().join(format!("xtask_todo_io_json_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let _ = std::fs::remove_file(".todo.json");
    cmd_todo(todo_args(TodoSub::Add(TodoAddArgs {
        title: "one".to_string(),
        description: None,
        due_date: None,
        priority: None,
        tags: None,
        repeat_rule: None,
        repeat_until: None,
        repeat_count: None,
    })))
    .unwrap();
    let export_path = dir.join("out.json");
    cmd_todo(TodoArgs {
        sub: TodoSub::Export(TodoExportArgs {
            file: export_path.clone(),
            format: None,
        }),
        json: true,
        dry_run: false,
    })
    .unwrap();
    assert!(export_path.exists());

    let import_path = dir.join("in.json");
    std::fs::write(
        &import_path,
        r#"[{"id":1,"title":"x","completed":false,"created_at_secs":0,"tags":[]}]"#,
    )
    .unwrap();
    cmd_todo(TodoArgs {
        sub: TodoSub::Import(TodoImportArgs {
            file: import_path,
            replace: false,
        }),
        json: true,
        dry_run: false,
    })
    .unwrap();
    let todos = load_todos().unwrap();
    assert_eq!(todos.len(), 2);
}

#[test]
fn save_todos_to_path_and_load_todos_from_path() {
    let _guard = cwd_test_lock();
    let dir = std::env::temp_dir().join(format!("xtask_todo_path_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let path = dir.join("custom.json");
    let mut list = xtask_todo_lib::TodoList::new();
    let _ = list.create("saved");
    save_todos_to_path(&list, &path).unwrap();
    assert!(path.exists());
    let loaded = load_todos_from_path(&path).unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].title, "saved");
    let empty_path = dir.join("nonexistent.json");
    let empty = load_todos_from_path(&empty_path).unwrap();
    assert!(empty.is_empty());
}

#[test]
fn cmd_todo_export_csv_and_import_csv() {
    let _guard = cwd_test_lock();
    let dir = std::env::temp_dir().join(format!("xtask_todo_csv_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let _ = std::fs::remove_file(".todo.json");
    cmd_todo(todo_args(TodoSub::Add(TodoAddArgs {
        title: "csv task".to_string(),
        description: None,
        due_date: None,
        priority: None,
        tags: None,
        repeat_rule: None,
        repeat_until: None,
        repeat_count: None,
    })))
    .unwrap();

    let csv_path = dir.join("data.csv");
    cmd_todo(todo_args(TodoSub::Export(TodoExportArgs {
        file: csv_path.clone(),
        format: Some("csv".to_string()),
    })))
    .unwrap();
    assert!(csv_path.exists());
    let content = std::fs::read_to_string(&csv_path).unwrap();
    assert!(content.contains("id,title,completed"));
    assert!(content.contains("csv task"));

    let _ = std::fs::remove_file(".todo.json");
    cmd_todo(todo_args(TodoSub::Import(TodoImportArgs {
        file: csv_path,
        replace: false,
    })))
    .unwrap();
    let todos = load_todos().unwrap();
    assert_eq!(todos.len(), 1);
    assert_eq!(todos[0].title, "csv task");
}

#[test]
fn load_todos_from_path_csv_with_special_chars() {
    let _guard = cwd_test_lock();
    let dir = std::env::temp_dir().join(format!("xtask_todo_csv_esc_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let mut list = xtask_todo_lib::TodoList::new();
    let _ = list.create("title with, comma");
    let path = dir.join("esc.csv");
    save_todos_to_path_with_format(&list, &path, "csv").unwrap();
    let loaded = load_todos_from_path(&path).unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].title, "title with, comma");
}

#[test]
fn save_todos_to_path_with_format_csv() {
    let _guard = cwd_test_lock();
    let dir = std::env::temp_dir().join(format!("xtask_todo_fmt_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let mut list = xtask_todo_lib::TodoList::new();
    let _ = list.create("one");
    let _ = list.create("two");
    let path = dir.join("out.csv");
    save_todos_to_path_with_format(&list, &path, "csv").unwrap();
    assert!(path.exists());
    let loaded = load_todos_from_path(&path).unwrap();
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0].title, "one");
    assert_eq!(loaded[1].title, "two");
}
