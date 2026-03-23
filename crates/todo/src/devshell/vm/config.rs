//! Environment-driven configuration for optional devshell VM execution (`DEVSHELL_VM`, backend, Lima name).

#![allow(clippy::pedantic, clippy::nursery)]

/// `DEVSHELL_VM` — **Release / binary default:** unset means **on** (use VM backend per [`ENV_DEVSHELL_VM_BACKEND`]).
/// Set to `off` / `0` / `false` / `no` (case-insensitive) to use **only** the host temp sandbox.
/// `on` / `1` / `true` / `yes` also enable VM mode.
///
/// **Unit tests** (`cfg(test)`): unset defaults to **off** so `cargo test` works without Lima.
pub const ENV_DEVSHELL_VM: &str = "DEVSHELL_VM";

/// Backend selector: `host`, `auto`, `lima`, `beta`, …
///
/// **Release / binary default on Unix:** `lima` (γ) when this variable is unset.
/// **Windows** default: **`beta`** (with **`beta-vm`** feature). Use **`DEVSHELL_VM_BACKEND=host`** for host-only sandbox.
/// **Other non-Unix (non-Windows):** `host`.
/// **`cfg(test)`:** unset → `auto` (host sandbox) for the same reason as `ENV_DEVSHELL_VM`.
pub const ENV_DEVSHELL_VM_BACKEND: &str = "DEVSHELL_VM_BACKEND";

/// When `1`/`true`/`yes`, start the VM session eagerly (future γ); default is lazy start on first rust tool.
pub const ENV_DEVSHELL_VM_EAGER: &str = "DEVSHELL_VM_EAGER";

/// Lima instance name for γ (`limactl shell <name>`).
pub const ENV_DEVSHELL_VM_LIMA_INSTANCE: &str = "DEVSHELL_VM_LIMA_INSTANCE";

/// Unix socket path for β client ↔ `devshell-vm --serve-socket` (see IPC draft).
pub const ENV_DEVSHELL_VM_SOCKET: &str = "DEVSHELL_VM_SOCKET";

/// When set (non-empty), β **`session_start`** sends this string as **`staging_dir`** to the sidecar instead of
/// `canonicalize(DEVSHELL_VM_WORKSPACE_PARENT / …)`. Use a **POSIX path** visible to the sidecar process
/// (e.g. **`/workspace`** inside a Podman/WSL Linux container) while **`DEVSHELL_VM_WORKSPACE_PARENT`** on the
/// host remains the real Windows path for push/pull. See **`docs/devshell-vm-windows.md`** (Podman).
///
/// On Windows, **`stdio`** (default) maps the host workspace to **`/mnt/<drive>/…`** inside Podman Machine for
/// `session_start` **`staging_dir`** unless you set this explicitly.
pub const ENV_DEVSHELL_VM_BETA_SESSION_STAGING: &str = "DEVSHELL_VM_BETA_SESSION_STAGING";

/// When set (any value), skip **`podman machine ssh`** bootstrap on Windows: no Podman check / no requirement
/// that the Linux `devshell-vm` binary exists (tests or fully manual β setup).
pub const ENV_DEVSHELL_VM_SKIP_PODMAN_BOOTSTRAP: &str = "DEVSHELL_VM_SKIP_PODMAN_BOOTSTRAP";

/// **Windows β:** optional full **Windows** path to the **Linux** `devshell-vm` binary
/// (`x86_64-unknown-linux-gnu` / ELF) for **`podman machine ssh`** transport.
///
/// If unset, the binary is searched under **`$repo_root/target/x86_64-unknown-linux-gnu/release/devshell-vm`**
/// where `repo_root` is discovered from cwd, [`ENV_DEVSHELL_VM_REPO_ROOT`], or walking up from the workspace
/// parent — **not** the ephemeral `cargo-devshell-exports` tree. If still not found, **automatic fallback
/// uses [`ENV_DEVSHELL_VM_CONTAINER_IMAGE`] with `podman run -i`** (see `podman_machine.rs`).
pub const ENV_DEVSHELL_VM_LINUX_BINARY: &str = "DEVSHELL_VM_LINUX_BINARY";

/// **Windows β:** optional **Windows** path to an **xtask_todo** repository root (directory containing
/// **`containers/devshell-vm/Containerfile`**). Locates **`target/x86_64-unknown-linux-gnu/release/devshell-vm`**
/// when [`ENV_DEVSHELL_VM_LINUX_BINARY`] is unset. Useful if you keep a checkout for building the sidecar but run
/// **`cargo devshell`** from other directories; **not** applicable when you only have a crates.io install and no clone.
pub const ENV_DEVSHELL_VM_REPO_ROOT: &str = "DEVSHELL_VM_REPO_ROOT";

/// **Windows β:** OCI image used when **no** host Linux `devshell-vm` ELF is found: `podman run -i` with
/// **`--serve-stdio`** and the workspace mounted at **`/workspace`** (no host TCP).
/// Default: **`ghcr.io/tangcan/xtask_todo/devshell-vm:v{CARGO_PKG_VERSION}`** (published by CI on release).
pub const ENV_DEVSHELL_VM_CONTAINER_IMAGE: &str = "DEVSHELL_VM_CONTAINER_IMAGE";

/// **Windows β:** stdio transport for `DEVSHELL_VM_SOCKET=stdio`: **`auto`** (default), **`machine-ssh`**
/// (host ELF + `podman machine ssh`), or **`podman-run`** (OCI image + `podman run -i`).
pub const ENV_DEVSHELL_VM_STDIO_TRANSPORT: &str = "DEVSHELL_VM_STDIO_TRANSPORT";

/// When set (any value), do **not** isolate **`USERPROFILE` / `HOME`** for `podman` subprocesses (Windows).
/// By default we point **`USERPROFILE`** (Go’s `UserHomeDir()` on Windows — not only `HOME`) at a writable
/// temp “profile” with an **empty default** `.ssh/known_hosts`, so a **locked, protected, or invalid**
/// **`%USERPROFILE%\.ssh\known_hosts`** is not read. An existing Podman Machine dir is **symlinked** in when
/// possible (see `podman_machine.rs`).
pub const ENV_DEVSHELL_VM_DISABLE_PODMAN_SSH_HOME: &str = "DEVSHELL_VM_DISABLE_PODMAN_SSH_HOME";

/// `DEVSHELL_VM_WORKSPACE_MODE` — **`sync`** (default) or **`guest`** (Mode P; guest filesystem as source of truth).
///
/// **`guest`** is effective only when the VM is enabled and the backend is **`lima`** or **`beta`**; otherwise
/// [`VmConfig::workspace_mode_effective`] returns [`WorkspaceMode::Sync`] (design `2026-03-20-devshell-guest-primary-design.md` §6).
///
/// **Unset** (including **`cfg(test)`**): [`WorkspaceMode::Sync`].
pub const ENV_DEVSHELL_VM_WORKSPACE_MODE: &str = "DEVSHELL_VM_WORKSPACE_MODE";

/// How the devshell workspace is backed: memory VFS + push/pull (**[`WorkspaceMode::Sync`]**) vs guest-primary (**[`WorkspaceMode::Guest`]**, planned).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceMode {
    /// Mode S: in-memory `Vfs` authority; `cargo`/`rustup` sync with guest when using γ.
    Sync,
    /// Mode P: guest mount is the source of truth for the project tree (incremental implementation).
    Guest,
}

/// Read [`ENV_DEVSHELL_VM_WORKSPACE_MODE`] from the environment.
#[must_use]
pub fn workspace_mode_from_env() -> WorkspaceMode {
    match std::env::var(ENV_DEVSHELL_VM_WORKSPACE_MODE) {
        Ok(s) if s.trim().eq_ignore_ascii_case("guest") => WorkspaceMode::Guest,
        _ => WorkspaceMode::Sync,
    }
}

/// Parsed VM-related environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VmConfig {
    /// `DEVSHELL_VM` enabled.
    pub enabled: bool,
    /// Raw backend string (trimmed); see `ENV_DEVSHELL_VM_BACKEND` for defaults.
    pub backend: String,
    /// Eager VM/session start when REPL opens (vs lazy on first `rustup`/`cargo`).
    pub eager_start: bool,
    /// Lima instance name.
    pub lima_instance: String,
}

fn truthy(s: &str) -> bool {
    let s = s.trim();
    s == "1"
        || s.eq_ignore_ascii_case("true")
        || s.eq_ignore_ascii_case("yes")
        || s.eq_ignore_ascii_case("on")
}

fn falsy(s: &str) -> bool {
    let s = s.trim();
    s == "0"
        || s.eq_ignore_ascii_case("false")
        || s.eq_ignore_ascii_case("no")
        || s.eq_ignore_ascii_case("off")
}

fn devshell_repo_root_walk(mut dir: std::path::PathBuf) -> Option<std::path::PathBuf> {
    loop {
        let cf = dir.join("containers/devshell-vm/Containerfile");
        if cf.is_file() {
            return Some(dir);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

/// Walk parents from [`std::env::current_dir`] looking for `containers/devshell-vm/Containerfile` (xtask_todo repo).
#[cfg_attr(not(windows), allow(dead_code))]
#[cfg(feature = "beta-vm")]
pub(crate) fn devshell_repo_root_with_containerfile() -> Option<std::path::PathBuf> {
    let dir = std::env::current_dir().ok()?;
    devshell_repo_root_walk(dir)
}

/// Same as [`devshell_repo_root_with_containerfile`] but starting from `start` (e.g. workspace parent).
#[cfg_attr(not(windows), allow(dead_code))]
#[cfg(feature = "beta-vm")]
pub(crate) fn devshell_repo_root_from_path(start: &std::path::Path) -> Option<std::path::PathBuf> {
    devshell_repo_root_walk(start.to_path_buf())
}

fn default_backend_for_release() -> String {
    #[cfg(all(windows, feature = "beta-vm"))]
    {
        return "beta".to_string();
    }
    #[cfg(unix)]
    {
        "lima".to_string()
    }
    #[cfg(not(any(unix, all(windows, feature = "beta-vm"))))]
    {
        "host".to_string()
    }
}

fn vm_enabled_from_env() -> bool {
    if cfg!(test) {
        return std::env::var(ENV_DEVSHELL_VM)
            .map(|s| truthy(&s))
            .unwrap_or(false);
    }
    match std::env::var(ENV_DEVSHELL_VM) {
        Err(_) => true,
        Ok(s) if s.trim().is_empty() => false,
        Ok(s) if falsy(&s) => false,
        Ok(s) => truthy(&s),
    }
}

fn backend_from_env() -> String {
    let from_var = std::env::var(ENV_DEVSHELL_VM_BACKEND)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    if let Some(b) = from_var {
        return b;
    }
    if cfg!(test) {
        "auto".to_string()
    } else {
        default_backend_for_release()
    }
}

impl VmConfig {
    /// Read configuration from process environment.
    #[must_use]
    pub fn from_env() -> Self {
        let enabled = vm_enabled_from_env();

        let backend = backend_from_env();

        let eager_start = std::env::var(ENV_DEVSHELL_VM_EAGER)
            .map(|s| truthy(&s))
            .unwrap_or(false);

        let lima_instance = std::env::var(ENV_DEVSHELL_VM_LIMA_INSTANCE)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "devshell-rust".to_string());

        Self {
            enabled,
            backend,
            eager_start,
            lima_instance,
        }
    }

    /// Config with VM mode off (for tests).
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            backend: String::new(),
            eager_start: false,
            lima_instance: String::new(),
        }
    }

    /// Normalized backend: `host` and `auto` use the host temp sandbox; `lima` uses γ (Unix; see `docs/devshell-vm-gamma.md`).
    #[must_use]
    pub fn use_host_sandbox(&self) -> bool {
        let b = self.backend.to_ascii_lowercase();
        b == "host" || b == "auto" || b.is_empty()
    }

    /// Effective workspace mode after combining [`workspace_mode_from_env`] with VM availability (guest-primary design §6).
    ///
    /// Returns [`WorkspaceMode::Guest`] only when the user requested **`guest`**, [`VmConfig::enabled`] is true,
    /// [`VmConfig::use_host_sandbox`] is false, and the backend is **`lima`** or **`beta`**. Otherwise returns
    /// [`WorkspaceMode::Sync`] without erroring.
    #[must_use]
    pub fn workspace_mode_effective(&self) -> WorkspaceMode {
        let requested = workspace_mode_from_env();
        if matches!(requested, WorkspaceMode::Sync) {
            return WorkspaceMode::Sync;
        }

        let effective = if !self.enabled || self.use_host_sandbox() {
            WorkspaceMode::Sync
        } else {
            let b = self.backend.to_ascii_lowercase();
            if b == "lima" || b == "beta" {
                WorkspaceMode::Guest
            } else {
                WorkspaceMode::Sync
            }
        };

        if matches!(requested, WorkspaceMode::Guest)
            && matches!(effective, WorkspaceMode::Sync)
            && !cfg!(test)
        {
            eprintln!(
                "dev_shell: DEVSHELL_VM_WORKSPACE_MODE=guest requires VM enabled and backend lima or beta; using sync mode."
            );
        }

        effective
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Mutex, OnceLock};

    use super::*;

    fn vm_env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|e| e.into_inner())
    }

    fn set_env(key: &str, val: Option<&str>) {
        match val {
            Some(v) => std::env::set_var(key, v),
            None => std::env::remove_var(key),
        }
    }

    #[test]
    fn from_env_devshell_vm_on() {
        let _g = vm_env_lock();
        let old_vm = std::env::var(ENV_DEVSHELL_VM).ok();
        let old_b = std::env::var(ENV_DEVSHELL_VM_BACKEND).ok();
        set_env(ENV_DEVSHELL_VM, Some("on"));
        set_env(ENV_DEVSHELL_VM_BACKEND, None);
        let c = VmConfig::from_env();
        assert!(c.enabled);
        assert_eq!(c.backend, "auto");
        set_env(ENV_DEVSHELL_VM, old_vm.as_deref());
        set_env(ENV_DEVSHELL_VM_BACKEND, old_b.as_deref());
    }

    #[test]
    fn from_env_defaults_off() {
        let _g = vm_env_lock();
        let old = std::env::var(ENV_DEVSHELL_VM).ok();
        set_env(ENV_DEVSHELL_VM, None);
        let c = VmConfig::from_env();
        assert!(!c.enabled);
        set_env(ENV_DEVSHELL_VM, old.as_deref());
    }

    #[test]
    fn from_env_explicit_off_disables_vm() {
        let _g = vm_env_lock();
        let old = std::env::var(ENV_DEVSHELL_VM).ok();
        set_env(ENV_DEVSHELL_VM, Some("off"));
        let c = VmConfig::from_env();
        assert!(!c.enabled);
        set_env(ENV_DEVSHELL_VM, old.as_deref());
    }

    #[test]
    fn use_host_sandbox_lima_false() {
        let mut c = VmConfig::disabled();
        c.backend = "lima".to_string();
        assert!(!c.use_host_sandbox());
    }

    #[test]
    fn workspace_mode_from_env_unset_defaults_sync() {
        let _g = vm_env_lock();
        let old = std::env::var(ENV_DEVSHELL_VM_WORKSPACE_MODE).ok();
        set_env(ENV_DEVSHELL_VM_WORKSPACE_MODE, None);
        assert_eq!(workspace_mode_from_env(), WorkspaceMode::Sync);
        set_env(ENV_DEVSHELL_VM_WORKSPACE_MODE, old.as_deref());
    }

    #[test]
    fn workspace_mode_from_env_guest() {
        let _g = vm_env_lock();
        let old = std::env::var(ENV_DEVSHELL_VM_WORKSPACE_MODE).ok();
        set_env(ENV_DEVSHELL_VM_WORKSPACE_MODE, Some("guest"));
        assert_eq!(workspace_mode_from_env(), WorkspaceMode::Guest);
        set_env(ENV_DEVSHELL_VM_WORKSPACE_MODE, Some("GUEST"));
        assert_eq!(workspace_mode_from_env(), WorkspaceMode::Guest);
        set_env(ENV_DEVSHELL_VM_WORKSPACE_MODE, old.as_deref());
    }

    #[test]
    fn workspace_mode_effective_guest_plus_host_sandbox_forces_sync() {
        let _g = vm_env_lock();
        let old_w = std::env::var(ENV_DEVSHELL_VM_WORKSPACE_MODE).ok();
        set_env(ENV_DEVSHELL_VM_WORKSPACE_MODE, Some("guest"));
        let mut c = VmConfig::disabled();
        c.enabled = true;
        c.backend = "host".to_string();
        assert_eq!(c.workspace_mode_effective(), WorkspaceMode::Sync);
        set_env(ENV_DEVSHELL_VM_WORKSPACE_MODE, old_w.as_deref());
    }

    #[test]
    fn workspace_mode_effective_guest_vm_off_forces_sync() {
        let _g = vm_env_lock();
        let old_w = std::env::var(ENV_DEVSHELL_VM_WORKSPACE_MODE).ok();
        set_env(ENV_DEVSHELL_VM_WORKSPACE_MODE, Some("guest"));
        let mut c = VmConfig::disabled();
        c.enabled = false;
        c.backend = "lima".to_string();
        assert_eq!(c.workspace_mode_effective(), WorkspaceMode::Sync);
        set_env(ENV_DEVSHELL_VM_WORKSPACE_MODE, old_w.as_deref());
    }

    #[test]
    fn workspace_mode_effective_guest_lima_enabled() {
        let _g = vm_env_lock();
        let old_w = std::env::var(ENV_DEVSHELL_VM_WORKSPACE_MODE).ok();
        set_env(ENV_DEVSHELL_VM_WORKSPACE_MODE, Some("guest"));
        let mut c = VmConfig::disabled();
        c.enabled = true;
        c.backend = "lima".to_string();
        assert_eq!(c.workspace_mode_effective(), WorkspaceMode::Guest);
        set_env(ENV_DEVSHELL_VM_WORKSPACE_MODE, old_w.as_deref());
    }

    #[test]
    fn workspace_mode_effective_sync_env_ignores_backend() {
        let _g = vm_env_lock();
        let old_w = std::env::var(ENV_DEVSHELL_VM_WORKSPACE_MODE).ok();
        set_env(ENV_DEVSHELL_VM_WORKSPACE_MODE, Some("sync"));
        let mut c = VmConfig::disabled();
        c.enabled = true;
        c.backend = "lima".to_string();
        assert_eq!(c.workspace_mode_effective(), WorkspaceMode::Sync);
        set_env(ENV_DEVSHELL_VM_WORKSPACE_MODE, old_w.as_deref());
    }

    #[cfg(unix)]
    #[test]
    fn try_from_config_lima_depends_on_limactl_in_path() {
        use super::super::{SessionHolder, VmError};
        use crate::devshell::sandbox;

        let _g = vm_env_lock();
        let old_vm = std::env::var(ENV_DEVSHELL_VM).ok();
        let old_b = std::env::var(ENV_DEVSHELL_VM_BACKEND).ok();
        set_env(ENV_DEVSHELL_VM, Some("1"));
        set_env(ENV_DEVSHELL_VM_BACKEND, Some("lima"));
        let c = VmConfig::from_env();
        assert!(c.enabled);
        assert!(!c.use_host_sandbox());
        let r = SessionHolder::try_from_config(&c);
        match sandbox::find_in_path("limactl") {
            Some(_) => assert!(
                matches!(r, Ok(SessionHolder::Gamma(_))),
                "expected Gamma session when limactl is in PATH, got {r:?}"
            ),
            None => assert!(
                matches!(r, Err(VmError::Lima(_))),
                "expected Lima error when limactl missing, got {r:?}"
            ),
        }
        set_env(ENV_DEVSHELL_VM, old_vm.as_deref());
        set_env(ENV_DEVSHELL_VM_BACKEND, old_b.as_deref());
    }

    #[cfg(not(unix))]
    #[test]
    fn try_from_config_lima_errors_on_non_unix() {
        use super::super::{SessionHolder, VmError};

        let mut c = VmConfig::disabled();
        c.enabled = true;
        c.backend = "lima".to_string();
        c.lima_instance = "devshell-rust".to_string();
        let r = SessionHolder::try_from_config(&c);
        assert!(matches!(r, Err(VmError::BackendNotImplemented(_))));
    }

    #[cfg(all(unix, not(feature = "beta-vm")))]
    #[test]
    fn try_from_config_beta_requires_feature_flag() {
        use super::super::{SessionHolder, VmError};

        let mut c = VmConfig::disabled();
        c.enabled = true;
        c.backend = "beta".to_string();
        c.lima_instance = "devshell-rust".to_string();
        let r = SessionHolder::try_from_config(&c);
        let Err(VmError::BackendNotImplemented(msg)) = r else {
            panic!("expected BackendNotImplemented, got {r:?}");
        };
        assert!(
            msg.contains("beta-vm"),
            "message should mention beta-vm: {msg}"
        );
    }

    /// Without `beta-vm` (e.g. `cargo test -p xtask-todo-lib --no-default-features`), `beta` backend is unavailable.
    #[cfg(all(not(unix), not(feature = "beta-vm")))]
    #[test]
    fn try_from_config_beta_errors_without_beta_vm_feature() {
        use super::super::{SessionHolder, VmError};

        let mut c = VmConfig::disabled();
        c.enabled = true;
        c.backend = "beta".to_string();
        c.lima_instance = "devshell-rust".to_string();
        let r = SessionHolder::try_from_config(&c);
        assert!(matches!(r, Err(VmError::BackendNotImplemented(_))));
    }
}
