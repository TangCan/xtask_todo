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
    #[serde(default)]
    pub repeat_until: Option<String>,
    #[serde(default)]
    pub repeat_count: Option<u32>,
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
        repeat_until: d.repeat_until,
        repeat_count: d.repeat_count,
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

/// Load todos from a file at the given path (format inferred from extension: .csv → CSV, else JSON).
///
/// # Errors
/// Returns an error if the file cannot be read or content is invalid.
pub fn load_todos_from_path(path: &Path) -> Result<Vec<Todo>, Box<dyn std::error::Error>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let is_csv = path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("csv"));
    let s = std::fs::read_to_string(path)?;
    if is_csv {
        Ok(load_todos_from_csv(&s))
    } else {
        let dtos: Vec<TodoDto> = serde_json::from_str(&s).unwrap_or_default();
        Ok(dtos.into_iter().filter_map(dto_to_todo).collect())
    }
}

/// Save todos to a file (format: "json" or "csv").
///
/// # Errors
/// Returns an error if the file cannot be written.
pub fn save_todos_to_path_with_format(
    list: &TodoList<InMemoryStore>,
    path: &Path,
    format: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if format.eq_ignore_ascii_case("csv") {
        save_todos_to_csv(list, path)
    } else {
        save_todos_to_path(list, path)
    }
}

fn csv_escape(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

fn save_todos_to_csv(
    list: &TodoList<InMemoryStore>,
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut w = std::io::BufWriter::new(std::fs::File::create(path)?);
    let header = "id,title,completed,created_at_secs,completed_at_secs,description,due_date,priority,tags,repeat_rule,repeat_until,repeat_count";
    std::io::Write::write_all(&mut w, header.as_bytes())?;
    std::io::Write::write_all(&mut w, b"\n")?;
    for t in list.list() {
        let created = t
            .created_at
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();
        let completed_secs = t
            .completed_at
            .and_then(|ct| ct.duration_since(UNIX_EPOCH).ok().map(|d| d.as_secs()));
        let tags_cell = t.tags.join(";");
        let row = format!(
            "{},{},{},{},{},{},{},{},{},{},{},{}\n",
            t.id.as_u64(),
            csv_escape(&t.title),
            t.completed,
            created,
            completed_secs.unwrap_or(0),
            csv_escape(t.description.as_deref().unwrap_or("")),
            csv_escape(t.due_date.as_deref().unwrap_or("")),
            csv_escape(
                t.priority
                    .as_ref()
                    .map(ToString::to_string)
                    .as_deref()
                    .unwrap_or("")
            ),
            csv_escape(&tags_cell),
            csv_escape(
                t.repeat_rule
                    .as_ref()
                    .map(ToString::to_string)
                    .as_deref()
                    .unwrap_or("")
            ),
            csv_escape(t.repeat_until.as_deref().unwrap_or("")),
            t.repeat_count.unwrap_or(0),
        );
        std::io::Write::write_all(&mut w, row.as_bytes())?;
    }
    Ok(())
}

#[derive(serde::Deserialize)]
struct CsvRow {
    id: u64,
    title: String,
    completed: bool,
    created_at_secs: u64,
    completed_at_secs: u64,
    description: String,
    due_date: String,
    priority: String,
    tags: String,
    repeat_rule: String,
    repeat_until: String,
    repeat_count: u64,
}

fn load_todos_from_csv(s: &str) -> Vec<Todo> {
    let mut rdr = csv::Reader::from_reader(s.as_bytes());
    let mut todos = Vec::new();
    for result in rdr.deserialize() {
        let row: CsvRow = result.unwrap_or_else(|_| CsvRow {
            id: 0,
            title: String::new(),
            completed: false,
            created_at_secs: 0,
            completed_at_secs: 0,
            description: String::new(),
            due_date: String::new(),
            priority: String::new(),
            tags: String::new(),
            repeat_rule: String::new(),
            repeat_until: String::new(),
            repeat_count: 0,
        });
        if row.id == 0 {
            continue;
        }
        let d = TodoDto {
            id: row.id,
            title: row.title,
            completed: row.completed,
            created_at_secs: row.created_at_secs,
            completed_at_secs: if row.completed_at_secs > 0 {
                Some(row.completed_at_secs)
            } else {
                None
            },
            description: if row.description.is_empty() {
                None
            } else {
                Some(row.description)
            },
            due_date: if row.due_date.is_empty() {
                None
            } else {
                Some(row.due_date)
            },
            priority: if row.priority.is_empty() {
                None
            } else {
                Some(row.priority)
            },
            tags: row
                .tags
                .split(';')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect(),
            repeat_rule: if row.repeat_rule.is_empty() {
                None
            } else {
                Some(row.repeat_rule)
            },
            repeat_until: if row.repeat_until.is_empty() {
                None
            } else {
                Some(row.repeat_until)
            },
            repeat_count: if row.repeat_count == 0 {
                None
            } else {
                u32::try_from(row.repeat_count).ok()
            },
        };
        if let Some(t) = dto_to_todo(d) {
            todos.push(t);
        }
    }
    todos
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
            repeat_until: t.repeat_until.clone(),
            repeat_count: t.repeat_count,
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
            repeat_until: t.repeat_until.clone(),
            repeat_count: t.repeat_count,
        })
        .collect();
    let s = serde_json::to_string_pretty(&dtos)?;
    std::fs::write(path, s)?;
    Ok(())
}
