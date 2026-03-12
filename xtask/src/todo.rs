//! `todo` subcommand - add, list, complete, delete (data in .todo.json).

use argh::FromArgs;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use xtask_todo_lib::{InMemoryStore, Priority, RepeatRule, Todo, TodoId, TodoList};

/// Path to the todo JSON file in the current directory.
///
/// # Errors
/// Returns an error if the current directory cannot be determined.
pub fn todo_file() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let cwd = std::env::current_dir()?;
    Ok(cwd.join(".todo.json"))
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct TodoDto {
    pub id: u64,
    pub title: String,
    pub completed: bool,
    pub created_at_secs: u64,
    #[serde(default)]
    pub completed_at_secs: Option<u64>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub due_date: Option<String>,
    #[serde(default)]
    pub priority: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub repeat_rule: Option<String>,
}

/// Load todos from `.todo.json` in the current directory.
///
/// # Errors
/// Returns an error if the file cannot be read or the path is invalid.
pub fn load_todos() -> Result<Vec<Todo>, Box<dyn std::error::Error>> {
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
            let priority = d
                .priority
                .as_deref()
                .and_then(|s| Priority::from_str(s).ok());
            let repeat_rule = d
                .repeat_rule
                .as_deref()
                .and_then(|s| RepeatRule::from_str(s).ok());
            Some(Todo {
                id,
                title: d.title,
                completed: d.completed,
                created_at,
                completed_at,
                description: d.description,
                due_date: d.due_date,
                priority,
                tags: d.tags,
                repeat_rule,
            })
        })
        .collect();
    Ok(todos)
}

/// Load todos from a JSON file at the given path.
///
/// # Errors
/// Returns an error if the file cannot be read or JSON is invalid.
pub fn load_todos_from_path(path: &Path) -> Result<Vec<Todo>, Box<dyn std::error::Error>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let s = std::fs::read_to_string(path)?;
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
            let priority = d
                .priority
                .as_deref()
                .and_then(|s| Priority::from_str(s).ok());
            let repeat_rule = d
                .repeat_rule
                .as_deref()
                .and_then(|s| RepeatRule::from_str(s).ok());
            Some(Todo {
                id,
                title: d.title,
                completed: d.completed,
                created_at,
                completed_at,
                description: d.description,
                due_date: d.due_date,
                priority,
                tags: d.tags,
                repeat_rule,
            })
        })
        .collect();
    Ok(todos)
}

/// Save todos to a JSON file at the given path.
///
/// # Errors
/// Returns an error if the file cannot be written.
pub fn save_todos_to_path(
    list: &TodoList<InMemoryStore>,
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
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
            description: t.description.clone(),
            due_date: t.due_date.clone(),
            priority: t.priority.map(|p| p.to_string()),
            tags: t.tags.clone(),
            repeat_rule: t.repeat_rule.as_ref().map(ToString::to_string),
        })
        .collect();
    let s = serde_json::to_string_pretty(&dtos)?;
    std::fs::write(path, s)?;
    Ok(())
}

pub const AGE_THRESHOLD_DAYS: u64 = 7;

#[must_use]
pub fn format_time_ago(when: SystemTime) -> String {
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

#[must_use]
pub fn format_duration(d: Duration) -> String {
    let s = d.as_secs();
    if s < 60 {
        format!("{s}s")
    } else if s < 3600 {
        format!("{}m", s / 60)
    } else if s < 86400 {
        format!("{}h", s / 3600)
    } else {
        format!("{}d", s / 86400)
    }
}

#[must_use]
pub fn is_old_open(t: &Todo, now: SystemTime) -> bool {
    if t.completed {
        return false;
    }
    let age = now.duration_since(t.created_at).unwrap_or(Duration::ZERO);
    age.as_secs() >= AGE_THRESHOLD_DAYS * 86400
}

/// Save todos to `.todo.json` in the current directory.
///
/// # Errors
/// Returns an error if the file cannot be written.
pub fn save_todos(list: &TodoList<InMemoryStore>) -> Result<(), Box<dyn std::error::Error>> {
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
            description: t.description.clone(),
            due_date: t.due_date.clone(),
            priority: t.priority.map(|p| p.to_string()),
            tags: t.tags.clone(),
            repeat_rule: t.repeat_rule.as_ref().map(ToString::to_string),
        })
        .collect();
    let s = serde_json::to_string_pretty(&dtos)?;
    std::fs::write(path, s)?;
    Ok(())
}

/// Prints todo list items to stdout. Used by list subcommand and tests.
#[allow(clippy::missing_panics_doc)]
pub fn print_todo_list_items(items: &[Todo], use_color: bool) {
    let now = SystemTime::now();
    if items.is_empty() {
        println!("No tasks.");
    } else {
        for t in items {
            let mark = if t.completed { "✓" } else { " " };
            let created = format_time_ago(t.created_at);
            let time_info = t.completed_at.as_ref().map_or_else(
                || format!("  创建 {created}"),
                |cat| {
                    let completed = format_time_ago(*cat);
                    let took = cat
                        .duration_since(t.created_at)
                        .ok()
                        .map(format_duration)
                        .map(|s| format!("  用时 {s}"))
                        .unwrap_or_default();
                    format!("  创建 {created}  完成 {completed}{took}")
                },
            );
            let line = format!("  [{}] {} {}  {}", t.id, mark, t.title, time_info);
            if use_color && is_old_open(t, now) {
                println!("\x1b[33m{line}\x1b[0m");
            } else {
                println!("{line}");
            }
        }
    }
}

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "todo")]
/// Todo list: add, list, complete, delete (data in .todo.json)
pub struct TodoArgs {
    #[argh(subcommand)]
    pub sub: TodoSub,
}

#[derive(FromArgs, Clone)]
#[argh(subcommand)]
pub enum TodoSub {
    Add(TodoAddArgs),
    List(TodoListArgs),
    Show(TodoShowArgs),
    Update(TodoUpdateArgs),
    Complete(TodoCompleteArgs),
    Delete(TodoDeleteArgs),
    Search(TodoSearchArgs),
    Stats(TodoStatsArgs),
    Export(TodoExportArgs),
    Import(TodoImportArgs),
}

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "search")]
/// Search tasks by keyword (title, description, tags)
pub struct TodoSearchArgs {
    #[argh(positional)]
    pub keyword: String,
}

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "stats")]
/// Show task counts (total, incomplete, complete)
pub struct TodoStatsArgs {}

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "export")]
/// Export tasks to a JSON file
pub struct TodoExportArgs {
    #[argh(positional)]
    pub file: PathBuf,
}

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "import")]
/// Import tasks from a JSON file (merge or replace)
pub struct TodoImportArgs {
    #[argh(positional)]
    pub file: PathBuf,
    #[argh(switch)]
    /// replace current list instead of merging
    pub replace: bool,
}

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "show")]
/// Show a single task by id
pub struct TodoShowArgs {
    #[argh(positional)]
    pub id: u64,
}

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "update")]
/// Update a task's title by id
pub struct TodoUpdateArgs {
    #[argh(positional)]
    pub id: u64,
    #[argh(positional)]
    pub title: String,
}

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "add")]
/// Add a task
pub struct TodoAddArgs {
    #[argh(positional)]
    pub title: String,
}

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "list")]
/// List all tasks
pub struct TodoListArgs {}

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "complete")]
/// Mark a task as completed by id
pub struct TodoCompleteArgs {
    #[argh(positional)]
    pub id: u64,
    #[argh(switch)]
    /// do not create next instance for recurring tasks
    pub no_next: bool,
}

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "delete")]
/// Delete a task by id
pub struct TodoDeleteArgs {
    #[argh(positional)]
    pub id: u64,
}

/// Handle todo subcommand (add, list, complete, delete).
///
/// # Errors
/// Returns an error on I/O, invalid id, or todo operations (e.g. not found).
pub fn cmd_todo(args: TodoArgs) -> Result<(), Box<dyn std::error::Error>> {
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
            print_todo_list_items(&items, use_color);
        }
        TodoSub::Show(a) => {
            let id = TodoId::from_raw(a.id).ok_or("invalid id 0")?;
            match list.get(id) {
                Some(t) => {
                    let mark = if t.completed { "✓" } else { " " };
                    let created = format_time_ago(t.created_at);
                    let time_info = t.completed_at.as_ref().map_or_else(
                        || format!("  创建 {created}"),
                        |cat| {
                            let completed = format_time_ago(*cat);
                            let took = cat
                                .duration_since(t.created_at)
                                .ok()
                                .map(format_duration)
                                .map(|s| format!("  用时 {s}"))
                                .unwrap_or_default();
                            format!("  创建 {created}  完成 {completed}{took}")
                        },
                    );
                    println!("  [{}] {} {}  {}", t.id, mark, t.title, time_info);
                }
                None => return Err(format!("todo not found: {id}").into()),
            }
        }
        TodoSub::Update(a) => {
            let id = TodoId::from_raw(a.id).ok_or("invalid id 0")?;
            list.update_title(id, &a.title)?;
            save_todos(&list)?;
            println!("Updated [{}] {}", id, a.title.trim());
        }
        TodoSub::Complete(a) => {
            let id = TodoId::from_raw(a.id).ok_or("invalid id 0")?;
            list.complete(id, a.no_next)?;
            save_todos(&list)?;
            println!("Completed [{id}]");
        }
        TodoSub::Delete(a) => {
            let id = TodoId::from_raw(a.id).ok_or("invalid id 0")?;
            list.delete(id)?;
            save_todos(&list)?;
            println!("Deleted [{id}]");
        }
        TodoSub::Search(a) => {
            let items = list.search(&a.keyword);
            let use_color = std::io::stdout().is_terminal();
            print_todo_list_items(&items, use_color);
        }
        TodoSub::Stats(_) => {
            let (total, incomplete, complete) = list.stats();
            println!("Total: {total}  Incomplete: {incomplete}  Complete: {complete}");
        }
        TodoSub::Export(a) => {
            save_todos_to_path(&list, &a.file)?;
            println!(
                "Exported {} tasks to {}",
                list.list().len(),
                a.file.display()
            );
        }
        TodoSub::Import(a) => {
            let imported = load_todos_from_path(&a.file)?;
            if a.replace {
                let store = InMemoryStore::from_todos(imported.clone());
                let new_list = TodoList::with_store(store);
                save_todos(&new_list)?;
                println!(
                    "Replaced with {} tasks from {}",
                    new_list.list().len(),
                    a.file.display()
                );
            } else {
                for t in &imported {
                    list.add_todo(t);
                }
                save_todos(&list)?;
                println!("Merged {} tasks from {}", imported.len(), a.file.display());
            }
        }
    }
    Ok(())
}
