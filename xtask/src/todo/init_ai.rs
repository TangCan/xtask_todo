//! Generate init-ai skill files for AI tools.

use std::path::{Path, PathBuf};

use crate::todo::error::TodoCliError;

/// Generate init-ai skill files under output dir.
///
/// # Errors
/// Returns error on I/O.
pub fn run_init_ai(_for_tool: Option<&str>, output: Option<&Path>) -> Result<(), TodoCliError> {
    let dir = output.map_or_else(
        || {
            std::env::current_dir().map_or_else(
                |_| PathBuf::from(".cursor/commands"),
                |cwd| cwd.join(".cursor").join("commands"),
            )
        },
        PathBuf::from,
    );
    std::fs::create_dir_all(&dir).map_err(|e| TodoCliError::General(Box::new(e)))?;
    let path = dir.join("todo.md");
    let content = r#"# Todo list commands

Run via: `cargo xtask todo <subcommand> [options]`

## Subcommands

- `add <title> [--description …] [--due-date …] [--priority …] [--tags …] [--repeat-rule …] [--repeat-until …] [--repeat-count …]` — add a task (optional fields)
- `list [--status completed|incomplete] [--priority …] [--tags …] [--due-before …] [--due-after …] [--sort created-at|due-date|priority|title]` — list tasks (optional filter and sort)
- `show <id>` — show one task (human-readable: description, due, priority, tags, repeat; use --json for full)
- `update <id> <title> [--description …] [--due-date …] [--priority …] [--tags …] [--repeat-rule …] [--repeat-until …] [--repeat-count …] [--clear-repeat-rule]` — update task (--clear-repeat-rule clears repeat)
- `complete <id> [--no-next]` — mark done (optional: do not create next for recurring)
- `delete <id>` — remove task
- `search <keyword>` — search in title/description/tags
- `stats` — total, incomplete, complete counts
- `export <file> [--format json|csv]` — export to file (format by extension or --format)
- `import <file> [--replace]` — import from file (merge or replace; .json or .csv)
- `init-ai` — generate this skill file

## JSON output

Use `todo --json <subcommand>` for machine-readable output:

- Success: `{"status":"success","data":...}` (for `list` / `search`, empty results include `"empty": true` and `"message": "No tasks."` alongside `"items": []`)
- Error: `{"status":"error","error":{"code":1|2|3,"message":"..."}}`

Exit codes: 0 success, 1 general, 2 parameter error, 3 data error (e.g. id not found).

## Dry run

Use `--dry-run` with add/update/complete/delete to print the intended operation without writing `.todo.json`.
"#;
    std::fs::write(&path, content).map_err(|e| TodoCliError::General(Box::new(e)))?;
    Ok(())
}
