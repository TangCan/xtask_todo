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

- `add <title>` — add a task
- `list` — list all tasks
- `show <id>` — show one task
- `update <id> <title>` — update task title
- `complete <id> [--no-next]` — mark done (optional: do not create next for recurring)
- `delete <id>` — remove task
- `search <keyword>` — search in title/description/tags
- `stats` — total, incomplete, complete counts
- `export <file>` — export JSON to file
- `import <file> [--replace]` — import from file (merge or replace)
- `init-ai` — generate this skill file

## JSON output

Append `--json` to any subcommand for machine-readable output:

- Success: `{"status":"success","data":...}`
- Error: `{"status":"error","error":{"code":1|2|3,"message":"..."}}`

Exit codes: 0 success, 1 general, 2 parameter error, 3 data error (e.g. id not found).

## Dry run

Use `--dry-run` with add/update/complete/delete to print the intended operation without writing `.todo.json`.
"#;
    std::fs::write(&path, content).map_err(|e| TodoCliError::General(Box::new(e)))?;
    Ok(())
}
