//! `todo` subcommand - add, list, complete, delete (data in .todo.json).

mod args;
mod cmd;
mod error;
mod format;
mod init_ai;
mod io;

#[cfg(test)]
pub use args::todo_args;
#[allow(unused_imports)]
pub use args::{
    TodoAddArgs, TodoArgs, TodoCompleteArgs, TodoDeleteArgs, TodoExportArgs, TodoImportArgs,
    TodoInitAiArgs, TodoListArgs, TodoSearchArgs, TodoShowArgs, TodoStatsArgs, TodoSub,
    TodoUpdateArgs,
};
pub use cmd::cmd_todo;
#[allow(unused_imports)]
pub use error::{TodoCliError, EXIT_DATA, EXIT_GENERAL, EXIT_PARAMETER};

/// Prints JSON error to stdout (for `--json` when `cmd_todo` fails). Used by the xtask runner.
pub fn print_json_error(code: i32, message: &str) {
    error::print_json_error(code, message);
}
#[allow(unused_imports)]
pub use format::{
    format_duration, format_time_ago, is_old_open, print_todo_list_items, AGE_THRESHOLD_DAYS,
};
#[allow(unused_imports)]
pub use init_ai::run_init_ai;
#[allow(unused_imports)]
pub use io::{
    load_todos, load_todos_from_path, save_todos, save_todos_to_path,
    save_todos_to_path_with_format, todo_file, TodoDto,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_json_error_wrapper_prints_valid_json() {
        print_json_error(2, "title must be non-empty");
    }
}
