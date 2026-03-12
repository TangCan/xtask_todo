//! `todo` subcommand - add, list, complete, delete (data in .todo.json).

mod args;
mod cmd;
mod error;
mod format;
mod init_ai;
mod io;

#[allow(unused_imports)]
pub use args::{
    todo_args, TodoAddArgs, TodoArgs, TodoCompleteArgs, TodoDeleteArgs, TodoExportArgs,
    TodoImportArgs, TodoInitAiArgs, TodoListArgs, TodoSearchArgs, TodoShowArgs, TodoStatsArgs,
    TodoSub, TodoUpdateArgs,
};
pub use cmd::cmd_todo;
#[allow(unused_imports)]
pub use error::{TodoCliError, EXIT_DATA, EXIT_GENERAL, EXIT_PARAMETER};
#[allow(unused_imports)]
pub use format::{
    format_duration, format_time_ago, is_old_open, print_todo_list_items, AGE_THRESHOLD_DAYS,
};
#[allow(unused_imports)]
pub use init_ai::run_init_ai;
#[allow(unused_imports)]
pub use io::{
    load_todos, load_todos_from_path, save_todos, save_todos_to_path, todo_file, TodoDto,
};
