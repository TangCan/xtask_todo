//! Copy VFS or host-backed trees to a host directory.

use std::path::Path;

use super::error::VfsError;
use super::node::Node;

/// Copy a host tree into `host_dir` (used when [`super::Vfs`] is host-backed).
pub(super) fn copy_host_path_to_host_dir(src: &Path, dst_root: &Path) -> Result<(), VfsError> {
    let meta = std::fs::metadata(src).map_err(VfsError::Io)?;
    if meta.is_file() {
        if let Some(parent) = dst_root.parent() {
            std::fs::create_dir_all(parent).map_err(VfsError::Io)?;
        }
        let data = std::fs::read(src).map_err(VfsError::Io)?;
        std::fs::write(dst_root, data).map_err(VfsError::Io)?;
        return Ok(());
    }
    if meta.is_dir() {
        std::fs::create_dir_all(dst_root).map_err(VfsError::Io)?;
        for e in std::fs::read_dir(src).map_err(VfsError::Io)? {
            let e = e.map_err(VfsError::Io)?;
            let ft = e.file_type().map_err(VfsError::Io)?;
            let s = e.path();
            let d = dst_root.join(e.file_name());
            if ft.is_dir() {
                copy_host_path_to_host_dir(&s, &d)?;
            } else {
                let data = std::fs::read(&s).map_err(VfsError::Io)?;
                std::fs::write(&d, data).map_err(VfsError::Io)?;
            }
        }
        return Ok(());
    }
    Err(VfsError::InvalidPath)
}

/// Returns true if the name is safe to use as a single path component (no .. or separators).
fn is_safe_component(name: &str) -> bool {
    !name.is_empty() && name != "." && name != ".." && !name.contains('/') && !name.contains('\\')
}

/// Recursively copy a VFS node to the host path. Creates dirs and writes file contents.
pub(super) fn copy_node_to_host(node: &Node, host_path: &Path) -> Result<(), VfsError> {
    match node {
        Node::Dir { name, children } => {
            let dir_path = if name.is_empty() {
                host_path.to_path_buf()
            } else {
                if !is_safe_component(name) {
                    return Err(VfsError::InvalidPath);
                }
                host_path.join(name)
            };
            std::fs::create_dir_all(&dir_path).map_err(VfsError::Io)?;
            for child in children {
                copy_node_to_host(child, &dir_path)?;
            }
            Ok(())
        }
        Node::File { name, content } => {
            if !is_safe_component(name) {
                return Err(VfsError::InvalidPath);
            }
            let file_path = host_path.join(name);
            std::fs::write(&file_path, content).map_err(VfsError::Io)?;
            Ok(())
        }
    }
}
