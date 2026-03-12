//! File I/O for todo list (.todo.json and custom paths).

use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::{Duration, UNIX_EPOCH};

use xtask_todo_lib::{InMemoryStore, Priority, RepeatRule, Todo, TodoId, TodoList};

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

/// Path to the todo JSON file in the current directory.
///
/// # Errors
/// Returns an error if the current directory cannot be determined.
pub fn todo_file() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let cwd = std::env::current_dir()?;
    Ok(cwd.join(".todo.json"))
}

fn dto_to_todo(d: TodoDto) -> Option<Todo> {
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
    let todos = dtos.into_iter().filter_map(dto_to_todo).collect();
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
    let todos = dtos.into_iter().filter_map(dto_to_todo).collect();
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
        .iter()
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

/// Save todos to `.todo.json` in the current directory.
///
/// # Errors
/// Returns an error if the file cannot be written.
pub fn save_todos(list: &TodoList<InMemoryStore>) -> Result<(), Box<dyn std::error::Error>> {
    let path = todo_file()?;
    let dtos: Vec<TodoDto> = list
        .list()
        .iter()
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
