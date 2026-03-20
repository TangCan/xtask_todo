//! Copy VFS subtree to a unique host directory.

use std::path::PathBuf;

use super::super::vfs::Vfs;
use super::error::SandboxError;
use super::paths::devshell_export_parent_dir;

/// Export the VFS subtree at `vfs_path` (e.g. current cwd) to a new temporary directory.
///
/// The directory lives under the system temp dir with a unique name (`devshell_<pid>_<nanos>`)
/// and on Unix has mode `0o700`. Returns the path to the created directory;
/// the caller is responsible for cleanup (remove dir when done).
///
/// # Errors
/// Returns `SandboxError::ExportFailed` if the temp dir cannot be created or permissions set.
/// Returns `SandboxError::CopyFailed` if VFS copy to host fails.
pub fn export_vfs_to_temp_dir(vfs: &Vfs, vfs_path: &str) -> Result<PathBuf, SandboxError> {
    let temp_base = devshell_export_parent_dir();
    std::fs::create_dir_all(&temp_base).map_err(SandboxError::ExportFailed)?;
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let name = format!("devshell_{}_{}", std::process::id(), nanos);
    let path = temp_base.join(name);

    std::fs::create_dir(&path).map_err(SandboxError::ExportFailed)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&path)
            .map_err(SandboxError::ExportFailed)?
            .permissions();
        perms.set_mode(0o700);
        std::fs::set_permissions(&path, perms).map_err(SandboxError::ExportFailed)?;
    }

    vfs.copy_tree_to_host(vfs_path, &path)
        .map_err(SandboxError::CopyFailed)?;

    Ok(path)
}
