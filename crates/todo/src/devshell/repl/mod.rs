//! REPL: read-eval-print loop with `parse_line` and `execute_pipeline`; handles exit/quit.
//!
//! TTY: rustyline Editor with path completion (`CompletionType::List`, bash-like);
//! non-TTY: `read_line` loop.
//! On exit (exit/quit or EOF), VFS is auto-saved to `bin_path`.
//! Shared loop body is in `process_line` so it can be unit-tested.

use std::cell::RefCell;
use std::io::{BufRead, Read, Write};
use std::path::Path;
use std::rc::Rc;

use rustyline::config::Configurer;
use rustyline::{CompletionType, Editor};

use super::command::{execute_pipeline, ExecContext, RunResult};
use super::completion::DevShellHelper;
use super::parser;
use super::script;
use super::serialization;
use super::session_store;
use super::vfs::Vfs;
use super::vm::SessionHolder;

/// Result of processing one REPL line: continue the loop or exit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepResult {
    Continue,
    Exit,
}

/// Process one input line: parse, optionally run pipeline, return whether to exit.
/// Handles `source path` and `. path` by loading script text via [`crate::devshell::script::read_script_source_text`]
/// (workspace guest / VFS, then host) and running it.
/// Used by both TTY and non-TTY loops; exposed for tests.
pub fn process_line<R, W1, W2>(
    vfs: &Rc<RefCell<Vfs>>,
    vm_session: &Rc<RefCell<SessionHolder>>,
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

    // REPL source: "source path" or ". path" — run file as script (VFS or host)
    if let Some(path) = line_trimmed.strip_prefix("source ") {
        let path = path.trim();
        if path.is_empty() {
            let _ = writeln!(stderr, "source: missing path");
            return StepResult::Continue;
        }
        let content = script::read_script_source_text(vfs, vm_session, path);
        match content {
            Some(c) => {
                let _ = script::run_script(
                    vfs,
                    vm_session,
                    &c,
                    Path::new(""),
                    false,
                    stdin,
                    stdout,
                    stderr,
                );
            }
            None => {
                let _ = writeln!(stderr, "source: cannot read {path}");
            }
        }
        return StepResult::Continue;
    }
    if let Some(path) = line_trimmed.strip_prefix(". ") {
        let path = path.trim();
        if path.is_empty() {
            let _ = writeln!(stderr, ".: missing path");
            return StepResult::Continue;
        }
        let content = script::read_script_source_text(vfs, vm_session, path);
        match content {
            Some(c) => {
                let _ = script::run_script(
                    vfs,
                    vm_session,
                    &c,
                    Path::new(""),
                    false,
                    stdin,
                    stdout,
                    stderr,
                );
            }
            None => {
                let _ = writeln!(stderr, ".: cannot read {path}");
            }
        }
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
    let mut sess_ref = vm_session.borrow_mut();
    let mut ctx = ExecContext {
        vfs: &mut vfs_ref,
        stdin,
        stdout,
        stderr,
        vm_session: &mut sess_ref,
    };
    match execute_pipeline(&mut ctx, &pipeline) {
        Ok(RunResult::Exit) => StepResult::Exit,
        Ok(RunResult::Continue) => StepResult::Continue,
        Err(e) => {
            let _ = writeln!(stderr, "error: {e}");
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
    vm_session: &Rc<RefCell<SessionHolder>>,
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
        run_tty(vfs, vm_session, bin_path, stdin, stdout, stderr)
    } else {
        run_readline(vfs, vm_session, bin_path, stdin, stdout, stderr)
    }
}

fn save_on_exit<W2: Write>(
    vfs: &Rc<RefCell<Vfs>>,
    vm_session: &Rc<RefCell<SessionHolder>>,
    bin_path: &Path,
    stderr: &mut W2,
) {
    let cwd = vfs.borrow().cwd().to_string();
    {
        let mut vfs_mut = vfs.borrow_mut();
        if let Err(e) = vm_session.borrow_mut().shutdown(&mut vfs_mut, &cwd) {
            let _ = writeln!(stderr, "dev_shell: session shutdown: {e}");
        }
    }
    if vfs.borrow().is_host_backed() {
        let _ = writeln!(
            stderr,
            "dev_shell: host workspace: skipping .dev_shell.bin (tree is on disk under DEVSHELL_WORKSPACE_ROOT)"
        );
        return;
    }
    if vm_session.borrow().is_guest_primary() {
        let _ = writeln!(
            stderr,
            "dev_shell: guest-primary mode: skipping legacy .dev_shell.bin save (design §10; guest workspace is authoritative)"
        );
        if let Err(e) = session_store::save_guest_primary(bin_path, vfs.borrow().cwd()) {
            let _ = writeln!(
                stderr,
                "dev_shell: failed to write guest-primary session {}: {e}",
                session_store::session_metadata_path(bin_path).display()
            );
        }
    } else if let Err(e) = serialization::save_to_file(&vfs.borrow(), bin_path) {
        let _ = writeln!(stderr, "save on exit failed: {e}");
    }
}

/// TTY branch: rustyline Editor with `DevShellHelper` (path completion via vfs).
fn run_tty<R, W1, W2>(
    vfs: &Rc<RefCell<Vfs>>,
    vm_session: &Rc<RefCell<SessionHolder>>,
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
    // Bash/readline-style: extend to longest common prefix; second Tab lists options.
    // Default rustyline `Circular` cycles candidates and then restores the pre-Tab line,
    // so e.g. `cat s` → Tab → `cat src` → Tab → `cat s` again (surprising vs shells).
    editor.set_completion_type(CompletionType::List);
    editor.set_helper(Some(DevShellHelper::new(vfs.clone(), vm_session.clone())));

    loop {
        let prompt = format!("{} $ ", vfs.borrow().cwd());
        let line = match editor.readline(&prompt) {
            Ok(line) => line,
            Err(rustyline::error::ReadlineError::Eof) => {
                save_on_exit(vfs, vm_session, bin_path, stderr);
                return Ok(());
            }
            Err(rustyline::error::ReadlineError::Interrupted) => continue,
            Err(e) => {
                let _ = writeln!(stderr, "readline error: {e}");
                continue;
            }
        };
        if process_line(vfs, vm_session, &line, stdin, stdout, stderr) == StepResult::Exit {
            break;
        }
    }
    save_on_exit(vfs, vm_session, bin_path, stderr);
    Ok(())
}

/// Non-TTY branch: `read_line` loop; `borrow_mut` once per iteration for prompt and `ExecContext`.
fn run_readline<R, W1, W2>(
    vfs: &Rc<RefCell<Vfs>>,
    vm_session: &Rc<RefCell<SessionHolder>>,
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
            save_on_exit(vfs, vm_session, bin_path, stderr);
            return Ok(());
        }
        if process_line(vfs, vm_session, &line, stdin, stdout, stderr) == StepResult::Exit {
            break;
        }
    }
    save_on_exit(vfs, vm_session, bin_path, stderr);
    Ok(())
}

#[cfg(test)]
mod tests;
