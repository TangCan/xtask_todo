//! REPL: read-eval-print loop with `parse_line` and `execute_pipeline`; handles exit/quit.
//!
//! TTY: rustyline Editor with path completion; non-TTY: `read_line` loop.
//! On exit (exit/quit or EOF), VFS is auto-saved to `bin_path`.

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
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let pipeline = match parser::parse_line(line) {
            Ok(p) => p,
            Err(e) => {
                let _ = writeln!(stderr, "parse error: {e}");
                continue;
            }
        };
        let first_argv0 = pipeline
            .commands
            .first()
            .and_then(|c| c.argv.first())
            .map(String::as_str);
        if first_argv0 == Some("exit") || first_argv0 == Some("quit") {
            break;
        }
        let mut vfs_ref = vfs.borrow_mut();
        let mut ctx = ExecContext {
            vfs: &mut vfs_ref,
            stdin,
            stdout,
            stderr,
        };
        match execute_pipeline(&mut ctx, &pipeline) {
            Ok(RunResult::Exit) => break,
            Ok(RunResult::Continue) => {}
            Err(e) => {
                let _ = writeln!(stderr, "error: {e:?}");
            }
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
        let line_trimmed = line.trim();
        if line_trimmed.is_empty() {
            continue;
        }
        let pipeline = match parser::parse_line(line_trimmed) {
            Ok(p) => p,
            Err(e) => {
                let _ = writeln!(stderr, "parse error: {e}");
                continue;
            }
        };
        let first_argv0 = pipeline
            .commands
            .first()
            .and_then(|c| c.argv.first())
            .map(String::as_str);
        if first_argv0 == Some("exit") || first_argv0 == Some("quit") {
            break;
        }
        let mut vfs_ref = vfs.borrow_mut();
        let mut ctx = ExecContext {
            vfs: &mut vfs_ref,
            stdin,
            stdout,
            stderr,
        };
        match execute_pipeline(&mut ctx, &pipeline) {
            Ok(RunResult::Exit) => break,
            Ok(RunResult::Continue) => {}
            Err(e) => {
                let _ = writeln!(stderr, "error: {e:?}");
            }
        }
    }
    save_on_exit(vfs, bin_path, stderr);
    Ok(())
}
