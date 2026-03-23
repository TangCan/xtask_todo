//! Windows: try to start the β `devshell-vm` sidecar via Podman so `cargo-devshell` works out of the box.

#![allow(clippy::pedantic, clippy::nursery)]

use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use super::config::{ENV_DEVSHELL_VM_BETA_SESSION_STAGING, ENV_DEVSHELL_VM_SKIP_PODMAN_BOOTSTRAP};
use super::VmError;

const SIDECAR_PORT: u16 = 9847;
const CONTAINER_NAME: &str = "cargo-devshell-sidecar";
const IMAGE_TAG: &str = "devshell-vm:local";

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

fn find_repo_root_with_containerfile() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        let cf = dir.join("containers/devshell-vm/Containerfile");
        if cf.is_file() {
            return Some(dir);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

fn try_install_podman_via_winget() {
    if !command_succeeds("winget", &["--version"]) {
        return;
    }
    eprintln!("dev_shell: Podman not found; trying: winget install Podman.Podman …");
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
        Ok(s) if s.success() => eprintln!("dev_shell: winget reported success; you may need to open a new terminal so `podman` is on PATH."),
        Ok(_) => eprintln!("dev_shell: winget install failed (try: run Terminal as Administrator, or install from https://podman.io/)."),
        Err(e) => eprintln!("dev_shell: could not run winget: {e}"),
    }
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
        .map_err(|e| VmError::Ipc(format!("podman build: {e}")))?;
    if !st.success() {
        return Err(VmError::Ipc(
            "podman build failed (see stderr above). Ensure you are in the xtask_todo repo.".into(),
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
        return Err(VmError::Ipc(
            "Podman is required for the devshell VM on Windows (beta backend). Install: https://podman.io/ \
             or: winget install -e --id Podman.Podman — then open a new terminal, or set DEVSHELL_VM_BACKEND=host."
                .into(),
        ));
    }

    let repo_root = find_repo_root_with_containerfile().ok_or_else(|| {
        VmError::Ipc(
            "could not find containers/devshell-vm/Containerfile — cd to the xtask_todo repository root, \
             or start devshell-vm manually and set DEVSHELL_VM_SOCKET / DEVSHELL_VM_SKIP_PODMAN_BOOTSTRAP=1."
                .into(),
        )
    })?;

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
        .map_err(|e| VmError::Ipc(format!("podman run: {e}")))?;
    if !st.success() {
        return Err(VmError::Ipc(
            "podman run failed (volume mount or port 9847 in use?).".into(),
        ));
    }

    if std::env::var(ENV_DEVSHELL_VM_BETA_SESSION_STAGING).is_err() {
        std::env::set_var(ENV_DEVSHELL_VM_BETA_SESSION_STAGING, "/workspace");
    }

    if !wait_for_sidecar(Duration::from_secs(45)) {
        return Err(VmError::Ipc(
            "sidecar did not open tcp/127.0.0.1:9847 in time".into(),
        ));
    }
    Ok(())
}
