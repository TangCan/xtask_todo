//! Merge host export tree back into the VFS.

use std::path::{Path, PathBuf};

use super::super::vfs::Vfs;
use super::error::SandboxError;

/// Host path that corresponds to the root of the exported VFS subtree.
///
/// `copy_tree_to_host` places the resolved node (with its name) directly under `export_dir`,
/// so for `/projects/hello` we get `export_dir/hello`, not `export_dir/projects/hello`.
pub fn host_export_root(export_dir: &Path, vfs_path: &str) -> PathBuf {
    let trimmed = vfs_path.trim_matches('/');
    if trimmed.is_empty() {
        export_dir.to_path_buf()
    } else {
        let last = trimmed.split('/').next_back().unwrap_or(".");
        export_dir.join(last)
    }
}

/// Sync the host export directory back into the VFS at `vfs_path`.
///
/// Walks the host subtree and creates/overwrites files and dirs in the VFS.
/// Does not remove VFS nodes that no longer exist on host (add/update only).
///
/// # Errors
/// Returns `SandboxError::SyncBackFailed` on host read or VFS write failure.
pub fn sync_host_dir_to_vfs(
    export_dir: &Path,
    vfs_path: &str,
    vfs: &mut Vfs,
) -> Result<(), SandboxError> {
    let host_root = host_export_root(export_dir, vfs_path);
    if !host_root.is_dir() {
        return Ok(());
    }
    sync_host_dir_to_vfs_recursive(&host_root, vfs_path, vfs)
}

fn sync_host_dir_to_vfs_recursive(
    host_dir: &Path,
    vfs_prefix: &str,
    vfs: &mut Vfs,
) -> Result<(), SandboxError> {
    let entries = std::fs::read_dir(host_dir).map_err(SandboxError::SyncBackFailed)?;
    for entry in entries {
        let entry = entry.map_err(SandboxError::SyncBackFailed)?;
        let name = entry.file_name();
        let name_str = name.to_str().ok_or_else(|| {
            SandboxError::SyncBackFailed(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "non-UTF8 path",
            ))
        })?;
        let vfs_path_here = if vfs_prefix == "/" || vfs_prefix.is_empty() {
            format!("/{name_str}")
        } else {
            format!("{vfs_prefix}/{name_str}")
        };

        if entry.path().is_dir() {
            vfs.mkdir(&vfs_path_here)
                .map_err(|e| SandboxError::SyncBackFailed(std::io::Error::other(e.to_string())))?;
            sync_host_dir_to_vfs_recursive(&entry.path(), &vfs_path_here, vfs)?;
        } else {
            let content = std::fs::read(entry.path()).map_err(SandboxError::SyncBackFailed)?;
            vfs.write_file(&vfs_path_here, &content)
                .map_err(|e| SandboxError::SyncBackFailed(std::io::Error::other(e.to_string())))?;
        }
    }
    Ok(())
}
