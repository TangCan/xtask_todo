//! Windows β: JSON lines over **`devshell-vm --serve-stdio`** on stdio — **no host TCP**.
//! - **Preferred:** host Linux ELF + **`podman machine ssh -T`** (VM sees `/mnt/...`).
//! - **Automatic fallback** (e.g. `cargo install` with no checkout): **`podman run -i`** with a published OCI image
//!   and the workspace mounted at **`/workspace`** (see `docs/devshell-vm-windows.md`).

#[cfg(windows)]
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

#[cfg(windows)]
pub fn ensure(workspace_parent: &std::path::Path) -> Result<(), VmError> {
    win::ensure(workspace_parent)
}

#[cfg(windows)]
pub fn spawn_devshell_vm_stdio(
    workspace_root: &std::path::Path,
) -> Result<std::process::Child, VmError> {
    win::spawn_devshell_vm_stdio(workspace_root)
}

#[cfg(windows)]
pub fn stdio_guest_mount(workspace_parent: &std::path::Path) -> String {
    win::stdio_guest_mount(workspace_parent)
}
