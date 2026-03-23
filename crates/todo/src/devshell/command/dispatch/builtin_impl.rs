//! Builtin `help`, `export-readonly`, rust tools, and core command match.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use super::super::super::sandbox;
use super::super::super::serialization;
use super::super::super::vfs::Vfs;
use super::super::super::vm::SessionHolder;
use super::super::super::vm::VmError;
use super::super::todo_builtin::run_todo_cmd;
use super::super::types::BuiltinError;
use super::workspace::{workspace_list_dir, workspace_mkdir, workspace_read_file, workspace_touch};

pub(super) fn run_builtin_help(stdout: &mut dyn Write) -> Result<(), BuiltinError> {
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
        "  export-readonly [path]  Mode S: copy VFS subtree to host temp dir; Mode P: mirror guest tree under a logical path in VFS"
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

pub(super) fn run_builtin_export_readonly(
    vfs: &mut Vfs,
    vm_session: &mut SessionHolder,
    stdout: &mut dyn Write,
    path: &str,
) -> Result<(), BuiltinError> {
    #[cfg(not(unix))]
    let _ = vm_session;
    #[cfg(unix)]
    if vm_session.is_guest_primary() {
        let dest = crate::devshell::workspace::guest_export_readonly_to_vfs(vfs, vm_session, path)
            .map_err(|e| BuiltinError::GuestFsOpFailed(e.to_string()))?;
        writeln!(stdout, "{dest}").map_err(|_| BuiltinError::RedirectWrite)?;
        return Ok(());
    }
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

pub(super) fn run_rust_tool_builtin(
    vfs: &mut Vfs,
    vm_session: &mut SessionHolder,
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

pub(super) fn run_builtin_core(
    vfs: &mut Vfs,
    vm_session: &mut SessionHolder,
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
            let names = workspace_list_dir(vfs, vm_session, path)?;
            for n in names {
                writeln!(stdout, "{n}").map_err(|_| BuiltinError::RedirectWrite)?;
            }
            Ok(())
        }
        "mkdir" => {
            let path = argv.get(1).ok_or(BuiltinError::MkdirFailed)?;
            workspace_mkdir(vfs, vm_session, path)?;
            Ok(())
        }
        "cat" => {
            if argv.len() <= 1 {
                std::io::copy(stdin, stdout).map_err(|_| BuiltinError::CatFailed)?;
            } else {
                for path in argv.iter().skip(1) {
                    let content = workspace_read_file(vfs, vm_session, path)?;
                    stdout
                        .write_all(&content)
                        .map_err(|_| BuiltinError::RedirectWrite)?;
                }
            }
            Ok(())
        }
        "touch" => {
            let path = argv.get(1).ok_or(BuiltinError::TouchFailed)?;
            workspace_touch(vfs, vm_session, path)?;
            Ok(())
        }
        "echo" => {
            let line = argv[1..].join(" ");
            writeln!(stdout, "{line}").map_err(|_| BuiltinError::RedirectWrite)?;
            Ok(())
        }
        "export-readonly" | "export_readonly" => {
            let path = argv.get(1).map_or(".", String::as_str);
            run_builtin_export_readonly(vfs, vm_session, stdout, path)
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
