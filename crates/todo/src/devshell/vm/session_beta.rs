//! β client: JSON-lines over a Unix socket to `devshell-vm --serve-socket <path>`.
//!
//! Build: `cargo build -p xtask-todo-lib --features beta-vm`. Env: `DEVSHELL_VM_SOCKET`, same workspace
//! layout as γ (`session_gamma::workspace_parent_for_instance`).

#![allow(clippy::pedantic, clippy::nursery)]

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::ExitStatus;

use std::os::unix::net::UnixStream;
use std::os::unix::process::ExitStatusExt;

use super::super::vfs::Vfs;
use super::config::ENV_DEVSHELL_VM_SOCKET;
use super::guest_fs_ops::{validate_guest_path_under_mount, GuestFsError, GuestFsOps};
use super::session_gamma::{self, guest_dir_for_vfs_cwd};
use super::sync::{pull_workspace_to_vfs, push_incremental};
use super::{VmConfig, VmError, VmExecutionSession, WorkspaceMode};

use base64::prelude::*;

/// IPC client session (sidecar must be started separately).
pub struct BetaSession {
    sock_path: PathBuf,
    stream: Option<UnixStream>,
    session_id: String,
    workspace_parent: PathBuf,
    guest_mount: String,
    handshake_ok: bool,
    session_started: bool,
    /// Same as γ: when `false` ([`WorkspaceMode::Guest`]), skip host↔VFS sync around rust tools.
    sync_vfs_with_workspace: bool,
}

impl std::fmt::Debug for BetaSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BetaSession")
            .field("sock_path", &self.sock_path)
            .field("session_id", &self.session_id)
            .field("workspace_parent", &self.workspace_parent)
            .field("guest_mount", &self.guest_mount)
            .field("handshake_ok", &self.handshake_ok)
            .field("session_started", &self.session_started)
            .field("sync_vfs_with_workspace", &self.sync_vfs_with_workspace)
            .finish_non_exhaustive()
    }
}

fn vfs_cwd_leaf(vfs_cwd: &str) -> String {
    let t = vfs_cwd.trim_matches('/');
    if t.is_empty() {
        ".".to_string()
    } else {
        t.split('/').next_back().unwrap_or(".").to_string()
    }
}

impl BetaSession {
    /// New β session; requires `DEVSHELL_VM_SOCKET` pointing at the sidecar listening path.
    pub fn new(config: &VmConfig) -> Result<Self, VmError> {
        let sock_path = std::env::var(ENV_DEVSHELL_VM_SOCKET).map_err(|_| {
            VmError::Ipc(format!(
                "{ENV_DEVSHELL_VM_SOCKET} is not set (start sidecar: devshell-vm --serve-socket <path>)"
            ))
        })?;
        let sock_path = sock_path.trim();
        if sock_path.is_empty() {
            return Err(VmError::Ipc(format!("{ENV_DEVSHELL_VM_SOCKET} is empty")));
        }
        let workspace_parent = session_gamma::workspace_parent_for_instance(&config.lima_instance);
        let guest_mount = std::env::var(session_gamma::ENV_DEVSHELL_VM_GUEST_WORKSPACE)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "/workspace".to_string());

        let sync_vfs_with_workspace =
            matches!(config.workspace_mode_effective(), WorkspaceMode::Sync);

        Ok(Self {
            sock_path: PathBuf::from(sock_path),
            stream: None,
            session_id: format!("pid-{}", std::process::id()),
            workspace_parent,
            guest_mount,
            handshake_ok: false,
            session_started: false,
            sync_vfs_with_workspace,
        })
    }

    fn conn(&mut self) -> Result<&mut UnixStream, VmError> {
        if self.stream.is_none() {
            let s = UnixStream::connect(&self.sock_path).map_err(|e| {
                VmError::Ipc(format!(
                    "connect {}: {e}; start sidecar with: devshell-vm --serve-socket {}",
                    self.sock_path.display(),
                    self.sock_path.display()
                ))
            })?;
            self.stream = Some(s);
        }
        Ok(self.stream.as_mut().expect("set above"))
    }

    fn exchange(&mut self, req: &serde_json::Value) -> Result<serde_json::Value, VmError> {
        let stream = self.conn()?;
        let line = serde_json::to_string(req).map_err(|e| VmError::Ipc(e.to_string()))?;
        writeln!(stream, "{line}").map_err(|e| VmError::Ipc(e.to_string()))?;
        stream.flush().map_err(|e| VmError::Ipc(e.to_string()))?;
        let mut reader = BufReader::new(
            stream
                .try_clone()
                .map_err(|e| VmError::Ipc(format!("stream clone: {e}")))?,
        );
        let mut out = String::new();
        reader
            .read_line(&mut out)
            .map_err(|e| VmError::Ipc(e.to_string()))?;
        let v: serde_json::Value =
            serde_json::from_str(out.trim()).map_err(|e| VmError::Ipc(e.to_string()))?;
        if v.get("op").and_then(|x| x.as_str()) == Some("error") {
            let msg = v
                .get("message")
                .and_then(|x| x.as_str())
                .unwrap_or("server error");
            return Err(VmError::Ipc(msg.to_string()));
        }
        Ok(v)
    }

    fn ensure_session_started(&mut self) -> Result<(), VmError> {
        if self.session_started {
            return Ok(());
        }
        std::fs::create_dir_all(&self.workspace_parent).map_err(|e| {
            VmError::Ipc(format!(
                "create staging dir {}: {e}",
                self.workspace_parent.display()
            ))
        })?;
        let staging = std::fs::canonicalize(&self.workspace_parent)
            .map_err(|e| VmError::Ipc(format!("canonicalize staging: {e}")))?;
        let staging_str = staging
            .to_str()
            .ok_or_else(|| VmError::Ipc("workspace path is not valid UTF-8".to_string()))?;
        let req = serde_json::json!({
            "op": "session_start",
            "session_id": &self.session_id,
            "staging_dir": staging_str,
            "guest_workspace": &self.guest_mount,
            "backend": "beta-stub",
            "backend_config": serde_json::json!({}),
        });
        let v = self.exchange(&req)?;
        if v.get("op").and_then(|x| x.as_str()) != Some("session_ok") {
            return Err(VmError::Ipc(format!("session_start: unexpected {v}")));
        }
        self.session_started = true;
        Ok(())
    }

    /// Guest workspace mount (e.g. `/workspace`), same env as γ.
    #[must_use]
    pub fn guest_mount(&self) -> &str {
        &self.guest_mount
    }

    /// `true` when Mode S (host↔VFS sync around rust tools); `false` in guest-primary ([`WorkspaceMode::Guest`]).
    #[must_use]
    pub fn syncs_vfs_with_host_workspace(&self) -> bool {
        self.sync_vfs_with_workspace
    }

    fn guest_fs_prep(&mut self) -> Result<(), GuestFsError> {
        let vfs = Vfs::new();
        self.ensure_ready(&vfs, "/").map_err(GuestFsError::from)?;
        self.ensure_session_started().map_err(GuestFsError::from)?;
        Ok(())
    }

    fn guest_fs_call(
        &mut self,
        operation: &str,
        guest_path: &str,
        data: Option<&[u8]>,
    ) -> Result<serde_json::Value, GuestFsError> {
        let mount = self.guest_mount();
        let p = validate_guest_path_under_mount(mount, guest_path)?;
        self.guest_fs_prep()?;
        let mut req = serde_json::json!({
            "op": "guest_fs",
            "session_id": &self.session_id,
            "operation": operation,
            "guest_path": p,
        });
        if let Some(bytes) = data {
            req["content_base64"] = serde_json::Value::String(BASE64_STANDARD.encode(bytes));
        }
        let v = self.exchange(&req).map_err(GuestFsError::from)?;
        match v.get("op").and_then(|x| x.as_str()) {
            Some("guest_fs_ok") => Ok(v),
            Some("guest_fs_error") => {
                let code = v
                    .get("code")
                    .and_then(|x| x.as_str())
                    .unwrap_or("guest_fs_error");
                let msg = v.get("message").and_then(|x| x.as_str()).unwrap_or(code);
                Err(match code {
                    "not_found" => GuestFsError::NotFound(msg.to_string()),
                    "not_a_directory" => GuestFsError::NotADirectory(msg.to_string()),
                    "is_a_directory" => GuestFsError::IsADirectory(msg.to_string()),
                    "invalid_path" => GuestFsError::InvalidPath(msg.to_string()),
                    _ => GuestFsError::Internal(format!("{code}: {msg}")),
                })
            }
            _ => Err(GuestFsError::Internal(format!(
                "unexpected guest_fs response: {v}"
            ))),
        }
    }
}

impl VmExecutionSession for BetaSession {
    fn ensure_ready(&mut self, _vfs: &Vfs, _vfs_cwd: &str) -> Result<(), VmError> {
        if self.handshake_ok {
            return Ok(());
        }
        let stream = self.conn()?;
        let req = serde_json::json!({
            "op": "handshake",
            "version": 1u64,
            "client": "cargo-devshell",
            "client_version": env!("CARGO_PKG_VERSION"),
        });
        let line = serde_json::to_string(&req).map_err(|e| VmError::Ipc(e.to_string()))?;
        writeln!(stream, "{line}").map_err(|e| VmError::Ipc(e.to_string()))?;
        stream.flush().map_err(|e| VmError::Ipc(e.to_string()))?;
        let mut reader = BufReader::new(
            stream
                .try_clone()
                .map_err(|e| VmError::Ipc(format!("stream clone: {e}")))?,
        );
        let mut out = String::new();
        reader
            .read_line(&mut out)
            .map_err(|e| VmError::Ipc(e.to_string()))?;
        let v: serde_json::Value =
            serde_json::from_str(out.trim()).map_err(|e| VmError::Ipc(e.to_string()))?;
        if v.get("op").and_then(|x| x.as_str()) != Some("handshake_ok") {
            return Err(VmError::Ipc(format!("handshake: unexpected {v}")));
        }
        self.handshake_ok = true;
        Ok(())
    }

    fn run_rust_tool(
        &mut self,
        vfs: &mut Vfs,
        vfs_cwd: &str,
        program: &str,
        args: &[String],
    ) -> Result<ExitStatus, VmError> {
        self.ensure_ready(vfs, vfs_cwd)?;
        self.ensure_session_started()?;

        let leaf = vfs_cwd_leaf(vfs_cwd);

        if self.sync_vfs_with_workspace {
            push_incremental(vfs, vfs_cwd, &self.workspace_parent).map_err(VmError::Sync)?;

            let push = serde_json::json!({
                "op": "sync_request",
                "session_id": &self.session_id,
                "direction": "push_to_guest",
                "vfs_cwd_leaf": &leaf,
            });
            self.exchange(&push)?;
        }

        let mut argv = vec![program.to_string()];
        argv.extend_from_slice(args);
        let guest_cwd = guest_dir_for_vfs_cwd(&self.guest_mount, vfs_cwd);
        let exec = serde_json::json!({
            "op": "exec",
            "session_id": &self.session_id,
            "guest_cwd": guest_cwd,
            "argv": argv,
            "env": serde_json::json!({}),
        });
        let res = self.exchange(&exec)?;
        let code = res.get("exit_code").and_then(|x| x.as_i64()).unwrap_or(1) as i32;

        if self.sync_vfs_with_workspace {
            let pull_req = serde_json::json!({
                "op": "sync_request",
                "session_id": &self.session_id,
                "direction": "pull_from_guest",
                "vfs_cwd_leaf": &leaf,
            });
            let _ = self.exchange(&pull_req);

            if let Err(e) = pull_workspace_to_vfs(&self.workspace_parent, vfs_cwd, vfs) {
                eprintln!(
                    "dev_shell: warning: vm workspace pull failed after `{program}` (VFS may be stale): {e}"
                );
            }
        }

        let code_u8 = code.clamp(0, 255) as i32;
        Ok(ExitStatusExt::from_raw(code_u8 << 8))
    }

    fn shutdown(&mut self, vfs: &mut Vfs, vfs_cwd: &str) -> Result<(), VmError> {
        if self.stream.is_some() {
            let req = serde_json::json!({
                "op": "session_shutdown",
                "session_id": &self.session_id,
                "stop_vm": false,
            });
            let _ = self.exchange(&req);
            if self.sync_vfs_with_workspace {
                let _ = pull_workspace_to_vfs(&self.workspace_parent, vfs_cwd, vfs);
            }
        }
        self.stream = None;
        Ok(())
    }
}

impl GuestFsOps for BetaSession {
    fn list_dir(&mut self, guest_path: &str) -> Result<Vec<String>, GuestFsError> {
        let v = self.guest_fs_call("list_dir", guest_path, None)?;
        let arr = v
            .get("names")
            .and_then(|x| x.as_array())
            .ok_or_else(|| GuestFsError::Internal("guest_fs_ok missing names".into()))?;
        let mut names = Vec::with_capacity(arr.len());
        for x in arr {
            let s = x
                .as_str()
                .ok_or_else(|| GuestFsError::Internal("names entry not string".into()))?;
            names.push(s.to_string());
        }
        Ok(names)
    }

    fn read_file(&mut self, guest_path: &str) -> Result<Vec<u8>, GuestFsError> {
        let v = self.guest_fs_call("read_file", guest_path, None)?;
        let b64 = v
            .get("content_base64")
            .and_then(|x| x.as_str())
            .ok_or_else(|| GuestFsError::Internal("guest_fs_ok missing content_base64".into()))?;
        BASE64_STANDARD
            .decode(b64)
            .map_err(|e| GuestFsError::Internal(e.to_string()))
    }

    fn write_file(&mut self, guest_path: &str, data: &[u8]) -> Result<(), GuestFsError> {
        self.guest_fs_call("write_file", guest_path, Some(data))?;
        Ok(())
    }

    fn mkdir(&mut self, guest_path: &str) -> Result<(), GuestFsError> {
        self.guest_fs_call("mkdir", guest_path, None)?;
        Ok(())
    }

    fn remove(&mut self, guest_path: &str) -> Result<(), GuestFsError> {
        self.guest_fs_call("remove", guest_path, None)?;
        Ok(())
    }
}
