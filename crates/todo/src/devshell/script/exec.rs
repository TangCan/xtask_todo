//! Script interpreter: run AST (assign, command, if/for/while, source).

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::io::{BufRead, Read, Write};
use std::path::Path;
use std::rc::Rc;

use super::ast::ScriptStmt;
use super::parse::parse_script;
use crate::devshell::command::{execute_pipeline, ExecContext, RunResult};
use crate::devshell::parser;
use crate::devshell::vfs::Vfs;

const MAX_SOURCE_DEPTH: u32 = 64;

/// Error from script execution (parse, command failure with `set_e`, or source failure).
#[derive(Debug)]
pub enum RunScriptError {
    Parse,
    CommandFailed,
    Source,
}

impl fmt::Display for RunScriptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse => f.write_str("script parse error"),
            Self::CommandFailed => f.write_str("script command failed"),
            Self::Source => f.write_str("script source error"),
        }
    }
}

impl std::error::Error for RunScriptError {}

/// Execution context for script interpretation: VFS, variables, streams, and source depth.
struct ExecScriptContext<'a, R, W1, W2> {
    vfs: &'a Rc<RefCell<Vfs>>,
    vars: &'a mut HashMap<String, String>,
    set_e: &'a mut bool,
    source_depth: u32,
    stdin: &'a mut R,
    stdout: &'a mut W1,
    stderr: &'a mut W2,
}

/// Execute a single `source` statement: read file, parse, run. Returns Ok(false) if exit requested.
fn exec_source<R, W1, W2>(
    ctx: &mut ExecScriptContext<'_, R, W1, W2>,
    path: &str,
) -> Result<bool, RunScriptError>
where
    R: BufRead + Read,
    W1: Write,
    W2: Write,
{
    if ctx.source_depth >= MAX_SOURCE_DEPTH {
        let _ = writeln!(ctx.stderr, "source: max depth {MAX_SOURCE_DEPTH} exceeded");
        return Err(RunScriptError::Source);
    }
    let content = ctx
        .vfs
        .borrow()
        .read_file(path)
        .ok()
        .and_then(|b| String::from_utf8(b).ok())
        .or_else(|| std::fs::read_to_string(path).ok());
    let Some(content) = content else {
        let _ = writeln!(ctx.stderr, "source: cannot read {path}");
        return Err(RunScriptError::Source);
    };
    let lines = logical_lines(&content);
    let sub = match parse_script(&lines) {
        Ok(s) => s,
        Err(e) => {
            let _ = writeln!(ctx.stderr, "source {path}: {e}");
            return Err(RunScriptError::Source);
        }
    };
    ctx.source_depth += 1;
    let result = exec_stmts(ctx, &sub);
    ctx.source_depth -= 1;
    result
}

/// Result of running one command line: success (exit 0), failed (non-zero), or exit requested.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmdOutcome {
    Success,
    Failed,
    Exit,
}

/// Run one expanded command line; returns Success / Failed / Exit.
fn run_command_line<R, W1, W2>(ctx: &mut ExecScriptContext<'_, R, W1, W2>, line: &str) -> CmdOutcome
where
    R: BufRead + Read,
    W1: Write,
    W2: Write,
{
    let line = expand_vars(line, ctx.vars);
    let line = line.trim();
    if line.is_empty() {
        return CmdOutcome::Success;
    }
    let pipeline = match parser::parse_line(line) {
        Ok(p) => p,
        Err(e) => {
            let _ = writeln!(ctx.stderr, "parse error: {e}");
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
    let mut vfs_ref = ctx.vfs.borrow_mut();
    let mut exec_ctx = ExecContext {
        vfs: &mut vfs_ref,
        stdin: ctx.stdin,
        stdout: ctx.stdout,
        stderr: ctx.stderr,
    };
    match execute_pipeline(&mut exec_ctx, &pipeline) {
        Ok(RunResult::Continue) => CmdOutcome::Success,
        Ok(RunResult::Exit) => CmdOutcome::Exit,
        Err(e) => {
            let _ = writeln!(ctx.stderr, "error: {e:?}");
            CmdOutcome::Failed
        }
    }
}

/// Execute a list of statements; returns Ok(false) if exit was requested, Ok(true) if done, Err on `set_e` failure or source error.
fn exec_stmts<R, W1, W2>(
    ctx: &mut ExecScriptContext<'_, R, W1, W2>,
    stmts: &[ScriptStmt],
) -> Result<bool, RunScriptError>
where
    R: BufRead + Read,
    W1: Write,
    W2: Write,
{
    for stmt in stmts {
        match stmt {
            ScriptStmt::Assign(n, v) => {
                ctx.vars.insert(n.clone(), v.clone());
            }
            ScriptStmt::SetE => *ctx.set_e = true,
            ScriptStmt::Command(line) => {
                let out = run_command_line(ctx, line);
                match out {
                    CmdOutcome::Exit => return Ok(false),
                    CmdOutcome::Failed if *ctx.set_e => return Err(RunScriptError::CommandFailed),
                    _ => {}
                }
            }
            ScriptStmt::If {
                cond,
                then_body,
                else_body,
            } => {
                let out = run_command_line(ctx, cond);
                let run_body = if out == CmdOutcome::Success {
                    then_body
                } else {
                    else_body.as_deref().unwrap_or(&[])
                };
                if !run_body.is_empty() {
                    let cont = exec_stmts(ctx, run_body)?;
                    if !cont {
                        return Ok(false);
                    }
                }
            }
            ScriptStmt::For { var, words, body } => {
                for w in words {
                    let w_expanded = expand_vars(w, ctx.vars);
                    ctx.vars.insert(var.clone(), w_expanded);
                    let cont = exec_stmts(ctx, body)?;
                    if !cont {
                        return Ok(false);
                    }
                }
            }
            ScriptStmt::While { cond, body } => loop {
                let out = run_command_line(ctx, cond);
                if out != CmdOutcome::Success {
                    break;
                }
                let cont = exec_stmts(ctx, body)?;
                if !cont {
                    return Ok(false);
                }
            },
            ScriptStmt::Source(path) => {
                let cont = exec_source(ctx, path)?;
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
/// Returns `Err(RunScriptError)` on parse error (message to stderr), when `set_e` is true and a command fails, or on source failure.
pub fn run_script<R, W1, W2>(
    vfs: &Rc<RefCell<Vfs>>,
    script_src: &str,
    _bin_path: &Path,
    set_e: bool,
    stdin: &mut R,
    stdout: &mut W1,
    stderr: &mut W2,
) -> Result<(), RunScriptError>
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
            return Err(RunScriptError::Parse);
        }
    };
    let mut vars = HashMap::new();
    let mut set_e_flag = set_e;
    let mut ctx = ExecScriptContext {
        vfs,
        vars: &mut vars,
        set_e: &mut set_e_flag,
        source_depth: 0,
        stdin,
        stdout,
        stderr,
    };
    let _ = exec_stmts(&mut ctx, &stmts)?;
    Ok(())
}
