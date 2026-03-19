//! Devshell REPL and VFS: same logic as the `cargo-devshell` binary, exposed so tests can cover it.

pub mod command;
pub mod completion;
pub mod parser;
pub mod script;
pub mod serialization;
pub mod todo_io;
pub mod vfs;

mod repl;

use std::cell::RefCell;
use std::io::{self, BufReader, IsTerminal};
use std::path::Path;
use std::rc::Rc;

use vfs::Vfs;

/// Error from `run_with` (usage or REPL failure).
#[derive(Debug)]
pub enum RunWithError {
    Usage,
    ReplFailed,
}

impl std::fmt::Display for RunWithError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Usage => f.write_str("usage error"),
            Self::ReplFailed => f.write_str("repl failed"),
        }
    }
}

impl std::error::Error for RunWithError {}

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
    let positionals: Vec<&str> = args
        .iter()
        .skip(1)
        .filter(|a| *a != "-e" && *a != "-f")
        .map(String::as_str)
        .collect();
    let set_e = args.iter().skip(1).any(|a| a == "-e");
    let run_script = args.iter().skip(1).any(|a| a == "-f");

    if run_script {
        if positionals.len() != 1 {
            writeln!(stderr, "usage: dev_shell [-e] -f script.dsh")?;
            return Err(Box::new(std::io::Error::other("usage")));
        }
        let script_path = positionals[0];
        let script_src = match std::fs::read_to_string(script_path) {
            Ok(s) => s,
            Err(e) => {
                writeln!(stderr, "dev_shell: {script_path}: {e}")?;
                return Err(e.into());
            }
        };
        let bin_path = Path::new(".dev_shell.bin");
        let vfs = match serialization::load_from_file(bin_path) {
            Ok(v) => v,
            Err(e) => {
                if e.kind() != io::ErrorKind::NotFound {
                    let _ = writeln!(stderr, "Failed to load {}: {}", bin_path.display(), e);
                }
                Vfs::new()
            }
        };
        let vfs = Rc::new(RefCell::new(vfs));
        script::run_script(&vfs, &script_src, bin_path, set_e, stdin, stdout, stderr).map_err(
            |()| Box::new(std::io::Error::other("script error")) as Box<dyn std::error::Error>,
        )
    } else {
        let path = match positionals.as_slice() {
            [] => Path::new(".dev_shell.bin"),
            [p] => Path::new(p),
            _ => {
                writeln!(stderr, "usage: dev_shell [options] [path]")?;
                return Err(Box::new(std::io::Error::other("usage")));
            }
        };
        let vfs = match serialization::load_from_file(path) {
            Ok(v) => v,
            Err(e) => {
                if e.kind() == io::ErrorKind::NotFound {
                    if positionals.len() > 1 {
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
}

/// Run the devshell with given args and streams (for tests).
///
/// # Errors
/// Returns `RunWithError::Usage` on invalid args; `RunWithError::ReplFailed` if the REPL exits with error.
pub fn run_with<R, W1, W2>(
    args: &[String],
    stdin: &mut R,
    stdout: &mut W1,
    stderr: &mut W2,
) -> Result<(), RunWithError>
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
            return Err(RunWithError::Usage);
        }
    };
    let vfs = serialization::load_from_file(path).unwrap_or_default();
    let vfs = Rc::new(RefCell::new(vfs));
    repl::run(&vfs, false, path, stdin, stdout, stderr).map_err(|()| RunWithError::ReplFailed)
}

#[cfg(test)]
mod tests;
