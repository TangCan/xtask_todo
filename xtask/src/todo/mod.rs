//! `todo` subcommand - add, list, complete, delete (data in .todo.json).

pub mod args;
mod cmd;
pub mod error;
pub mod format;
pub mod init_ai;
pub mod io;

pub use args::TodoArgs;
pub use cmd::cmd_todo;

/// Prints JSON error to stdout (for `--json` when `cmd_todo` fails). Used by the xtask runner.
pub fn print_json_error(code: i32, message: &str) {
    error::print_json_error(code, message);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_json_error_wrapper_prints_valid_json() {
        print_json_error(2, "title must be non-empty");
    }
}
