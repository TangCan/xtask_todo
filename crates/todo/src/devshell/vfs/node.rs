//! In-memory VFS tree nodes.

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
