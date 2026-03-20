//! VFS error type.

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
