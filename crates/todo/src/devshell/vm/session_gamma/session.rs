//! [`GammaSession`] struct and Lima `limactl` helpers.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};

use super::super::super::vfs::Vfs;
use super::super::lima_diagnostics;
use super::super::sync::{pull_workspace_to_vfs, push_incremental};
use super::super::{VmConfig, VmError, VmExecutionSession, WorkspaceMode};
use super::env::ENV_DEVSHELL_VM_GUEST_WORKSPACE;
use super::helpers::{
    auto_build_essential_enabled, guest_dir_for_cwd_inner, resolve_limactl,
    workspace_parent_for_instance,
};

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
    /// After first `ensure_ready`, skip repeating guest C toolchain probe/install.
    guest_build_essential_done: bool,
    /// When `true` (Mode S), push/pull VFS around each `cargo`/`rustup`. When `false` ([`WorkspaceMode::Guest`]), guest tree is authoritative — no sync (see guest-primary design §1c).
    sync_vfs_with_workspace: bool,
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
            guest_build_essential_done: false,
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

    /// Non-interactive `limactl shell -y … -- /bin/sh -c …` with captured output (VM started first).
    fn limactl_shell_script_sh(
        &mut self,
        guest_workdir: &str,
        sh_script: &str,
    ) -> Result<std::process::Output, VmError> {
        self.limactl_ensure_running()?;
        Command::new(&self.limactl)
            .arg("shell")
            .arg("-y")
            .arg("--workdir")
            .arg(guest_workdir)
            .arg(&self.lima_instance)
            .arg("--")
            .arg("/bin/sh")
            .arg("-c")
            .arg(sh_script)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| VmError::Lima(format!("limactl shell: {e}")))
    }

    /// If guest has no `gcc`, try Debian/Ubuntu `apt-get install -y build-essential` (non-interactive sudo).
    fn maybe_ensure_guest_build_essential(&mut self) -> Result<(), VmError> {
        if self.guest_build_essential_done {
            return Ok(());
        }

        if !auto_build_essential_enabled() {
            self.guest_build_essential_done = true;
            return Ok(());
        }

        let probe = self.limactl_shell_script_sh("/", "command -v gcc >/dev/null 2>&1")?;
        if probe.status.success() {
            self.guest_build_essential_done = true;
            return Ok(());
        }

        eprintln!("dev_shell: guest: no C compiler (gcc) in PATH; attempting apt install build-essential…");

        let has_apt =
            self.limactl_shell_script_sh("/", "test -x /usr/bin/apt-get && test -x /usr/bin/dpkg")?;
        if !has_apt.status.success() {
            eprintln!(
                "dev_shell: guest: no apt-get/dpkg; install gcc + binutils manually (see docs/devshell-vm-gamma.md)."
            );
            self.guest_build_essential_done = true;
            return Ok(());
        }

        // `sudo -n` fails if a password is required (no TTY here).
        const INSTALL_SH: &str = r"set -e
export DEBIAN_FRONTEND=noninteractive
if ! sudo -n true 2>/dev/null; then
  echo 'dev_shell: guest: sudo needs a password; run in the VM: sudo apt update && sudo apt install -y build-essential' >&2
  exit 1
fi
sudo apt-get update -qq
sudo apt-get install -y -qq build-essential
";

        let out = self.limactl_shell_script_sh("/", INSTALL_SH)?;
        if out.status.success() {
            eprintln!(
                "dev_shell: guest: build-essential installed (gcc available for cargo link)."
            );
        } else {
            let stderr = String::from_utf8_lossy(&out.stderr);
            let stdout = String::from_utf8_lossy(&out.stdout);
            eprintln!(
                "dev_shell: guest: automatic build-essential install failed (exit {:?}).",
                out.status.code()
            );
            if !stdout.trim().is_empty() {
                eprintln!("dev_shell: guest stdout: {stdout}");
            }
            if !stderr.trim().is_empty() {
                eprintln!("dev_shell: guest stderr: {stderr}");
            }
            eprintln!(
                "dev_shell: hint: in the guest shell: sudo apt update && sudo apt install -y build-essential"
            );
        }
        self.guest_build_essential_done = true;
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

    /// Path to `limactl` binary (for `exec` delegation).
    #[must_use]
    pub fn limactl_path(&self) -> &Path {
        &self.limactl
    }

    /// Lima instance name (`limactl shell <this> …`).
    #[must_use]
    pub fn lima_instance_name(&self) -> &str {
        &self.lima_instance
    }
}

impl VmExecutionSession for GammaSession {
    fn ensure_ready(&mut self, _vfs: &Vfs, _vfs_cwd: &str) -> Result<(), VmError> {
        self.limactl_ensure_running()?;
        self.maybe_ensure_guest_build_essential()?;
        Ok(())
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
        use super::env::ENV_DEVSHELL_VM_STOP_ON_EXIT;
        use super::helpers::truthy_env;

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
