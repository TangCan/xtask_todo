//! Windows: Podman machine SSH / `podman run` stdio transport for β.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use super::super::config::{
    devshell_repo_root_from_path, devshell_repo_root_with_containerfile,
    ENV_DEVSHELL_VM_CONTAINER_IMAGE, ENV_DEVSHELL_VM_DISABLE_PODMAN_SSH_HOME,
    ENV_DEVSHELL_VM_LINUX_BINARY, ENV_DEVSHELL_VM_REPO_ROOT, ENV_DEVSHELL_VM_SKIP_PODMAN_BOOTSTRAP,
    ENV_DEVSHELL_VM_STDIO_TRANSPORT,
};
use super::super::VmError;
use super::WindowsStdioTransport;

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

fn push_repo_candidate(out: &mut Vec<PathBuf>, p: PathBuf) {
    if out.iter().any(|x| x == &p) {
        return;
    }
    out.push(p);
}

/// `cargo-devshell` workspace parent under `%LOCALAPPDATA%\cargo-devshell-exports\…` never contains a built
/// `devshell-vm`; do not use it as the default search root.
fn workspace_parent_is_ephemeral_export(host_workspace: &Path) -> bool {
    host_workspace
        .to_string_lossy()
        .to_ascii_lowercase()
        .contains("cargo-devshell-exports")
}

fn default_container_image() -> String {
    std::env::var(ENV_DEVSHELL_VM_CONTAINER_IMAGE)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            format!(
                "ghcr.io/tangcan/xtask_todo/devshell-vm:v{}",
                env!("CARGO_PKG_VERSION")
            )
        })
}

enum StdioTransportPref {
    Auto,
    MachineSsh,
    PodmanRun,
}

fn stdio_transport_pref() -> StdioTransportPref {
    match std::env::var(ENV_DEVSHELL_VM_STDIO_TRANSPORT) {
        Ok(s) if s.trim().eq_ignore_ascii_case("machine-ssh") => StdioTransportPref::MachineSsh,
        Ok(s) if s.trim().eq_ignore_ascii_case("podman-run") => StdioTransportPref::PodmanRun,
        _ => StdioTransportPref::Auto,
    }
}

fn find_host_elf_in_repos(workspace_root: &Path) -> Option<PathBuf> {
    let mut repos: Vec<PathBuf> = Vec::new();
    if let Some(p) = devshell_repo_root_with_containerfile() {
        push_repo_candidate(&mut repos, p);
    }
    if let Ok(s) = std::env::var(ENV_DEVSHELL_VM_REPO_ROOT) {
        let p = PathBuf::from(s.trim());
        if p.is_dir() {
            push_repo_candidate(&mut repos, p);
        }
    }
    if let Some(p) = devshell_repo_root_from_path(workspace_root) {
        push_repo_candidate(&mut repos, p);
    }
    if !workspace_parent_is_ephemeral_export(workspace_root) {
        push_repo_candidate(&mut repos, workspace_root.to_path_buf());
    }

    let rel = Path::new("target/x86_64-unknown-linux-gnu/release/devshell-vm");
    for wr in &repos {
        let p = wr.join(rel);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

fn machine_ssh_unavailable_err(workspace_root: &Path) -> VmError {
    let rel = Path::new("target/x86_64-unknown-linux-gnu/release/devshell-vm");
    let show = find_host_elf_in_repos(workspace_root)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| workspace_root.join(rel).display().to_string());
    VmError::Ipc(format!(
        "DEVSHELL_VM_STDIO_TRANSPORT=machine-ssh but no Linux devshell-vm ELF found (tried {show}).\n\
         Build: rustup target add x86_64-unknown-linux-gnu && cargo build -p devshell-vm --release --target x86_64-unknown-linux-gnu\n\
         Or unset {} to use automatic OCI image fallback, or set {}.",
        ENV_DEVSHELL_VM_STDIO_TRANSPORT,
        ENV_DEVSHELL_VM_LINUX_BINARY
    ))
}

pub(super) fn resolve_stdio_transport(
    workspace_root: &Path,
) -> Result<WindowsStdioTransport, VmError> {
    if let Ok(s) = std::env::var(ENV_DEVSHELL_VM_LINUX_BINARY) {
        let t = s.trim();
        if !t.is_empty() {
            let p = PathBuf::from(t);
            if p.is_file() {
                return Ok(WindowsStdioTransport::MachineSsh { host_bin: p });
            }
            return Err(VmError::Ipc(format!(
                "{} points to a missing file: {}",
                ENV_DEVSHELL_VM_LINUX_BINARY,
                p.display()
            )));
        }
    }

    match stdio_transport_pref() {
        StdioTransportPref::MachineSsh => find_host_elf_in_repos(workspace_root)
            .map(|host_bin| WindowsStdioTransport::MachineSsh { host_bin })
            .ok_or_else(|| machine_ssh_unavailable_err(workspace_root)),
        StdioTransportPref::PodmanRun => Ok(WindowsStdioTransport::PodmanRun {
            image: default_container_image(),
        }),
        StdioTransportPref::Auto => Ok(
            if let Some(host_bin) = find_host_elf_in_repos(workspace_root) {
                WindowsStdioTransport::MachineSsh { host_bin }
            } else {
                WindowsStdioTransport::PodmanRun {
                    image: default_container_image(),
                }
            },
        ),
    }
}

pub(super) fn stdio_guest_mount(workspace_parent: &Path) -> String {
    match resolve_stdio_transport(workspace_parent) {
        Ok(WindowsStdioTransport::MachineSsh { .. }) => {
            if let Ok(staging) = std::fs::canonicalize(workspace_parent) {
                if let Some(m) = windows_host_path_to_vm_mnt(&staging) {
                    return m;
                }
            }
            "/workspace".to_string()
        }
        Ok(WindowsStdioTransport::PodmanRun { .. }) => "/workspace".to_string(),
        Err(_) => "/workspace".to_string(),
    }
}

fn podman_pull(image: &str) -> Result<(), VmError> {
    let st = podman_command()
        .args(["pull", image])
        .status()
        .map_err(|e| VmError::Ipc(format!("podman pull {image}: {e}\n{MSG_PODMAN_INSTALL}")))?;
    if st.success() {
        return Ok(());
    }
    Err(VmError::Ipc(format!(
        "podman pull {image} failed (offline, auth, or image not published).\n\
         Options: set {} to a Linux devshell-vm ELF path, or {} to an image you can pull, or {}=machine-ssh with a built ELF.",
        ENV_DEVSHELL_VM_LINUX_BINARY,
        ENV_DEVSHELL_VM_CONTAINER_IMAGE,
        ENV_DEVSHELL_VM_STDIO_TRANSPORT
    )))
}

fn spawn_machine_ssh_elf(host_bin: &Path) -> Result<std::process::Child, VmError> {
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

fn spawn_podman_run_stdio(
    workspace_root: &Path,
    image: &str,
) -> Result<std::process::Child, VmError> {
    let ws = workspace_root.canonicalize().map_err(|e| {
        VmError::Ipc(format!(
            "canonicalize workspace {} for podman run: {e}",
            workspace_root.display()
        ))
    })?;
    let ws_s = ws
        .to_str()
        .ok_or_else(|| VmError::Ipc("workspace path is not valid UTF-8 for podman -v".into()))?;

    let mut cmd = podman_command();
    cmd.arg("run");
    cmd.arg("--rm");
    cmd.arg("-i");
    cmd.arg("--volume");
    cmd.arg(format!("{ws_s}:/workspace:Z"));
    cmd.arg("--workdir");
    cmd.arg("/workspace");
    cmd.arg(image);
    cmd.arg("/usr/local/bin/devshell-vm");
    cmd.arg("--serve-stdio");
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::inherit());

    cmd.spawn().map_err(|e| {
        VmError::Ipc(format!(
            "podman run (β OCI stdio): {e}\n{MSG_PODMAN_INSTALL}"
        ))
    })
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

    match resolve_stdio_transport(workspace_parent)? {
        WindowsStdioTransport::MachineSsh { .. } => Ok(()),
        WindowsStdioTransport::PodmanRun { image } => {
            eprintln!("dev_shell: β VM using OCI image (automatic fallback; no host devshell-vm ELF): {image}");
            podman_pull(&image)
        }
    }
}

pub(super) fn spawn_devshell_vm_stdio(
    workspace_root: &Path,
) -> Result<std::process::Child, VmError> {
    match resolve_stdio_transport(workspace_root)? {
        WindowsStdioTransport::MachineSsh { host_bin } => spawn_machine_ssh_elf(&host_bin),
        WindowsStdioTransport::PodmanRun { image } => {
            spawn_podman_run_stdio(workspace_root, &image)
        }
    }
}
