//! Script interpreter: run AST (assign, command, if/for/while, source).

use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{BufRead, Read, Write};
use std::path::Path;
use std::rc::Rc;

use super::ast::ScriptStmt;
use super::parse::parse_script;
use crate::devshell::command::{execute_pipeline, ExecContext, RunResult};
use crate::devshell::parser;
use crate::devshell::vfs::Vfs;

const MAX_SOURCE_DEPTH: u32 = 64;

/// Execute a single `source` statement: read file, parse, run. Returns Ok(false) if exit requested.
#[allow(clippy::too_many_arguments)]
fn exec_source<R, W1, W2>(
    vfs: &Rc<RefCell<Vfs>>,
    path: &str,
    vars: &mut HashMap<String, String>,
    set_e: &mut bool,
    source_depth: u32,
    stdin: &mut R,
    stdout: &mut W1,
    stderr: &mut W2,
) -> Result<bool, ()>
where
    R: BufRead + Read,
    W1: Write,
    W2: Write,
{
    if source_depth >= MAX_SOURCE_DEPTH {
        let _ = writeln!(stderr, "source: max depth {MAX_SOURCE_DEPTH} exceeded");
        return Err(());
    }
    let content = vfs
        .borrow()
        .read_file(path)
        .ok()
        .and_then(|b| String::from_utf8(b).ok())
        .or_else(|| std::fs::read_to_string(path).ok());
    let Some(content) = content else {
        let _ = writeln!(stderr, "source: cannot read {path}");
        return Err(());
    };
    let lines = logical_lines(&content);
    let sub = match parse_script(&lines) {
        Ok(s) => s,
        Err(e) => {
            let _ = writeln!(stderr, "source {path}: {e}");
            return Err(());
        }
    };
    exec_stmts(
        vfs,
        &sub,
        vars,
        set_e,
        source_depth + 1,
        stdin,
        stdout,
        stderr,
    )
}

/// Result of running one command line: success (exit 0), failed (non-zero), or exit requested.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmdOutcome {
    Success,
    Failed,
    Exit,
}

/// Run one expanded command line; returns Success / Failed / Exit.
fn run_command_line<R, W1, W2>(
    vfs: &Rc<RefCell<Vfs>>,
    line: &str,
    vars: &HashMap<String, String>,
    stdin: &mut R,
    stdout: &mut W1,
    stderr: &mut W2,
) -> CmdOutcome
where
    R: BufRead + Read,
    W1: Write,
    W2: Write,
{
    let line = expand_vars(line, vars);
    let line = line.trim();
    if line.is_empty() {
        return CmdOutcome::Success;
    }
    let pipeline = match parser::parse_line(line) {
        Ok(p) => p,
        Err(e) => {
            let _ = writeln!(stderr, "parse error: {e}");
            return CmdOutcome::Failed;
        }
    };
    let first_argv0 = pipeline
        .commands
        .first()
        .and_then(|c| c.argv.first())
        .map(String::as_str);
    if first_argv0 == Some("exit") || first_argv0 == Some("quit") {
        return CmdOutcome::Exit;
    }
    let mut vfs_ref = vfs.borrow_mut();
    let mut ctx = ExecContext {
        vfs: &mut vfs_ref,
        stdin,
        stdout,
        stderr,
    };
    match execute_pipeline(&mut ctx, &pipeline) {
        Ok(RunResult::Continue) => CmdOutcome::Success,
        Ok(RunResult::Exit) => CmdOutcome::Exit,
        Err(e) => {
            let _ = writeln!(stderr, "error: {e:?}");
            CmdOutcome::Failed
        }
    }
}

/// Execute a list of statements; returns Ok(false) if exit was requested, Ok(true) if done, Err(()) on `set_e` failure or source error.
#[allow(clippy::too_many_arguments)]
fn exec_stmts<R, W1, W2>(
    vfs: &Rc<RefCell<Vfs>>,
    stmts: &[ScriptStmt],
    vars: &mut HashMap<String, String>,
    set_e: &mut bool,
    source_depth: u32,
    stdin: &mut R,
    stdout: &mut W1,
    stderr: &mut W2,
) -> Result<bool, ()>
where
    R: BufRead + Read,
    W1: Write,
    W2: Write,
{
    for stmt in stmts {
        match stmt {
            ScriptStmt::Assign(n, v) => {
                vars.insert(n.clone(), v.clone());
            }
            ScriptStmt::SetE => *set_e = true,
            ScriptStmt::Command(line) => {
                let out = run_command_line(vfs, line, vars, stdin, stdout, stderr);
                match out {
                    CmdOutcome::Exit => return Ok(false),
                    CmdOutcome::Failed if *set_e => return Err(()),
                    _ => {}
                }
            }
            ScriptStmt::If {
                cond,
                then_body,
                else_body,
            } => {
                let out = run_command_line(vfs, cond, vars, stdin, stdout, stderr);
                let run_body = if out == CmdOutcome::Success {
                    then_body
                } else {
                    else_body.as_deref().unwrap_or(&[])
                };
                if !run_body.is_empty() {
                    let cont = exec_stmts(
                        vfs,
                        run_body,
                        vars,
                        set_e,
                        source_depth,
                        stdin,
                        stdout,
                        stderr,
                    )?;
                    if !cont {
                        return Ok(false);
                    }
                }
            }
            ScriptStmt::For { var, words, body } => {
                for w in words {
                    let w_expanded = expand_vars(w, vars);
                    vars.insert(var.clone(), w_expanded);
                    let cont =
                        exec_stmts(vfs, body, vars, set_e, source_depth, stdin, stdout, stderr)?;
                    if !cont {
                        return Ok(false);
                    }
                }
            }
            ScriptStmt::While { cond, body } => loop {
                let out = run_command_line(vfs, cond, vars, stdin, stdout, stderr);
                if out != CmdOutcome::Success {
                    break;
                }
                let cont = exec_stmts(vfs, body, vars, set_e, source_depth, stdin, stdout, stderr)?;
                if !cont {
                    return Ok(false);
                }
            },
            ScriptStmt::Source(path) => {
                let cont =
                    exec_source(vfs, path, vars, set_e, source_depth, stdin, stdout, stderr)?;
                if !cont {
                    return Ok(false);
                }
            }
        }
    }
    Ok(true)
}

/// Expand `$VAR` and `${VAR}` in `s` using `vars`; undefined names expand to empty string.
#[must_use]
pub fn expand_vars<S: std::hash::BuildHasher>(
    s: &str,
    vars: &HashMap<String, String, S>,
) -> String {
    let mut out = String::new();
    let mut i = 0;
    let bytes = s.as_bytes();
    while i < bytes.len() {
        if bytes[i] == b'$' && i + 1 < bytes.len() {
            if bytes[i + 1] == b'{' {
                let start = i + 2;
                let mut end = start;
                while end < bytes.len()
                    && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_')
                {
                    end += 1;
                }
                if end < bytes.len() && bytes[end] == b'}' {
                    let name = std::str::from_utf8(&bytes[start..end]).unwrap_or("");
                    out.push_str(vars.get(name).map_or("", String::as_str));
                    i = end + 1;
                    continue;
                }
            } else if bytes[i + 1] == b'_' || bytes[i + 1].is_ascii_alphabetic() {
                let start = i + 1;
                let mut end = start;
                while end < bytes.len()
                    && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_')
                {
                    end += 1;
                }
                let name = std::str::from_utf8(&bytes[start..end]).unwrap_or("");
                out.push_str(vars.get(name).map_or("", String::as_str));
                i = end;
                continue;
            }
        }
        out.push(char::from(bytes[i]));
        i += 1;
    }
    out
}

/// Turn script source into logical lines: join lines ending with `\`, strip `#` comments, skip blank.
#[must_use]
pub fn logical_lines(source: &str) -> Vec<String> {
    let raw_lines: Vec<&str> = source.lines().collect();
    let mut merged: Vec<String> = Vec::new();
    let mut current = String::new();

    for line in raw_lines {
        let line = line.trim_end();
        if current.ends_with('\\') {
            current.pop();
            current.push_str(line.trim_start());
        } else {
            if !current.is_empty() {
                merged.push(std::mem::take(&mut current));
            }
            current = line.to_string();
        }
    }
    if !current.is_empty() {
        merged.push(current);
    }

    let mut out: Vec<String> = Vec::new();
    for line in merged {
        let comment_start = line.find('#').unwrap_or(line.len());
        let line = line[..comment_start].trim();
        if !line.is_empty() {
            out.push(line.to_string());
        }
    }
    out
}

/// Run script source: logical lines → parse to AST → interpret.
///
/// # Errors
/// Returns `Err(())` on parse error (message to stderr) or when `set_e` is true and a command fails.
#[allow(clippy::result_unit_err)]
pub fn run_script<R, W1, W2>(
    vfs: &Rc<RefCell<Vfs>>,
    script_src: &str,
    _bin_path: &Path,
    set_e: bool,
    stdin: &mut R,
    stdout: &mut W1,
    stderr: &mut W2,
) -> Result<(), ()>
where
    R: BufRead + Read,
    W1: Write,
    W2: Write,
{
    let lines = logical_lines(script_src);
    let stmts = match parse_script(&lines) {
        Ok(s) => s,
        Err(e) => {
            let _ = writeln!(stderr, "script parse error: {e}");
            return Err(());
        }
    };
    let mut vars = HashMap::new();
    let mut set_e_flag = set_e;
    let _ = exec_stmts(
        vfs,
        &stmts,
        &mut vars,
        &mut set_e_flag,
        0,
        stdin,
        stdout,
        stderr,
    )?;
    Ok(())
}
