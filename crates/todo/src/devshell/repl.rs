//! REPL: read-eval-print loop with `parse_line` and `execute_pipeline`; handles exit/quit.
//!
//! TTY: rustyline Editor with path completion; non-TTY: `read_line` loop.
//! On exit (exit/quit or EOF), VFS is auto-saved to `bin_path`.
//! Shared loop body is in `process_line` so it can be unit-tested.

use std::cell::RefCell;
use std::io::{BufRead, Read, Write};
use std::path::Path;
use std::rc::Rc;

use rustyline::Editor;

use super::command::{execute_pipeline, ExecContext, RunResult};
use super::completion::DevShellHelper;
use super::parser;
use super::serialization;
use super::vfs::Vfs;

/// Result of processing one REPL line: continue the loop or exit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepResult {
    Continue,
    Exit,
}

/// Process one input line: parse, optionally run pipeline, return whether to exit.
/// Used by both TTY and non-TTY loops; exposed for tests.
pub fn process_line<R, W1, W2>(
    vfs: &Rc<RefCell<Vfs>>,
    line: &str,
    stdin: &mut R,
    stdout: &mut W1,
    stderr: &mut W2,
) -> StepResult
where
    R: BufRead + Read,
    W1: Write,
    W2: Write,
{
    let line_trimmed = line.trim();
    if line_trimmed.is_empty() {
        return StepResult::Continue;
    }
    let pipeline = match parser::parse_line(line_trimmed) {
        Ok(p) => p,
        Err(e) => {
            let _ = writeln!(stderr, "parse error: {e}");
            return StepResult::Continue;
        }
    };
    let first_argv0 = pipeline
        .commands
        .first()
        .and_then(|c| c.argv.first())
        .map(String::as_str);
    if first_argv0 == Some("exit") || first_argv0 == Some("quit") {
        return StepResult::Exit;
    }
    let mut vfs_ref = vfs.borrow_mut();
    let mut ctx = ExecContext {
        vfs: &mut vfs_ref,
        stdin,
        stdout,
        stderr,
    };
    match execute_pipeline(&mut ctx, &pipeline) {
        Ok(RunResult::Exit) => StepResult::Exit,
        Ok(RunResult::Continue) => StepResult::Continue,
        Err(e) => {
            let _ = writeln!(stderr, "error: {e:?}");
            StepResult::Continue
        }
    }
}

/// Run the REPL until exit/quit or EOF.
///
/// When `is_tty`: uses rustyline Editor with tab completion (command + path).
/// When not TTY: uses `stdin.read_line` (pipe/script compatible).
/// On exit, the VFS is automatically saved to `bin_path`.
pub fn run<R, W1, W2>(
    vfs: &Rc<RefCell<Vfs>>,
    is_tty: bool,
    bin_path: &Path,
    stdin: &mut R,
    stdout: &mut W1,
    stderr: &mut W2,
) -> Result<(), ()>
where
    R: BufRead + Read,
    W1: Write,
    W2: Write,
{
    if is_tty {
        run_tty(vfs, bin_path, stdin, stdout, stderr)
    } else {
        run_readline(vfs, bin_path, stdin, stdout, stderr)
    }
}

fn save_on_exit<W2: Write>(vfs: &Rc<RefCell<Vfs>>, bin_path: &Path, stderr: &mut W2) {
    if let Err(e) = serialization::save_to_file(&vfs.borrow(), bin_path) {
        let _ = writeln!(stderr, "save on exit failed: {e}");
    }
}

/// TTY branch: rustyline Editor with `DevShellHelper` (path completion via vfs).
fn run_tty<R, W1, W2>(
    vfs: &Rc<RefCell<Vfs>>,
    bin_path: &Path,
    stdin: &mut R,
    stdout: &mut W1,
    stderr: &mut W2,
) -> Result<(), ()>
where
    R: BufRead + Read,
    W1: Write,
    W2: Write,
{
    let mut editor = Editor::new().map_err(|_| ())?;
    editor.set_helper(Some(DevShellHelper::new(vfs.clone())));

    loop {
        let prompt = format!("{} $ ", vfs.borrow().cwd());
        let line = match editor.readline(&prompt) {
            Ok(line) => line,
            Err(rustyline::error::ReadlineError::Eof) => {
                save_on_exit(vfs, bin_path, stderr);
                return Ok(());
            }
            Err(rustyline::error::ReadlineError::Interrupted) => continue,
            Err(e) => {
                let _ = writeln!(stderr, "readline error: {e}");
                continue;
            }
        };
        if process_line(vfs, &line, stdin, stdout, stderr) == StepResult::Exit {
            break;
        }
    }
    save_on_exit(vfs, bin_path, stderr);
    Ok(())
}

/// Non-TTY branch: `read_line` loop; `borrow_mut` once per iteration for prompt and `ExecContext`.
fn run_readline<R, W1, W2>(
    vfs: &Rc<RefCell<Vfs>>,
    bin_path: &Path,
    stdin: &mut R,
    stdout: &mut W1,
    stderr: &mut W2,
) -> Result<(), ()>
where
    R: BufRead + Read,
    W1: Write,
    W2: Write,
{
    let mut line = String::new();
    loop {
        line.clear();
        let cwd = vfs.borrow().cwd().to_string();
        let _ = write!(stdout, "{cwd} $ ");
        let _ = stdout.flush();
        let n = stdin.read_line(&mut line).map_err(|_| ())?;
        if n == 0 {
            save_on_exit(vfs, bin_path, stderr);
            return Ok(());
        }
        if process_line(vfs, &line, stdin, stdout, stderr) == StepResult::Exit {
            break;
        }
    }
    save_on_exit(vfs, bin_path, stderr);
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::io::Cursor;
    use std::rc::Rc;

    use super::super::vfs::Vfs;
    use super::{process_line, StepResult};

    #[test]
    fn process_line_empty_returns_continue() {
        let vfs = Rc::new(RefCell::new(Vfs::new()));
        let mut stdin = Cursor::new(b"");
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let r = process_line(&vfs, "  \n", &mut stdin, &mut stdout, &mut stderr);
        assert_eq!(r, StepResult::Continue);
    }

    #[test]
    fn process_line_exit_returns_exit() {
        let vfs = Rc::new(RefCell::new(Vfs::new()));
        let mut stdin = Cursor::new(b"");
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let r = process_line(&vfs, "exit", &mut stdin, &mut stdout, &mut stderr);
        assert_eq!(r, StepResult::Exit);
    }

    #[test]
    fn process_line_quit_returns_exit() {
        let vfs = Rc::new(RefCell::new(Vfs::new()));
        let mut stdin = Cursor::new(b"");
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let r = process_line(&vfs, "quit", &mut stdin, &mut stdout, &mut stderr);
        assert_eq!(r, StepResult::Exit);
    }

    #[test]
    fn process_line_parse_error_returns_continue() {
        let vfs = Rc::new(RefCell::new(Vfs::new()));
        let mut stdin = Cursor::new(b"");
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let r = process_line(&vfs, "echo >", &mut stdin, &mut stdout, &mut stderr);
        assert_eq!(r, StepResult::Continue);
        let err = String::from_utf8(stderr).unwrap();
        assert!(err.contains("parse error"));
    }

    #[test]
    fn process_line_pwd_continues_and_writes() {
        let vfs = Rc::new(RefCell::new(Vfs::new()));
        let mut stdin = Cursor::new(b"");
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let r = process_line(&vfs, "pwd", &mut stdin, &mut stdout, &mut stderr);
        assert_eq!(r, StepResult::Continue);
        let out = String::from_utf8(stdout).unwrap();
        assert!(out.contains('/'));
    }

    #[test]
    fn process_line_unknown_command_continues_and_stderr() {
        let vfs = Rc::new(RefCell::new(Vfs::new()));
        let mut stdin = Cursor::new(b"");
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let r = process_line(&vfs, "unknowncmd", &mut stdin, &mut stdout, &mut stderr);
        assert_eq!(r, StepResult::Continue);
        let err = String::from_utf8(stderr).unwrap();
        assert!(err.contains("unknown command"));
    }
}
