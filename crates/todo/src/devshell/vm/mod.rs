//! Optional session-scoped VM execution (γ CLI / β sidecar): host [`SessionHolder::Host`], Unix γ [`SessionHolder::Gamma`].

#![allow(clippy::pedantic, clippy::nursery)]

use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

mod config;
mod guest_fs_ops;
#[cfg(unix)]
mod lima_diagnostics;
#[cfg(all(windows, feature = "beta-vm"))]
mod podman_sidecar;
#[cfg(feature = "beta-vm")]
mod session_beta;
#[cfg(unix)]
mod session_gamma;
mod session_host;
pub mod sync;
mod workspace_host;

pub use config::{
    workspace_mode_from_env, VmConfig, WorkspaceMode, ENV_DEVSHELL_VM, ENV_DEVSHELL_VM_BACKEND,
    ENV_DEVSHELL_VM_BETA_SESSION_STAGING, ENV_DEVSHELL_VM_DISABLE_PODMAN_SSH_HOME,
    ENV_DEVSHELL_VM_EAGER, ENV_DEVSHELL_VM_LIMA_INSTANCE, ENV_DEVSHELL_VM_SKIP_PODMAN_BOOTSTRAP,
    ENV_DEVSHELL_VM_SOCKET, ENV_DEVSHELL_VM_WORKSPACE_MODE,
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
    GammaSession, ENV_DEVSHELL_VM_AUTO_BUILD_ESSENTIAL, ENV_DEVSHELL_VM_AUTO_BUILD_TODO_GUEST,
    ENV_DEVSHELL_VM_AUTO_TODO_PATH, ENV_DEVSHELL_VM_GUEST_HOST_DIR,
    ENV_DEVSHELL_VM_GUEST_TODO_HINT, ENV_DEVSHELL_VM_GUEST_WORKSPACE, ENV_DEVSHELL_VM_LIMACTL,
    ENV_DEVSHELL_VM_STOP_ON_EXIT, ENV_DEVSHELL_VM_WORKSPACE_PARENT,
    ENV_DEVSHELL_VM_WORKSPACE_USE_CARGO_ROOT,
};
pub use session_host::HostSandboxSession;
pub use sync::{pull_workspace_to_vfs, push_full, push_incremental, VmSyncError};
pub use workspace_host::workspace_parent_for_instance;

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
    /// β: JSON-lines IPC to `devshell-vm` (Unix socket or TCP; `beta-vm` feature).
    #[cfg(feature = "beta-vm")]
    Beta(session_beta::BetaSession),
}

/// Single-quoted POSIX shell word (safe for `export PATH=…`).
#[cfg(unix)]
pub(crate) fn bash_single_quoted(s: &str) -> String {
    let mut o = String::from("'");
    for c in s.chars() {
        if c == '\'' {
            o.push_str("'\"'\"'");
        } else {
            o.push(c);
        }
    }
    o.push('\'');
    o
}

#[cfg(all(unix, test))]
mod bash_single_quoted_tests {
    use super::bash_single_quoted;

    #[test]
    fn wraps_plain_path() {
        assert_eq!(
            bash_single_quoted("/workspace/p/target/release"),
            "'/workspace/p/target/release'"
        );
    }
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
        #[cfg(feature = "beta-vm")]
        if config.backend.eq_ignore_ascii_case("beta") {
            return session_beta::BetaSession::new(config).map(SessionHolder::Beta);
        }
        #[cfg(not(feature = "beta-vm"))]
        if config.backend.eq_ignore_ascii_case("beta") {
            return Err(VmError::BackendNotImplemented(
                "DEVSHELL_VM_BACKEND=beta requires building xtask-todo-lib with `--features beta-vm`",
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
            #[cfg(feature = "beta-vm")]
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
            #[cfg(feature = "beta-vm")]
            Self::Beta(s) => VmExecutionSession::run_rust_tool(s, vfs, vfs_cwd, program, args),
        }
    }

    pub fn shutdown(&mut self, vfs: &mut Vfs, vfs_cwd: &str) -> Result<(), VmError> {
        match self {
            Self::Host(s) => VmExecutionSession::shutdown(s, vfs, vfs_cwd),
            #[cfg(unix)]
            Self::Gamma(s) => VmExecutionSession::shutdown(s, vfs, vfs_cwd),
            #[cfg(feature = "beta-vm")]
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

    /// γ **or** β guest-primary: [`GuestFsOps`] + guest mount for [`crate::devshell::workspace::logical_path_to_guest`].
    ///
    /// Returns `None` for host sandbox, Mode S sync, or non–guest-primary sessions.
    /// Mount is owned so the returned trait object does not alias a borrow of the session.
    #[must_use]
    pub fn guest_primary_fs_ops_mut(&mut self) -> Option<(&mut dyn GuestFsOps, String)> {
        match self {
            #[cfg(unix)]
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
        match self {
            #[cfg(unix)]
            Self::Gamma(g) if !g.syncs_vfs_with_host_workspace() => true,
            #[cfg(feature = "beta-vm")]
            Self::Beta(b) if !b.syncs_vfs_with_host_workspace() => true,
            _ => false,
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

    /// `true` when using the host temp sandbox ([`HostSandboxSession`]) rather than γ/β.
    #[must_use]
    pub fn is_host_only(&self) -> bool {
        matches!(self, Self::Host(_))
    }

    /// Replace this process with an interactive `limactl shell` (`bash -l`) under the guest workspace mount.
    ///
    /// On success, does not return. On failure, returns the [`std::io::Error`] from [`std::os::unix::process::CommandExt::exec`].
    #[cfg(unix)]
    pub fn exec_lima_interactive_shell(&self) -> std::io::Error {
        use std::os::unix::process::CommandExt;
        use std::process::Command;
        match self {
            Self::Gamma(g) => {
                let (workdir, inner) = g.lima_interactive_shell_workdir_and_inner();
                Command::new(g.limactl_path())
                    .arg("shell")
                    .arg("-y")
                    .arg("--workdir")
                    .arg(workdir)
                    .arg(g.lima_instance_name())
                    .arg("--")
                    .arg("bash")
                    .arg("-lc")
                    .arg(inner)
                    .exec()
            }
            _ => std::io::Error::other("exec_lima_interactive_shell: not a Lima gamma session"),
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

/// Like [`try_session_rc`], but on failure uses [`SessionHolder::Host`] so the REPL can run against
/// [`workspace_parent_for_instance`] (same tree as the Lima mount).
pub fn try_session_rc_or_host(stderr: &mut dyn Write) -> Rc<RefCell<SessionHolder>> {
    match try_session_rc(stderr) {
        Ok(s) => s,
        Err(()) => {
            let _ = writeln!(
                stderr,
                "dev_shell: VM unavailable — in-process REPL uses the same host directory as the Lima workspace (DEVSHELL_WORKSPACE_ROOT)."
            );
            Rc::new(RefCell::new(SessionHolder::Host(HostSandboxSession::new())))
        }
    }
}

#[cfg(unix)]
pub fn export_devshell_workspace_root_env() {
    #[cfg(test)]
    let _workspace_env_test_guard = crate::test_support::devshell_workspace_env_mutex();
    let c = config::VmConfig::from_env();
    let p = session_gamma::workspace_parent_for_instance(&c.lima_instance);
    let _ = std::fs::create_dir_all(&p);
    if let Ok(can) = p.canonicalize() {
        std::env::set_var("DEVSHELL_WORKSPACE_ROOT", can.as_os_str());
    }
}

#[cfg(not(unix))]
pub fn export_devshell_workspace_root_env() {}

/// Host directory that Lima mounts at the guest workspace (e.g. `/workspace`).
#[cfg(unix)]
#[must_use]
pub fn vm_workspace_host_root() -> std::path::PathBuf {
    let c = config::VmConfig::from_env();
    session_gamma::workspace_parent_for_instance(&c.lima_instance)
}

/// Stub for non-Unix targets: `devshell/mod.rs` uses `if cfg!(unix) && …` but the branch is still
/// type-checked; this is never called when `cfg!(unix)` is false.
#[cfg(not(unix))]
#[must_use]
pub fn vm_workspace_host_root() -> std::path::PathBuf {
    std::path::PathBuf::new()
}

#[cfg(unix)]
#[must_use]
pub fn should_delegate_lima_shell(
    vm_session: &Rc<RefCell<SessionHolder>>,
    is_tty: bool,
    run_script: bool,
) -> bool {
    if run_script || !is_tty {
        return false;
    }
    if std::env::var("DEVSHELL_VM_INTERNAL_REPL")
        .map(|s| {
            let s = s.trim();
            s == "1" || s.eq_ignore_ascii_case("true") || s.eq_ignore_ascii_case("yes")
        })
        .unwrap_or(false)
    {
        return false;
    }
    matches!(*vm_session.borrow(), SessionHolder::Gamma(_))
}
