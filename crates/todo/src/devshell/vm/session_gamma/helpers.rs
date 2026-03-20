//! Path helpers, `limactl` resolution, and guest cwd mapping.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

use super::super::super::sandbox;
use super::super::VmError;
use super::env::{
    ENV_DEVSHELL_VM_AUTO_BUILD_ESSENTIAL, ENV_DEVSHELL_VM_AUTO_BUILD_TODO_GUEST,
    ENV_DEVSHELL_VM_AUTO_TODO_PATH, ENV_DEVSHELL_VM_GUEST_HOST_DIR,
    ENV_DEVSHELL_VM_GUEST_TODO_HINT, ENV_DEVSHELL_VM_WORKSPACE_PARENT,
    ENV_DEVSHELL_VM_WORKSPACE_USE_CARGO_ROOT,
};

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

/// Default `true` when unset; `0`/`false`/`no`/`off` disables guest `PATH` for `todo` under mount.
pub(super) fn auto_todo_path_enabled() -> bool {
    match std::env::var(ENV_DEVSHELL_VM_AUTO_TODO_PATH) {
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

/// Default `true` when unset; `0`/`false`/`no`/`off` disables guest `todo` probe + hints before `limactl shell`.
pub(super) fn guest_todo_hint_enabled() -> bool {
    match std::env::var(ENV_DEVSHELL_VM_GUEST_TODO_HINT) {
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

/// `1`/`true`/`yes`: after hints, try `cargo build -p xtask --release --bin todo` in the guest when workspace is under the mount.
pub(super) fn auto_build_todo_guest_enabled() -> bool {
    truthy_env(ENV_DEVSHELL_VM_AUTO_BUILD_TODO_GUEST)
}

/// Shell single-quote `s` for safe embedding in `/bin/sh -c '…'`.
pub(super) fn shell_single_quote_sh(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for c in s.chars() {
        if c == '\'' {
            out.push_str("'\"'\"'");
        } else {
            out.push(c);
        }
    }
    out.push('\'');
    out
}

pub(super) fn cargo_metadata_workspace_and_target(cwd: &Path) -> Result<(PathBuf, PathBuf), ()> {
    let out = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .current_dir(cwd)
        .output()
        .map_err(|_| ())?;
    if !out.status.success() {
        return Err(());
    }
    let v: Value = serde_json::from_slice(&out.stdout).map_err(|_| ())?;
    let wr = v
        .get("workspace_root")
        .and_then(|x| x.as_str())
        .map(PathBuf::from)
        .ok_or(())?;
    let td = v
        .get("target_directory")
        .and_then(|x| x.as_str())
        .map(PathBuf::from)
        .ok_or(())?;
    Ok((wr, td))
}

fn cargo_metadata_target_dir(cwd: &Path) -> Result<PathBuf, ()> {
    Ok(cargo_metadata_workspace_and_target(cwd)?.1)
}

/// If `host_path` lies under `workspace_parent`, return the corresponding guest path under `guest_mount`.
pub(super) fn guest_dir_for_host_path_under_workspace(
    workspace_parent: &Path,
    guest_mount: &str,
    host_path: &Path,
) -> Option<String> {
    let hs = host_path.canonicalize().ok()?;
    let ws = workspace_parent.canonicalize().ok()?;
    let rel = hs.strip_prefix(&ws).ok()?;
    let guest = if rel.as_os_str().is_empty() {
        PathBuf::from(guest_mount)
    } else {
        Path::new(guest_mount).join(rel)
    };
    Some(guest.to_string_lossy().replace('\\', "/"))
}

/// Symlink name under guest `$HOME` for the host project (`host_dir` by default); `None` = disable symlinks.
pub(super) fn guest_host_dir_link_name() -> Option<String> {
    match std::env::var(ENV_DEVSHELL_VM_GUEST_HOST_DIR) {
        Err(_) => Some("host_dir".to_string()),
        Ok(s) => {
            let s = s.trim();
            if s.is_empty()
                || s == "0"
                || s.eq_ignore_ascii_case("false")
                || s.eq_ignore_ascii_case("off")
                || s.eq_ignore_ascii_case("no")
            {
                None
            } else if s
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
            {
                Some(s.to_string())
            } else {
                None
            }
        }
    }
}

/// If `cargo metadata` workspace root lies under `workspace_parent`, return guest path to that workspace (for `cd` + `cargo build`).
pub(super) fn guest_cargo_workspace_dir_for_cwd(
    workspace_parent: &Path,
    guest_mount: &str,
    cwd: &Path,
) -> Option<String> {
    let ws_root = cargo_metadata_workspace_and_target(cwd).ok()?.0;
    let ws = workspace_parent.canonicalize().ok()?;
    let ws_root_canon = ws_root.canonicalize().ok()?;
    let rel = ws_root_canon.strip_prefix(&ws).ok()?;
    Some(
        Path::new(guest_mount)
            .join(rel)
            .to_string_lossy()
            .replace('\\', "/"),
    )
}

/// If host `target/release` (from `cargo metadata` in `cwd`) is under `workspace_parent`, return the
/// guest directory to prepend to `PATH` (e.g. `/workspace/proj/target/release`).
pub(super) fn guest_todo_release_dir_for_cwd(
    workspace_parent: &Path,
    guest_mount: &str,
    cwd: &Path,
) -> Option<String> {
    if !auto_todo_path_enabled() {
        return None;
    }
    let release_dir = cargo_metadata_target_dir(cwd).ok()?.join("release");
    let todo_bin = release_dir.join("todo");
    if !todo_bin.is_file() {
        return None;
    }
    let ws = workspace_parent.canonicalize().ok()?;
    let release_canon = release_dir.canonicalize().ok()?;
    let rel = release_canon.strip_prefix(&ws).ok()?;
    let guest = Path::new(guest_mount).join(rel);
    let s = guest.to_string_lossy().replace('\\', "/");
    Some(s)
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
///
/// Resolution order:
/// 1. [`ENV_DEVSHELL_VM_WORKSPACE_PARENT`] if set and non-empty.
/// 2. If [`ENV_DEVSHELL_VM_WORKSPACE_USE_CARGO_ROOT`] is unset or truthy: `cargo metadata` workspace root from
///    [`std::env::current_dir`] (so the repo you start from is the logical mount root).
/// 3. Legacy fallback: `…/vm-workspace/<instance>/` under the devshell export cache.
#[must_use]
pub fn workspace_parent_for_instance(instance: &str) -> PathBuf {
    if let Ok(p) = std::env::var(ENV_DEVSHELL_VM_WORKSPACE_PARENT) {
        let p = p.trim();
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    let use_cargo = match std::env::var(ENV_DEVSHELL_VM_WORKSPACE_USE_CARGO_ROOT) {
        Err(_) => true,
        Ok(s) => {
            let s = s.trim();
            if s.is_empty() {
                true
            } else {
                !(s == "0"
                    || s.eq_ignore_ascii_case("false")
                    || s.eq_ignore_ascii_case("no")
                    || s.eq_ignore_ascii_case("off"))
            }
        }
    };
    if use_cargo {
        if let Ok(cwd) = std::env::current_dir() {
            if let Ok((wr, _)) = cargo_metadata_workspace_and_target(&cwd) {
                return wr.canonicalize().unwrap_or(wr);
            }
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
