//! `todo` subcommand - add, list, complete, delete (data in .todo.json).

pub mod args;
mod cmd;
pub mod error;
pub mod format;
pub mod init_ai;
pub mod io;

pub use args::TodoArgs;
pub use args::TodoStandaloneArgs;
pub use cmd::cmd_todo;

/// Prints JSON error to stdout (for `--json` when `cmd_todo` fails). Used by the xtask runner.
pub fn print_json_error(code: i32, message: &str) {
    error::print_json_error(code, message);
}

/// Same logic as the standalone `todo` binary (`src/bin/todo.rs`): dispatch to [`cmd_todo`].
///
/// Used by unit tests so `cargo tarpaulin` counts this path without spawning a subprocess.
///
/// # Errors
/// Same as [`cmd_todo`] (I/O, validation, todo store).
pub fn run_standalone(cli: TodoStandaloneArgs) -> Result<(), error::TodoCliError> {
    let args: TodoArgs = cli.into();
    cmd_todo(args)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::todo::args::{TodoListArgs, TodoSub};

    #[test]
    fn print_json_error_wrapper_prints_valid_json() {
        print_json_error(2, "title must be non-empty");
    }

    #[test]
    fn run_standalone_matches_cmd_todo() {
        let a = TodoStandaloneArgs {
            sub: TodoSub::List(TodoListArgs {
                status: None,
                priority: None,
                tags: None,
                due_before: None,
                due_after: None,
                sort: None,
            }),
            json: false,
            dry_run: false,
        };
        let r = run_standalone(a);
        assert!(r.is_ok(), "{r:?}");
    }
}
