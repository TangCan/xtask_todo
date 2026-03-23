//! Host workspace paths shared by γ (Lima) and β (`devshell-vm` over socket / TCP / Windows stdio), without pulling in `limactl` / Lima session.
//!
//! Used on **all targets** when `beta-vm` is enabled (Windows β uses TCP); γ continues to use the same
//! resolution via `session_gamma`.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

use super::super::sandbox;
use super::guest_fs_ops::guest_project_dir_on_guest;

// Keep in sync with `session_gamma/env.rs`.
const ENV_DEVSHELL_VM_WORKSPACE_PARENT: &str = "DEVSHELL_VM_WORKSPACE_PARENT";
const ENV_DEVSHELL_VM_WORKSPACE_USE_CARGO_ROOT: &str = "DEVSHELL_VM_WORKSPACE_USE_CARGO_ROOT";

fn cargo_metadata_workspace_and_target(cwd: &Path) -> Result<(PathBuf, PathBuf), ()> {
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

fn sanitize_instance_segment(name: &str) -> String {
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

/// Host workspace root shared with the VM / β staging tree (see `docs/devshell-vm-gamma.md`).
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

/// Guest working directory for VFS cwd (mount + last path segment).
#[must_use]
pub fn guest_dir_for_vfs_cwd(guest_mount: &str, vfs_cwd: &str) -> String {
    guest_project_dir_on_guest(guest_mount, vfs_cwd)
}
