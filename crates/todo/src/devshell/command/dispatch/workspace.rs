//! Workspace / guest-primary path mapping for builtins.

#[cfg(any(unix, feature = "beta-vm"))]
use crate::devshell::workspace::logical_path_to_guest;
use crate::devshell::workspace::WorkspaceBackendError;

use crate::devshell::workspace::read_logical_file_bytes;
use crate::devshell::workspace::WorkspaceReadError;

use super::super::super::vfs::Vfs;
#[cfg(any(unix, feature = "beta-vm"))]
use super::super::super::vm::GuestFsOps;
use super::super::super::vm::SessionHolder;
use super::super::types::BuiltinError;

pub(super) fn map_workspace_to_builtin(e: WorkspaceBackendError) -> BuiltinError {
    match e {
        WorkspaceBackendError::PathOutsideWorkspace => BuiltinError::WorkspacePathOutside,
        WorkspaceBackendError::Guest(err) => BuiltinError::GuestFsOpFailed(err.to_string()),
        _ => BuiltinError::GuestFsOpFailed(e.to_string()),
    }
}

pub(super) fn map_workspace_read_err(e: WorkspaceReadError) -> BuiltinError {
    match e {
        WorkspaceReadError::Vfs(_) => BuiltinError::CatFailed,
        WorkspaceReadError::Guest(err) => BuiltinError::GuestFsOpFailed(err.to_string()),
        WorkspaceReadError::PathOutsideWorkspace => BuiltinError::WorkspacePathOutside,
        WorkspaceReadError::Backend(err) => map_workspace_to_builtin(err),
    }
}

pub(super) fn workspace_read_file(
    vfs: &mut Vfs,
    vm_session: &mut SessionHolder,
    path: &str,
) -> Result<Vec<u8>, BuiltinError> {
    read_logical_file_bytes(vfs, vm_session, path).map_err(map_workspace_read_err)
}

pub(super) fn workspace_write_file(
    vfs: &mut Vfs,
    vm_session: &mut SessionHolder,
    path: &str,
    data: &[u8],
) -> Result<(), BuiltinError> {
    #[cfg(not(any(unix, feature = "beta-vm")))]
    let _ = vm_session;
    #[cfg(any(unix, feature = "beta-vm"))]
    if let Some((ops, mount)) = vm_session.guest_primary_fs_ops_mut() {
        let gp =
            logical_path_to_guest(&mount, vfs.cwd(), path).map_err(map_workspace_to_builtin)?;
        return GuestFsOps::write_file(ops, &gp, data)
            .map_err(|e| BuiltinError::GuestFsOpFailed(e.to_string()));
    }
    vfs.write_file(path, data)
        .map_err(|_| BuiltinError::RedirectWrite)
}

pub(super) fn workspace_list_dir(
    vfs: &Vfs,
    vm_session: &mut SessionHolder,
    path: &str,
) -> Result<Vec<String>, BuiltinError> {
    #[cfg(not(any(unix, feature = "beta-vm")))]
    let _ = vm_session;
    #[cfg(any(unix, feature = "beta-vm"))]
    if let Some((ops, mount)) = vm_session.guest_primary_fs_ops_mut() {
        let gp =
            logical_path_to_guest(&mount, vfs.cwd(), path).map_err(map_workspace_to_builtin)?;
        return GuestFsOps::list_dir(ops, &gp)
            .map_err(|e| BuiltinError::GuestFsOpFailed(e.to_string()));
    }
    vfs.list_dir(path).map_err(|_| BuiltinError::LsFailed)
}

pub(super) fn workspace_mkdir(
    vfs: &mut Vfs,
    vm_session: &mut SessionHolder,
    path: &str,
) -> Result<(), BuiltinError> {
    #[cfg(not(any(unix, feature = "beta-vm")))]
    let _ = vm_session;
    #[cfg(any(unix, feature = "beta-vm"))]
    if let Some((ops, mount)) = vm_session.guest_primary_fs_ops_mut() {
        let gp =
            logical_path_to_guest(&mount, vfs.cwd(), path).map_err(map_workspace_to_builtin)?;
        return GuestFsOps::mkdir(ops, &gp)
            .map_err(|e| BuiltinError::GuestFsOpFailed(e.to_string()));
    }
    vfs.mkdir(path).map_err(|_| BuiltinError::MkdirFailed)
}

pub(super) fn workspace_touch(
    vfs: &mut Vfs,
    vm_session: &mut SessionHolder,
    path: &str,
) -> Result<(), BuiltinError> {
    #[cfg(not(any(unix, feature = "beta-vm")))]
    let _ = vm_session;
    #[cfg(any(unix, feature = "beta-vm"))]
    if let Some((ops, mount)) = vm_session.guest_primary_fs_ops_mut() {
        let gp =
            logical_path_to_guest(&mount, vfs.cwd(), path).map_err(map_workspace_to_builtin)?;
        return GuestFsOps::write_file(ops, &gp, &[])
            .map_err(|e| BuiltinError::GuestFsOpFailed(e.to_string()));
    }
    vfs.touch(path).map_err(|_| BuiltinError::TouchFailed)
}
