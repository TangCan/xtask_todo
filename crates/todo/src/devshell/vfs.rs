use std::path::Path;

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
    /// Returns the root for "" or "/", otherwise walks segments; Err(()) if any segment is missing.
    pub fn resolve_absolute(&self, path: &str) -> Result<Node, ()> {
        let path = path.trim_end_matches('/');
        if path.is_empty() || path == "/" {
            return Ok(self.root.clone());
        }
        let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let mut current = &self.root;
        for segment in segments {
            current = current.child(segment).ok_or(())?;
        }
        Ok(current.clone())
    }

    /// 将任意路径（相对或绝对）归一化并解析为绝对路径字符串
    #[must_use]
    pub fn resolve_to_absolute(&self, path: &str) -> String {
        let path = normalize_path(path);
        if path.starts_with('/') && path != "/" {
            return path;
        }
        if path == "/" {
            return self.cwd.clone();
        }
        let base = self.cwd.trim_end_matches('/');
        let p = path.trim_start_matches('/');
        let result = normalize_path(&format!("{base}/{p}"));
        if result.starts_with('/') {
            result
        } else {
            format!("/{result}")
        }
    }

    /// Create directory at path (`mkdir_all` style). Creates any missing parent directories.
    /// Returns Err(()) if any path component exists and is not a directory.
    pub fn mkdir(&mut self, path: &str) -> Result<(), ()> {
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
                            return Err(());
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
                Node::File { .. } => return Err(()),
            }
        }
        Ok(())
    }

    /// Create or overwrite a file at path. Parent directory must exist and be a directory.
    pub fn write_file(&mut self, path: &str, content: &[u8]) -> Result<(), ()> {
        let abs = self.resolve_to_absolute(path);
        let segments: Vec<&str> = abs.split('/').filter(|s| !s.is_empty()).collect();
        let (parent_segments, file_name) = match segments.split_last() {
            Some((last, rest)) => (rest, *last),
            None => return Err(()), // path is "/" or empty
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
                                return Err(());
                            }
                            indices.push(i);
                        }
                        None => return Err(()), // parent path does not exist
                    }
                }
                Node::File { .. } => return Err(()),
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
            Node::File { .. } => Err(()),
        }
    }

    /// Create an empty file at path (touch). Parent directory must exist.
    pub fn touch(&mut self, path: &str) -> Result<(), ()> {
        self.write_file(path, &[])
    }

    pub fn read_file(&self, path: &str) -> Result<Vec<u8>, ()> {
        let abs = self.resolve_to_absolute(path);
        let n = self.resolve_absolute(&abs)?;
        match n {
            Node::File { content, .. } => Ok(content),
            Node::Dir { .. } => Err(()),
        }
    }

    pub fn list_dir(&self, path: &str) -> Result<Vec<String>, ()> {
        let abs = self.resolve_to_absolute(path);
        let n = self.resolve_absolute(&abs)?;
        match n {
            Node::Dir { children, .. } => {
                Ok(children.iter().map(|c| c.name().to_string()).collect())
            }
            Node::File { .. } => Err(()),
        }
    }

    pub fn set_cwd(&mut self, path: &str) -> Result<(), ()> {
        let abs = self.resolve_to_absolute(path);
        let n = self.resolve_absolute(&abs)?;
        if !n.is_dir() {
            return Err(());
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
    /// Paths are safe: no ".." or other components can escape the export root.
    pub fn copy_tree_to_host(&self, vfs_path: &str, host_dir: &Path) -> Result<(), ()> {
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
fn copy_node_to_host(node: &Node, host_path: &Path) -> Result<(), ()> {
    match node {
        Node::Dir { name, children } => {
            let dir_path = if name.is_empty() {
                host_path.to_path_buf()
            } else {
                if !is_safe_component(name) {
                    return Err(());
                }
                host_path.join(name)
            };
            std::fs::create_dir_all(&dir_path).map_err(|_| ())?;
            for child in children {
                copy_node_to_host(child, &dir_path)?;
            }
            Ok(())
        }
        Node::File { name, content } => {
            if !is_safe_component(name) {
                return Err(());
            }
            let file_path = host_path.join(name);
            std::fs::write(&file_path, content).map_err(|_| ())?;
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
mod tests {
    use super::*;

    #[test]
    fn vfs_new_cwd_root() {
        let vfs = Vfs::new();
        assert_eq!(vfs.cwd(), "/");
    }

    #[test]
    fn vfs_mkdir_and_list() {
        let mut vfs = Vfs::new();
        vfs.mkdir("/foo").unwrap();
        vfs.mkdir("/foo/bar").unwrap();
        assert_eq!(vfs.list_dir("/").unwrap(), vec!["foo"]);
        assert_eq!(vfs.list_dir("/foo").unwrap(), vec!["bar"]);
    }

    #[test]
    fn vfs_write_and_read_file() {
        let mut vfs = Vfs::new();
        vfs.mkdir("/dir").unwrap();
        vfs.write_file("/dir/f", b"hello").unwrap();
        assert_eq!(vfs.read_file("/dir/f").unwrap(), b"hello");
    }

    #[test]
    fn vfs_set_cwd() {
        let mut vfs = Vfs::new();
        vfs.mkdir("/a").unwrap();
        vfs.set_cwd("/a").unwrap();
        assert_eq!(vfs.cwd(), "/a");
    }

    #[test]
    fn vfs_touch() {
        let mut vfs = Vfs::new();
        vfs.mkdir("/d").unwrap();
        vfs.touch("/d/empty").unwrap();
        assert_eq!(vfs.read_file("/d/empty").unwrap(), b"");
    }

    #[test]
    fn normalize_path_dot_dot() {
        assert_eq!(normalize_path("/a/b/.."), "/a");
        assert_eq!(normalize_path("a/../b"), "b");
    }

    #[test]
    fn vfs_copy_tree_to_host() {
        let mut vfs = Vfs::new();
        vfs.mkdir("/sub").unwrap();
        vfs.write_file("/sub/f.txt", b"data").unwrap();
        let dir = std::env::temp_dir().join(format!("vfs_export_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        vfs.copy_tree_to_host("/sub", &dir).unwrap();
        let sub_dir = dir.join("sub");
        let content = std::fs::read(sub_dir.join("f.txt")).unwrap();
        assert_eq!(content, b"data");
        let _ = std::fs::remove_file(sub_dir.join("f.txt"));
        let _ = std::fs::remove_dir(sub_dir);
        let _ = std::fs::remove_dir(dir);
    }

    #[test]
    fn node_methods() {
        let dir = Node::Dir {
            name: "d".into(),
            children: vec![Node::File {
                name: "f".into(),
                content: vec![1, 2, 3],
            }],
        };
        assert_eq!(dir.name(), "d");
        assert!(dir.is_dir());
        assert!(!dir.is_file());
        let f = dir.child("f").unwrap();
        assert_eq!(f.name(), "f");
        assert!(f.is_file());
        assert!(!f.is_dir());
        assert!(dir.child("x").is_none());
        assert!(f.child("x").is_none());
    }

    #[test]
    fn resolve_to_absolute_relative() {
        let mut vfs = Vfs::new();
        vfs.mkdir("/a").unwrap();
        vfs.set_cwd("/a").unwrap();
        let abs = vfs.resolve_to_absolute("b");
        assert!(abs.contains('a'));
        assert!(abs.ends_with('b') || abs.contains('b'));
    }

    #[test]
    fn normalize_path_windows_drive() {
        let out = normalize_path("C:\\foo\\bar");
        assert!(out.contains("foo"));
        assert!(out.contains("bar"));
        assert!(out.starts_with('/') || out == "foo/bar");
    }

    #[test]
    fn mkdir_when_component_is_file_returns_err() {
        let mut vfs = Vfs::new();
        vfs.write_file("/f", b"").unwrap();
        assert!(vfs.mkdir("/f/sub").is_err());
    }

    #[test]
    fn write_file_when_parent_is_file_returns_err() {
        let mut vfs = Vfs::new();
        vfs.write_file("/f", b"").unwrap();
        assert!(vfs.write_file("/f/child", b"x").is_err());
    }

    #[test]
    fn write_file_when_parent_does_not_exist_returns_err() {
        let mut vfs = Vfs::new();
        vfs.mkdir("/d").unwrap();
        assert!(vfs.write_file("/d/nonexistent/sub", b"x").is_err());
    }

    #[test]
    fn write_file_overwrite_existing() {
        let mut vfs = Vfs::new();
        vfs.mkdir("/d").unwrap();
        vfs.write_file("/d/f", b"first").unwrap();
        vfs.write_file("/d/f", b"second").unwrap();
        assert_eq!(vfs.read_file("/d/f").unwrap(), b"second");
    }

    #[test]
    fn read_file_on_dir_errors() {
        let mut vfs = Vfs::new();
        vfs.mkdir("/d").unwrap();
        assert!(vfs.read_file("/d").is_err());
    }

    #[test]
    fn list_dir_on_file_errors() {
        let mut vfs = Vfs::new();
        vfs.mkdir("/d").unwrap();
        vfs.write_file("/d/f", b"x").unwrap();
        assert!(vfs.list_dir("/d/f").is_err());
    }

    #[test]
    fn set_cwd_to_file_errors() {
        let mut vfs = Vfs::new();
        vfs.mkdir("/d").unwrap();
        vfs.write_file("/d/f", b"x").unwrap();
        assert!(vfs.set_cwd("/d/f").is_err());
    }
}
