//! Run host binaries against an exported workspace and sync back.

use std::path::Path;
use std::process::Command;

use super::super::vfs::Vfs;
use super::elf::restore_execute_bits_for_build_artifacts;
use super::error::SandboxError;
use super::export::export_vfs_to_temp_dir;
use super::paths::find_in_path;
use super::sync::{host_export_root, sync_host_dir_to_vfs};

/// Run a subprocess with cwd set to `export_dir`.
///
/// Child inherits the process stdin/stdout/stderr (redirects for this builtin can be added later).
/// Returns the process exit status; the caller should then sync back and remove the dir.
///
/// # Errors
/// Returns `SandboxError::ExportFailed` if spawning the process fails (e.g. program not found).
pub fn run_in_export_dir<P: AsRef<Path>>(
    export_dir: &Path,
    program: P,
    args: &[String],
) -> Result<std::process::ExitStatus, SandboxError> {
    let mut cmd = Command::new(program.as_ref());
    cmd.args(args)
        .current_dir(export_dir)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit());

    #[cfg(target_os = "linux")]
    if super::linux_mount::linux_mount_namespace_enabled() {
        super::linux_mount::apply_linux_private_mount_namespace(&mut cmd);
    }

    let mut child = cmd.spawn().map_err(SandboxError::ExportFailed)?;
    child.wait().map_err(SandboxError::ExportFailed)
}

/// Export VFS subtree at `vfs_path`, run `program` with `args` in that dir, sync changes back, then cleanup.
/// Returns the child's exit status. Caller should check `status.success()`.
///
/// # Isolation
/// See parent module docs: temp export + host `PATH` binary; optional Linux mount namespace via
/// `DEVSHELL_RUST_MOUNT_NAMESPACE` (no container engine).
///
/// # Errors
/// Returns `SandboxError` if binary not in PATH (`ExportFailed` with a message), export fails, spawn fails, or sync fails.
pub fn run_rust_tool(
    vfs: &mut Vfs,
    vfs_path: &str,
    program: &str,
    args: &[String],
) -> Result<std::process::ExitStatus, SandboxError> {
    let export_dir = export_vfs_to_temp_dir(vfs, vfs_path)?;
    let work_dir = host_export_root(&export_dir, vfs_path);
    restore_execute_bits_for_build_artifacts(&work_dir)?;

    let program_path = find_in_path(program).ok_or_else(|| {
        SandboxError::ExportFailed(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("{program} not found in PATH"),
        ))
    })?;
    let status = run_in_export_dir(&work_dir, &program_path, args);

    let sync_result = sync_host_dir_to_vfs(&export_dir, vfs_path, vfs);
    let _ = std::fs::remove_dir_all(&export_dir);

    sync_result?;
    status
}
