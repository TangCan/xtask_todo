//! Shared workspace file reads: Mode P γ ([`GuestFsOps`]) vs Mode S ([`Vfs`]).
//!
//! Used by [`crate::devshell::command::dispatch`] and script / REPL `source` loading (design §9).

#![allow(clippy::pedantic, clippy::nursery)]

use std::cell::RefCell;
use std::rc::Rc;

use crate::devshell::vfs::{Vfs, VfsError};
#[cfg(any(unix, feature = "beta-vm"))]
use crate::devshell::vm::GuestFsOps;
use crate::devshell::vm::{GuestFsError, SessionHolder};

#[cfg(unix)]
use crate::devshell::vm::GammaSession;
#[cfg(any(unix, feature = "beta-vm"))]
use crate::devshell::workspace::logical_path_to_guest;
use crate::devshell::workspace::WorkspaceBackendError;

/// Failure reading a logical path from the active workspace (guest or VFS).
#[derive(Debug)]
pub enum WorkspaceReadError {
    Vfs(VfsError),
    Guest(GuestFsError),
    PathOutsideWorkspace,
    Backend(WorkspaceBackendError),
}

impl std::fmt::Display for WorkspaceReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Vfs(e) => write!(f, "{e}"),
            Self::Guest(e) => write!(f, "{e}"),
            Self::PathOutsideWorkspace => f.write_str("path outside workspace"),
            Self::Backend(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for WorkspaceReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Vfs(e) => Some(e),
            Self::Guest(e) => Some(e),
            Self::Backend(e) => Some(e),
            Self::PathOutsideWorkspace => None,
        }
    }
}

/// Map a logical path to a guest absolute path (γ guest-primary).
#[cfg(unix)]
pub fn logical_to_guest_abs(
    vfs: &Vfs,
    g: &GammaSession,
    logical_path: &str,
) -> Result<String, WorkspaceBackendError> {
    logical_path_to_guest(g.guest_mount(), vfs.cwd(), logical_path)
}

/// Read file bytes for a logical path: guest-primary (γ or β) first, else in-memory [`Vfs`].
pub fn read_logical_file_bytes(
    vfs: &mut Vfs,
    vm_session: &mut SessionHolder,
    path: &str,
) -> Result<Vec<u8>, WorkspaceReadError> {
    #[cfg(any(unix, feature = "beta-vm"))]
    {
        if let Some((ops, mount)) = vm_session.guest_primary_fs_ops_mut() {
            let gp = match logical_path_to_guest(&mount, vfs.cwd(), path) {
                Ok(p) => p,
                Err(WorkspaceBackendError::PathOutsideWorkspace) => {
                    return Err(WorkspaceReadError::PathOutsideWorkspace);
                }
                Err(e) => return Err(WorkspaceReadError::Backend(e)),
            };
            return GuestFsOps::read_file(ops, &gp).map_err(WorkspaceReadError::Guest);
        }
    }
    #[cfg(not(any(unix, feature = "beta-vm")))]
    let _ = vm_session;
    vfs.read_file(path).map_err(WorkspaceReadError::Vfs)
}

/// Same as [`read_logical_file_bytes`] with shared [`Rc`] handles (script / REPL).
pub fn read_logical_file_bytes_rc(
    vfs: &Rc<RefCell<Vfs>>,
    vm_session: &Rc<RefCell<SessionHolder>>,
    path: &str,
) -> Result<Vec<u8>, WorkspaceReadError> {
    read_logical_file_bytes(&mut vfs.borrow_mut(), &mut vm_session.borrow_mut(), path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::devshell::vfs::Vfs;

    #[test]
    fn read_host_session_uses_vfs() {
        let vfs = Rc::new(RefCell::new(Vfs::new()));
        vfs.borrow_mut().mkdir("/a").unwrap();
        vfs.borrow_mut().write_file("/a/x", b"hi").unwrap();
        let vm = Rc::new(RefCell::new(SessionHolder::new_host()));
        let got = read_logical_file_bytes_rc(&vfs, &vm, "/a/x").unwrap();
        assert_eq!(got, b"hi");
    }
}
