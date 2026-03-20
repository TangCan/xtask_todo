//! Built-in `todo` subcommands (shared `.todo.json` with `cargo xtask todo`).
//! Storage path is host [`crate::devshell::todo_io`] (design §11 **A**; unchanged in Mode P).

use std::io::Write;

use crate::{InMemoryStore, TodoId, TodoList};

use super::super::todo_io::{list_from_todos, load_todos, save_todos};
use super::types::BuiltinError;

fn run_todo_list(
    stdout: &mut dyn Write,
    list: &TodoList<InMemoryStore>,
    rest: &[String],
) -> Result<(), BuiltinError> {
    let use_json = rest.iter().any(|a| a == "--json");
    if use_json {
        #[derive(serde::Serialize)]
        struct TodoRow {
            id: u64,
            title: String,
            completed: bool,
        }
        let rows: Vec<TodoRow> = list
            .list()
            .into_iter()
            .map(|t| TodoRow {
                id: t.id.as_u64(),
                title: t.title.clone(),
                completed: t.completed,
            })
            .collect();
        let json = serde_json::to_string_pretty(&rows).map_err(|_| BuiltinError::TodoDataError)?;
        writeln!(stdout, "{json}").map_err(|_| BuiltinError::RedirectWrite)?;
    } else {
        for t in list.list() {
            let done = if t.completed { " [done]" } else { "" };
            writeln!(stdout, "{}. {}{}", t.id.as_u64(), t.title, done)
                .map_err(|_| BuiltinError::RedirectWrite)?;
        }
    }
    Ok(())
}

fn run_todo_add(
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    rest: &[String],
    list: &mut TodoList<InMemoryStore>,
) -> Result<(), BuiltinError> {
    let title = rest.join(" ").trim().to_string();
    if title.is_empty() {
        writeln!(stderr, "todo add: title must be non-empty")
            .map_err(|_| BuiltinError::RedirectWrite)?;
        return Err(BuiltinError::TodoArgError);
    }
    let id = list.create(&title).map_err(|e| {
        let _ = writeln!(stderr, "todo add: {e}");
        BuiltinError::TodoArgError
    })?;
    save_todos(list).map_err(|_| BuiltinError::TodoSaveFailed)?;
    writeln!(stdout, "{}", id.as_u64()).map_err(|_| BuiltinError::RedirectWrite)?;
    Ok(())
}

fn run_todo_show(
    stdout: &mut dyn Write,
    rest: &[String],
    list: &TodoList<InMemoryStore>,
) -> Result<(), BuiltinError> {
    let id_str = rest.first().ok_or(BuiltinError::TodoArgError)?;
    let id_raw: u64 = id_str.parse().map_err(|_| BuiltinError::TodoArgError)?;
    let id = TodoId::from_raw(id_raw).ok_or(BuiltinError::TodoArgError)?;
    let t = list.get(id).ok_or(BuiltinError::TodoDataError)?;
    let done = if t.completed { " [done]" } else { "" };
    writeln!(stdout, "{}. {}{}", t.id.as_u64(), t.title, done)
        .map_err(|_| BuiltinError::RedirectWrite)?;
    if let Some(ref d) = t.description {
        writeln!(stdout, "  {d}").map_err(|_| BuiltinError::RedirectWrite)?;
    }
    if let Some(ref due) = t.due_date {
        writeln!(stdout, "  due: {due}").map_err(|_| BuiltinError::RedirectWrite)?;
    }
    Ok(())
}

fn run_todo_update(
    stderr: &mut dyn Write,
    rest: &[String],
    list: &mut TodoList<InMemoryStore>,
) -> Result<(), BuiltinError> {
    let id_str = rest.first().ok_or(BuiltinError::TodoArgError)?;
    let id_raw: u64 = id_str.parse().map_err(|_| BuiltinError::TodoArgError)?;
    let id = TodoId::from_raw(id_raw).ok_or(BuiltinError::TodoArgError)?;
    let title = rest
        .get(1..)
        .map(|a| a.join(" ").trim().to_string())
        .unwrap_or_default();
    if title.is_empty() {
        writeln!(stderr, "todo update: new title must be non-empty")
            .map_err(|_| BuiltinError::RedirectWrite)?;
        return Err(BuiltinError::TodoArgError);
    }
    list.update_title(id, &title).map_err(|e| {
        let _ = writeln!(stderr, "todo update: {e}");
        BuiltinError::TodoDataError
    })?;
    save_todos(list).map_err(|_| BuiltinError::TodoSaveFailed)?;
    Ok(())
}

fn run_todo_complete(
    stderr: &mut dyn Write,
    rest: &[String],
    list: &mut TodoList<InMemoryStore>,
) -> Result<(), BuiltinError> {
    let id_str = rest.first().ok_or(BuiltinError::TodoArgError)?;
    let id_raw: u64 = id_str.parse().map_err(|_| BuiltinError::TodoArgError)?;
    let id = TodoId::from_raw(id_raw).ok_or(BuiltinError::TodoArgError)?;
    list.complete(id, false).map_err(|e| {
        let _ = writeln!(stderr, "todo complete: {e}");
        BuiltinError::TodoDataError
    })?;
    save_todos(list).map_err(|_| BuiltinError::TodoSaveFailed)?;
    Ok(())
}

fn run_todo_delete(
    stderr: &mut dyn Write,
    rest: &[String],
    list: &mut TodoList<InMemoryStore>,
) -> Result<(), BuiltinError> {
    let id_str = rest.first().ok_or(BuiltinError::TodoArgError)?;
    let id_raw: u64 = id_str.parse().map_err(|_| BuiltinError::TodoArgError)?;
    let id = TodoId::from_raw(id_raw).ok_or(BuiltinError::TodoArgError)?;
    list.delete(id).map_err(|e| {
        let _ = writeln!(stderr, "todo delete: {e}");
        BuiltinError::TodoDataError
    })?;
    save_todos(list).map_err(|_| BuiltinError::TodoSaveFailed)?;
    Ok(())
}

fn run_todo_search(
    stdout: &mut dyn Write,
    rest: &[String],
    list: &TodoList<InMemoryStore>,
) -> Result<(), BuiltinError> {
    let keyword = rest.join(" ").trim().to_string();
    for t in list.search(&keyword) {
        let done = if t.completed { " [done]" } else { "" };
        writeln!(stdout, "{}. {}{}", t.id.as_u64(), t.title, done)
            .map_err(|_| BuiltinError::RedirectWrite)?;
    }
    Ok(())
}

fn run_todo_stats(
    stdout: &mut dyn Write,
    list: &TodoList<InMemoryStore>,
) -> Result<(), BuiltinError> {
    let (total, open, completed) = list.stats();
    writeln!(
        stdout,
        "open: {open}  completed: {completed}  total: {total}"
    )
    .map_err(|_| BuiltinError::RedirectWrite)?;
    Ok(())
}

pub(super) fn run_todo_cmd(
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    argv: &[String],
) -> Result<(), BuiltinError> {
    let sub = argv.get(1).map_or("list", String::as_str);
    let rest = argv.get(2..).unwrap_or(&[]);

    let todos = load_todos().map_err(|_| BuiltinError::TodoLoadFailed)?;
    let mut list = list_from_todos(todos);

    match sub {
        "list" => run_todo_list(stdout, &list, rest),
        "add" => run_todo_add(stdout, stderr, rest, &mut list),
        "show" => run_todo_show(stdout, rest, &list),
        "update" => run_todo_update(stderr, rest, &mut list),
        "complete" => run_todo_complete(stderr, rest, &mut list),
        "delete" => run_todo_delete(stderr, rest, &mut list),
        "search" => run_todo_search(stdout, rest, &list),
        "stats" => run_todo_stats(stdout, &list),
        _ => {
            writeln!(stderr, "todo: unknown subcommand '{sub}' (list, add, show, update, complete, delete, search, stats)")
                .map_err(|_| BuiltinError::RedirectWrite)?;
            Err(BuiltinError::TodoArgError)
        }
    }
}
