//! Todo subcommand dispatch and handlers.

use std::io::IsTerminal;
use std::path::Path;

use xtask_todo_lib::{InMemoryStore, TodoError, TodoId, TodoList};

use super::args::{TodoArgs, TodoSub};
use super::error::{print_json_success, todo_to_json, TodoCliError};
use super::format::{format_duration, format_time_ago, print_todo_list_items};
use super::init_ai::run_init_ai;
use super::io::{load_todos, load_todos_from_path, save_todos, save_todos_to_path};

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
            let id = list.create(&a.title).map_err(|e| match e {
                TodoError::InvalidInput => {
                    TodoCliError::Parameter("invalid input: title must be non-empty".into())
                }
                TodoError::NotFound(id) => TodoCliError::Data(format!("todo not found: {id}")),
            })?;
            if !dry_run {
                save_todos(&list).map_err(TodoCliError::General)?;
            }
            if json {
                print_json_success(
                    &serde_json::json!({ "id": id.as_u64(), "title": a.title.trim() }),
                );
            } else {
                println!("Added [{}] {}", id, a.title.trim());
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
            list.update_title(id, &a.title)
                .map_err(|e| TodoCliError::Data(e.to_string()))?;
            if !dry_run {
                save_todos(&list).map_err(TodoCliError::General)?;
            }
            if json {
                print_json_success(
                    &serde_json::json!({ "id": id.as_u64(), "title": a.title.trim() }),
                );
            } else {
                println!("Updated [{}] {}", id, a.title.trim());
            }
        }
        TodoSub::Complete(a) => {
            let id = TodoId::from_raw(a.id)
                .ok_or_else(|| TodoCliError::Parameter("invalid id 0".into()))?;
            list.complete(id, a.no_next)
                .map_err(|e| TodoCliError::Data(e.to_string()))?;
            if !dry_run {
                save_todos(&list).map_err(TodoCliError::General)?;
            }
            if json {
                print_json_success(&serde_json::json!({ "id": id.as_u64(), "completed": true }));
            } else {
                println!("Completed [{id}]");
            }
        }
        TodoSub::Delete(a) => {
            let id = TodoId::from_raw(a.id)
                .ok_or_else(|| TodoCliError::Parameter("invalid id 0".into()))?;
            list.delete(id)
                .map_err(|e| TodoCliError::Data(e.to_string()))?;
            if !dry_run {
                save_todos(&list).map_err(TodoCliError::General)?;
            }
            if json {
                print_json_success(&serde_json::json!({ "id": id.as_u64(), "deleted": true }));
            } else {
                println!("Deleted [{id}]");
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
            save_todos_to_path(&list, &a.file).map_err(TodoCliError::General)?;
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
