//! `cargo` / filesystem / `limactl` helpers.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value as JsonValue;

/// Run `cargo metadata` in `workspace` and return (`workspace_root`, `target_directory`).
pub(super) fn cargo_metadata_target(workspace: &Path) -> Result<(PathBuf, PathBuf), String> {
    let out = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .current_dir(workspace)
        .output()
        .map_err(|e| format!("cargo metadata: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let v: JsonValue =
        serde_json::from_slice(&out.stdout).map_err(|e| format!("cargo metadata JSON: {e}"))?;
    let root = v
        .get("workspace_root")
        .and_then(|x| x.as_str())
        .ok_or_else(|| "metadata: missing workspace_root".to_string())?;
    let target = v
        .get("target_directory")
        .and_then(|x| x.as_str())
        .ok_or_else(|| "metadata: missing target_directory".to_string())?;
    Ok((PathBuf::from(root), PathBuf::from(target)))
}

/// Absolute `target/release` path string (matches Lima `mounts[].location` after merge).
pub(super) fn host_release_str_for_target_dir(target_dir: &Path) -> Result<String, String> {
    let release_dir = target_dir.join("release");
    if let Ok(p) = release_dir.canonicalize() {
        return Ok(p.to_string_lossy().replace('\\', "/"));
    }
    let td = target_dir
        .canonicalize()
        .map_err(|e| format!("canonicalize {}: {e}", target_dir.display()))?;
    let r = td.join("release");
    Ok(r.to_string_lossy().replace('\\', "/"))
}

pub(super) fn build_todo_release(workspace: &Path) -> Result<(), String> {
    let st = Command::new("cargo")
        .args(["build", "-p", "xtask", "--release", "--bin", "todo"])
        .current_dir(workspace)
        .status()
        .map_err(|e| format!("cargo build: {e}"))?;
    if !st.success() {
        return Err("cargo build -p xtask --release --bin todo failed".to_string());
    }
    Ok(())
}

pub(super) fn backup_path(path: &Path) -> PathBuf {
    let mut s = path.as_os_str().to_os_string();
    s.push(".bak");
    PathBuf::from(s)
}

pub(super) fn backup_and_write(path: &Path, content: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("create_dir_all {}: {e}", parent.display()))?;
    }
    if path.is_file() {
        let bak = backup_path(path);
        std::fs::copy(path, &bak)
            .map_err(|e| format!("backup {} -> {}: {e}", path.display(), bak.display()))?;
    }
    std::fs::write(path, content).map_err(|e| format!("write {}: {e}", path.display()))
}

pub(super) fn limactl_restart(instance: &str) -> Result<(), String> {
    let st_stop = Command::new("limactl")
        .args(["stop", instance])
        .status()
        .map_err(|e| format!("limactl stop: {e}"))?;
    if !st_stop.success() {
        return Err(format!(
            "limactl stop {instance} failed (exit {:?}); fix instance name or stop manually",
            st_stop.code()
        ));
    }
    let st_start = Command::new("limactl")
        .args(["start", "-y", instance])
        .status()
        .map_err(|e| format!("limactl start: {e}"))?;
    if !st_start.success() {
        return Err(format!(
            "limactl start -y {instance} failed (exit {:?})",
            st_start.code()
        ));
    }
    Ok(())
}
