//! File I/O for todo list (.todo.json). Same format as xtask so `dev_shell` and `cargo xtask todo` share data.

use std::path::PathBuf;
use std::str::FromStr;
use std::time::{Duration, UNIX_EPOCH};

use crate::{InMemoryStore, Priority, RepeatRule, Todo, TodoId, TodoList};

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
    #[serde(default)]
    pub repeat_until: Option<String>,
    #[serde(default)]
    pub repeat_count: Option<u32>,
}

/// Path to `.todo.json` in the current directory.
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
        repeat_until: d.repeat_until,
        repeat_count: d.repeat_count,
    })
}

/// Load todos from `.todo.json` in the current directory.
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

/// Build a `TodoList` from loaded todos (call after `load_todos`).
#[must_use]
pub fn list_from_todos(todos: Vec<Todo>) -> TodoList<InMemoryStore> {
    TodoList::with_store(InMemoryStore::from_todos(todos))
}

/// Save todos to `.todo.json` in the current directory.
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
            repeat_until: t.repeat_until.clone(),
            repeat_count: t.repeat_count,
        })
        .collect();
    let s = serde_json::to_string_pretty(&dtos)?;
    std::fs::write(path, s)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_todos_when_file_missing_returns_empty() {
        let dir = std::env::temp_dir().join(format!("todo_io_nojson_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let cwd = std::env::current_dir().unwrap();
        let _ = std::env::set_current_dir(&dir);
        let result = load_todos();
        let _ = std::env::set_current_dir(&cwd);
        let _ = std::fs::remove_dir(&dir);
        let todos = result.unwrap();
        assert!(todos.is_empty());
    }
}
