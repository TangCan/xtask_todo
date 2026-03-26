//! Guest `build-essential` / `todo` setup and [`VmExecutionSession`] for γ.

use std::process::ExitStatus;

use super::super::super::super::vfs::Vfs;
use super::super::super::lima_diagnostics;
use super::super::super::sync::{pull_workspace_to_vfs, push_incremental};
use super::super::super::{VmError, VmExecutionSession};
use super::super::env::ENV_DEVSHELL_VM_STOP_ON_EXIT;
use super::super::helpers::{
    auto_build_essential_enabled, auto_build_todo_guest_enabled,
    cargo_metadata_workspace_and_target, guest_cargo_workspace_dir_for_cwd,
    guest_dir_for_cwd_inner, guest_todo_hint_enabled, guest_todo_release_dir_for_cwd,
    shell_single_quote_sh, truthy_env,
};
use super::GammaSession;

impl GammaSession {
    const INSTALL_SH: &str = r"set -e
export DEBIAN_FRONTEND=noninteractive
if ! sudo -n true 2>/dev/null; then
  echo 'dev_shell: guest: sudo needs a password; run in the VM: sudo apt update && sudo apt install -y build-essential' >&2
  exit 1
fi
sudo apt-get update -qq
sudo apt-get install -y -qq build-essential
";

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

        let out = self.limactl_shell_script_sh("/", Self::INSTALL_SH)?;
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

    /// If guest has no `todo` in PATH (and no executable under expected `target/release` when mapped), print install hints; optionally run `cargo build` in guest.
    fn maybe_guest_todo_probe_hint_and_install(&mut self) -> Result<(), VmError> {
        if self.guest_todo_hint_done {
            return Ok(());
        }
        if !guest_todo_hint_enabled() {
            self.guest_todo_hint_done = true;
            return Ok(());
        }

        let cwd =
            std::env::current_dir().map_err(|e| VmError::Lima(format!("current_dir: {e}")))?;

        let grel = guest_todo_release_dir_for_cwd(&self.workspace_parent, &self.guest_mount, &cwd);
        let script = grel.as_ref().map_or_else(
            || "command -v todo >/dev/null 2>&1".to_string(),
            |gr| {
                format!(
                    "command -v todo >/dev/null 2>&1 || test -x {}",
                    shell_single_quote_sh(&format!("{gr}/todo"))
                )
            },
        );

        let probe = self.limactl_shell_script_sh("/", &script)?;
        if probe.status.success() {
            self.guest_todo_hint_done = true;
            return Ok(());
        }

        eprintln!(
            "dev_shell: guest: `todo` not found (do not use apt `devtodo` — unrelated package)."
        );

        let meta = cargo_metadata_workspace_and_target(&cwd);
        let host_has_todo = meta
            .as_ref()
            .is_ok_and(|(_, td)| td.join("release").join("todo").is_file());

        if meta.is_err() {
            eprintln!("dev_shell: hint: from a repo checkout, run: cargo build -p xtask --release --bin todo");
            eprintln!(
                "dev_shell: hint: repo outside Lima mount: cargo xtask lima-todo (merges ~/.lima/{}/lima.yaml + restarts VM; use --print-only for fragment only)",
                self.lima_instance
            );
        } else {
            eprintln!("dev_shell: host: cargo build -p xtask --release --bin todo");
            if grel.is_none() && host_has_todo {
                eprintln!(
                    "dev_shell: host: workspace outside Lima mount — run: cargo xtask lima-todo"
                );
                eprintln!(
                    "dev_shell: host: (merges `mounts` + `env.PATH` into ~/.lima/{}/lima.yaml and runs limactl stop/start unless --no-restart)",
                    self.lima_instance
                );
            }
        }

        if let Some(ref gw) =
            guest_cargo_workspace_dir_for_cwd(&self.workspace_parent, &self.guest_mount, &cwd)
        {
            eprintln!("dev_shell: guest: cd {gw} && cargo build -p xtask --release --bin todo");
            if auto_build_todo_guest_enabled() {
                let q = shell_single_quote_sh(gw);
                let build_sh = format!(
                    "set -e
cd {q}
if ! command -v cargo >/dev/null 2>&1; then
  echo 'dev_shell: guest: cargo not in PATH; install Rust in the VM first (see docs/devshell-vm-gamma.md).' >&2
  exit 1
fi
cargo build -p xtask --release --bin todo
"
                );
                let out = self.limactl_shell_script_sh("/", &build_sh)?;
                if out.status.success() {
                    eprintln!("dev_shell: guest: built target/release/todo.");
                } else {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    eprintln!(
                        "dev_shell: guest: automatic `cargo build` for todo failed (exit {:?}).",
                        out.status.code()
                    );
                    if !stdout.trim().is_empty() {
                        eprintln!("dev_shell: guest stdout: {stdout}");
                    }
                    if !stderr.trim().is_empty() {
                        eprintln!("dev_shell: guest stderr: {stderr}");
                    }
                }
            }
        }

        self.guest_todo_hint_done = true;
        Ok(())
    }
}

impl VmExecutionSession for GammaSession {
    fn ensure_ready(&mut self, _vfs: &Vfs, _vfs_cwd: &str) -> Result<(), VmError> {
        self.limactl_ensure_running()?;
        self.maybe_ensure_guest_build_essential()?;
        self.maybe_guest_todo_probe_hint_and_install()?;
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
                status,
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
