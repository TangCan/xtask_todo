//! Todo subcommand dispatch: `cmd_todo` and per-subcommand handling.

use std::fmt::Write;
use std::io::IsTerminal;
use std::path::Path;

use xtask_todo_lib::{InMemoryStore, TodoError, TodoId, TodoList};

use super::super::args::{
    TodoAddArgs, TodoArgs, TodoCompleteArgs, TodoDeleteArgs, TodoExportArgs, TodoImportArgs,
    TodoListArgs, TodoSearchArgs, TodoShowArgs, TodoSub, TodoUpdateArgs,
};
use super::super::error::{print_json_success, todo_list_json_payload, todo_to_json, TodoCliError};
use super::super::format::{format_duration, format_time_ago, print_todo_list_items};
use super::super::init_ai::run_init_ai;
use super::super::io::{
    load_todos, load_todos_for_import, save_todos, save_todos_to_path_with_format,
};
use super::parse::{list_options_from_args, patch_from_add_args};

/// Handle todo subcommand.
///
/// # Errors
/// Returns `TodoCliError` on I/O, invalid id, or todo operations (e.g. not found); use `exit_code()` for process exit.
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
        TodoSub::Add(a) => handle_add(&mut list, &a, json, dry_run),
        TodoSub::List(a) => handle_list(&list, &a, json),
        TodoSub::Show(a) => handle_show(&list, &a, json),
        TodoSub::Update(a) => handle_update(&mut list, &a, json, dry_run),
        TodoSub::Complete(a) => handle_complete(&mut list, &a, json, dry_run),
        TodoSub::Delete(a) => handle_delete(&mut list, &a, json, dry_run),
        TodoSub::Search(a) => {
            handle_search(&list, &a, json);
            Ok(())
        }
        TodoSub::Stats(_) => {
            handle_stats(&list, json);
            Ok(())
        }
        TodoSub::Export(a) => handle_export(&list, &a, json),
        TodoSub::Import(a) => handle_import(&mut list, &a, json, dry_run),
        TodoSub::InitAi(_) => Ok(()),
    }
}

fn handle_add(
    list: &mut TodoList<InMemoryStore>,
    a: &TodoAddArgs,
    json: bool,
    dry_run: bool,
) -> Result<(), TodoCliError> {
    let has_opts = a.description.is_some()
        || a.due_date.is_some()
        || a.priority.is_some()
        || a.tags.is_some()
        || a.repeat_rule.is_some()
        || a.repeat_until.is_some()
        || a.repeat_count.is_some();
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
        return Ok(());
    }
    // Validate optional fields before `create` so invalid flags never allocate a new id (TC-T1-3 / FR1).
    let patch_opt = if has_opts {
        Some(patch_from_add_args(
            a.description.as_deref(),
            a.due_date.as_deref(),
            a.priority.as_deref(),
            a.tags.as_deref(),
            a.repeat_rule.as_deref(),
            a.repeat_until.as_deref(),
            a.repeat_count.as_deref(),
        )?)
    } else {
        None
    };
    let id = list.create(&a.title).map_err(|e| match e {
        TodoError::InvalidInput => {
            TodoCliError::Parameter("invalid input: title must be non-empty".into())
        }
        TodoError::NotFound(id) => TodoCliError::Data(format!("todo not found: {id}")),
    })?;
    if let Some(patch) = patch_opt {
        list.update(id, patch)
            .map_err(|e| TodoCliError::Data(e.to_string()))?;
    }
    save_todos(list).map_err(TodoCliError::General)?;
    if json {
        print_json_success(&serde_json::json!({ "id": id.as_u64(), "title": a.title.trim() }));
    } else {
        println!("Added [{}] {}", id, a.title.trim());
    }
    Ok(())
}

fn handle_list(
    list: &TodoList<InMemoryStore>,
    a: &TodoListArgs,
    json: bool,
) -> Result<(), TodoCliError> {
    let options = list_options_from_args(
        a.status.as_deref(),
        a.priority.as_deref(),
        a.tags.as_deref(),
        a.due_before.as_deref(),
        a.due_after.as_deref(),
        a.sort.as_deref(),
    )?;
    let items = list.list_with_options(&options);
    if json {
        print_json_success(&todo_list_json_payload(&items));
    } else {
        let use_color = std::io::stdout().is_terminal();
        print_todo_list_items(&items, use_color);
    }
    Ok(())
}

fn handle_show(
    list: &TodoList<InMemoryStore>,
    a: &TodoShowArgs,
    json: bool,
) -> Result<(), TodoCliError> {
    let id =
        TodoId::from_raw(a.id).ok_or_else(|| TodoCliError::Parameter("invalid id 0".into()))?;
    let t = list
        .get(id)
        .ok_or_else(|| TodoCliError::Data(format!("todo not found: {id}")))?;
    if json {
        print_json_success(&todo_to_json(&t));
        return Ok(());
    }
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
    if let Some(ref d) = t.description {
        println!("    描述: {d}");
    }
    if let Some(ref d) = t.due_date {
        println!("    截止: {d}");
    }
    if let Some(p) = t.priority {
        println!("    优先级: {p}");
    }
    if !t.tags.is_empty() {
        println!("    标签: {}", t.tags.join(", "));
    }
    if let Some(ref r) = t.repeat_rule {
        let mut repeat_line = format!("    重复: {r}");
        if let Some(ref u) = t.repeat_until {
            let _ = write!(repeat_line, "  截止 {u}");
        }
        if let Some(c) = t.repeat_count {
            let _ = write!(repeat_line, "  剩余 {c} 次");
        }
        println!("{repeat_line}");
    }
    Ok(())
}

fn handle_update(
    list: &mut TodoList<InMemoryStore>,
    a: &TodoUpdateArgs,
    json: bool,
    dry_run: bool,
) -> Result<(), TodoCliError> {
    let id =
        TodoId::from_raw(a.id).ok_or_else(|| TodoCliError::Parameter("invalid id 0".into()))?;
    if dry_run {
        if list.get(id).is_none() {
            return Err(TodoCliError::Data(format!("todo not found: {id}")));
        }
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
        return Ok(());
    }
    let mut patch = patch_from_add_args(
        a.description.as_deref(),
        a.due_date.as_deref(),
        a.priority.as_deref(),
        a.tags.as_deref(),
        a.repeat_rule.as_deref(),
        a.repeat_until.as_deref(),
        a.repeat_count.as_deref(),
    )?;
    patch.title = Some(a.title.trim().to_string());
    patch.repeat_rule_clear = a.clear_repeat_rule;
    list.update(id, patch)
        .map_err(|e| TodoCliError::Data(e.to_string()))?;
    save_todos(list).map_err(TodoCliError::General)?;
    if json {
        print_json_success(&serde_json::json!({ "id": id.as_u64(), "title": a.title.trim() }));
    } else {
        println!("Updated [{}] {}", id, a.title.trim());
    }
    Ok(())
}

fn handle_complete(
    list: &mut TodoList<InMemoryStore>,
    a: &TodoCompleteArgs,
    json: bool,
    dry_run: bool,
) -> Result<(), TodoCliError> {
    let id =
        TodoId::from_raw(a.id).ok_or_else(|| TodoCliError::Parameter("invalid id 0".into()))?;
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
        return Ok(());
    }
    list.complete(id, a.no_next)
        .map_err(|e| TodoCliError::Data(e.to_string()))?;
    save_todos(list).map_err(TodoCliError::General)?;
    if json {
        print_json_success(&serde_json::json!({ "id": id.as_u64(), "completed": true }));
    } else {
        println!("Completed [{id}]");
    }
    Ok(())
}

fn handle_delete(
    list: &mut TodoList<InMemoryStore>,
    a: &TodoDeleteArgs,
    json: bool,
    dry_run: bool,
) -> Result<(), TodoCliError> {
    let id =
        TodoId::from_raw(a.id).ok_or_else(|| TodoCliError::Parameter("invalid id 0".into()))?;
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
        return Ok(());
    }
    list.delete(id)
        .map_err(|e| TodoCliError::Data(e.to_string()))?;
    save_todos(list).map_err(TodoCliError::General)?;
    if json {
        print_json_success(&serde_json::json!({ "id": id.as_u64(), "deleted": true }));
    } else {
        println!("Deleted [{id}]");
    }
    Ok(())
}

fn handle_search(list: &TodoList<InMemoryStore>, a: &TodoSearchArgs, json: bool) {
    let items = list.search(&a.keyword);
    if json {
        print_json_success(&todo_list_json_payload(&items));
    } else {
        let use_color = std::io::stdout().is_terminal();
        print_todo_list_items(&items, use_color);
    }
}

fn handle_stats(list: &TodoList<InMemoryStore>, json: bool) {
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

fn handle_export(
    list: &TodoList<InMemoryStore>,
    a: &TodoExportArgs,
    json: bool,
) -> Result<(), TodoCliError> {
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
    save_todos_to_path_with_format(list, &a.file, format).map_err(TodoCliError::General)?;
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
    Ok(())
}

fn handle_import(
    list: &mut TodoList<InMemoryStore>,
    a: &TodoImportArgs,
    json: bool,
    dry_run: bool,
) -> Result<(), TodoCliError> {
    let imported = load_todos_for_import(&a.file).map_err(TodoCliError::General)?;
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
            save_todos(list).map_err(TodoCliError::General)?;
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
    Ok(())
}
