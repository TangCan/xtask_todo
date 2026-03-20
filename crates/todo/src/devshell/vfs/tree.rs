//! [`Vfs`] — in-memory or host-backed virtual filesystem.

use std::path::{Path, PathBuf};

use super::copy_to_host::{copy_host_path_to_host_dir, copy_node_to_host};
use super::error::VfsError;
use super::node::Node;
use super::path::resolve_path_with_cwd;

pub struct Vfs {
    root: Node,
    cwd: String,
    /// When set, project-tree operations use this host directory (logical `/` = this path).
    /// Matches the Lima workspace mount so offline REPL and VM see the same files.
    host_root: Option<PathBuf>,
}

impl Default for Vfs {
    fn default() -> Self {
        Self::new()
    }
}

impl Vfs {
    #[must_use]
    pub fn new() -> Self {
        Self {
            root: Node::Dir {
                name: String::new(),
                children: vec![],
            },
            cwd: "/".to_string(),
            host_root: None,
        }
    }

    /// Project tree backed by a host directory (same path as the Lima `workspace_parent` mount).
    ///
    /// # Errors
    /// I/O errors from [`std::fs::create_dir_all`] or [`std::fs::canonicalize`].
    pub fn new_host_root(root: impl AsRef<Path>) -> std::io::Result<Self> {
        let root = root.as_ref();
        std::fs::create_dir_all(root)?;
        let root = root.canonicalize()?;
        Ok(Self {
            root: Node::Dir {
                name: String::new(),
                children: vec![],
            },
            cwd: "/".to_string(),
            host_root: Some(root),
        })
    }

    /// `true` when this instance uses the host directory ([`Self::new_host_root`]) instead of the in-memory tree.
    #[must_use]
    pub const fn is_host_backed(&self) -> bool {
        self.host_root.is_some()
    }

    /// Construct VFS from root node and cwd (used by deserialization).
    #[must_use]
    pub const fn from_parts(root: Node, cwd: String) -> Self {
        Self {
            root,
            cwd,
            host_root: None,
        }
    }

    fn logical_to_host_path(&self, abs_logical: &str) -> PathBuf {
        let root = self.host_root.as_ref().unwrap();
        let abs_logical = abs_logical.trim_end_matches('/');
        let mut p = root.clone();
        if abs_logical.is_empty() || abs_logical == "/" {
            return p;
        }
        for seg in abs_logical.split('/').filter(|s| !s.is_empty()) {
            p.push(seg);
        }
        p
    }
    #[must_use]
    pub fn cwd(&self) -> &str {
        &self.cwd
    }
    #[must_use]
    pub const fn root(&self) -> &Node {
        &self.root
    }

    /// Resolve an absolute path to a node. Path must be absolute (normalized). Trailing '/' is trimmed.
    ///
    /// # Errors
    /// Returns `VfsError::InvalidPath` if any segment is missing.
    pub fn resolve_absolute(&self, path: &str) -> Result<Node, VfsError> {
        if self.host_root.is_some() {
            let path = path.trim_end_matches('/');
            let p = self.logical_to_host_path(path);
            let meta = std::fs::metadata(&p).map_err(|_| VfsError::InvalidPath)?;
            if meta.is_file() {
                let content = std::fs::read(&p).map_err(VfsError::Io)?;
                let name = p
                    .file_name()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default();
                return Ok(Node::File { name, content });
            }
            if meta.is_dir() {
                let name = p
                    .file_name()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default();
                return Ok(Node::Dir {
                    name,
                    children: vec![],
                });
            }
            return Err(VfsError::InvalidPath);
        }
        let path = path.trim_end_matches('/');
        if path.is_empty() || path == "/" {
            return Ok(self.root.clone());
        }
        let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let mut current = &self.root;
        for segment in segments {
            current = current.child(segment).ok_or(VfsError::InvalidPath)?;
        }
        Ok(current.clone())
    }

    /// 将任意路径（相对或绝对）归一化并解析为绝对路径字符串。
    /// 相对路径先与 cwd 拼接再归一化，这样 ".." 能正确退到上级目录。
    #[must_use]
    pub fn resolve_to_absolute(&self, path: &str) -> String {
        resolve_path_with_cwd(&self.cwd, path)
    }

    /// Create directory at path (`mkdir_all` style). Creates any missing parent directories.
    ///
    /// # Errors
    /// Returns `VfsError::InvalidPath` if any path component exists and is not a directory.
    pub fn mkdir(&mut self, path: &str) -> Result<(), VfsError> {
        if self.host_root.is_some() {
            let abs = self.resolve_to_absolute(path);
            let p = self.logical_to_host_path(&abs);
            std::fs::create_dir_all(&p).map_err(VfsError::Io)?;
            return Ok(());
        }
        let abs = self.resolve_to_absolute(path);
        let segments: Vec<&str> = abs.split('/').filter(|s| !s.is_empty()).collect();
        if segments.is_empty() {
            return Ok(());
        }
        let mut indices: Vec<usize> = vec![];
        for segment in segments {
            let current = Self::get_mut_at(&mut self.root, &indices);
            match current {
                Node::Dir { children, .. } => {
                    let pos = children.iter().position(|c| c.name() == segment);
                    if let Some(i) = pos {
                        if !children[i].is_dir() {
                            return Err(VfsError::InvalidPath);
                        }
                        indices.push(i);
                    } else {
                        children.push(Node::Dir {
                            name: segment.to_string(),
                            children: vec![],
                        });
                        indices.push(children.len() - 1);
                    }
                }
                Node::File { .. } => return Err(VfsError::InvalidPath),
            }
        }
        Ok(())
    }

    /// Create or overwrite a file at path. Parent directory must exist and be a directory.
    ///
    /// # Errors
    /// Returns `VfsError::InvalidPath` if parent path does not exist or a component is not a directory.
    pub fn write_file(&mut self, path: &str, content: &[u8]) -> Result<(), VfsError> {
        if self.host_root.is_some() {
            let abs = self.resolve_to_absolute(path);
            let p = self.logical_to_host_path(&abs);
            if let Some(parent) = p.parent() {
                std::fs::create_dir_all(parent).map_err(VfsError::Io)?;
            }
            std::fs::write(&p, content).map_err(VfsError::Io)?;
            return Ok(());
        }
        let abs = self.resolve_to_absolute(path);
        let segments: Vec<&str> = abs.split('/').filter(|s| !s.is_empty()).collect();
        let (parent_segments, file_name) = match segments.split_last() {
            Some((last, rest)) => (rest, *last),
            None => return Err(VfsError::InvalidPath), // path is "/" or empty
        };
        let mut indices: Vec<usize> = vec![];
        for segment in parent_segments {
            let current = Self::get_mut_at(&mut self.root, &indices);
            match current {
                Node::Dir { children, .. } => {
                    let pos = children.iter().position(|c| c.name() == *segment);
                    match pos {
                        Some(i) => {
                            if !children[i].is_dir() {
                                return Err(VfsError::InvalidPath);
                            }
                            indices.push(i);
                        }
                        None => return Err(VfsError::InvalidPath), // parent path does not exist
                    }
                }
                Node::File { .. } => return Err(VfsError::InvalidPath),
            }
        }
        let parent = Self::get_mut_at(&mut self.root, &indices);
        match parent {
            Node::Dir { children, .. } => {
                let pos = children.iter().position(|c| c.name() == file_name);
                let node = Node::File {
                    name: file_name.to_string(),
                    content: content.to_vec(),
                };
                match pos {
                    Some(i) => children[i] = node,
                    None => children.push(node),
                }
                Ok(())
            }
            Node::File { .. } => Err(VfsError::InvalidPath),
        }
    }

    /// Create an empty file at path (touch). Parent directory must exist.
    ///
    /// # Errors
    /// Same as `write_file`.
    pub fn touch(&mut self, path: &str) -> Result<(), VfsError> {
        self.write_file(path, &[])
    }

    /// Read file content at path.
    ///
    /// # Errors
    /// Returns `VfsError::InvalidPath` if path does not exist or is not a file.
    pub fn read_file(&self, path: &str) -> Result<Vec<u8>, VfsError> {
        if self.host_root.is_some() {
            let abs = self.resolve_to_absolute(path);
            let p = self.logical_to_host_path(&abs);
            if !p.is_file() {
                return Err(VfsError::InvalidPath);
            }
            return std::fs::read(&p).map_err(VfsError::Io);
        }
        let abs = self.resolve_to_absolute(path);
        let n = self.resolve_absolute(&abs)?;
        match n {
            Node::File { content, .. } => Ok(content),
            Node::Dir { .. } => Err(VfsError::InvalidPath),
        }
    }

    /// List directory entries at path.
    ///
    /// # Errors
    /// Returns `VfsError::InvalidPath` if path does not exist or is not a directory.
    pub fn list_dir(&self, path: &str) -> Result<Vec<String>, VfsError> {
        if self.host_root.is_some() {
            let abs = self.resolve_to_absolute(path);
            let p = self.logical_to_host_path(&abs);
            if !p.is_dir() {
                return Err(VfsError::InvalidPath);
            }
            let mut out = Vec::new();
            for e in std::fs::read_dir(&p).map_err(VfsError::Io)? {
                let e = e.map_err(VfsError::Io)?;
                out.push(e.file_name().to_string_lossy().into_owned());
            }
            out.sort();
            return Ok(out);
        }
        let abs = self.resolve_to_absolute(path);
        let n = self.resolve_absolute(&abs)?;
        match n {
            Node::Dir { children, .. } => {
                Ok(children.iter().map(|c| c.name().to_string()).collect())
            }
            Node::File { .. } => Err(VfsError::InvalidPath),
        }
    }

    /// Set current working directory to path.
    ///
    /// # Errors
    /// Returns `VfsError::InvalidPath` if path does not exist or is not a directory.
    pub fn set_cwd(&mut self, path: &str) -> Result<(), VfsError> {
        if self.host_root.is_some() {
            let abs = self.resolve_to_absolute(path);
            let p = self.logical_to_host_path(&abs);
            let meta = std::fs::metadata(&p).map_err(|_| VfsError::InvalidPath)?;
            if !meta.is_dir() {
                return Err(VfsError::InvalidPath);
            }
            self.cwd = if abs == "/" { "/".to_string() } else { abs };
            return Ok(());
        }
        let abs = self.resolve_to_absolute(path);
        let n = self.resolve_absolute(&abs)?;
        if !n.is_dir() {
            return Err(VfsError::InvalidPath);
        }
        self.cwd = if abs == "/" { "/".to_string() } else { abs };
        Ok(())
    }

    /// Mutable reference to node at path of child indices from the given node.
    fn get_mut_at<'a>(node: &'a mut Node, path: &[usize]) -> &'a mut Node {
        if path.is_empty() {
            return node;
        }
        match node {
            Node::Dir { children, .. } => {
                let i = path[0];
                Self::get_mut_at(&mut children[i], &path[1..])
            }
            Node::File { .. } => unreachable!("path must follow dirs only"),
        }
    }

    /// Recursively copy the VFS subtree at `vfs_path` to the host directory `host_dir`.
    /// For each Dir creates a directory; for each File writes file content.
    ///
    /// # Errors
    /// Returns `VfsError::InvalidPath` if path does not exist; `VfsError::Io` on host I/O failure.
    pub fn copy_tree_to_host(&self, vfs_path: &str, host_dir: &Path) -> Result<(), VfsError> {
        let abs = self.resolve_to_absolute(vfs_path);
        if self.host_root.is_some() {
            let src = self.logical_to_host_path(&abs);
            copy_host_path_to_host_dir(&src, host_dir)
        } else {
            let node = self.resolve_absolute(&abs)?;
            copy_node_to_host(&node, host_dir)
        }
    }
}
