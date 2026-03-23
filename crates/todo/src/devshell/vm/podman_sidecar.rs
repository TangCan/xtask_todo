//! Windows: try to start the β `devshell-vm` sidecar via Podman so `cargo-devshell` works out of the box.

#![allow(clippy::pedantic, clippy::nursery)]

use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use super::config::{
    devshell_repo_root_with_containerfile, ENV_DEVSHELL_VM_BETA_SESSION_STAGING,
    ENV_DEVSHELL_VM_DISABLE_PODMAN_SSH_HOME, ENV_DEVSHELL_VM_SKIP_PODMAN_BOOTSTRAP,
};
use super::VmError;

const SIDECAR_PORT: u16 = 9847;
const CONTAINER_NAME: &str = "cargo-devshell-sidecar";
const IMAGE_TAG: &str = "devshell-vm:local";

/// Shown when `podman` is missing or unusable after auto-install attempts.
const MSG_PODMAN_INSTALL: &str = "\
dev_shell (beta VM): Podman is not available or not on PATH.

Try in order:
  1) Install:    winget install -e --id Podman.Podman
  2) Verify:     podman version
  3) If needed:  podman machine start
  4) Docs:       https://podman.io/getting-started/installation
  5) Host-only:  set DEVSHELL_VM_BACKEND=host (no VM sidecar)";

/// Shown when `podman` exists but auto-start has no Containerfile (not in xtask_todo clone).
const MSG_NO_CONTAINERFILE: &str = "\
dev_shell (beta VM): cannot auto-build the sidecar image — no containers/devshell-vm/Containerfile in parent directories.

Do one of:
  A) cd to your xtask_todo clone root (the repo that contains containers/devshell-vm/), then run cargo-devshell again; or
  B) Build once from that repo:  podman build -f containers/devshell-vm/Containerfile -t devshell-vm:local .
     then run the container manually (see docs/devshell-vm-windows.md); or
  C) Skip auto Podman and use a running sidecar:  set DEVSHELL_VM_SKIP_PODMAN_BOOTSTRAP=1
     and ensure DEVSHELL_VM_SOCKET=tcp:127.0.0.1:9847 reaches devshell-vm; or
  D) Host-only:  set DEVSHELL_VM_BACKEND=host";

/// Podman’s Go/SSH stack resolves `~/.ssh/known_hosts` via **`os.UserHomeDir()`**.
/// - On **Unix**, that is typically **`$HOME`**.
/// - On **Windows**, **`UserHomeDir()` uses `%USERPROFILE%` first**, not `HOME`. So we must set
///   **`USERPROFILE`** (and **`HOME`** for POSIX-style tools) to a stable temp “profile” directory with a
///   **writable empty** `.ssh/known_hosts`, so a **locked, protected, or invalid**
///   **`%USERPROFILE%\.ssh\known_hosts`** is never read — effectively “default empty known_hosts”.
///
/// Podman Machine stores state under **`%USERPROFILE%\.local\share\containers\podman`**. When we point
/// `USERPROFILE` at the temp dir, we **symlink** that subtree from the **real** profile when it exists,
/// so existing machines keep working.
#[cfg(windows)]
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

#[cfg(not(windows))]
fn link_podman_machine_into_ssh_home(_ssh_home: &Path) -> std::io::Result<()> {
    Ok(())
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
             Podman may not see an existing machine until you enable Windows Developer Mode (symlinks) or run elevated once.\n\
             Or set {}=1 to use your real profile (and fix or unlock .ssh\\known_hosts).",
            ENV_DEVSHELL_VM_DISABLE_PODMAN_SSH_HOME
        );
    });
    Ok(home)
}

#[cfg(windows)]
fn apply_windows_podman_profile_env(cmd: &mut Command, home: &Path) {
    cmd.env("USERPROFILE", home);
    cmd.env("HOME", home);
    // Some Windows APIs / legacy code use HOMEDRIVE + HOMEPATH when USERPROFILE is odd.
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
        Ok(home) => {
            #[cfg(windows)]
            apply_windows_podman_profile_env(cmd, &home);
            #[cfg(not(windows))]
            cmd.env("HOME", &home);
        }
        Err(e) => {
            eprintln!("dev_shell: could not create temp HOME for podman (SSH known_hosts workaround): {e}");
        }
    }
}

/// `podman` subprocess with optional Windows `HOME` workaround for SSH `known_hosts`.
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

fn sidecar_tcp_reachable() -> bool {
    let addr: SocketAddr = format!("127.0.0.1:{SIDECAR_PORT}")
        .parse()
        .expect("valid addr");
    TcpStream::connect_timeout(&addr, Duration::from_millis(300)).is_ok()
}

fn wait_for_sidecar(timeout: Duration) -> bool {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if sidecar_tcp_reachable() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(150));
    }
    false
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

/// Podman on Windows often needs a running machine before `podman run` works.
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
    std::thread::sleep(Duration::from_secs(2));
}

fn podman_image_exists(image: &str) -> bool {
    podman_command()
        .args(["image", "exists", image])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn podman_build(repo_root: &Path, image: &str) -> Result<(), VmError> {
    let st = podman_command()
        .args([
            "build",
            "-f",
            "containers/devshell-vm/Containerfile",
            "-t",
            image,
            ".",
        ])
        .current_dir(repo_root)
        .status()
        .map_err(|e| VmError::Ipc(format!("podman build: {e}\n{MSG_PODMAN_INSTALL}")))?;
    if !st.success() {
        return Err(VmError::Ipc(
            "podman build failed (see stderr above: network, disk, or Containerfile).\n\
             Retry after fixing, or: set DEVSHELL_VM_BACKEND=host"
                .to_string(),
        ));
    }
    Ok(())
}

fn podman_rm_force() {
    let _ = podman_command().args(["rm", "-f", CONTAINER_NAME]).output();
}

/// Ensure `127.0.0.1:9847` accepts TCP: if nothing listens, try Podman build + run.
pub fn ensure(workspace_parent: &Path) -> Result<(), VmError> {
    if std::env::var(ENV_DEVSHELL_VM_SKIP_PODMAN_BOOTSTRAP).is_ok() {
        return Ok(());
    }
    if sidecar_tcp_reachable() {
        return Ok(());
    }

    if !podman_version_succeeds() {
        try_install_podman_via_winget();
    }
    if !podman_version_succeeds() {
        return Err(VmError::Ipc(MSG_PODMAN_INSTALL.to_string()));
    }

    try_podman_engine_ready();

    let repo_root = match devshell_repo_root_with_containerfile() {
        Some(p) => p,
        None => {
            eprintln!("{MSG_NO_CONTAINERFILE}");
            return Ok(());
        }
    };

    if !podman_image_exists(IMAGE_TAG) {
        eprintln!("dev_shell: building {IMAGE_TAG} (first run may take a few minutes)…");
        podman_build(&repo_root, IMAGE_TAG)?;
    }

    podman_rm_force();

    let ws = workspace_parent.as_os_str().to_string_lossy();
    let vol = format!("{ws}:/workspace");
    let st = podman_command()
        .args([
            "run",
            "-d",
            "--name",
            CONTAINER_NAME,
            "-p",
            "9847:9847",
            "-v",
            &vol,
            IMAGE_TAG,
        ])
        .current_dir(&repo_root)
        .status()
        .map_err(|e| VmError::Ipc(format!("podman run: {e}\n{MSG_PODMAN_INSTALL}")))?;
    if !st.success() {
        return Err(VmError::Ipc(
            "podman run failed (check: podman machine start, port 9847, volume path).\n\
             Verify:  podman ps -a\n\
             Host-only:  set DEVSHELL_VM_BACKEND=host"
                .to_string(),
        ));
    }

    if std::env::var(ENV_DEVSHELL_VM_BETA_SESSION_STAGING).is_err() {
        std::env::set_var(ENV_DEVSHELL_VM_BETA_SESSION_STAGING, "/workspace");
    }

    if !wait_for_sidecar(Duration::from_secs(45)) {
        return Err(VmError::Ipc(
            "sidecar did not open tcp/127.0.0.1:9847 in time.\n\
             Check:  podman logs cargo-devshell-sidecar\n\
             Or:     set DEVSHELL_VM_BACKEND=host"
                .to_string(),
        ));
    }
    Ok(())
}
