//! Heuristic checks for Lima γ setup: guest probes + light `lima.yaml` parsing.
//!
//! See `docs/devshell-vm-gamma.md`.

#![allow(clippy::pedantic, clippy::nursery)]

use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};

/// Set to `0`/`false`/`no`/`off` to silence Lima configuration hints.
pub const ENV_DEVSHELL_VM_LIMA_HINTS: &str = "DEVSHELL_VM_LIMA_HINTS";

fn hints_enabled() -> bool {
    match std::env::var(ENV_DEVSHELL_VM_LIMA_HINTS) {
        Err(_) => true,
        Ok(s) => {
            let s = s.trim();
            !(s.is_empty()
                || s == "0"
                || s.eq_ignore_ascii_case("false")
                || s.eq_ignore_ascii_case("no")
                || s.eq_ignore_ascii_case("off"))
        }
    }
}

fn lima_home() -> PathBuf {
    if let Ok(h) = std::env::var("LIMA_HOME") {
        let h = h.trim();
        if !h.is_empty() {
            return PathBuf::from(h);
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        let home = home.trim();
        if !home.is_empty() {
            return PathBuf::from(home).join(".lima");
        }
    }
    PathBuf::from(".lima")
}

/// `~/.lima/<instance>/lima.yaml` (or `$LIMA_HOME/<instance>/lima.yaml`).
#[must_use]
pub fn lima_yaml_path(instance: &str) -> PathBuf {
    lima_home().join(instance).join("lima.yaml")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GuestProbe {
    pub guest_mount_exists: bool,
    pub project_dir_exists: bool,
    pub cargo_in_path: bool,
}

/// Run a non-interactive guest script via `limactl shell -y` and parse `w p c` flags.
pub fn probe_guest(
    limactl: &Path,
    instance: &str,
    guest_mount: &str,
    guest_project_dir: &str,
) -> Option<GuestProbe> {
    let gm = sh_word(guest_mount)?;
    let gd = sh_word(guest_project_dir)?;
    let script = format!(
        "w=0; test -d {gm} && w=1; p=0; test -d {gd} && p=1; c=0; command -v cargo >/dev/null 2>&1 && c=1; printf '%s %s %s\\n' \"$w\" \"$p\" \"$c\""
    );
    let out = Command::new(limactl)
        .args([
            "shell",
            "-y",
            "--workdir",
            "/",
            instance,
            "--",
            "/bin/sh",
            "-c",
            &script,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let line = String::from_utf8_lossy(&out.stdout);
    let mut parts = line.split_whitespace();
    let w = parts.next()? == "1";
    let p = parts.next()? == "1";
    let c = parts.next()? == "1";
    Some(GuestProbe {
        guest_mount_exists: w,
        project_dir_exists: p,
        cargo_in_path: c,
    })
}

/// Shell-embed a path: safe subset without quotes; otherwise single-quoted with escapes.
fn sh_word(path: &str) -> Option<String> {
    if path.is_empty() {
        return None;
    }
    if path
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "/._-+:@".contains(c))
    {
        Some(path.to_string())
    } else {
        Some(format!("'{}'", path.replace('\'', "'\\''")))
    }
}

/// True if `lima.yaml` text contains a `mountPoint:` line for `guest_mount`.
#[must_use]
pub fn yaml_has_mount_point(content: &str, guest_mount: &str) -> bool {
    let gm = guest_mount.trim_end_matches('/');
    for line in content.lines() {
        let t = line.trim();
        let rest = t.strip_prefix("mountPoint:").or_else(|| {
            t.find("mountPoint:")
                .map(|pos| &t[pos + "mountPoint:".len()..])
        });
        if let Some(r) = rest {
            let v = r
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .trim_end_matches('/');
            if v == gm {
                return true;
            }
        }
    }
    false
}

/// Heuristic: yaml mentions the devshell workspace staging path (tilde or cache segment).
#[cfg(test)]
fn yaml_mentions_workspace_staging(content: &str, workspace_parent: &Path) -> bool {
    if content.contains("vm-workspace") {
        return true;
    }
    if let Some(name) = workspace_parent.file_name().and_then(|n| n.to_str()) {
        if content.contains(name) {
            return true;
        }
    }
    let lossy = workspace_parent.to_string_lossy();
    content.contains(lossy.as_ref())
}

#[must_use]
pub fn yaml_mentions_host_toolchain_mounts(content: &str) -> bool {
    content.contains("host-rustup") && content.contains("host-cargo")
}

#[must_use]
pub fn yaml_has_rust_env(content: &str) -> bool {
    content.contains("RUSTUP_HOME:") && content.contains("CARGO_HOME:")
}

/// After `limactl start` fails (non-zero exit).
pub fn emit_start_failure_hints(instance: &str) {
    if !hints_enabled() {
        return;
    }
    eprintln!("dev_shell: lima: `limactl start` failed — hints:");
    if let Some(msg) = tail_ha_stderr_kvm_hint(instance) {
        eprintln!("dev_shell: lima: - {msg}");
    }
    let yaml_path = lima_yaml_path(instance);
    if !yaml_path.exists() {
        eprintln!(
            "dev_shell: lima: - no `{}` yet — first start creates the instance; if you see `template \"default.yaml\" not found`, install Lima `share/lima` next to `limactl` (see docs/devshell-vm-gamma.md).",
            yaml_path.display()
        );
    }
    if let Ok(data) = std::fs::read_to_string(lima_home().join(instance).join("ha.stderr.log")) {
        if data.contains("template") && data.contains("not found") {
            eprintln!(
                "dev_shell: lima: - host log mentions missing template — ensure Lima was installed with `share/lima/templates` (docs/devshell-vm-gamma.md)."
            );
        }
    }
    eprintln!(
        "dev_shell: lima: - see `~/.lima/{instance}/ha.stderr.log` and run `limactl list`; disable hints: {ENV_DEVSHELL_VM_LIMA_HINTS}=0.",
        instance = instance
    );
}

fn tail_ha_stderr_kvm_hint(instance: &str) -> Option<String> {
    let path = lima_home().join(instance).join("ha.stderr.log");
    let data = std::fs::read_to_string(&path).ok()?;
    if data.contains("Could not access KVM kernel module")
        || data.contains("failed to initialize kvm")
    {
        Some(
            "host log mentions KVM permission denied — add user to group `kvm` and re-login (or run `newgrp kvm` / `sg kvm -c '…'`). See docs/devshell-vm-gamma.md."
                .to_string(),
        )
    } else {
        None
    }
}

/// After VM is up: warn if guest layout or yaml looks wrong (once per session).
pub fn warn_if_guest_misconfigured(
    limactl: &Path,
    instance: &str,
    workspace_parent: &Path,
    guest_mount: &str,
    guest_project_dir: &str,
) {
    if !hints_enabled() {
        return;
    }
    let yaml_path = lima_yaml_path(instance);
    let yaml_text = std::fs::read_to_string(&yaml_path).unwrap_or_default();

    let Some(probe) = probe_guest(limactl, instance, guest_mount, guest_project_dir) else {
        eprintln!(
            "dev_shell: lima: could not run guest probe via `limactl shell`; check `limactl list` and instance name `{instance}`."
        );
        return;
    };

    if !probe.guest_mount_exists {
        eprintln!(
            "dev_shell: lima: guest directory `{guest_mount}` is missing — mount host `{}` at `{guest_mount}` in `{}` (writable), then `limactl stop/start`. See docs/devshell-vm-gamma.md and docs/snippets/lima-devshell-workspace-mount.yaml.",
            workspace_parent.display(),
            yaml_path.display()
        );
    } else if !yaml_has_mount_point(&yaml_text, guest_mount) {
        eprintln!(
            "dev_shell: lima: `{}` has no `mountPoint: {guest_mount}` line; keep it in sync with your mounts. See docs/devshell-vm-gamma.md.",
            yaml_path.display()
        );
    }

    if probe.guest_mount_exists && !probe.project_dir_exists {
        eprintln!(
            "dev_shell: lima: guest project dir `{guest_project_dir}` missing after sync — check VFS cwd leaf matches a directory under `{}` on the host.",
            workspace_parent.display()
        );
    }

    if !probe.cargo_in_path {
        if yaml_mentions_host_toolchain_mounts(&yaml_text) && yaml_has_rust_env(&yaml_text) {
            eprintln!(
                "dev_shell: lima: `cargo` not found in guest but lima.yaml mentions host toolchain mounts — verify mounts and `env:` PATH (then `limactl stop/start`). See docs/snippets/lima-devshell-rust-toolchain-mount.yaml."
            );
        } else {
            eprintln!(
                "dev_shell: lima: `cargo` not found in guest — install rustup in the VM, or read-only mount `~/.rustup` and `~/.cargo` plus `env:` (docs/devshell-vm-gamma.md, \"做法二\")."
            );
        }
    }
}

/// After `cargo`/`rustup` fails inside the guest.
pub fn emit_tool_failure_hints(
    limactl: &Path,
    instance: &str,
    workspace_parent: &Path,
    guest_mount: &str,
    guest_project_dir: &str,
    program: &str,
    status: &ExitStatus,
) {
    if !hints_enabled() {
        return;
    }
    if program != "cargo" && program != "rustup" {
        return;
    }

    let code = status.code();
    eprintln!(
        "dev_shell: lima: `{program}` exited with {:?} — diagnostic hints:",
        code
    );

    if let Some(msg) = tail_ha_stderr_kvm_hint(instance) {
        eprintln!("dev_shell: lima: - {msg}");
    }

    let yaml_path = lima_yaml_path(instance);
    if !yaml_path.exists() {
        eprintln!(
            "dev_shell: lima: - no `{}` — create instance `{}` or fix DEVSHELL_VM_LIMA_INSTANCE / LIMA_HOME.",
            yaml_path.display(),
            instance
        );
        return;
    }

    let yaml_text = std::fs::read_to_string(&yaml_path).unwrap_or_default();

    if let Some(probe) = probe_guest(limactl, instance, guest_mount, guest_project_dir) {
        if !probe.guest_mount_exists {
            eprintln!(
                "dev_shell: lima: - guest `{guest_mount}` missing: add writable mount of `{}` → `{guest_mount}` in lima.yaml; see docs/snippets/lima-devshell-workspace-mount.yaml",
                workspace_parent.display()
            );
        }
        if probe.guest_mount_exists && !probe.project_dir_exists {
            eprintln!(
                "dev_shell: lima: - guest `{guest_project_dir}` missing: ensure push created it under `{}` on the host, or fix VFS cwd.",
                workspace_parent.display()
            );
        }
        if code == Some(127) || !probe.cargo_in_path {
            eprintln!(
                "dev_shell: lima: - `cargo`/`rustup` not on PATH in guest: install rustup in VM, or mount host `~/.rustup` + `~/.cargo` with `env:` PATH (docs/devshell-vm-gamma.md)."
            );
        }
    } else {
        eprintln!(
            "dev_shell: lima: - could not probe guest; try `limactl shell -y --workdir / {instance} -- true`"
        );
    }

    if !yaml_has_mount_point(&yaml_text, guest_mount) {
        eprintln!(
            "dev_shell: lima: - `{}` should contain `mountPoint: {guest_mount}` for devshell γ.",
            yaml_path.display()
        );
    }

    if (code == Some(127) || code == Some(126)) && !yaml_mentions_host_toolchain_mounts(&yaml_text)
    {
        eprintln!(
            "dev_shell: lima: - for host toolchain sharing, add mounts for `host-rustup` / `host-cargo` and `env:` block; see docs/snippets/lima-devshell-rust-toolchain-mount.yaml."
        );
    }

    eprintln!(
        "dev_shell: lima: - full guide: docs/devshell-vm-gamma.md (disable hints: {}=0).",
        ENV_DEVSHELL_VM_LIMA_HINTS
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yaml_has_mount_point_detects_workspace() {
        let y = "mounts:\n  - location: \"~\"\n  - location: \"~/.cache/foo\"\n    mountPoint: /workspace\n    writable: true\n";
        assert!(yaml_has_mount_point(y, "/workspace"));
        assert!(!yaml_has_mount_point(y, "/work"));
    }

    #[test]
    fn yaml_has_rust_env_detects() {
        let y = "env:\n  RUSTUP_HOME: /host-rustup\n  CARGO_HOME: /host-cargo\n";
        assert!(yaml_has_rust_env(y));
    }

    #[test]
    fn yaml_mentions_workspace_staging_paths() {
        let p = Path::new("/home/x/.cache/cargo-devshell-exports/vm-workspace/devshell-rust");
        let y = "location: /home/x/.cache/cargo-devshell-exports/vm-workspace/devshell-rust\n";
        assert!(yaml_mentions_workspace_staging(y, p));
    }
}
