//! Windows β: JSON lines over **`devshell-vm --serve-stdio`** on stdio — **no host TCP**.
//! - **Preferred:** host Linux ELF + **`podman machine ssh -T`** (VM sees `/mnt/...`).
//! - **Automatic fallback** (e.g. `cargo install` with no checkout): **`podman run -i`** with a published OCI image
//!   and the workspace mounted at **`/workspace`** (see `docs/devshell-vm-windows.md`).

#![allow(clippy::pedantic, clippy::nursery)]

use std::path::Path;

use super::VmError;

#[cfg(windows)]
use std::path::PathBuf;

#[cfg(windows)]
mod win;

/// How Windows β attaches stdio when `DEVSHELL_VM_SOCKET=stdio`.
#[cfg(windows)]
#[derive(Debug, Clone)]
pub enum WindowsStdioTransport {
    /// Host ELF + `podman machine ssh` (guest staging path `/mnt/<drive>/...`).
    MachineSsh {
        /// Linux `devshell-vm` on the Windows filesystem.
        host_bin: PathBuf,
    },
    /// OCI image + `podman run -i` (guest staging path `/workspace`).
    PodmanRun {
        /// Image reference (default: `ghcr.io/.../devshell-vm:v{crate version}`).
        image: String,
    },
}

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

/// Maps a Windows path to the Podman Machine **`/mnt/<drive>/…`** form (for diagnostics or custom tooling).
#[cfg(windows)]
#[must_use]
#[allow(dead_code)] // Public API; not referenced from every workspace crate.
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

/// Resolves **machine-ssh** vs **podman-run** (OCI) stdio transport for the given workspace parent.
#[cfg(windows)]
#[allow(dead_code)] // Public API for embedders.
pub fn resolve_stdio_transport(workspace_parent: &Path) -> Result<WindowsStdioTransport, VmError> {
    win::resolve_stdio_transport(workspace_parent)
}

#[cfg(windows)]
pub fn stdio_guest_mount(workspace_parent: &Path) -> String {
    win::stdio_guest_mount(workspace_parent)
}
