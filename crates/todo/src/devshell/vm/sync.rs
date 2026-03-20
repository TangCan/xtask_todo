//! Push/pull between VFS and a persistent host workspace directory (session VM staging).
//!
//! Layout matches [`super::super::sandbox::export_vfs_to_temp_dir`]: `workspace_parent` holds the
//! leaf directory named after the last segment of `vfs_path` (see [`super::super::sandbox::host_export_root`]).

#![allow(clippy::pedantic, clippy::nursery)]

use std::path::{Path, PathBuf};

use super::super::sandbox;
use super::super::vfs::{Node, Vfs, VfsError};

/// Errors from workspace sync helpers.
#[derive(Debug)]
pub enum VmSyncError {
    Io(std::io::Error),
    Vfs(VfsError),
    Sandbox(sandbox::SandboxError),
}

impl std::fmt::Display for VmSyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{e}"),
            Self::Vfs(e) => write!(f, "{e}"),
            Self::Sandbox(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for VmSyncError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Vfs(e) => Some(e),
            Self::Sandbox(e) => Some(e),
        }
    }
}

fn vfs_child_vfs_path(parent: &str, name: &str) -> String {
    let p = parent.trim_end_matches('/');
    if p.is_empty() || p == "/" {
        format!("/{name}")
    } else {
        format!("{p}/{name}")
    }
}

fn walk_vfs_files_recurse(
    vfs: &Vfs,
    dir_vfs: &str,
    rel: PathBuf,
    out: &mut Vec<(PathBuf, Vec<u8>)>,
) -> Result<(), VfsError> {
    for name in vfs.list_dir(dir_vfs)? {
        let child = vfs_child_vfs_path(dir_vfs, &name);
        let n = vfs.resolve_absolute(&child)?;
        match n {
            Node::File { content, .. } => {
                out.push((rel.join(&name), content));
            }
            Node::Dir { .. } => {
                walk_vfs_files_recurse(vfs, &child, rel.join(&name), out)?;
            }
        }
    }
    Ok(())
}

/// Remove the exported leaf directory (if present), then copy the full VFS subtree at `vfs_path`
/// into `workspace_parent`, and restore ELF execute bits under `target/`.
///
/// # Errors
/// Returns [`VmSyncError`] on I/O or VFS failure.
pub fn push_full(vfs: &Vfs, vfs_path: &str, workspace_parent: &Path) -> Result<(), VmSyncError> {
    std::fs::create_dir_all(workspace_parent).map_err(VmSyncError::Io)?;
    let host_leaf = sandbox::host_export_root(workspace_parent, vfs_path);
    if host_leaf.exists() {
        std::fs::remove_dir_all(&host_leaf).map_err(VmSyncError::Io)?;
    }
    vfs.copy_tree_to_host(vfs_path, workspace_parent)
        .map_err(VmSyncError::Vfs)?;
    let work_dir = sandbox::host_export_root(workspace_parent, vfs_path);
    sandbox::restore_execute_bits_for_build_artifacts(&work_dir).map_err(VmSyncError::Sandbox)?;
    Ok(())
}

/// For each file under the VFS subtree at `vfs_path`, write to the host workspace only when missing or content differs.
///
/// # Errors
/// Returns [`VmSyncError`] on failure.
pub fn push_incremental(
    vfs: &Vfs,
    vfs_path: &str,
    workspace_parent: &Path,
) -> Result<(), VmSyncError> {
    let abs = vfs.resolve_to_absolute(vfs_path);
    let host_root = sandbox::host_export_root(workspace_parent, vfs_path);
    std::fs::create_dir_all(&host_root).map_err(VmSyncError::Io)?;

    let mut files: Vec<(PathBuf, Vec<u8>)> = Vec::new();
    walk_vfs_files_recurse(vfs, &abs, PathBuf::new(), &mut files).map_err(VmSyncError::Vfs)?;

    for (rel, content) in files {
        let host_file = host_root.join(&rel);
        let write_it = match std::fs::read(&host_file) {
            Ok(existing) => existing != content,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => true,
            Err(e) => return Err(VmSyncError::Io(e)),
        };
        if write_it {
            if let Some(d) = host_file.parent() {
                std::fs::create_dir_all(d).map_err(VmSyncError::Io)?;
            }
            std::fs::write(&host_file, &content).map_err(VmSyncError::Io)?;
        }
    }

    sandbox::restore_execute_bits_for_build_artifacts(&host_root).map_err(VmSyncError::Sandbox)?;
    Ok(())
}

/// Merge host workspace tree into the VFS at `vfs_path` (add/update only; same semantics as [`sandbox::sync_host_dir_to_vfs`]).
///
/// Used for both “full” and “incremental” pull until delete-on-host is specified.
pub fn pull_workspace_to_vfs(
    workspace_parent: &Path,
    vfs_path: &str,
    vfs: &mut Vfs,
) -> Result<(), sandbox::SandboxError> {
    sandbox::sync_host_dir_to_vfs(workspace_parent, vfs_path, vfs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_incremental_writes_only_changed_file() {
        let mut vfs = Vfs::new();
        vfs.mkdir("/p").unwrap();
        vfs.write_file("/p/a.txt", b"one").unwrap();
        vfs.write_file("/p/b.txt", b"fix").unwrap();

        let base = std::env::temp_dir().join(format!(
            "devshell_vm_sync_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&base);
        push_full(&vfs, "/p", &base).unwrap();

        let p_host = base.join("p");
        assert_eq!(
            std::fs::read_to_string(p_host.join("a.txt")).unwrap(),
            "one"
        );
        assert_eq!(
            std::fs::read_to_string(p_host.join("b.txt")).unwrap(),
            "fix"
        );

        vfs.write_file("/p/a.txt", b"two").unwrap();
        push_incremental(&vfs, "/p", &base).unwrap();
        assert_eq!(
            std::fs::read_to_string(p_host.join("a.txt")).unwrap(),
            "two"
        );
        assert_eq!(
            std::fs::read_to_string(p_host.join("b.txt")).unwrap(),
            "fix"
        );

        let mut vfs2 = Vfs::new();
        vfs2.mkdir("/p").unwrap();
        pull_workspace_to_vfs(&base, "/p", &mut vfs2).unwrap();
        assert_eq!(vfs2.read_file("/p/a.txt").unwrap(), b"two");
        assert_eq!(vfs2.read_file("/p/b.txt").unwrap(), b"fix");

        let _ = std::fs::remove_dir_all(&base);
    }
}
