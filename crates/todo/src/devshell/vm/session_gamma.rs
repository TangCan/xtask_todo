//! γ backend: [Lima](https://github.com/lima-vm/lima) via `limactl start` / `limactl shell`.
//!
//! The host directory [`GammaSession::workspace_parent`] must be mounted in the guest at
//! [`GammaSession::guest_mount`] (default `/workspace`). See `docs/devshell-vm-gamma.md`.

#![allow(clippy::pedantic, clippy::nursery)]

use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};

use super::super::sandbox;
use super::super::vfs::Vfs;
use super::lima_diagnostics;
use super::sync::{pull_workspace_to_vfs, push_incremental};
use super::{VmConfig, VmError, VmExecutionSession, WorkspaceMode};

/// Override path to `limactl` (default: `PATH`).
pub const ENV_DEVSHELL_VM_LIMACTL: &str = "DEVSHELL_VM_LIMACTL";

/// Host directory we push/pull (must be mounted at [`GammaSession::guest_mount`] in the Lima VM).
pub const ENV_DEVSHELL_VM_WORKSPACE_PARENT: &str = "DEVSHELL_VM_WORKSPACE_PARENT";

/// Guest mount point for that directory (default `/workspace`).
pub const ENV_DEVSHELL_VM_GUEST_WORKSPACE: &str = "DEVSHELL_VM_GUEST_WORKSPACE";

/// When set truthy, run `limactl stop` on session shutdown.
pub const ENV_DEVSHELL_VM_STOP_ON_EXIT: &str = "DEVSHELL_VM_STOP_ON_EXIT";

/// Lima-backed session: sync VFS ↔ host workspace, run tools inside the VM.
#[derive(Debug)]
pub struct GammaSession {
    lima_instance: String,
    /// Same layout as temp export: subtree leaves under this dir (see `sandbox::host_export_root`).
    workspace_parent: PathBuf,
    /// Guest path where `workspace_parent` is mounted.
    guest_mount: String,
    limactl: PathBuf,
    vm_started: bool,
    /// After first successful `limactl start`, run one guest/yaml diagnostic pass.
    lima_hints_checked: bool,
    /// When `true` (Mode S), push/pull VFS around each `cargo`/`rustup`. When `false` ([`WorkspaceMode::Guest`]), guest tree is authoritative — no sync (see guest-primary design §1c).
    sync_vfs_with_workspace: bool,
}

fn truthy_env(key: &str) -> bool {
    std::env::var(key)
        .map(|s| {
            let s = s.trim();
            s == "1" || s.eq_ignore_ascii_case("true") || s.eq_ignore_ascii_case("yes")
        })
        .unwrap_or(false)
}

fn resolve_limactl() -> Result<PathBuf, VmError> {
    if let Ok(p) = std::env::var(ENV_DEVSHELL_VM_LIMACTL) {
        let p = p.trim();
        if !p.is_empty() {
            return Ok(PathBuf::from(p));
        }
    }
    sandbox::find_in_path("limactl").ok_or_else(|| {
        VmError::Lima(
            "limactl not found in PATH; install Lima (https://lima-vm.io/) or set DEVSHELL_VM_LIMACTL"
                .to_string(),
        )
    })
}

fn sanitize_instance_segment(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Host workspace root shared with β (`session_start.staging_dir`).
#[must_use]
pub(crate) fn workspace_parent_for_instance(instance: &str) -> PathBuf {
    if let Ok(p) = std::env::var(ENV_DEVSHELL_VM_WORKSPACE_PARENT) {
        let p = p.trim();
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    let seg = sanitize_instance_segment(instance);
    sandbox::devshell_export_parent_dir()
        .join("vm-workspace")
        .join(seg)
}

/// Guest working directory for VFS cwd (mount + last path segment).
#[cfg_attr(not(feature = "beta-vm"), allow(dead_code))]
#[must_use]
pub(crate) fn guest_dir_for_vfs_cwd(guest_mount: &str, vfs_cwd: &str) -> String {
    super::guest_fs_ops::guest_project_dir_on_guest(guest_mount, vfs_cwd)
}

fn guest_dir_for_cwd_inner(guest_mount: &str, vfs_cwd: &str) -> String {
    super::guest_fs_ops::guest_project_dir_on_guest(guest_mount, vfs_cwd)
}

impl GammaSession {
    /// Build a γ session from VM config (does not start the VM yet).
    ///
    /// # Errors
    /// Returns [`VmError::Lima`] if `limactl` cannot be resolved.
    pub fn new(config: &VmConfig) -> Result<Self, VmError> {
        let limactl = resolve_limactl()?;
        let workspace_parent = workspace_parent_for_instance(&config.lima_instance);
        let guest_mount = std::env::var(ENV_DEVSHELL_VM_GUEST_WORKSPACE)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "/workspace".to_string());

        let sync_vfs_with_workspace =
            matches!(config.workspace_mode_effective(), WorkspaceMode::Sync);

        Ok(Self {
            lima_instance: config.lima_instance.clone(),
            workspace_parent,
            guest_mount,
            limactl,
            vm_started: false,
            lima_hints_checked: false,
            sync_vfs_with_workspace,
        })
    }

    /// Whether this session push/pulls the in-memory VFS around rust tools (Mode S). `false` = guest-primary ([`WorkspaceMode::Guest`]).
    #[must_use]
    pub fn syncs_vfs_with_host_workspace(&self) -> bool {
        self.sync_vfs_with_workspace
    }

    fn limactl_ensure_running(&mut self) -> Result<(), VmError> {
        if self.vm_started {
            return Ok(());
        }
        std::fs::create_dir_all(&self.workspace_parent).map_err(|e| {
            VmError::Lima(format!(
                "create workspace dir {}: {e}",
                self.workspace_parent.display()
            ))
        })?;

        // `-y` / non-TUI: devshell runs `limactl` as a child without a usable TTY; Lima's TUI
        // otherwise fails with EOF and confuses first-time create flows.
        let st = Command::new(&self.limactl)
            .args(["start", "-y", &self.lima_instance])
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| VmError::Lima(format!("limactl start: {e}")))?;

        if !st.success() {
            lima_diagnostics::emit_start_failure_hints(&self.lima_instance);
            return Err(VmError::Lima(format!(
                "limactl start '{}' failed (exit code {:?}); check instance name and `limactl list`",
                self.lima_instance,
                st.code()
            )));
        }
        self.vm_started = true;
        Ok(())
    }

    fn limactl_shell(
        &self,
        guest_workdir: &str,
        program: &str,
        args: &[String],
    ) -> Result<ExitStatus, VmError> {
        let st = Command::new(&self.limactl)
            .arg("shell")
            .arg("--workdir")
            .arg(guest_workdir)
            .arg(&self.lima_instance)
            .arg("--")
            .arg(program)
            .args(args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| VmError::Lima(format!("limactl shell: {e}")))?;
        Ok(st)
    }

    /// Run `limactl shell … -- program args` with captured stdout/stderr (for guest FS ops / Mode P).
    ///
    /// Starts the VM on first use (same as interactive `limactl shell`).
    pub(crate) fn limactl_shell_output(
        &mut self,
        guest_workdir: &str,
        program: &str,
        args: &[String],
    ) -> Result<std::process::Output, VmError> {
        self.limactl_ensure_running()?;
        Command::new(&self.limactl)
            .arg("shell")
            .arg("--workdir")
            .arg(guest_workdir)
            .arg(&self.lima_instance)
            .arg("--")
            .arg(program)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| VmError::Lima(format!("limactl shell: {e}")))
    }

    /// Pipe `stdin_data` to a guest process stdin (e.g. `dd of=…`).
    pub(crate) fn limactl_shell_stdin(
        &mut self,
        guest_workdir: &str,
        program: &str,
        args: &[String],
        stdin_data: &[u8],
    ) -> Result<std::process::Output, VmError> {
        self.limactl_ensure_running()?;
        use std::io::Write;
        let mut child = Command::new(&self.limactl)
            .arg("shell")
            .arg("--workdir")
            .arg(guest_workdir)
            .arg(&self.lima_instance)
            .arg("--")
            .arg(program)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| VmError::Lima(format!("limactl shell: {e}")))?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(stdin_data)
                .map_err(|e| VmError::Lima(format!("limactl shell: write stdin: {e}")))?;
        }
        child
            .wait_with_output()
            .map_err(|e| VmError::Lima(format!("limactl shell: {e}")))
    }

    /// Guest mount point (e.g. `/workspace`).
    #[must_use]
    pub fn guest_mount(&self) -> &str {
        &self.guest_mount
    }

    fn limactl_stop(&self) -> Result<(), VmError> {
        let st = Command::new(&self.limactl)
            .args(["stop", &self.lima_instance])
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| VmError::Lima(format!("limactl stop: {e}")))?;
        if !st.success() {
            return Err(VmError::Lima(format!(
                "limactl stop '{}' failed (exit code {:?})",
                self.lima_instance,
                st.code()
            )));
        }
        Ok(())
    }

    /// Host workspace root (for docs / debugging).
    #[must_use]
    pub fn workspace_parent(&self) -> &Path {
        &self.workspace_parent
    }
}

impl VmExecutionSession for GammaSession {
    fn ensure_ready(&mut self, _vfs: &Vfs, _vfs_cwd: &str) -> Result<(), VmError> {
        self.limactl_ensure_running()
    }

    fn run_rust_tool(
        &mut self,
        vfs: &mut Vfs,
        vfs_cwd: &str,
        program: &str,
        args: &[String],
    ) -> Result<ExitStatus, VmError> {
        self.limactl_ensure_running()?;
        if self.sync_vfs_with_workspace {
            push_incremental(vfs, vfs_cwd, &self.workspace_parent).map_err(VmError::Sync)?;
        }

        let guest_dir = guest_dir_for_cwd_inner(&self.guest_mount, vfs_cwd);
        if !self.lima_hints_checked {
            self.lima_hints_checked = true;
            lima_diagnostics::warn_if_guest_misconfigured(
                &self.limactl,
                &self.lima_instance,
                &self.workspace_parent,
                &self.guest_mount,
                &guest_dir,
            );
        }

        let status = self.limactl_shell(&guest_dir, program, args)?;

        if !status.success() && (program == "cargo" || program == "rustup") {
            lima_diagnostics::emit_tool_failure_hints(
                &self.limactl,
                &self.lima_instance,
                &self.workspace_parent,
                &self.guest_mount,
                &guest_dir,
                program,
                &status,
            );
        }

        if self.sync_vfs_with_workspace {
            if let Err(e) = pull_workspace_to_vfs(&self.workspace_parent, vfs_cwd, vfs) {
                eprintln!(
                    "dev_shell: warning: vm workspace pull failed after `{program}` (VFS may be stale): {e}"
                );
            }
        }

        Ok(status)
    }

    fn shutdown(&mut self, vfs: &mut Vfs, vfs_cwd: &str) -> Result<(), VmError> {
        if self.vm_started && self.sync_vfs_with_workspace {
            if let Err(e) = pull_workspace_to_vfs(&self.workspace_parent, vfs_cwd, vfs) {
                return Err(VmError::Sandbox(e));
            }
        }
        if truthy_env(ENV_DEVSHELL_VM_STOP_ON_EXIT) {
            let _ = self.limactl_stop();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guest_dir_for_cwd_root() {
        assert_eq!(guest_dir_for_cwd_inner("/workspace", "/"), "/workspace");
    }

    #[test]
    fn guest_dir_for_cwd_nested() {
        assert_eq!(
            guest_dir_for_cwd_inner("/workspace", "/projects/hello"),
            "/workspace/hello"
        );
    }

    #[test]
    fn sanitize_instance_replaces_dots() {
        assert_eq!(sanitize_instance_segment("a.b"), "a_b");
    }

    /// When `DEVSHELL_VM_WORKSPACE_MODE=guest` and γ is available, push/pull is disabled for rust tools.
    #[cfg(unix)]
    #[test]
    fn gamma_session_sync_flag_follows_workspace_mode() {
        use std::sync::{Mutex, OnceLock};

        use crate::devshell::sandbox;
        use crate::devshell::vm::{
            GammaSession, VmConfig, ENV_DEVSHELL_VM, ENV_DEVSHELL_VM_BACKEND,
            ENV_DEVSHELL_VM_WORKSPACE_MODE,
        };

        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let _g = LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();

        if sandbox::find_in_path("limactl").is_none() {
            return;
        }

        let old_wm = std::env::var(ENV_DEVSHELL_VM_WORKSPACE_MODE).ok();
        let old_vm = std::env::var(ENV_DEVSHELL_VM).ok();
        let old_b = std::env::var(ENV_DEVSHELL_VM_BACKEND).ok();

        std::env::set_var(ENV_DEVSHELL_VM, "1");
        std::env::set_var(ENV_DEVSHELL_VM_BACKEND, "lima");
        std::env::set_var(ENV_DEVSHELL_VM_WORKSPACE_MODE, "guest");
        let c = VmConfig::from_env();
        let g = GammaSession::new(&c).expect("gamma");
        assert!(
            !g.syncs_vfs_with_host_workspace(),
            "guest mode should skip VFS sync"
        );

        std::env::set_var(ENV_DEVSHELL_VM_WORKSPACE_MODE, "sync");
        let c2 = VmConfig::from_env();
        let g2 = GammaSession::new(&c2).expect("gamma");
        assert!(g2.syncs_vfs_with_host_workspace());

        match old_wm {
            Some(ref v) => std::env::set_var(ENV_DEVSHELL_VM_WORKSPACE_MODE, v),
            None => std::env::remove_var(ENV_DEVSHELL_VM_WORKSPACE_MODE),
        }
        match old_vm {
            Some(ref v) => std::env::set_var(ENV_DEVSHELL_VM, v),
            None => std::env::remove_var(ENV_DEVSHELL_VM),
        }
        match old_b {
            Some(ref v) => std::env::set_var(ENV_DEVSHELL_VM_BACKEND, v),
            None => std::env::remove_var(ENV_DEVSHELL_VM_BACKEND),
        }
    }
}
