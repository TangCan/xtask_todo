//! Command execution: `ExecContext`, redirects, and builtin dispatch (pwd, cd, ls, mkdir, cat, touch, echo, export-readonly, save, todo, exit/quit).

use std::io::Cursor;
use std::io::{Read, Write};
use std::path::Path;
use std::path::PathBuf;

use super::parser::{Pipeline, SimpleCommand};
use super::serialization;
use super::todo_io::{list_from_todos, load_todos, save_todos};
use super::vfs::Vfs;
use crate::TodoId;

/// Result of running a pipeline: continue the REPL loop or exit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunResult {
    Continue,
    Exit,
}

/// Execution context: VFS and standard streams for one command.
pub struct ExecContext<'a> {
    pub vfs: &'a mut Vfs,
    pub stdin: &'a mut dyn Read,
    pub stdout: &'a mut dyn Write,
    pub stderr: &'a mut dyn Write,
}

/// Error from builtin execution (redirect or VFS failure, unknown command).
#[derive(Debug)]
pub enum BuiltinError {
    UnknownCommand(String),
    RedirectRead,
    RedirectWrite,
    CdFailed,
    MkdirFailed,
    CatFailed,
    TouchFailed,
    LsFailed,
    ExportFailed,
    SaveFailed,
    TodoLoadFailed,
    TodoSaveFailed,
    TodoArgError,
    TodoDataError,
}

/// Run a single command with given streams. Redirects override the provided stdin/stdout/stderr.
fn run_builtin_with_streams(
    vfs: &mut Vfs,
    default_stdin: &mut dyn Read,
    default_stdout: &mut dyn Write,
    default_stderr: &mut dyn Write,
    cmd: &SimpleCommand,
) -> Result<(), BuiltinError> {
    let redirects = &cmd.redirects;
    let argv = &cmd.argv;

    let mut stdin_override: Option<Cursor<Vec<u8>>> = None;
    let mut stdout_override: Option<Vec<u8>> = None;
    let mut stderr_override: Option<Vec<u8>> = None;
    let mut stdout_redirect_path: Option<String> = None;
    let mut stderr_redirect_path: Option<String> = None;

    for r in redirects {
        match r.fd {
            0 => {
                let content = vfs
                    .read_file(&r.path)
                    .map_err(|()| BuiltinError::RedirectRead)?;
                stdin_override = Some(Cursor::new(content));
            }
            1 => {
                stdout_override = Some(Vec::new());
                stdout_redirect_path = Some(r.path.clone());
            }
            2 => {
                stderr_override = Some(Vec::new());
                stderr_redirect_path = Some(r.path.clone());
            }
            _ => {}
        }
    }

    let stdin: &mut dyn Read = stdin_override
        .as_mut()
        .map_or(default_stdin, |c| c as &mut dyn Read);
    let stdout: &mut dyn Write = stdout_override
        .as_mut()
        .map_or(default_stdout, |v| v as &mut dyn Write);
    let stderr: &mut dyn Write = stderr_override
        .as_mut()
        .map_or(default_stderr, |v| v as &mut dyn Write);

    let result = run_builtin_core(vfs, stdin, stdout, stderr, argv);

    if let Some(path) = stdout_redirect_path {
        if let Some(buf) = &stdout_override {
            vfs.write_file(&path, buf)
                .map_err(|()| BuiltinError::RedirectWrite)?;
        }
    }
    if let Some(path) = stderr_redirect_path {
        if let Some(buf) = &stderr_override {
            vfs.write_file(&path, buf)
                .map_err(|()| BuiltinError::RedirectWrite)?;
        }
    }

    result
}

/// Apply redirects: build optional stdin (Cursor over file content), stdout/stderr buffers.
/// Then run the builtin with the effective streams; after that write stdout/stderr buffers to vfs if redirected.
pub fn run_builtin(ctx: &mut ExecContext<'_>, cmd: &SimpleCommand) -> Result<(), BuiltinError> {
    run_builtin_with_streams(ctx.vfs, ctx.stdin, ctx.stdout, ctx.stderr, cmd)
}

/// Execute a pipeline: run each command with stdin from previous stage (or ctx.stdin for first)
/// and stdout to a buffer; last command's stdout is written to ctx.stdout. Redirects override pipe.
/// If the first command is "exit" or "quit", returns `Ok(RunResult::Exit)` without running.
pub fn execute_pipeline(
    ctx: &mut ExecContext<'_>,
    pipeline: &Pipeline,
) -> Result<RunResult, BuiltinError> {
    let commands = &pipeline.commands;
    if commands.is_empty() {
        return Ok(RunResult::Continue);
    }

    let first_argv0 = commands
        .first()
        .and_then(|c| c.argv.first())
        .map(String::as_str);
    if first_argv0 == Some("exit") || first_argv0 == Some("quit") {
        return Ok(RunResult::Exit);
    }

    let mut prev_output: Option<Vec<u8>> = None;

    for (i, cmd) in commands.iter().enumerate() {
        let is_first = i == 0;
        let is_last = i == commands.len() - 1;

        let mut pipe_stdin: Option<Cursor<Vec<u8>>> = prev_output.take().map(Cursor::new);
        let mut next_buffer = Vec::new();

        let stdin: &mut dyn Read = if is_first {
            ctx.stdin
        } else {
            pipe_stdin.as_mut().unwrap() as &mut dyn Read
        };
        let stdout: &mut dyn Write = if is_last {
            ctx.stdout
        } else {
            &mut next_buffer
        };

        run_builtin_with_streams(ctx.vfs, stdin, stdout, ctx.stderr, cmd)?;

        if !is_last {
            prev_output = Some(next_buffer);
        }
    }

    Ok(RunResult::Continue)
}

fn run_todo_cmd(
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    argv: &[String],
) -> Result<(), BuiltinError> {
    let sub = argv.get(1).map_or("list", String::as_str);
    let rest = argv.get(2..).unwrap_or(&[]);

    let todos = load_todos().map_err(|_| BuiltinError::TodoLoadFailed)?;
    let mut list = list_from_todos(todos);

    match sub {
        "list" => {
            let use_json = rest.iter().any(|a| a == "--json");
            if use_json {
                #[derive(serde::Serialize)]
                struct TodoRow {
                    id: u64,
                    title: String,
                    completed: bool,
                }
                let rows: Vec<TodoRow> = list
                    .list()
                    .into_iter()
                    .map(|t| TodoRow {
                        id: t.id.as_u64(),
                        title: t.title.clone(),
                        completed: t.completed,
                    })
                    .collect();
                let json =
                    serde_json::to_string_pretty(&rows).map_err(|_| BuiltinError::TodoDataError)?;
                writeln!(stdout, "{json}").map_err(|_| BuiltinError::RedirectWrite)?;
            } else {
                for t in list.list() {
                    let done = if t.completed { " [done]" } else { "" };
                    writeln!(stdout, "{}. {}{}", t.id.as_u64(), t.title, done)
                        .map_err(|_| BuiltinError::RedirectWrite)?;
                }
            }
            Ok(())
        }
        "add" => {
            let title = rest.join(" ").trim().to_string();
            if title.is_empty() {
                writeln!(stderr, "todo add: title must be non-empty")
                    .map_err(|_| BuiltinError::RedirectWrite)?;
                return Err(BuiltinError::TodoArgError);
            }
            let id = list.create(&title).map_err(|e| {
                let _ = writeln!(stderr, "todo add: {e}");
                BuiltinError::TodoArgError
            })?;
            save_todos(&list).map_err(|_| BuiltinError::TodoSaveFailed)?;
            writeln!(stdout, "{}", id.as_u64()).map_err(|_| BuiltinError::RedirectWrite)?;
            Ok(())
        }
        "show" => {
            let id_str = rest.first().ok_or(BuiltinError::TodoArgError)?;
            let id_raw: u64 = id_str.parse().map_err(|_| BuiltinError::TodoArgError)?;
            let id = TodoId::from_raw(id_raw).ok_or(BuiltinError::TodoArgError)?;
            let t = list.get(id).ok_or(BuiltinError::TodoDataError)?;
            let done = if t.completed { " [done]" } else { "" };
            writeln!(stdout, "{}. {}{}", t.id.as_u64(), t.title, done)
                .map_err(|_| BuiltinError::RedirectWrite)?;
            if let Some(ref d) = t.description {
                writeln!(stdout, "  {d}").map_err(|_| BuiltinError::RedirectWrite)?;
            }
            if let Some(ref due) = t.due_date {
                writeln!(stdout, "  due: {due}").map_err(|_| BuiltinError::RedirectWrite)?;
            }
            Ok(())
        }
        "update" => {
            let id_str = rest.first().ok_or(BuiltinError::TodoArgError)?;
            let id_raw: u64 = id_str.parse().map_err(|_| BuiltinError::TodoArgError)?;
            let id = TodoId::from_raw(id_raw).ok_or(BuiltinError::TodoArgError)?;
            let title = rest
                .get(1..)
                .map(|a| a.join(" ").trim().to_string())
                .unwrap_or_default();
            if title.is_empty() {
                writeln!(stderr, "todo update: new title must be non-empty")
                    .map_err(|_| BuiltinError::RedirectWrite)?;
                return Err(BuiltinError::TodoArgError);
            }
            list.update_title(id, &title).map_err(|e| {
                let _ = writeln!(stderr, "todo update: {e}");
                BuiltinError::TodoDataError
            })?;
            save_todos(&list).map_err(|_| BuiltinError::TodoSaveFailed)?;
            Ok(())
        }
        "complete" => {
            let id_str = rest.first().ok_or(BuiltinError::TodoArgError)?;
            let id_raw: u64 = id_str.parse().map_err(|_| BuiltinError::TodoArgError)?;
            let id = TodoId::from_raw(id_raw).ok_or(BuiltinError::TodoArgError)?;
            list.complete(id, false).map_err(|e| {
                let _ = writeln!(stderr, "todo complete: {e}");
                BuiltinError::TodoDataError
            })?;
            save_todos(&list).map_err(|_| BuiltinError::TodoSaveFailed)?;
            Ok(())
        }
        "delete" => {
            let id_str = rest.first().ok_or(BuiltinError::TodoArgError)?;
            let id_raw: u64 = id_str.parse().map_err(|_| BuiltinError::TodoArgError)?;
            let id = TodoId::from_raw(id_raw).ok_or(BuiltinError::TodoArgError)?;
            list.delete(id).map_err(|e| {
                let _ = writeln!(stderr, "todo delete: {e}");
                BuiltinError::TodoDataError
            })?;
            save_todos(&list).map_err(|_| BuiltinError::TodoSaveFailed)?;
            Ok(())
        }
        "search" => {
            let keyword = rest.join(" ").trim().to_string();
            for t in list.search(&keyword) {
                let done = if t.completed { " [done]" } else { "" };
                writeln!(stdout, "{}. {}{}", t.id.as_u64(), t.title, done)
                    .map_err(|_| BuiltinError::RedirectWrite)?;
            }
            Ok(())
        }
        "stats" => {
            let (total, open, completed) = list.stats();
            writeln!(
                stdout,
                "open: {open}  completed: {completed}  total: {total}"
            )
            .map_err(|_| BuiltinError::RedirectWrite)?;
            Ok(())
        }
        _ => {
            writeln!(stderr, "todo: unknown subcommand '{sub}' (list, add, show, update, complete, delete, search, stats)")
                .map_err(|_| BuiltinError::RedirectWrite)?;
            Err(BuiltinError::TodoArgError)
        }
    }
}

fn run_builtin_core(
    vfs: &mut Vfs,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    argv: &[String],
) -> Result<(), BuiltinError> {
    let name = argv.first().map_or("", String::as_str);
    match name {
        "pwd" => {
            writeln!(stdout, "{}", vfs.cwd()).map_err(|_| BuiltinError::RedirectWrite)?;
            Ok(())
        }
        "cd" => {
            let path = argv.get(1).map_or("/", String::as_str);
            vfs.set_cwd(path).map_err(|()| BuiltinError::CdFailed)?;
            Ok(())
        }
        "ls" => {
            let path = argv.get(1).map_or(".", String::as_str);
            let names = vfs.list_dir(path).map_err(|()| BuiltinError::LsFailed)?;
            for n in names {
                writeln!(stdout, "{n}").map_err(|_| BuiltinError::RedirectWrite)?;
            }
            Ok(())
        }
        "mkdir" => {
            let path = argv.get(1).ok_or(BuiltinError::MkdirFailed)?;
            vfs.mkdir(path).map_err(|()| BuiltinError::MkdirFailed)?;
            Ok(())
        }
        "cat" => {
            if argv.len() <= 1 {
                std::io::copy(stdin, stdout).map_err(|_| BuiltinError::CatFailed)?;
            } else {
                for path in argv.iter().skip(1) {
                    let content = vfs.read_file(path).map_err(|()| BuiltinError::CatFailed)?;
                    stdout
                        .write_all(&content)
                        .map_err(|_| BuiltinError::RedirectWrite)?;
                }
            }
            Ok(())
        }
        "touch" => {
            let path = argv.get(1).ok_or(BuiltinError::TouchFailed)?;
            vfs.touch(path).map_err(|()| BuiltinError::TouchFailed)?;
            Ok(())
        }
        "echo" => {
            let line = argv[1..].join(" ");
            writeln!(stdout, "{line}").map_err(|_| BuiltinError::RedirectWrite)?;
            Ok(())
        }
        "export-readonly" | "export_readonly" => {
            let path = argv.get(1).map_or(".", String::as_str);
            let temp_base = std::env::temp_dir();
            let subdir_name = format!(
                "dev_shell_export_{}_{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
            );
            let temp_dir = temp_base.join(&subdir_name);
            std::fs::create_dir_all(&temp_dir).map_err(|_| BuiltinError::ExportFailed)?;
            vfs.copy_tree_to_host(path, &temp_dir)
                .map_err(|()| BuiltinError::ExportFailed)?;
            let abs_path: PathBuf =
                std::fs::canonicalize(&temp_dir).map_err(|_| BuiltinError::ExportFailed)?;
            writeln!(stdout, "{}", abs_path.display()).map_err(|_| BuiltinError::RedirectWrite)?;
            Ok(())
        }
        "save" => {
            let path = argv.get(1).map_or(".dev_shell.bin", String::as_str);
            serialization::save_to_file(vfs, Path::new(path))
                .map_err(|_| BuiltinError::SaveFailed)?;
            Ok(())
        }
        "todo" => run_todo_cmd(stdout, stderr, argv),
        "help" => {
            writeln!(stdout, "Supported commands:").map_err(|_| BuiltinError::RedirectWrite)?;
            writeln!(stdout, "  pwd              print current working directory")
                .map_err(|_| BuiltinError::RedirectWrite)?;
            writeln!(stdout, "  cd <path>        change directory")
                .map_err(|_| BuiltinError::RedirectWrite)?;
            writeln!(stdout, "  ls [path]        list directory contents")
                .map_err(|_| BuiltinError::RedirectWrite)?;
            writeln!(stdout, "  mkdir <path>     create directory (and parents)")
                .map_err(|_| BuiltinError::RedirectWrite)?;
            writeln!(
                stdout,
                "  cat [path...]    print file contents (or stdin if no path)"
            )
            .map_err(|_| BuiltinError::RedirectWrite)?;
            writeln!(stdout, "  touch <path>     create empty file")
                .map_err(|_| BuiltinError::RedirectWrite)?;
            writeln!(stdout, "  echo [args...]   print arguments")
                .map_err(|_| BuiltinError::RedirectWrite)?;
            writeln!(stdout, "  save [path]      save virtual FS to .bin file")
                .map_err(|_| BuiltinError::RedirectWrite)?;
            writeln!(
                stdout,
                "  export-readonly [path]  copy VFS subtree to host temp dir (read-only)"
            )
            .map_err(|_| BuiltinError::RedirectWrite)?;
            writeln!(stdout, "  todo [list|add|show|update|complete|delete|search|stats] ...  todo list (shares .todo.json with cargo xtask todo)").map_err(|_| BuiltinError::RedirectWrite)?;
            writeln!(stdout, "  exit, quit       exit the shell")
                .map_err(|_| BuiltinError::RedirectWrite)?;
            writeln!(stdout, "  help             show this help")
                .map_err(|_| BuiltinError::RedirectWrite)?;
            Ok(())
        }
        _ => {
            writeln!(stderr, "unknown command: {name}").map_err(|_| BuiltinError::RedirectWrite)?;
            Err(BuiltinError::UnknownCommand(name.to_string()))
        }
    }
}
