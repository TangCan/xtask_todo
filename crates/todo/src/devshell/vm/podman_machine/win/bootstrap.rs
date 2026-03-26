use std::path::{Path, PathBuf};
use std::process::Command;

use super::super::super::config::{
    ENV_DEVSHELL_VM_DISABLE_PODMAN_SSH_HOME, ENV_DEVSHELL_VM_SKIP_PODMAN_BOOTSTRAP,
};
use super::{VmError, MSG_PODMAN_INSTALL};

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

pub(super) fn podman_command() -> Command {
    let mut c = Command::new("podman");
    apply_podman_ssh_home_env(&mut c);
    c
}

pub(super) fn podman_version_succeeds() -> bool {
    podman_command()
        .args(["--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub(super) fn try_install_podman_via_winget() {
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

pub(super) fn try_podman_engine_ready() {
    if std::env::var(ENV_DEVSHELL_VM_SKIP_PODMAN_BOOTSTRAP).is_ok() {
        return;
    }
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

pub(super) fn podman_not_available_error() -> VmError {
    VmError::Ipc(MSG_PODMAN_INSTALL.to_string())
}
