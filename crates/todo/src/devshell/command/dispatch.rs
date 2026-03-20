//! Redirect handling, pipeline execution, and builtin dispatch.

use std::io::Cursor;
use std::io::{Read, Write};
use std::path::Path;
use std::path::PathBuf;

use super::super::parser::{Pipeline, SimpleCommand};
use super::super::sandbox;
use super::super::serialization;
use super::super::vfs::Vfs;
use super::super::vm::VmError;
use super::todo_builtin::run_todo_cmd;
use super::types::{BuiltinError, ExecContext, RunResult};

/// Run a single command with given streams. Redirects override the provided stdin/stdout/stderr.
fn run_builtin_with_streams(
    vfs: &mut Vfs,
    vm_session: &mut super::super::vm::SessionHolder,
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
                    .map_err(|_| BuiltinError::RedirectRead)?;
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

    let result = run_builtin_core(vfs, vm_session, stdin, stdout, stderr, argv);

    if let Some(path) = stdout_redirect_path {
        if let Some(buf) = &stdout_override {
            vfs.write_file(&path, buf)
                .map_err(|_| BuiltinError::RedirectWrite)?;
        }
    }
    if let Some(path) = stderr_redirect_path {
        if let Some(buf) = &stderr_override {
            vfs.write_file(&path, buf)
                .map_err(|_| BuiltinError::RedirectWrite)?;
        }
    }

    result
}

/// Apply redirects: build optional stdin (Cursor over file content), stdout/stderr buffers.
/// Then run the builtin with the effective streams; after that write stdout/stderr buffers to vfs if redirected.
///
/// # Errors
/// Returns `BuiltinError` on redirect or builtin execution failure.
pub fn run_builtin(ctx: &mut ExecContext<'_>, cmd: &SimpleCommand) -> Result<(), BuiltinError> {
    run_builtin_with_streams(
        ctx.vfs,
        ctx.vm_session,
        ctx.stdin,
        ctx.stdout,
        ctx.stderr,
        cmd,
    )
}

/// Execute a pipeline: run each command with stdin from previous stage (or `ctx.stdin` for first),
/// stdout to a buffer; last command's stdout is written to `ctx.stdout`. Redirects override pipe.
///
/// # Errors
/// Returns `BuiltinError` if any command or redirect fails.
///
/// # Panics
/// Panics if the pipeline state is inconsistent (non-first stage without pipe input); this is a programming error.
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
            pipe_stdin
                .as_mut()
                .expect("non-first pipeline stage has pipe input") as &mut dyn Read
        };
        let stdout: &mut dyn Write = if is_last {
            ctx.stdout
        } else {
            &mut next_buffer
        };

        run_builtin_with_streams(ctx.vfs, ctx.vm_session, stdin, stdout, ctx.stderr, cmd)?;

        if !is_last {
            prev_output = Some(next_buffer);
        }
    }

    Ok(RunResult::Continue)
}

fn run_builtin_help(stdout: &mut dyn Write) -> Result<(), BuiltinError> {
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
    writeln!(stdout, "  todo [list|add|show|update|complete|delete|search|stats] ...  todo list (shares .todo.json with cargo xtask todo)")
        .map_err(|_| BuiltinError::RedirectWrite)?;
    writeln!(
        stdout,
        "  rustup [args...] run rustup in sandbox (exports VFS cwd, runs, syncs back)"
    )
    .map_err(|_| BuiltinError::RedirectWrite)?;
    writeln!(
        stdout,
        "  cargo [args...]  run cargo in sandbox (exports VFS cwd, runs, syncs back)"
    )
    .map_err(|_| BuiltinError::RedirectWrite)?;
    writeln!(stdout, "  exit, quit       exit the shell")
        .map_err(|_| BuiltinError::RedirectWrite)?;
    writeln!(stdout, "  help             show this help")
        .map_err(|_| BuiltinError::RedirectWrite)?;
    Ok(())
}

fn run_builtin_export_readonly(
    vfs: &Vfs,
    stdout: &mut dyn Write,
    path: &str,
) -> Result<(), BuiltinError> {
    let temp_base = sandbox::devshell_export_parent_dir();
    std::fs::create_dir_all(&temp_base).map_err(|_| BuiltinError::ExportFailed)?;
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
        .map_err(|_| BuiltinError::ExportFailed)?;
    let abs_path: PathBuf =
        std::fs::canonicalize(&temp_dir).map_err(|_| BuiltinError::ExportFailed)?;
    writeln!(stdout, "{}", abs_path.display()).map_err(|_| BuiltinError::RedirectWrite)?;
    Ok(())
}

fn run_rust_tool_builtin(
    vfs: &mut Vfs,
    vm_session: &mut super::super::vm::SessionHolder,
    stderr: &mut dyn Write,
    program: &str,
    argv: &[String],
) -> Result<(), BuiltinError> {
    let tool_args: Vec<String> = argv.get(1..).unwrap_or_default().to_vec();
    let cwd = vfs.cwd().to_string();
    match vm_session.run_rust_tool(vfs, &cwd, program, &tool_args) {
        Ok(status) => {
            if status.success() {
                Ok(())
            } else {
                Err(BuiltinError::RustToolNonZeroExit {
                    program: program.to_string(),
                    code: status.code(),
                })
            }
        }
        Err(VmError::Sandbox(sandbox::SandboxError::ExportFailed(e))) => {
            let _ = writeln!(stderr, "{program}: {e}");
            if e.kind() == std::io::ErrorKind::NotFound {
                Err(if program == "rustup" {
                    BuiltinError::RustupNotFound
                } else {
                    BuiltinError::CargoNotFound
                })
            } else {
                Err(BuiltinError::SandboxExportFailed)
            }
        }
        Err(VmError::Sandbox(sandbox::SandboxError::CopyFailed(_))) => {
            let _ = writeln!(stderr, "{program}: export failed");
            Err(BuiltinError::SandboxExportFailed)
        }
        Err(VmError::Sandbox(sandbox::SandboxError::SyncBackFailed(e))) => {
            let _ = writeln!(stderr, "{program}: sync back failed: {e}");
            Err(BuiltinError::SandboxSyncFailed)
        }
        Err(VmError::Sync(e)) => {
            let _ = writeln!(stderr, "{program}: {e}");
            Err(BuiltinError::VmWorkspaceSyncFailed)
        }
        Err(VmError::BackendNotImplemented(msg)) => {
            let _ = writeln!(stderr, "{program}: {msg}");
            Err(BuiltinError::VmSessionError(msg.to_string()))
        }
        Err(VmError::Lima(msg) | VmError::Ipc(msg)) => {
            let _ = writeln!(stderr, "{program}: {msg}");
            Err(BuiltinError::VmSessionError(msg))
        }
    }
}

fn run_builtin_core(
    vfs: &mut Vfs,
    vm_session: &mut super::super::vm::SessionHolder,
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
            vfs.set_cwd(path).map_err(|_| BuiltinError::CdFailed)?;
            Ok(())
        }
        "ls" => {
            let path = argv.get(1).map_or(".", String::as_str);
            let names = vfs.list_dir(path).map_err(|_| BuiltinError::LsFailed)?;
            for n in names {
                writeln!(stdout, "{n}").map_err(|_| BuiltinError::RedirectWrite)?;
            }
            Ok(())
        }
        "mkdir" => {
            let path = argv.get(1).ok_or(BuiltinError::MkdirFailed)?;
            vfs.mkdir(path).map_err(|_| BuiltinError::MkdirFailed)?;
            Ok(())
        }
        "cat" => {
            if argv.len() <= 1 {
                std::io::copy(stdin, stdout).map_err(|_| BuiltinError::CatFailed)?;
            } else {
                for path in argv.iter().skip(1) {
                    let content = vfs.read_file(path).map_err(|_| BuiltinError::CatFailed)?;
                    stdout
                        .write_all(&content)
                        .map_err(|_| BuiltinError::RedirectWrite)?;
                }
            }
            Ok(())
        }
        "touch" => {
            let path = argv.get(1).ok_or(BuiltinError::TouchFailed)?;
            vfs.touch(path).map_err(|_| BuiltinError::TouchFailed)?;
            Ok(())
        }
        "echo" => {
            let line = argv[1..].join(" ");
            writeln!(stdout, "{line}").map_err(|_| BuiltinError::RedirectWrite)?;
            Ok(())
        }
        "export-readonly" | "export_readonly" => {
            let path = argv.get(1).map_or(".", String::as_str);
            run_builtin_export_readonly(&*vfs, stdout, path)
        }
        "save" => {
            let path = argv.get(1).map_or(".dev_shell.bin", String::as_str);
            serialization::save_to_file(vfs, Path::new(path))
                .map_err(|_| BuiltinError::SaveFailed)?;
            Ok(())
        }
        "todo" => run_todo_cmd(stdout, stderr, argv),
        "rustup" => run_rust_tool_builtin(vfs, vm_session, stderr, "rustup", argv),
        "cargo" => run_rust_tool_builtin(vfs, vm_session, stderr, "cargo", argv),
        "help" => run_builtin_help(stdout),
        _ => {
            writeln!(stderr, "unknown command: {name}").map_err(|_| BuiltinError::RedirectWrite)?;
            Err(BuiltinError::UnknownCommand(name.to_string()))
        }
    }
}
