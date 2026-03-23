//! Windows β: run **`devshell-vm --serve-stdio`** inside **Podman Machine** via **`podman machine ssh -T`**.
//! JSON lines go over the SSH session’s **stdin/stdout** — no host TCP listener (see `docs/devshell-vm-windows.md`).

#![allow(clippy::pedantic, clippy::nursery)]

use std::path::Path;

use super::VmError;

#[cfg(not(windows))]
#[must_use]
#[allow(dead_code)]
pub fn windows_host_path_to_vm_mnt(_host: &Path) -> Option<String> {
    None
}

#[cfg(not(windows))]
#[allow(dead_code)]
pub fn ensure(_workspace_parent: &Path) -> Result<(), VmError> {
    let _ = _workspace_parent;
    Ok(())
}

#[cfg(not(windows))]
#[allow(dead_code)]
pub fn spawn_devshell_vm_stdio(_workspace_root: &Path) -> Result<std::process::Child, VmError> {
    let _ = _workspace_root;
    Err(VmError::Ipc(
        "DEVSHELL_VM_SOCKET=stdio is only supported on Windows".into(),
    ))
}

#[cfg(windows)]
mod win {
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};

    use super::super::config::{
        devshell_repo_root_with_containerfile, ENV_DEVSHELL_VM_DISABLE_PODMAN_SSH_HOME,
        ENV_DEVSHELL_VM_LINUX_BINARY, ENV_DEVSHELL_VM_SKIP_PODMAN_BOOTSTRAP,
    };
    use super::VmError;

    /// Shown when `podman` is missing or unusable after auto-install attempts.
    const MSG_PODMAN_INSTALL: &str = "\
dev_shell (beta VM): Podman is not available or not on PATH.

Try in order:
  1) Install:    winget install -e --id Podman.Podman
  2) Verify:     podman version
  3) If needed:  podman machine start
  4) Docs:       https://podman.io/getting-started/installation
  5) Host-only:  set DEVSHELL_VM_BACKEND=host (no VM sidecar)";

    fn link_podman_machine_into_ssh_home(ssh_home: &Path) -> std::io::Result<()> {
        let real_profile = match std::env::var("USERPROFILE") {
            Ok(s) if !s.trim().is_empty() => PathBuf::from(s.trim()),
            _ => return Ok(()),
        };
        let real_podman = real_profile
            .join(".local")
            .join("share")
            .join("containers")
            .join("podman");
        if !real_podman.is_dir() {
            return Ok(());
        }
        let link = ssh_home
            .join(".local")
            .join("share")
            .join("containers")
            .join("podman");
        if link.exists() {
            return Ok(());
        }
        if let Some(parent) = link.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::os::windows::fs::symlink_dir(&real_podman, &link)
    }

    fn ssh_home_for_podman() -> std::io::Result<PathBuf> {
        let home = std::env::temp_dir().join("cargo-devshell-ssh-home");
        let ssh = home.join(".ssh");
        std::fs::create_dir_all(&ssh)?;
        let kh = ssh.join("known_hosts");
        if !kh.exists() {
            std::fs::write(&kh, "")?;
        }
        link_podman_machine_into_ssh_home(&home).unwrap_or_else(|e| {
            eprintln!(
                "dev_shell: could not symlink %USERPROFILE%\\.local\\share\\containers\\podman into isolated profile (SSH workaround): {e}\n\
                 Or set {}=1 to use your real profile (and fix or unlock .ssh\\known_hosts).",
                ENV_DEVSHELL_VM_DISABLE_PODMAN_SSH_HOME
            );
        });
        Ok(home)
    }

    fn apply_windows_podman_profile_env(cmd: &mut Command, home: &Path) {
        cmd.env("USERPROFILE", home);
        cmd.env("HOME", home);
        if let Some(s) = home.to_str() {
            let b = s.as_bytes();
            if b.len() >= 2 && b[1] == b':' {
                if let (Some(drive), Some(rest)) = (s.get(..2), s.get(2..)) {
                    cmd.env("HOMEDRIVE", drive);
                    cmd.env("HOMEPATH", rest);
                }
            }
        }
    }

    fn apply_podman_ssh_home_env(cmd: &mut Command) {
        if std::env::var(ENV_DEVSHELL_VM_DISABLE_PODMAN_SSH_HOME).is_ok() {
            return;
        }
        match ssh_home_for_podman() {
            Ok(home) => apply_windows_podman_profile_env(cmd, &home),
            Err(e) => {
                eprintln!("dev_shell: could not create temp HOME for podman (SSH known_hosts workaround): {e}");
            }
        }
    }

    fn podman_command() -> Command {
        let mut c = Command::new("podman");
        apply_podman_ssh_home_env(&mut c);
        c
    }

    fn podman_version_succeeds() -> bool {
        podman_command()
            .args(["--version"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn try_install_podman_via_winget() {
        if !Command::new("winget")
            .args(["--version"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            eprintln!("dev_shell: winget not found — install Podman manually from https://podman.io/ or use Chocolatey/choco install podman.");
            return;
        }
        eprintln!("dev_shell: Podman not on PATH; trying: winget install -e --id Podman.Podman");
        let status = Command::new("winget")
            .args([
                "install",
                "-e",
                "--id",
                "Podman.Podman",
                "--accept-package-agreements",
                "--accept-source-agreements",
            ])
            .status();
        match status {
            Ok(s) if s.success() => eprintln!(
                "dev_shell: winget reported success. Open a NEW terminal, then run:  podman version"
            ),
            Ok(_) => eprintln!(
                "dev_shell: winget install failed — try an elevated (Administrator) terminal, or install from https://podman.io/"
            ),
            Err(e) => eprintln!("dev_shell: could not run winget: {e}"),
        }
    }

    fn try_podman_engine_ready() {
        if podman_command()
            .args(["info"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return;
        }
        eprintln!("dev_shell: podman info failed — trying: podman machine start");
        let _ = podman_command().args(["machine", "start"]).status();
        std::thread::sleep(std::time::Duration::from_secs(2));
    }

    /// Windows path `D:\a\b` → `/mnt/d/a/b` for paths inside Podman Machine (Fedora/WSL-like mount).
    pub(super) fn windows_host_path_to_vm_mnt(host: &Path) -> Option<String> {
        let s = host.to_str()?;
        let norm = s.trim_start_matches(r"\\?\").replace('\\', "/");
        if norm.len() < 2 {
            return None;
        }
        let b = norm.as_bytes();
        if b[1] != b':' {
            return None;
        }
        let drive = norm.chars().next()?.to_ascii_lowercase();
        let rest = &norm[2..];
        let rest = rest.trim_start_matches('/');
        Some(format!("/mnt/{drive}/{rest}"))
    }

    fn linux_devshell_vm_host_path(workspace_root: &Path) -> Result<PathBuf, VmError> {
        if let Ok(s) = std::env::var(ENV_DEVSHELL_VM_LINUX_BINARY) {
            let p = PathBuf::from(s.trim());
            if p.is_file() {
                return Ok(p);
            }
            return Err(VmError::Ipc(format!(
                "{} points to a missing file: {}",
                ENV_DEVSHELL_VM_LINUX_BINARY,
                p.display()
            )));
        }
        let wr =
            devshell_repo_root_with_containerfile().unwrap_or_else(|| workspace_root.to_path_buf());
        let p = wr.join("target/x86_64-unknown-linux-gnu/release/devshell-vm");
        if p.is_file() {
            return Ok(p);
        }
        Err(VmError::Ipc(format!(
            "devshell-vm Linux binary not found at {}.\n\
             Build: rustup target add x86_64-unknown-linux-gnu\n\
             cargo build -p devshell-vm --release --target x86_64-unknown-linux-gnu\n\
             Or set {} to its Windows path.",
            p.display(),
            ENV_DEVSHELL_VM_LINUX_BINARY
        )))
    }

    pub(super) fn ensure(workspace_parent: &Path) -> Result<(), VmError> {
        if std::env::var(ENV_DEVSHELL_VM_SKIP_PODMAN_BOOTSTRAP).is_ok() {
            return Ok(());
        }

        if !podman_version_succeeds() {
            try_install_podman_via_winget();
        }
        if !podman_version_succeeds() {
            return Err(VmError::Ipc(MSG_PODMAN_INSTALL.to_string()));
        }

        try_podman_engine_ready();

        let _ = linux_devshell_vm_host_path(workspace_parent)?;
        Ok(())
    }

    pub(super) fn spawn_devshell_vm_stdio(
        workspace_root: &Path,
    ) -> Result<std::process::Child, VmError> {
        let host_bin = linux_devshell_vm_host_path(workspace_root)?;
        let host_bin = host_bin
            .canonicalize()
            .map_err(|e| VmError::Ipc(format!("canonicalize {}: {e}", host_bin.display())))?;
        let vm_path = windows_host_path_to_vm_mnt(&host_bin).ok_or_else(|| {
            VmError::Ipc(format!(
                "could not map host path {} to a /mnt/... path inside Podman Machine",
                host_bin.display()
            ))
        })?;
        let escaped = vm_path.replace('\'', "'\"'\"'");
        let script = format!("exec '{escaped}' --serve-stdio");

        let mut cmd = podman_command();
        cmd.args(["machine", "ssh", "-T", "--", "sh", "-c", &script]);
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::inherit());

        cmd.spawn()
            .map_err(|e| VmError::Ipc(format!("podman machine ssh: {e}\n{MSG_PODMAN_INSTALL}")))
    }
}

#[cfg(windows)]
#[must_use]
pub fn windows_host_path_to_vm_mnt(host: &Path) -> Option<String> {
    win::windows_host_path_to_vm_mnt(host)
}

#[cfg(windows)]
pub fn ensure(workspace_parent: &Path) -> Result<(), VmError> {
    win::ensure(workspace_parent)
}

#[cfg(windows)]
pub fn spawn_devshell_vm_stdio(workspace_root: &Path) -> Result<std::process::Child, VmError> {
    win::spawn_devshell_vm_stdio(workspace_root)
}
