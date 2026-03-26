//! [`WorkspaceBackend`] — Mode S ([`MemoryVfsBackend`]) vs Mode P ([`GuestPrimaryBackend`] skeleton).
//!
//! See `docs/superpowers/specs/2026-03-20-devshell-guest-primary-design.md` §4.

use std::cell::RefCell;
use std::process::ExitStatus;
use std::rc::Rc;

use crate::devshell::vfs::{resolve_path_with_cwd, Vfs, VfsError};
use crate::devshell::vm::{
    guest_path_is_under_mount, guest_project_dir_on_guest, normalize_guest_path, GuestFsError,
    GuestFsOps, SessionHolder,
};

/// Unified error for workspace operations (until dispatch is wired).
#[derive(Debug)]
pub enum WorkspaceBackendError {
    Vfs(VfsError),
    Guest(GuestFsError),
    Vm(crate::devshell::vm::VmError),
    /// Logical path is not under the current logical cwd project subtree.
    PathOutsideWorkspace,
    /// [`MemoryVfsBackend`] does not map to guest paths.
    ModeSOnly,
    /// Not yet implemented (Sprint 3+).
    Unsupported(&'static str),
}

impl std::fmt::Display for WorkspaceBackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Vfs(e) => write!(f, "{e}"),
            Self::Guest(e) => write!(f, "{e}"),
            Self::Vm(e) => write!(f, "{e}"),
            Self::PathOutsideWorkspace => f.write_str("path outside workspace cwd"),
            Self::ModeSOnly => f.write_str("guest path resolution not available in Mode S"),
            Self::Unsupported(msg) => write!(f, "unsupported: {msg}"),
        }
    }
}

impl std::error::Error for WorkspaceBackendError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Vfs(e) => Some(e),
            Self::Guest(e) => Some(e),
            Self::Vm(e) => Some(e),
            _ => None,
        }
    }
}

impl From<VfsError> for WorkspaceBackendError {
    fn from(e: VfsError) -> Self {
        Self::Vfs(e)
    }
}

impl From<GuestFsError> for WorkspaceBackendError {
    fn from(e: GuestFsError) -> Self {
        Self::Guest(e)
    }
}

impl From<crate::devshell::vm::VmError> for WorkspaceBackendError {
    fn from(e: crate::devshell::vm::VmError) -> Self {
        Self::Vm(e)
    }
}

/// Map a logical absolute path to a guest path (same layout as γ push: project root = `guest_project_dir_on_guest`).
///
/// # Errors
/// [`WorkspaceBackendError::PathOutsideWorkspace`] if `logical_path` is not under the project root
/// implied by `logical_cwd` (after normalization).
pub fn logical_path_to_guest(
    guest_mount: &str,
    logical_cwd: &str,
    logical_path: &str,
) -> Result<String, WorkspaceBackendError> {
    let abs_cwd = resolve_path_with_cwd("/", logical_cwd);
    let abs_path = resolve_path_with_cwd(logical_cwd, logical_path);
    let prefix = if abs_cwd.ends_with('/') {
        abs_cwd.clone()
    } else {
        format!("{abs_cwd}/")
    };
    if abs_path != abs_cwd && !abs_path.starts_with(&prefix) {
        return Err(WorkspaceBackendError::PathOutsideWorkspace);
    }
    let rel = if abs_path == abs_cwd {
        ""
    } else {
        &abs_path[prefix.len()..]
    };
    let guest_root = guest_project_dir_on_guest(guest_mount, logical_cwd);
    if !guest_path_is_under_mount(guest_mount, &guest_root) {
        return Err(WorkspaceBackendError::PathOutsideWorkspace);
    }
    let guest_path = if rel.is_empty() {
        guest_root
    } else {
        format!("{guest_root}/{rel}")
    };
    let guest_path =
        normalize_guest_path(&guest_path).ok_or(WorkspaceBackendError::PathOutsideWorkspace)?;
    if !guest_path_is_under_mount(guest_mount, &guest_path) {
        return Err(WorkspaceBackendError::PathOutsideWorkspace);
    }
    Ok(guest_path)
}

/// Virtual workspace for devshell: Mode S (memory) or Mode P (guest-primary).
pub trait WorkspaceBackend {
    fn logical_cwd(&self) -> String;
    /// # Errors
    /// Returns [`WorkspaceBackendError`] when path normalization or backend cwd updates fail.
    fn set_logical_cwd(&mut self, path: &str) -> Result<(), WorkspaceBackendError>;
    /// # Errors
    /// Returns [`WorkspaceBackendError`] when the backend cannot read `path`.
    fn read_file(&mut self, path: &str) -> Result<Vec<u8>, WorkspaceBackendError>;
    /// # Errors
    /// Returns [`WorkspaceBackendError`] when the backend cannot write `path`.
    fn write_file(&mut self, path: &str, data: &[u8]) -> Result<(), WorkspaceBackendError>;
    /// # Errors
    /// Returns [`WorkspaceBackendError`] when directory listing fails.
    fn list_dir(&mut self, path: &str) -> Result<Vec<String>, WorkspaceBackendError>;
    /// # Errors
    /// Returns [`WorkspaceBackendError`] when directory creation fails.
    fn mkdir(&mut self, path: &str) -> Result<(), WorkspaceBackendError>;
    /// # Errors
    /// Returns [`WorkspaceBackendError`] when remove is unsupported or fails.
    fn remove(&mut self, path: &str) -> Result<(), WorkspaceBackendError>;
    /// # Errors
    /// Returns [`WorkspaceBackendError`] when existence checks fail unexpectedly.
    fn exists(&mut self, path: &str) -> Result<bool, WorkspaceBackendError>;

    /// Mode P: logical path → guest absolute path. Mode S: [`WorkspaceBackendError::ModeSOnly`].
    ///
    /// # Errors
    /// Returns [`WorkspaceBackendError::ModeSOnly`] in Mode S or mapping failures in Mode P.
    fn try_resolve_guest_path(&self, logical_path: &str) -> Result<String, WorkspaceBackendError>;

    /// Run `rustup` / `cargo` (Mode S: sync VFS↔host/VM). Mode P skeleton: [`WorkspaceBackendError::Unsupported`].
    ///
    /// # Errors
    /// Returns [`WorkspaceBackendError`] when tool execution/sync is unsupported or fails.
    fn run_rust_tool(
        &mut self,
        vm_session: &mut SessionHolder,
        program: &str,
        args: &[String],
    ) -> Result<ExitStatus, WorkspaceBackendError>;
}

/// Mode S: [`Vfs`] in memory (`Rc<RefCell<Vfs>>` matches REPL sharing).
pub struct MemoryVfsBackend {
    vfs: Rc<RefCell<Vfs>>,
}

impl MemoryVfsBackend {
    #[must_use]
    pub const fn new(vfs: Rc<RefCell<Vfs>>) -> Self {
        Self { vfs }
    }
}

impl WorkspaceBackend for MemoryVfsBackend {
    fn logical_cwd(&self) -> String {
        self.vfs.borrow().cwd().to_string()
    }

    fn set_logical_cwd(&mut self, path: &str) -> Result<(), WorkspaceBackendError> {
        self.vfs.borrow_mut().set_cwd(path)?;
        Ok(())
    }

    fn read_file(&mut self, path: &str) -> Result<Vec<u8>, WorkspaceBackendError> {
        Ok(self.vfs.borrow().read_file(path)?)
    }

    fn write_file(&mut self, path: &str, data: &[u8]) -> Result<(), WorkspaceBackendError> {
        self.vfs.borrow_mut().write_file(path, data)?;
        Ok(())
    }

    fn list_dir(&mut self, path: &str) -> Result<Vec<String>, WorkspaceBackendError> {
        Ok(self.vfs.borrow().list_dir(path)?)
    }

    fn mkdir(&mut self, path: &str) -> Result<(), WorkspaceBackendError> {
        self.vfs.borrow_mut().mkdir(path)?;
        Ok(())
    }

    fn remove(&mut self, _path: &str) -> Result<(), WorkspaceBackendError> {
        Err(WorkspaceBackendError::Unsupported(
            "MemoryVfsBackend::remove — add Vfs::remove or use dispatch path",
        ))
    }

    fn exists(&mut self, path: &str) -> Result<bool, WorkspaceBackendError> {
        let vfs = self.vfs.borrow();
        let abs = resolve_path_with_cwd(vfs.cwd(), path);
        Ok(vfs.resolve_absolute(&abs).is_ok())
    }

    fn try_resolve_guest_path(&self, _logical_path: &str) -> Result<String, WorkspaceBackendError> {
        Err(WorkspaceBackendError::ModeSOnly)
    }

    fn run_rust_tool(
        &mut self,
        vm_session: &mut SessionHolder,
        program: &str,
        args: &[String],
    ) -> Result<ExitStatus, WorkspaceBackendError> {
        let vfs_cwd = self.vfs.borrow().cwd().to_string();
        let mut vfs = self.vfs.borrow_mut();
        Ok(vm_session.run_rust_tool(&mut vfs, &vfs_cwd, program, args)?)
    }
}

/// Mode P skeleton: [`GuestFsOps`] + logical cwd + guest mount (see design §5).
pub struct GuestPrimaryBackend {
    ops: Box<dyn GuestFsOps>,
    guest_mount: String,
    logical_cwd: String,
}

impl GuestPrimaryBackend {
    #[must_use]
    pub fn new(guest_mount: String, logical_cwd: String, ops: Box<dyn GuestFsOps>) -> Self {
        Self {
            ops,
            guest_mount,
            logical_cwd,
        }
    }

    #[must_use]
    pub fn guest_mount(&self) -> &str {
        &self.guest_mount
    }
}

impl WorkspaceBackend for GuestPrimaryBackend {
    fn logical_cwd(&self) -> String {
        self.logical_cwd.clone()
    }

    fn set_logical_cwd(&mut self, path: &str) -> Result<(), WorkspaceBackendError> {
        self.logical_cwd = resolve_path_with_cwd(&self.logical_cwd, path);
        Ok(())
    }

    fn read_file(&mut self, path: &str) -> Result<Vec<u8>, WorkspaceBackendError> {
        let g = logical_path_to_guest(&self.guest_mount, &self.logical_cwd, path)?;
        Ok(self.ops.read_file(&g)?)
    }

    fn write_file(&mut self, path: &str, data: &[u8]) -> Result<(), WorkspaceBackendError> {
        let g = logical_path_to_guest(&self.guest_mount, &self.logical_cwd, path)?;
        Ok(self.ops.write_file(&g, data)?)
    }

    fn list_dir(&mut self, path: &str) -> Result<Vec<String>, WorkspaceBackendError> {
        let g = logical_path_to_guest(&self.guest_mount, &self.logical_cwd, path)?;
        Ok(self.ops.list_dir(&g)?)
    }

    fn mkdir(&mut self, path: &str) -> Result<(), WorkspaceBackendError> {
        let g = logical_path_to_guest(&self.guest_mount, &self.logical_cwd, path)?;
        Ok(self.ops.mkdir(&g)?)
    }

    fn remove(&mut self, path: &str) -> Result<(), WorkspaceBackendError> {
        let g = logical_path_to_guest(&self.guest_mount, &self.logical_cwd, path)?;
        Ok(self.ops.remove(&g)?)
    }

    fn exists(&mut self, path: &str) -> Result<bool, WorkspaceBackendError> {
        let g = logical_path_to_guest(&self.guest_mount, &self.logical_cwd, path)?;
        match self.ops.read_file(&g) {
            Ok(_) | Err(GuestFsError::IsADirectory(_)) => Ok(true),
            Err(GuestFsError::NotFound(_)) => match self.ops.list_dir(&g) {
                Ok(_) => Ok(true),
                Err(GuestFsError::NotFound(_) | GuestFsError::NotADirectory(_)) => Ok(false),
                Err(e) => Err(WorkspaceBackendError::Guest(e)),
            },
            Err(e) => Err(WorkspaceBackendError::Guest(e)),
        }
    }

    fn try_resolve_guest_path(&self, logical_path: &str) -> Result<String, WorkspaceBackendError> {
        logical_path_to_guest(&self.guest_mount, &self.logical_cwd, logical_path)
    }

    fn run_rust_tool(
        &mut self,
        _vm_session: &mut SessionHolder,
        _program: &str,
        _args: &[String],
    ) -> Result<ExitStatus, WorkspaceBackendError> {
        Err(WorkspaceBackendError::Unsupported(
            "GuestPrimaryBackend::run_rust_tool (Sprint 3: push/pull skip)",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::devshell::vm::MockGuestFsOps;

    #[test]
    fn logical_path_to_guest_under_cwd() {
        let g = logical_path_to_guest("/workspace", "/projects/hello", "/projects/hello/src/a.rs")
            .unwrap();
        assert_eq!(g, "/workspace/hello/src/a.rs");
    }

    #[test]
    fn logical_path_to_guest_rejects_escape() {
        assert!(logical_path_to_guest("/workspace", "/projects/hello", "/etc/passwd").is_err());
    }

    #[test]
    fn memory_backend_roundtrip() {
        let vfs = Rc::new(RefCell::new(Vfs::new()));
        let mut b = MemoryVfsBackend::new(Rc::clone(&vfs));
        b.mkdir("/a").unwrap();
        b.write_file("/a/f", b"x").unwrap();
        assert_eq!(b.read_file("/a/f").unwrap(), b"x");
        assert!(b.exists("/a/f").unwrap());
        assert!(b.try_resolve_guest_path("/a/f").is_err());
    }

    #[test]
    fn guest_primary_backend_mock_resolves_and_writes() {
        let mut b = GuestPrimaryBackend::new(
            "/workspace".to_string(),
            "/projects/foo".to_string(),
            Box::new(MockGuestFsOps::new()),
        );
        b.write_file("/projects/foo/x.txt", b"hi").unwrap();
        assert_eq!(b.read_file("/projects/foo/x.txt").unwrap(), b"hi");
        assert_eq!(
            b.try_resolve_guest_path("/projects/foo/x.txt").unwrap(),
            "/workspace/foo/x.txt"
        );
    }
}
