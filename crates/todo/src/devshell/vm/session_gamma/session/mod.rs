//! [`GammaSession`] struct and Lima `limactl` helpers.

mod exec;

use std::fmt::Write as _;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};

use super::super::bash_single_quoted;
use super::super::lima_diagnostics;
use super::super::{VmConfig, VmError, WorkspaceMode};
use super::env::ENV_DEVSHELL_VM_GUEST_WORKSPACE;
use super::helpers::{
    guest_dir_for_host_path_under_workspace, guest_host_dir_link_name,
    guest_todo_release_dir_for_cwd, resolve_limactl, workspace_parent_for_instance,
};

/// Lima-backed session: sync VFS ↔ host workspace, run tools inside the VM.
#[derive(Debug)]
#[allow(clippy::struct_excessive_bools)]
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
    /// After first `ensure_ready`, skip repeating guest `todo` probe / install hints.
    guest_todo_hint_done: bool,
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
            guest_todo_hint_done: false,
            sync_vfs_with_workspace,
        })
    }

    /// Whether this session push/pulls the in-memory VFS around rust tools (Mode S). `false` = guest-primary ([`WorkspaceMode::Guest`]).
    #[must_use]
    pub const fn syncs_vfs_with_host_workspace(&self) -> bool {
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

    /// If host `cwd` has `target/release/todo` under [`Self::workspace_parent`], return guest path
    /// to that `release` dir for `PATH` (same logic as `guest_todo_release_dir_for_cwd` in session helpers).
    #[must_use]
    pub fn guest_todo_release_path_for_shell(&self) -> Option<String> {
        let cwd = std::env::current_dir().ok()?;
        guest_todo_release_dir_for_cwd(&self.workspace_parent, &self.guest_mount, &cwd)
    }

    /// Guest `--workdir` and `bash -lc` body for [`super::super::SessionHolder::exec_lima_interactive_shell`]:
    /// `cd` to the host `current_dir` project under the Lima workspace mount, optional `$HOME/host_dir` symlink,
    /// and `~/.todo.json` → `~/host_dir/.todo.json`.
    #[must_use]
    pub fn lima_interactive_shell_workdir_and_inner(&self) -> (String, String) {
        let cwd = std::env::current_dir().ok();
        let guest_proj = cwd.as_ref().and_then(|c| {
            guest_dir_for_host_path_under_workspace(&self.workspace_parent, &self.guest_mount, c)
        });

        if guest_proj.is_none() {
            if let Some(ref c) = cwd {
                let _ = writeln!(
                    std::io::stderr(),
                    "dev_shell: lima: host cwd {} is outside workspace_parent {} — shell starts at {}.\n\
dev_shell: hint: `workspace_parent` defaults to the Cargo workspace root (or set DEVSHELL_VM_WORKSPACE_PARENT). \
In ~/.lima/<instance>/lima.yaml, mount that host path, e.g.:\n  - location: \"{}\"\n    mountPoint: {}\n    writable: true\n\
Then limactl stop/start the instance. See docs/devshell-vm-gamma.md.",
                    c.display(),
                    self.workspace_parent.display(),
                    self.guest_mount,
                    self.workspace_parent.display(),
                    self.guest_mount
                );
            }
        }

        let workdir = guest_proj
            .clone()
            .unwrap_or_else(|| self.guest_mount.clone());

        let mut inner = String::new();
        if let Some(p) = self.guest_todo_release_path_for_shell() {
            let _ = writeln!(
                std::io::stderr(),
                "dev_shell: prepending guest PATH with {p} (host todo under Lima workspace mount)"
            );
            let _ = write!(inner, "export PATH={}:$PATH; ", bash_single_quoted(&p));
        }

        if let Some(ref gp) = guest_proj {
            match guest_host_dir_link_name() {
                Some(hd) => {
                    let _ = write!(
                        inner,
                        "export GUEST_PROJ={}; export HD={}; \
if [ -d \"$GUEST_PROJ\" ]; then cd \"$GUEST_PROJ\" || true; \
  ln -sfn \"$GUEST_PROJ\" \"$HOME/$HD\" 2>/dev/null || true; \
  ln -sf \"$HOME/$HD/.todo.json\" \"$HOME/.todo.json\" 2>/dev/null || true; \
fi; ",
                        bash_single_quoted(gp),
                        bash_single_quoted(&hd)
                    );
                }
                None => {
                    let _ = write!(
                        inner,
                        "export GUEST_PROJ={}; \
if [ -d \"$GUEST_PROJ\" ]; then cd \"$GUEST_PROJ\" || true; fi; ",
                        bash_single_quoted(gp)
                    );
                }
            }
        }

        inner.push_str("exec bash -l");
        (workdir, inner)
    }
}
