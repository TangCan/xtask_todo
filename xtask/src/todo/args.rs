//! Argh CLI types for the todo subcommand.

use argh::FromArgs;
use std::path::PathBuf;

/// Top-level CLI for the standalone **`todo`** binary (`cargo build -p xtask --bin todo`).
///
/// Same flags and subcommands as `cargo xtask todo` (see [`TodoArgs`]).
#[derive(FromArgs, Clone)]
#[argh(name = "todo")]
/// Todo list: add, list, complete, delete (data in .todo.json)
pub struct TodoStandaloneArgs {
    #[argh(subcommand)]
    pub sub: TodoSub,
    #[argh(switch)]
    /// output JSON only (unified structure)
    pub json: bool,
    #[argh(switch)]
    /// do not persist; show intended operation only
    pub dry_run: bool,
}

impl From<TodoStandaloneArgs> for TodoArgs {
    fn from(s: TodoStandaloneArgs) -> Self {
        Self {
            sub: s.sub,
            json: s.json,
            dry_run: s.dry_run,
        }
    }
}

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "todo")]
/// Todo list: add, list, complete, delete (data in .todo.json)
pub struct TodoArgs {
    #[argh(subcommand)]
    pub sub: TodoSub,
    #[argh(switch)]
    /// output JSON only (unified structure)
    pub json: bool,
    #[argh(switch)]
    /// do not persist; show intended operation only
    pub dry_run: bool,
}

/// Builds `TodoArgs` with `json: false`, `dry_run: false` (for tests).
#[cfg(test)]
#[must_use]
pub const fn todo_args(sub: TodoSub) -> TodoArgs {
    TodoArgs {
        sub,
        json: false,
        dry_run: false,
    }
}

#[derive(FromArgs, Clone)]
#[argh(subcommand)]
pub enum TodoSub {
    Add(TodoAddArgs),
    List(TodoListArgs),
    Show(TodoShowArgs),
    Update(TodoUpdateArgs),
    Complete(TodoCompleteArgs),
    Delete(TodoDeleteArgs),
    Search(TodoSearchArgs),
    Stats(TodoStatsArgs),
    Export(TodoExportArgs),
    Import(TodoImportArgs),
    InitAi(TodoInitAiArgs),
}

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "init-ai")]
/// generate skill/command files for AI tools (e.g. Cursor)
pub struct TodoInitAiArgs {
    #[argh(option)]
    /// target tool: cursor (default), or other
    pub for_tool: Option<String>,
    #[argh(option)]
    /// output directory (default: .cursor/commands)
    pub output: Option<PathBuf>,
}

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "search")]
/// Search tasks by keyword (title, description, tags)
pub struct TodoSearchArgs {
    #[argh(positional)]
    pub keyword: String,
}

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "stats")]
/// Show task counts (total, incomplete, complete)
pub struct TodoStatsArgs {}

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "export")]
/// Export tasks to a file (format by extension or --format: json, csv)
pub struct TodoExportArgs {
    #[argh(positional)]
    pub file: PathBuf,
    #[argh(option)]
    /// output format: json (default), csv
    pub format: Option<String>,
}

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "import")]
/// Import tasks from a JSON file (merge or replace)
pub struct TodoImportArgs {
    #[argh(positional)]
    pub file: PathBuf,
    #[argh(switch)]
    /// replace current list instead of merging
    pub replace: bool,
}

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "show")]
/// Show a single task by id
pub struct TodoShowArgs {
    #[argh(positional)]
    pub id: u64,
}

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "update")]
/// Update a task by id (title and optional fields)
pub struct TodoUpdateArgs {
    #[argh(positional)]
    pub id: u64,
    #[argh(positional)]
    pub title: String,
    #[argh(option)]
    /// optional description
    pub description: Option<String>,
    #[argh(option)]
    /// optional due date (YYYY-MM-DD)
    pub due_date: Option<String>,
    #[argh(option)]
    /// optional priority: low, medium, high
    pub priority: Option<String>,
    #[argh(option)]
    /// optional tags, comma-separated
    pub tags: Option<String>,
    #[argh(option)]
    /// optional repeat rule: daily, weekly, monthly, yearly, weekdays, 2d, 3w, custom:N
    pub repeat_rule: Option<String>,
    #[argh(option)]
    /// optional repeat end date (YYYY-MM-DD); no next instance if next due > this
    pub repeat_until: Option<String>,
    #[argh(option)]
    /// optional remaining repeat count (1 = last occurrence)
    pub repeat_count: Option<String>,
    #[argh(switch)]
    /// clear repeat rule (set to none)
    pub clear_repeat_rule: bool,
}

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "add")]
/// Add a task
pub struct TodoAddArgs {
    #[argh(positional)]
    pub title: String,
    #[argh(option)]
    /// optional description
    pub description: Option<String>,
    #[argh(option)]
    /// optional due date (YYYY-MM-DD)
    pub due_date: Option<String>,
    #[argh(option)]
    /// optional priority: low, medium, high
    pub priority: Option<String>,
    #[argh(option)]
    /// optional tags, comma-separated
    pub tags: Option<String>,
    #[argh(option)]
    /// optional repeat rule: daily, weekly, monthly, yearly, weekdays, 2d, 3w, custom:N
    pub repeat_rule: Option<String>,
    #[argh(option)]
    /// optional repeat end date (YYYY-MM-DD)
    pub repeat_until: Option<String>,
    #[argh(option)]
    /// optional remaining repeat count (1 = last)
    pub repeat_count: Option<String>,
}

#[derive(FromArgs, Clone, Default)]
#[argh(subcommand, name = "list")]
/// List tasks (optional filter and sort)
pub struct TodoListArgs {
    #[argh(option)]
    /// filter by status: completed, incomplete
    pub status: Option<String>,
    #[argh(option)]
    /// filter by priority: low, medium, high
    pub priority: Option<String>,
    #[argh(option)]
    /// filter by tags (comma-separated; match any)
    pub tags: Option<String>,
    #[argh(option)]
    /// filter due date on or before (YYYY-MM-DD)
    pub due_before: Option<String>,
    #[argh(option)]
    /// filter due date on or after (YYYY-MM-DD)
    pub due_after: Option<String>,
    #[argh(option)]
    /// sort by: created-at, due-date, priority, title
    pub sort: Option<String>,
}

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "complete")]
/// Mark a task as completed by id
pub struct TodoCompleteArgs {
    #[argh(positional)]
    pub id: u64,
    #[argh(switch)]
    /// do not create next instance for recurring tasks
    pub no_next: bool,
}

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "delete")]
/// Delete a task by id
pub struct TodoDeleteArgs {
    #[argh(positional)]
    pub id: u64,
}
