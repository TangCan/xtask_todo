//! Virtual filesystem for the devshell.

use std::path::Path;

/// Error from VFS operations (path not found, not a directory/file, or I/O).
#[derive(Debug)]
pub enum VfsError {
    InvalidPath,
    Io(std::io::Error),
}

impl std::fmt::Display for VfsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPath => f.write_str("invalid path"),
            Self::Io(e) => write!(f, "io: {e}"),
        }
    }
}

impl std::error::Error for VfsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::InvalidPath => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Dir { name: String, children: Vec<Self> },
    File { name: String, content: Vec<u8> },
}

impl Node {
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Dir { name, .. } | Self::File { name, .. } => name,
        }
    }
    #[must_use]
    pub const fn is_dir(&self) -> bool {
        matches!(self, Self::Dir { .. })
    }
    #[must_use]
    pub const fn is_file(&self) -> bool {
        matches!(self, Self::File { .. })
    }

    /// Returns a reference to the direct child with the given name, if any (Dir only).
    #[must_use]
    pub fn child(&self, name: &str) -> Option<&Self> {
        match self {
            Self::Dir { children, .. } => children.iter().find(|c| c.name() == name),
            Self::File { .. } => None,
        }
    }
}

pub struct Vfs {
    root: Node,
    cwd: String,
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
        }
    }

    /// Construct VFS from root node and cwd (used by deserialization).
    #[must_use]
    pub const fn from_parts(root: Node, cwd: String) -> Self {
        Self { root, cwd }
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
        let path = path.trim();
        let path_normalized = normalize_path(path);
        // 绝对路径：直接归一化后返回
        if path_normalized.starts_with('/') {
            return path_normalized;
        }
        if path_normalized == "/" {
            return self.cwd.clone();
        }
        // 相对路径：先与 cwd 拼接再归一化，避免单独 ".." 被归一成 "." 导致无法退回根目录
        let base = self.cwd.trim_end_matches('/');
        let p = path.trim_start_matches('/');
        let combined = if base.is_empty() {
            format!("/{p}")
        } else {
            format!("{base}/{p}")
        };
        let result = normalize_path(&combined);
        if result.is_empty() || result == "." {
            "/".to_string()
        } else if result.starts_with('/') {
            result
        } else {
            format!("/{result}")
        }
    }

    /// Create directory at path (`mkdir_all` style). Creates any missing parent directories.
    ///
    /// # Errors
    /// Returns `VfsError::InvalidPath` if any path component exists and is not a directory.
    pub fn mkdir(&mut self, path: &str) -> Result<(), VfsError> {
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
        let node = self.resolve_absolute(&abs)?;
        copy_node_to_host(&node, host_dir)
    }
}

/// Returns true if the name is safe to use as a single path component (no .. or separators).
fn is_safe_component(name: &str) -> bool {
    !name.is_empty() && name != "." && name != ".." && !name.contains('/') && !name.contains('\\')
}

/// Recursively copy a VFS node to the host path. Creates dirs and writes file contents.
fn copy_node_to_host(node: &Node, host_path: &Path) -> Result<(), VfsError> {
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

/// Normalize a path to Unix style: backslash -> slash, strip Windows drive,
/// resolve . and .., preserve absolute vs relative.
#[must_use]
pub fn normalize_path(input: &str) -> String {
    let s = input.replace('\\', "/");

    // Strip Windows drive letter prefix (e.g. C:) and treat as absolute.
    let (rest, absolute) = if s.len() >= 2
        && s.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
        && s.chars().nth(1) == Some(':')
    {
        (&s[2..], true)
    } else {
        (s.as_str(), s.starts_with('/'))
    };

    let rest = rest.trim_start_matches('/');
    let mut out: Vec<&str> = Vec::new();
    for p in rest.split('/') {
        match p {
            "" | "." => {}
            ".." => {
                out.pop();
            }
            _ => out.push(p),
        }
    }

    if absolute {
        "/".to_string() + &out.join("/")
    } else if out.is_empty() {
        ".".to_string()
    } else {
        out.join("/")
    }
}

#[cfg(test)]
mod tests;
