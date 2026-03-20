//! Optional session-scoped VM execution (γ CLI / β sidecar): host [`SessionHolder::Host`], Unix γ [`SessionHolder::Gamma`].

#![allow(clippy::pedantic, clippy::nursery)]

use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

mod config;
mod guest_fs_ops;
#[cfg(unix)]
mod lima_diagnostics;
#[cfg(all(unix, feature = "beta-vm"))]
mod session_beta;
#[cfg(unix)]
mod session_gamma;
mod session_host;
pub mod sync;

pub use config::{
    workspace_mode_from_env, VmConfig, WorkspaceMode, ENV_DEVSHELL_VM, ENV_DEVSHELL_VM_BACKEND,
    ENV_DEVSHELL_VM_EAGER, ENV_DEVSHELL_VM_LIMA_INSTANCE, ENV_DEVSHELL_VM_SOCKET,
    ENV_DEVSHELL_VM_WORKSPACE_MODE,
};
#[cfg(unix)]
pub use guest_fs_ops::LimaGuestFsOps;
pub use guest_fs_ops::{
    guest_path_is_under_mount, guest_project_dir_on_guest, normalize_guest_path, GuestFsError,
    GuestFsOps, MockGuestFsOps,
};
#[cfg(unix)]
pub use lima_diagnostics::ENV_DEVSHELL_VM_LIMA_HINTS;
#[cfg(unix)]
pub use session_gamma::{
    GammaSession, ENV_DEVSHELL_VM_GUEST_WORKSPACE, ENV_DEVSHELL_VM_LIMACTL,
    ENV_DEVSHELL_VM_STOP_ON_EXIT, ENV_DEVSHELL_VM_WORKSPACE_PARENT,
};
pub use session_host::HostSandboxSession;
pub use sync::{pull_workspace_to_vfs, push_full, push_incremental, VmSyncError};

use std::process::ExitStatus;

use super::sandbox;
use super::vfs::Vfs;

/// Errors from VM session operations.
#[derive(Debug)]
pub enum VmError {
    Sandbox(sandbox::SandboxError),
    Sync(VmSyncError),
    /// Backend not implemented on this OS or not wired yet.
    BackendNotImplemented(&'static str),
    /// Lima / `limactl` or γ orchestration failure (message for stderr).
    Lima(String),
    /// β IPC / `devshell-vm` protocol failure.
    Ipc(String),
}

impl std::fmt::Display for VmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sandbox(e) => write!(f, "{e}"),
            Self::Sync(e) => write!(f, "{e}"),
            Self::BackendNotImplemented(s) => write!(f, "vm backend not implemented: {s}"),
            Self::Lima(s) => f.write_str(s),
            Self::Ipc(s) => f.write_str(s),
        }
    }
}

impl std::error::Error for VmError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Sandbox(e) => Some(e),
            Self::Sync(e) => Some(e),
            Self::BackendNotImplemented(_) | Self::Lima(_) | Self::Ipc(_) => None,
        }
    }
}

/// Abstraction for a devshell execution session (host temp dir, γ VM, or β sidecar).
pub trait VmExecutionSession {
    /// Prepare the session (e.g. start VM, initial push). No-op for host temp export.
    fn ensure_ready(&mut self, vfs: &Vfs, vfs_cwd: &str) -> Result<(), VmError>;

    /// Run `rustup` or `cargo` with cwd matching `vfs_cwd`; update `vfs` as defined by the backend.
    fn run_rust_tool(
        &mut self,
        vfs: &mut Vfs,
        vfs_cwd: &str,
        program: &str,
        args: &[String],
    ) -> Result<ExitStatus, VmError>;

    /// Tear down (e.g. final pull, stop VM).
    fn shutdown(&mut self, vfs: &mut Vfs, vfs_cwd: &str) -> Result<(), VmError>;
}

/// Active VM / sandbox backend for one REPL or script run.
#[derive(Debug)]
pub enum SessionHolder {
    Host(HostSandboxSession),
    /// γ: Lima + host workspace sync (Unix only).
    #[cfg(unix)]
    Gamma(GammaSession),
    /// β: JSON-lines over Unix socket to `devshell-vm` (Unix + `beta-vm` feature).
    #[cfg(all(unix, feature = "beta-vm"))]
    Beta(session_beta::BetaSession),
}

impl SessionHolder {
    /// Build session from config.
    ///
    /// # Errors
    /// On Unix, `DEVSHELL_VM_BACKEND=lima` uses [`GammaSession`]; fails with [`VmError::Lima`] if `limactl` is missing.
    /// On non-Unix, `lima` returns [`VmError::BackendNotImplemented`].
    pub fn try_from_config(config: &VmConfig) -> Result<Self, VmError> {
        if !config.enabled {
            return Ok(Self::Host(HostSandboxSession::new()));
        }
        if config.use_host_sandbox() {
            return Ok(Self::Host(HostSandboxSession::new()));
        }
        #[cfg(all(unix, feature = "beta-vm"))]
        if config.backend.eq_ignore_ascii_case("beta") {
            return session_beta::BetaSession::new(config).map(SessionHolder::Beta);
        }
        #[cfg(not(all(unix, feature = "beta-vm")))]
        if config.backend.eq_ignore_ascii_case("beta") {
            return Err(VmError::BackendNotImplemented(
                "DEVSHELL_VM_BACKEND=beta requires Unix and building xtask-todo-lib with `--features beta-vm`",
            ));
        }
        #[cfg(unix)]
        if config.backend.eq_ignore_ascii_case("lima") {
            return GammaSession::new(config).map(SessionHolder::Gamma);
        }
        #[cfg(not(unix))]
        if config.backend.eq_ignore_ascii_case("lima") {
            return Err(VmError::BackendNotImplemented(
                "lima backend is only supported on Linux and macOS",
            ));
        }
        Err(VmError::BackendNotImplemented(
            "unknown DEVSHELL_VM_BACKEND (try host, auto, lima, or beta); see docs/devshell-vm-gamma.md",
        ))
    }

    /// Host sandbox only (tests and callers that do not read `VmConfig`).
    #[must_use]
    pub fn new_host() -> Self {
        Self::Host(HostSandboxSession::new())
    }

    pub fn ensure_ready(&mut self, vfs: &Vfs, vfs_cwd: &str) -> Result<(), VmError> {
        match self {
            Self::Host(s) => VmExecutionSession::ensure_ready(s, vfs, vfs_cwd),
            #[cfg(unix)]
            Self::Gamma(s) => VmExecutionSession::ensure_ready(s, vfs, vfs_cwd),
            #[cfg(all(unix, feature = "beta-vm"))]
            Self::Beta(s) => VmExecutionSession::ensure_ready(s, vfs, vfs_cwd),
        }
    }

    pub fn run_rust_tool(
        &mut self,
        vfs: &mut Vfs,
        vfs_cwd: &str,
        program: &str,
        args: &[String],
    ) -> Result<ExitStatus, VmError> {
        match self {
            Self::Host(s) => VmExecutionSession::run_rust_tool(s, vfs, vfs_cwd, program, args),
            #[cfg(unix)]
            Self::Gamma(s) => VmExecutionSession::run_rust_tool(s, vfs, vfs_cwd, program, args),
            #[cfg(all(unix, feature = "beta-vm"))]
            Self::Beta(s) => VmExecutionSession::run_rust_tool(s, vfs, vfs_cwd, program, args),
        }
    }

    pub fn shutdown(&mut self, vfs: &mut Vfs, vfs_cwd: &str) -> Result<(), VmError> {
        match self {
            Self::Host(s) => VmExecutionSession::shutdown(s, vfs, vfs_cwd),
            #[cfg(unix)]
            Self::Gamma(s) => VmExecutionSession::shutdown(s, vfs, vfs_cwd),
            #[cfg(all(unix, feature = "beta-vm"))]
            Self::Beta(s) => VmExecutionSession::shutdown(s, vfs, vfs_cwd),
        }
    }

    /// When γ is in guest-primary mode ([`WorkspaceMode::Guest`]), returns the session for direct guest FS ops.
    ///
    /// Returns `None` for host sandbox, β, or Mode S γ (push/pull sync).
    #[cfg(unix)]
    #[must_use]
    pub fn guest_primary_gamma_mut(&mut self) -> Option<&mut GammaSession> {
        match self {
            Self::Gamma(g) if !g.syncs_vfs_with_host_workspace() => Some(g),
            _ => None,
        }
    }

    /// γ **or** β guest-primary: [`GuestFsOps`] + guest mount for [`logical_path_to_guest`].
    ///
    /// Returns `None` for host sandbox, Mode S sync, or non–guest-primary sessions.
    /// Mount is owned so the returned trait object does not alias a borrow of the session.
    #[cfg(unix)]
    #[must_use]
    pub fn guest_primary_fs_ops_mut(&mut self) -> Option<(&mut dyn GuestFsOps, String)> {
        match self {
            Self::Gamma(g) if !g.syncs_vfs_with_host_workspace() => {
                let mount = g.guest_mount().to_string();
                Some((g as &mut dyn GuestFsOps, mount))
            }
            #[cfg(feature = "beta-vm")]
            Self::Beta(b) if !b.syncs_vfs_with_host_workspace() => {
                let mount = b.guest_mount().to_string();
                Some((b as &mut dyn GuestFsOps, mount))
            }
            _ => None,
        }
    }

    /// `true` when **any** VM session runs in guest-primary mode (γ or β: no VFS↔host project-tree sync).
    #[must_use]
    pub fn is_guest_primary(&self) -> bool {
        #[cfg(unix)]
        {
            match self {
                Self::Gamma(g) if !g.syncs_vfs_with_host_workspace() => true,
                #[cfg(feature = "beta-vm")]
                Self::Beta(b) if !b.syncs_vfs_with_host_workspace() => true,
                _ => false,
            }
        }
        #[cfg(not(unix))]
        {
            false
        }
    }

    /// `true` when γ runs in guest-primary mode (no VFS↔host project-tree sync).
    #[must_use]
    pub fn is_guest_primary_gamma(&self) -> bool {
        #[cfg(unix)]
        {
            matches!(
                self,
                Self::Gamma(g) if !g.syncs_vfs_with_host_workspace()
            )
        }
        #[cfg(not(unix))]
        {
            false
        }
    }
}

/// Build [`SessionHolder`] from the environment. On failure (e.g. default γ Lima but `limactl` missing), writes to `stderr` and returns `Err(())`.
/// Use **`DEVSHELL_VM=off`** or **`DEVSHELL_VM_BACKEND=host`** to force the host temp sandbox.
#[allow(clippy::result_unit_err)] // binary entry uses `()`; message already on stderr
pub fn try_session_rc(stderr: &mut dyn Write) -> Result<Rc<RefCell<SessionHolder>>, ()> {
    let config = VmConfig::from_env();
    match SessionHolder::try_from_config(&config) {
        Ok(s) => Ok(Rc::new(RefCell::new(s))),
        Err(e) => {
            let _ = writeln!(stderr, "dev_shell: {e}");
            Err(())
        }
    }
}
