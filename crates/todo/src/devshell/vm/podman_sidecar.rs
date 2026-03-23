//! Windows: try to start the β `devshell-vm` sidecar via Podman so `cargo-devshell` works out of the box.

#![allow(clippy::pedantic, clippy::nursery)]

use std::net::{SocketAddr, TcpStream};
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

use super::config::{
    devshell_repo_root_with_containerfile, ENV_DEVSHELL_VM_BETA_SESSION_STAGING,
    ENV_DEVSHELL_VM_SKIP_PODMAN_BOOTSTRAP,
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

fn command_succeeds(cmd: &str, args: &[&str]) -> bool {
    Command::new(cmd)
        .args(args)
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
    if !command_succeeds("winget", &["--version"]) {
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
    if Command::new("podman")
        .args(["info"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return;
    }
    eprintln!("dev_shell: podman info failed — trying: podman machine start");
    let _ = Command::new("podman").args(["machine", "start"]).status();
    std::thread::sleep(Duration::from_secs(2));
}

fn podman_image_exists(image: &str) -> bool {
    command_succeeds("podman", &["image", "exists", image])
}

fn podman_build(repo_root: &Path, image: &str) -> Result<(), VmError> {
    let st = Command::new("podman")
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
    let _ = Command::new("podman")
        .args(["rm", "-f", CONTAINER_NAME])
        .output();
}

/// Ensure `127.0.0.1:9847` accepts TCP: if nothing listens, try Podman build + run.
pub fn ensure(workspace_parent: &Path) -> Result<(), VmError> {
    if std::env::var(ENV_DEVSHELL_VM_SKIP_PODMAN_BOOTSTRAP).is_ok() {
        return Ok(());
    }
    if sidecar_tcp_reachable() {
        return Ok(());
    }

    if !command_succeeds("podman", &["--version"]) {
        try_install_podman_via_winget();
    }
    if !command_succeeds("podman", &["--version"]) {
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
    let st = Command::new("podman")
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
