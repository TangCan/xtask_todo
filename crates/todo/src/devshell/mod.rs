//! Devshell REPL and VFS: same logic as the `cargo-devshell` binary, exposed so tests can cover it.
#![allow(
    dead_code,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::too_long_first_doc_paragraph,
    clippy::too_many_lines,
    clippy::result_unit_err,
    clippy::cast_possible_truncation,
    clippy::branches_sharing_code,
    clippy::needless_pass_by_value,
    clippy::match_wildcard_for_single_variants,
    clippy::map_identity
)]

pub mod command;
pub mod completion;
pub mod parser;
pub mod serialization;
pub mod todo_io;
pub mod vfs;

mod repl;

use std::cell::RefCell;
use std::io::{self, BufReader, IsTerminal};
use std::path::Path;
use std::rc::Rc;

use vfs::Vfs;

/// Run the devshell using process args and standard I/O (for the binary).
///
/// # Errors
/// Returns an error if usage is wrong or I/O fails critically.
pub fn run_main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let is_tty = io::stdin().is_terminal();
    let mut stdin = BufReader::new(io::stdin());
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();
    run_main_from_args(&args, is_tty, &mut stdin, &mut stdout, &mut stderr)
}

/// Same as `run_main` but takes args, `is_tty`, and streams (for tests and callers that supply I/O).
///
/// # Errors
/// Returns an error if usage is wrong or I/O fails critically.
pub fn run_main_from_args<R, W1, W2>(
    args: &[String],
    is_tty: bool,
    stdin: &mut R,
    stdout: &mut W1,
    stderr: &mut W2,
) -> Result<(), Box<dyn std::error::Error>>
where
    R: std::io::BufRead + std::io::Read,
    W1: std::io::Write,
    W2: std::io::Write,
{
    let path = match args {
        [] | [_] => Path::new(".dev_shell.bin"),
        [_, path] => Path::new(path),
        _ => {
            writeln!(stderr, "usage: dev_shell [path]")?;
            return Err(Box::new(std::io::Error::other("usage")));
        }
    };
    let vfs = match serialization::load_from_file(path) {
        Ok(v) => v,
        Err(e) => {
            if e.kind() == io::ErrorKind::NotFound {
                if args.len() > 1 {
                    let _ = writeln!(stderr, "File not found, starting with empty VFS");
                }
            } else {
                let _ = writeln!(stderr, "Failed to load {}: {}", path.display(), e);
            }
            Vfs::new()
        }
    };
    let vfs = Rc::new(RefCell::new(vfs));
    repl::run(&vfs, is_tty, path, stdin, stdout, stderr).map_err(|()| {
        Box::new(std::io::Error::other("repl error")) as Box<dyn std::error::Error>
    })?;
    Ok(())
}

/// Run the devshell with given args and streams (for tests).
pub fn run_with<R, W1, W2>(
    args: &[String],
    stdin: &mut R,
    stdout: &mut W1,
    stderr: &mut W2,
) -> Result<(), ()>
where
    R: std::io::BufRead + std::io::Read,
    W1: std::io::Write,
    W2: std::io::Write,
{
    let path = match args {
        [] | [_] => Path::new(".dev_shell.bin"),
        [_, path] => Path::new(path),
        _ => {
            let _ = writeln!(stderr, "usage: dev_shell [path]");
            return Err(());
        }
    };
    let vfs = serialization::load_from_file(path).unwrap_or_default();
    let vfs = Rc::new(RefCell::new(vfs));
    repl::run(&vfs, false, path, stdin, stdout, stderr)
}

#[cfg(test)]
mod tests;
