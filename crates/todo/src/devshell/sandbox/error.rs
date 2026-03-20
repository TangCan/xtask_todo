//! `SandboxError` for export/sync failures.

/// Errors from sandbox export/sync.
#[derive(Debug)]
pub enum SandboxError {
    /// Failed to create temp dir or set permissions.
    ExportFailed(std::io::Error),
    /// VFS copy to host failed.
    CopyFailed(super::super::vfs::VfsError),
    /// Sync from host back to VFS failed.
    SyncBackFailed(std::io::Error),
}

impl std::fmt::Display for SandboxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExportFailed(e) => write!(f, "export failed: {e}"),
            Self::CopyFailed(e) => write!(f, "copy to host failed: {e}"),
            Self::SyncBackFailed(e) => write!(f, "sync back failed: {e}"),
        }
    }
}

impl std::error::Error for SandboxError {}
