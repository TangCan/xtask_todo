//! xtask - custom cargo tasks
//!
//! Run with: `cargo xtask <command>`

use argh::FromArgs;
use std::io::IsTerminal;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use todo::{InMemoryStore, Todo, TodoId, TodoList};

fn main() {
    let cmd: XtaskCmd = argh::from_env();
    if let Err(e) = run(cmd) {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

fn run(cmd: XtaskCmd) -> Result<(), Box<dyn std::error::Error>> {
    match cmd.sub {
        XtaskSub::Run(args) => cmd_run(args),
        XtaskSub::Todo(args) => cmd_todo(args),
    }
}

#[derive(FromArgs)]
/// Cargo xtask - custom build/tooling tasks
struct XtaskCmd {
    #[argh(subcommand)]
    sub: XtaskSub,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum XtaskSub {
    Run(RunArgs),
    Todo(TodoArgs),
}

#[derive(FromArgs)]
#[argh(subcommand, name = "run")]
/// Run the main project (example task)
struct RunArgs {}

fn cmd_run(_args: RunArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!("xtask run: placeholder - add your task logic here");
    Ok(())
}

#[derive(FromArgs)]
#[argh(subcommand, name = "todo")]
/// Todo list: add, list, complete, delete (data in .todo.json)
struct TodoArgs {
    #[argh(subcommand)]
    sub: TodoSub,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum TodoSub {
    Add(TodoAddArgs),
    List(TodoListArgs),
    Complete(TodoCompleteArgs),
    Delete(TodoDeleteArgs),
}

#[derive(FromArgs)]
#[argh(subcommand, name = "add")]
/// Add a task
struct TodoAddArgs {
    #[argh(positional)]
    title: String,
}

#[derive(FromArgs)]
#[argh(subcommand, name = "list")]
/// List all tasks
struct TodoListArgs {}

#[derive(FromArgs)]
#[argh(subcommand, name = "complete")]
/// Mark a task as completed by id
struct TodoCompleteArgs {
    #[argh(positional)]
    id: u64,
}

#[derive(FromArgs)]
#[argh(subcommand, name = "delete")]
/// Delete a task by id
struct TodoDeleteArgs {
    #[argh(positional)]
    id: u64,
}

fn todo_file() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let cwd = std::env::current_dir()?;
    Ok(cwd.join(".todo.json"))
}

#[derive(serde::Serialize, serde::Deserialize)]
struct TodoDto {
    id: u64,
    title: String,
    completed: bool,
    created_at_secs: u64,
    #[serde(default)]
    completed_at_secs: Option<u64>,
}

fn load_todos() -> Result<Vec<Todo>, Box<dyn std::error::Error>> {
    let path = todo_file()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let s = std::fs::read_to_string(&path)?;
    let dtos: Vec<TodoDto> = serde_json::from_str(&s).unwrap_or_default();
    let todos = dtos
        .into_iter()
        .filter_map(|d| {
            let id = TodoId::from_raw(d.id)?;
            let created_at = UNIX_EPOCH + Duration::from_secs(d.created_at_secs);
            let completed_at = d
                .completed_at_secs
                .filter(|&s| s > 0)
                .map(|s| UNIX_EPOCH + Duration::from_secs(s));
            Some(Todo {
                id,
                title: d.title,
                completed: d.completed,
                created_at,
                completed_at,
            })
        })
        .collect();
    Ok(todos)
}

const AGE_THRESHOLD_DAYS: u64 = 7;

fn format_time_ago(when: SystemTime) -> String {
    let now = SystemTime::now();
    let d = now.duration_since(when).unwrap_or(Duration::ZERO);
    let s = d.as_secs();
    if s < 60 {
        "just now".into()
    } else if s < 3600 {
        format!("{}m ago", s / 60)
    } else if s < 86400 {
        format!("{}h ago", s / 3600)
    } else {
        format!("{}d ago", s / 86400)
    }
}

fn format_duration(d: Duration) -> String {
    let s = d.as_secs();
    if s < 60 {
        format!("{}s", s)
    } else if s < 3600 {
        format!("{}m", s / 60)
    } else if s < 86400 {
        format!("{}h", s / 3600)
    } else {
        format!("{}d", s / 86400)
    }
}

fn is_old_open(t: &Todo, now: SystemTime) -> bool {
    if t.completed {
        return false;
    }
    let age = now.duration_since(t.created_at).unwrap_or(Duration::ZERO);
    age.as_secs() >= AGE_THRESHOLD_DAYS * 86400
}

fn save_todos(list: &TodoList<InMemoryStore>) -> Result<(), Box<dyn std::error::Error>> {
    let path = todo_file()?;
    let dtos: Vec<TodoDto> = list
        .list()
        .into_iter()
        .map(|t| TodoDto {
            id: t.id.as_u64(),
            title: t.title.clone(),
            completed: t.completed,
            created_at_secs: t
                .created_at
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_secs(),
            completed_at_secs: t
                .completed_at
                .and_then(|ct| ct.duration_since(UNIX_EPOCH).ok().map(|d| d.as_secs())),
        })
        .collect();
    let s = serde_json::to_string_pretty(&dtos)?;
    std::fs::write(path, s)?;
    Ok(())
}

fn cmd_todo(args: TodoArgs) -> Result<(), Box<dyn std::error::Error>> {
    let todos = load_todos()?;
    let store = InMemoryStore::from_todos(todos);
    let mut list = TodoList::with_store(store);

    match args.sub {
        TodoSub::Add(a) => {
            let id = list.create(&a.title)?;
            save_todos(&list)?;
            println!("Added [{}] {}", id, a.title.trim());
        }
        TodoSub::List(_) => {
            let items = list.list();
            let use_color = std::io::stdout().is_terminal();
            let now = SystemTime::now();
            if items.is_empty() {
                println!("No tasks.");
            } else {
                for t in items {
                    let mark = if t.completed { "✓" } else { " " };
                    let created = format_time_ago(t.created_at);
                    let time_info = match &t.completed_at {
                        Some(cat) => {
                            let completed = format_time_ago(*cat);
                            let took = cat
                                .duration_since(t.created_at)
                                .ok()
                                .map(format_duration)
                                .map(|s| format!("  用时 {}", s))
                                .unwrap_or_default();
                            format!("  创建 {}  完成 {}{}", created, completed, took)
                        }
                        None => format!("  创建 {}", created),
                    };
                    let line = format!("  [{}] {} {}  {}", t.id, mark, t.title, time_info);
                    if use_color && is_old_open(&t, now) {
                        println!("\x1b[33m{}\x1b[0m", line);
                    } else {
                        println!("{}", line);
                    }
                }
            }
        }
        TodoSub::Complete(a) => {
            let id = TodoId::from_raw(a.id).ok_or("invalid id 0")?;
            list.complete(id)?;
            save_todos(&list)?;
            println!("Completed [{}]", id);
        }
        TodoSub::Delete(a) => {
            let id = TodoId::from_raw(a.id).ok_or("invalid id 0")?;
            list.delete(id)?;
            save_todos(&list)?;
            println!("Deleted [{}]", id);
        }
    }
    Ok(())
}
