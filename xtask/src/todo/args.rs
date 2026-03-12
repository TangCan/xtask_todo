//! Argh CLI types for the todo subcommand.

use argh::FromArgs;
use std::path::PathBuf;

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
#[must_use]
#[allow(dead_code, clippy::missing_const_for_fn)]
pub fn todo_args(sub: TodoSub) -> TodoArgs {
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
    /// optional repeat rule: daily, weekly, monthly, yearly, weekdays, 2d, 3w, custom:N; empty to clear
    pub repeat_rule: Option<String>,
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
}

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "list")]
/// List all tasks
pub struct TodoListArgs {}

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
