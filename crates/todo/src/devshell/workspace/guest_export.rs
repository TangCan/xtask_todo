//! Guest → in-memory VFS copy for `export-readonly` in guest-primary mode (design §8.1).
//!
//! Does **not** export to a host temp directory; produces a read-only **mirror** under a dedicated
//! logical path in the process VFS so the user can `ls` / `cat` without leaving the virtual workspace.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::devshell::vfs::{Vfs, VfsError};
use crate::devshell::vm::{GuestFsError, GuestFsOps, SessionHolder};
use crate::devshell::workspace::logical_path_to_guest;
use crate::devshell::workspace::WorkspaceBackendError;

const MAX_DEPTH: u32 = 64;
const MAX_ENTRIES: usize = 5000;

fn join_guest(parent: &str, name: &str) -> String {
    let p = parent.trim_end_matches('/');
    format!("{p}/{name}")
}

fn vfs_parent(path: &str) -> Option<String> {
    let t = path.trim_end_matches('/');
    if t.is_empty() || t == "/" {
        return None;
    }
    let idx = t.rfind('/')?;
    if idx == 0 {
        Some("/".to_string())
    } else {
        Some(t[..idx].to_string())
    }
}

fn vfs_mkdir_all(vfs: &mut Vfs, abs: &str) -> Result<(), VfsError> {
    vfs.mkdir(abs)
}

fn copy_guest_entry(
    ops: &mut dyn GuestFsOps,
    guest_path: &str,
    vfs: &mut Vfs,
    vfs_dest: &str,
    depth: u32,
    budget: &mut usize,
) -> Result<(), GuestFsError> {
    if depth > MAX_DEPTH {
        return Err(GuestFsError::Internal(
            "export-readonly: depth limit".into(),
        ));
    }
    if *budget == 0 {
        return Err(GuestFsError::Internal(
            "export-readonly: entry limit".into(),
        ));
    }

    match ops.list_dir(guest_path) {
        Ok(names) => {
            *budget -= 1;
            vfs_mkdir_all(vfs, vfs_dest).map_err(|e| GuestFsError::Internal(e.to_string()))?;
            for n in names {
                if *budget == 0 {
                    return Err(GuestFsError::Internal(
                        "export-readonly: entry limit".into(),
                    ));
                }
                let g = join_guest(guest_path, &n);
                let v = join_guest(vfs_dest, &n);
                copy_guest_entry(ops, &g, vfs, &v, depth + 1, budget)?;
            }
            Ok(())
        }
        Err(e1) => {
            *budget -= 1;
            match ops.read_file(guest_path) {
                Ok(bytes) => {
                    if let Some(parent) = vfs_parent(vfs_dest) {
                        vfs_mkdir_all(vfs, &parent).map_err(|e| {
                            GuestFsError::Internal(format!("export mkdir parent: {e}"))
                        })?;
                    }
                    vfs.write_file(vfs_dest, &bytes)
                        .map_err(|e| GuestFsError::Internal(format!("export vfs write: {e}")))?;
                    Ok(())
                }
                Err(_) => Err(e1),
            }
        }
    }
}

/// Copy the logical subtree at `logical_src` from the guest into a fresh directory under
/// `/.__export_ro_<ms>/…` and return that **absolute logical path** (printed by `export-readonly`).
///
/// # Errors
/// Not guest-primary, path mapping failure, or guest/VFS errors.
pub fn guest_export_readonly_to_vfs(
    vfs: &mut Vfs,
    vm_session: &mut SessionHolder,
    logical_src: &str,
) -> Result<String, GuestFsError> {
    let Some((ops, mount)) = vm_session.guest_primary_fs_ops_mut() else {
        return Err(GuestFsError::Internal(
            "export-readonly: not in guest-primary mode".into(),
        ));
    };
    let gp = logical_path_to_guest(&mount, vfs.cwd(), logical_src).map_err(|e| match e {
        WorkspaceBackendError::PathOutsideWorkspace => {
            GuestFsError::InvalidPath("path outside workspace".into())
        }
        WorkspaceBackendError::Guest(ge) => ge,
        _ => GuestFsError::InvalidPath(e.to_string()),
    })?;
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis());
    let leaf = gp
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or("export");
    let leaf = if leaf.is_empty() { "export" } else { leaf };
    let vfs_root = format!("/.__export_ro_{ms}/{leaf}");
    let mut budget = MAX_ENTRIES;
    copy_guest_entry(ops, &gp, vfs, &vfs_root, 0, &mut budget)?;
    Ok(vfs_root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::devshell::vm::MockGuestFsOps;

    #[test]
    fn export_copies_file_to_vfs() {
        let mut ops = MockGuestFsOps::new();
        ops.mkdir("/workspace/p").unwrap();
        ops.write_file("/workspace/p/a.txt", b"hi").unwrap();

        let mut vfs = Vfs::new();
        let mut budget = 100;
        copy_guest_entry(
            &mut ops,
            "/workspace/p/a.txt",
            &mut vfs,
            "/.__export_ro_test/a.txt",
            0,
            &mut budget,
        )
        .unwrap();
        assert_eq!(vfs.read_file("/.__export_ro_test/a.txt").unwrap(), b"hi");
    }

    #[test]
    fn export_copies_dir_tree() {
        let mut ops = MockGuestFsOps::new();
        ops.mkdir("/workspace/d").unwrap();
        ops.write_file("/workspace/d/x", b"z").unwrap();

        let mut vfs = Vfs::new();
        let mut budget = 100;
        copy_guest_entry(
            &mut ops,
            "/workspace/d",
            &mut vfs,
            "/.__export_ro_t/d",
            0,
            &mut budget,
        )
        .unwrap();
        assert_eq!(vfs.read_file("/.__export_ro_t/d/x").unwrap(), b"z");
    }
}
