//! Todo subcommand dispatch and handlers.

use std::io::IsTerminal;
use std::path::Path;
use std::str::FromStr;

use xtask_todo_lib::{InMemoryStore, Priority, RepeatRule, TodoError, TodoId, TodoList, TodoPatch};

use super::args::{TodoArgs, TodoSub};
use super::error::{print_json_success, todo_to_json, TodoCliError};
use super::format::{format_duration, format_time_ago, print_todo_list_items};
use super::init_ai::run_init_ai;
use super::io::{load_todos, load_todos_from_path, save_todos, save_todos_to_path_with_format};

/// Build `TodoPatch` from add/update CLI optional fields (title set separately for update).
fn patch_from_add_args(
    description: Option<&str>,
    due_date: Option<&str>,
    priority: Option<&str>,
    tags: Option<&str>,
    repeat_rule: Option<&str>,
) -> Result<TodoPatch, TodoCliError> {
    let priority_parsed = priority
        .filter(|s| !s.is_empty())
        .and_then(|s| Priority::from_str(s).ok());
    let repeat_parsed = repeat_rule
        .filter(|s| !s.is_empty())
        .and_then(|s| RepeatRule::from_str(s).ok());
    let tags_vec = tags.map(|s| {
        s.split(',')
            .map(str::trim)
            .filter(|x| !x.is_empty())
            .map(String::from)
            .collect::<Vec<_>>()
    });
    if let Some(p) = priority {
        if !p.is_empty() && priority_parsed.is_none() {
            return Err(TodoCliError::Parameter(format!("invalid priority: {p}")));
        }
    }
    if let Some(r) = repeat_rule {
        if !r.is_empty() && repeat_parsed.is_none() {
            return Err(TodoCliError::Parameter(format!("invalid repeat_rule: {r}")));
        }
    }
    Ok(TodoPatch {
        title: None,
        description: description.map(String::from),
        due_date: due_date.map(String::from),
        priority: priority_parsed,
        tags: tags_vec,
        repeat_rule: repeat_parsed,
        repeat_until: None,
        repeat_count: None,
    })
}

/// Handle todo subcommand.
///
/// # Errors
/// Returns `TodoCliError` on I/O, invalid id, or todo operations (e.g. not found); use `exit_code()` for process exit.
#[allow(clippy::too_many_lines)]
pub fn cmd_todo(args: TodoArgs) -> Result<(), TodoCliError> {
    if let TodoSub::InitAi(a) = &args.sub {
        run_init_ai(a.for_tool.as_deref(), a.output.as_deref().map(Path::new))?;
        if args.json {
            print_json_success(&serde_json::json!({ "generated": true }));
        } else {
            println!("Generated init-ai skill file.");
        }
        return Ok(());
    }

    let todos = load_todos().map_err(TodoCliError::General)?;
    let store = InMemoryStore::from_todos(todos);
    let mut list = TodoList::with_store(store);
    let json = args.json;
    let dry_run = args.dry_run;

    match args.sub {
        TodoSub::Add(a) => {
            let has_opts = a.description.is_some()
                || a.due_date.is_some()
                || a.priority.is_some()
                || a.tags.is_some()
                || a.repeat_rule.is_some();
            if dry_run {
                if json {
                    print_json_success(&serde_json::json!({
                        "would_add": true,
                        "title": a.title.trim(),
                        "description": a.description,
                        "due_date": a.due_date,
                        "priority": a.priority,
                        "tags": a.tags,
                        "repeat_rule": a.repeat_rule
                    }));
                } else {
                    println!("Would add: {} (dry-run)", a.title.trim());
                }
            } else {
                let id = list.create(&a.title).map_err(|e| match e {
                    TodoError::InvalidInput => {
                        TodoCliError::Parameter("invalid input: title must be non-empty".into())
                    }
                    TodoError::NotFound(id) => TodoCliError::Data(format!("todo not found: {id}")),
                })?;
                if has_opts {
                    let patch = patch_from_add_args(
                        a.description.as_deref(),
                        a.due_date.as_deref(),
                        a.priority.as_deref(),
                        a.tags.as_deref(),
                        a.repeat_rule.as_deref(),
                    )?;
                    list.update(id, patch)
                        .map_err(|e| TodoCliError::Data(e.to_string()))?;
                }
                save_todos(&list).map_err(TodoCliError::General)?;
                if json {
                    print_json_success(
                        &serde_json::json!({ "id": id.as_u64(), "title": a.title.trim() }),
                    );
                } else {
                    println!("Added [{}] {}", id, a.title.trim());
                }
            }
        }
        TodoSub::List(_) => {
            let items = list.list();
            if json {
                let data: Vec<serde_json::Value> = items.iter().map(todo_to_json).collect();
                print_json_success(&serde_json::json!({ "items": data }));
            } else {
                let use_color = std::io::stdout().is_terminal();
                print_todo_list_items(&items, use_color);
            }
        }
        TodoSub::Show(a) => {
            let id = TodoId::from_raw(a.id)
                .ok_or_else(|| TodoCliError::Parameter("invalid id 0".into()))?;
            match list.get(id) {
                Some(t) => {
                    if json {
                        print_json_success(&todo_to_json(&t));
                    } else {
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
                }
                None => return Err(TodoCliError::Data(format!("todo not found: {id}"))),
            }
        }
        TodoSub::Update(a) => {
            let id = TodoId::from_raw(a.id)
                .ok_or_else(|| TodoCliError::Parameter("invalid id 0".into()))?;
            if dry_run {
                if json {
                    print_json_success(&serde_json::json!({
                        "would_update": true,
                        "id": id.as_u64(),
                        "title": a.title.trim(),
                        "description": a.description,
                        "due_date": a.due_date,
                        "priority": a.priority,
                        "tags": a.tags,
                        "repeat_rule": a.repeat_rule
                    }));
                } else {
                    println!("Would update [{}] {} (dry-run)", id, a.title.trim());
                }
            } else {
                let mut patch = patch_from_add_args(
                    a.description.as_deref(),
                    a.due_date.as_deref(),
                    a.priority.as_deref(),
                    a.tags.as_deref(),
                    a.repeat_rule.as_deref(),
                )?;
                patch.title = Some(a.title.trim().to_string());
                list.update(id, patch)
                    .map_err(|e| TodoCliError::Data(e.to_string()))?;
                save_todos(&list).map_err(TodoCliError::General)?;
                if json {
                    print_json_success(
                        &serde_json::json!({ "id": id.as_u64(), "title": a.title.trim() }),
                    );
                } else {
                    println!("Updated [{}] {}", id, a.title.trim());
                }
            }
        }
        TodoSub::Complete(a) => {
            let id = TodoId::from_raw(a.id)
                .ok_or_else(|| TodoCliError::Parameter("invalid id 0".into()))?;
            if dry_run {
                if list.get(id).is_none() {
                    return Err(TodoCliError::Data(format!("todo not found: {id}")));
                }
                if json {
                    print_json_success(&serde_json::json!({
                        "would_complete": true,
                        "id": id.as_u64(),
                        "no_next": a.no_next
                    }));
                } else {
                    println!("Would complete [{id}] (dry-run)");
                }
            } else {
                list.complete(id, a.no_next)
                    .map_err(|e| TodoCliError::Data(e.to_string()))?;
                save_todos(&list).map_err(TodoCliError::General)?;
                if json {
                    print_json_success(
                        &serde_json::json!({ "id": id.as_u64(), "completed": true }),
                    );
                } else {
                    println!("Completed [{id}]");
                }
            }
        }
        TodoSub::Delete(a) => {
            let id = TodoId::from_raw(a.id)
                .ok_or_else(|| TodoCliError::Parameter("invalid id 0".into()))?;
            if dry_run {
                if list.get(id).is_none() {
                    return Err(TodoCliError::Data(format!("todo not found: {id}")));
                }
                if json {
                    print_json_success(&serde_json::json!({
                        "would_delete": true,
                        "id": id.as_u64()
                    }));
                } else {
                    println!("Would delete [{id}] (dry-run)");
                }
            } else {
                list.delete(id)
                    .map_err(|e| TodoCliError::Data(e.to_string()))?;
                save_todos(&list).map_err(TodoCliError::General)?;
                if json {
                    print_json_success(&serde_json::json!({ "id": id.as_u64(), "deleted": true }));
                } else {
                    println!("Deleted [{id}]");
                }
            }
        }
        TodoSub::Search(a) => {
            let items = list.search(&a.keyword);
            if json {
                let data: Vec<serde_json::Value> = items.iter().map(todo_to_json).collect();
                print_json_success(&serde_json::json!({ "items": data }));
            } else {
                let use_color = std::io::stdout().is_terminal();
                print_todo_list_items(&items, use_color);
            }
        }
        TodoSub::Stats(_) => {
            let (total, incomplete, complete) = list.stats();
            if json {
                print_json_success(&serde_json::json!({
                    "total": total,
                    "incomplete": incomplete,
                    "complete": complete
                }));
            } else {
                println!("Total: {total}  Incomplete: {incomplete}  Complete: {complete}");
            }
        }
        TodoSub::Export(a) => {
            let format = a
                .format
                .as_deref()
                .or_else(|| {
                    a.file.extension().and_then(|e| e.to_str()).map(|s| {
                        if s.eq_ignore_ascii_case("csv") {
                            "csv"
                        } else {
                            "json"
                        }
                    })
                })
                .unwrap_or("json");
            save_todos_to_path_with_format(&list, &a.file, format)
                .map_err(TodoCliError::General)?;
            if json {
                print_json_success(&serde_json::json!({
                    "exported": list.list().len(),
                    "file": a.file.display().to_string()
                }));
            } else {
                println!(
                    "Exported {} tasks to {}",
                    list.list().len(),
                    a.file.display()
                );
            }
        }
        TodoSub::Import(a) => {
            let imported = load_todos_from_path(&a.file).map_err(TodoCliError::General)?;
            if a.replace {
                let store = InMemoryStore::from_todos(imported.clone());
                let new_list = TodoList::with_store(store);
                if !dry_run {
                    save_todos(&new_list).map_err(TodoCliError::General)?;
                }
                if json {
                    print_json_success(&serde_json::json!({
                        "replaced": true,
                        "count": new_list.list().len(),
                        "file": a.file.display().to_string()
                    }));
                } else {
                    println!(
                        "Replaced with {} tasks from {}",
                        new_list.list().len(),
                        a.file.display()
                    );
                }
            } else {
                for t in &imported {
                    list.add_todo(t);
                }
                if !dry_run {
                    save_todos(&list).map_err(TodoCliError::General)?;
                }
                if json {
                    print_json_success(&serde_json::json!({
                        "merged": true,
                        "count": imported.len(),
                        "file": a.file.display().to_string()
                    }));
                } else {
                    println!("Merged {} tasks from {}", imported.len(), a.file.display());
                }
            }
        }
        TodoSub::InitAi(_) => {}
    }
    Ok(())
}
