//! Path helpers, `limactl` resolution, and guest cwd mapping.

use std::path::PathBuf;

use super::super::super::sandbox;
use super::super::VmError;
use super::env::ENV_DEVSHELL_VM_AUTO_BUILD_ESSENTIAL;

pub(super) fn truthy_env(key: &str) -> bool {
    std::env::var(key)
        .map(|s| {
            let s = s.trim();
            s == "1" || s.eq_ignore_ascii_case("true") || s.eq_ignore_ascii_case("yes")
        })
        .unwrap_or(false)
}

/// Default `true` when unset; `0`/`false`/`no`/`off` disables auto `build-essential` install in guest.
pub(super) fn auto_build_essential_enabled() -> bool {
    match std::env::var(ENV_DEVSHELL_VM_AUTO_BUILD_ESSENTIAL) {
        Err(_) => true,
        Ok(s) => {
            let s = s.trim();
            if s.is_empty() {
                return true;
            }
            !(s == "0"
                || s.eq_ignore_ascii_case("false")
                || s.eq_ignore_ascii_case("no")
                || s.eq_ignore_ascii_case("off"))
        }
    }
}

pub(super) fn resolve_limactl() -> Result<PathBuf, VmError> {
    use super::env::ENV_DEVSHELL_VM_LIMACTL;
    if let Ok(p) = std::env::var(ENV_DEVSHELL_VM_LIMACTL) {
        let p = p.trim();
        if !p.is_empty() {
            return Ok(PathBuf::from(p));
        }
    }
    sandbox::find_in_path("limactl").ok_or_else(|| {
        VmError::Lima(
            "limactl not found in PATH; install Lima (https://lima-vm.io/) or set DEVSHELL_VM_LIMACTL"
                .to_string(),
        )
    })
}

pub(super) fn sanitize_instance_segment(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Host workspace root shared with β (`session_start.staging_dir`).
///
/// Same directory is mounted at [`super::GammaSession::guest_mount`] in the VM; offline devshell uses this path
/// for host-backed [`crate::devshell::vfs::Vfs`] so it stays unified with the guest tree.
#[must_use]
pub fn workspace_parent_for_instance(instance: &str) -> PathBuf {
    use super::env::ENV_DEVSHELL_VM_WORKSPACE_PARENT;
    if let Ok(p) = std::env::var(ENV_DEVSHELL_VM_WORKSPACE_PARENT) {
        let p = p.trim();
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    let seg = sanitize_instance_segment(instance);
    sandbox::devshell_export_parent_dir()
        .join("vm-workspace")
        .join(seg)
}

pub(super) fn guest_dir_for_vfs_cwd(guest_mount: &str, vfs_cwd: &str) -> String {
    super::super::guest_fs_ops::guest_project_dir_on_guest(guest_mount, vfs_cwd)
}

pub(super) fn guest_dir_for_cwd_inner(guest_mount: &str, vfs_cwd: &str) -> String {
    super::super::guest_fs_ops::guest_project_dir_on_guest(guest_mount, vfs_cwd)
}
