//! File I/O for todo list (`.todo.json`). Same format as xtask so `dev_shell` and `cargo xtask todo` share data.
//!
//! **Mode P / guest-primary:** paths stay on the **host** current directory (design §11 **A**); they are
//! **not** mapped into the guest project tree.

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
///
/// # Errors
/// Returns error if `current_dir()` fails.
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
///
/// # Errors
/// Returns error on I/O or invalid JSON (invalid entries are skipped).
pub fn load_todos() -> Result<Vec<Todo>, Box<dyn std::error::Error>> {
    let path = todo_file()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let s = super::host_text::read_host_text(&path)?;
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
///
/// # Errors
/// Returns error on I/O or serialization failure.
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
    fn todo_file_points_to_current_dir_dot_todo_json() {
        let _g = crate::test_support::cwd_mutex();
        let dir = std::env::temp_dir().join(format!(
            "todo_io_path_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).expect("create dir");
        let cwd = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(&dir).expect("set cwd");
        let file = todo_file().expect("todo_file");
        std::env::set_current_dir(&cwd).expect("restore cwd");
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(file, dir.join(".todo.json"));
    }

    #[test]
    fn load_todos_when_file_missing_returns_empty() {
        let _g = crate::test_support::cwd_mutex();
        let dir = std::env::temp_dir().join(format!(
            "todo_io_nojson_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let todo_json = dir.join(".todo.json");
        let _ = std::fs::remove_file(&todo_json);
        let cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();
        let result = load_todos();
        std::env::set_current_dir(&cwd).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        let todos = result.unwrap();
        assert!(todos.is_empty());
    }
}
