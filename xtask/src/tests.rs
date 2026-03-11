//! Unit tests for xtask commands.

use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use crate::clippy::{status_to_result, ClippyArgs};
use crate::git::{cmd_git, GitAddArgs, GitArgs, GitCommitArgs, GitSub};
use crate::run::RunArgs;
use crate::todo::{
    cmd_todo, format_duration, format_time_ago, is_old_open, load_todos, print_todo_list_items,
    todo_file, TodoAddArgs, TodoArgs, TodoCompleteArgs, TodoDeleteArgs, TodoDto, TodoListArgs,
    TodoSub, AGE_THRESHOLD_DAYS,
};
use crate::{run_with, XtaskCmd, XtaskSub};
use todo::{Todo, TodoId};

#[test]
fn status_to_result_success() {
    let status = std::process::Command::new("true").status().unwrap();
    assert!(status_to_result(status, "test").is_ok());
}

#[test]
fn run_subcommand_run() {
    let cmd = XtaskCmd {
        sub: XtaskSub::Run(RunArgs {}),
    };
    let out = run_with(cmd);
    assert!(out.is_ok());
}

#[test]
fn run_subcommand_clippy() {
    let cmd = XtaskCmd {
        sub: XtaskSub::Clippy(ClippyArgs {}),
    };
    let _ = run_with(cmd);
}

#[test]
fn run_subcommand_git_add() {
    let cmd = XtaskCmd {
        sub: XtaskSub::Git(GitArgs {
            sub: GitSub::Add(GitAddArgs {}),
        }),
    };
    let _ = run_with(cmd);
}

#[test]
fn run_subcommand_git_commit() {
    let cmd = XtaskCmd {
        sub: XtaskSub::Git(GitArgs {
            sub: GitSub::Commit(GitCommitArgs {}),
        }),
    };
    let _ = run_with(cmd);
}

#[test]
fn cmd_git_add_in_nongit_dir_returns_err() {
    let dir = std::env::temp_dir().join(format!("xtask_nongit_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let cmd = GitArgs {
        sub: GitSub::Add(GitAddArgs {}),
    };
    let result = cmd_git(&cmd);
    assert!(result.is_err());
}

#[test]
fn cmd_clippy_fail_returns_err() {
    let dir = std::env::temp_dir().join(format!("xtask_clippy_fail_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    std::fs::create_dir_all("src").unwrap();
    std::fs::write(
        "Cargo.toml",
        r#"[package]
name = "fail"
version = "0.1.0"
[dependencies]
nonexistent_crate_xyz = "999"
"#,
    )
    .unwrap();
    std::fs::write("src/lib.rs", "pub fn f() {}").unwrap();
    let result = crate::clippy::cmd_clippy(ClippyArgs {});
    assert!(result.is_err());
}

#[test]
fn cmd_git_commit_with_nothing_to_commit_returns_err() {
    let dir = std::env::temp_dir().join(format!("xtask_git_commit_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _guard = RestoreCwd::new(&dir, &cwd);
    let _ = std::process::Command::new("git").args(["init"]).status();
    let cmd = GitArgs {
        sub: GitSub::Commit(GitCommitArgs {}),
    };
    let result = cmd_git(&cmd);
    assert!(result.is_err());
}

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
    let when = SystemTime::now() - Duration::from_secs(2 * 3600); // 2h
    let s = format_time_ago(when);
    assert!(s.ends_with("h ago"));
}

#[test]
fn format_time_ago_days() {
    let when = SystemTime::now() - Duration::from_secs(48 * 3600); // 2 days
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
    assert_eq!(format_duration(Duration::from_secs(2 * 3600)), "2h");
}

#[test]
fn format_duration_days() {
    assert_eq!(format_duration(Duration::from_secs(72 * 3600)), "3d");
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

struct RestoreCwd(PathBuf);
impl RestoreCwd {
    fn new(dir: &std::path::Path, cwd: &std::path::Path) -> Self {
        std::env::set_current_dir(dir).unwrap();
        Self(cwd.to_path_buf())
    }
}
impl Drop for RestoreCwd {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.0);
    }
}

#[test]
fn cmd_todo_complete_and_delete() {
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
