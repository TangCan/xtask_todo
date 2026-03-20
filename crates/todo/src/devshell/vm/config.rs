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
/// **Non-Unix** default: `host` (Lima not available).
/// **`cfg(test)`:** unset → `auto` (host sandbox) for the same reason as `ENV_DEVSHELL_VM`.
pub const ENV_DEVSHELL_VM_BACKEND: &str = "DEVSHELL_VM_BACKEND";

/// When `1`/`true`/`yes`, start the VM session eagerly (future γ); default is lazy start on first rust tool.
pub const ENV_DEVSHELL_VM_EAGER: &str = "DEVSHELL_VM_EAGER";

/// Lima instance name for γ (`limactl shell <name>`).
pub const ENV_DEVSHELL_VM_LIMA_INSTANCE: &str = "DEVSHELL_VM_LIMA_INSTANCE";

/// Unix socket path for β client ↔ `devshell-vm --serve-socket` (see IPC draft).
pub const ENV_DEVSHELL_VM_SOCKET: &str = "DEVSHELL_VM_SOCKET";

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

fn default_backend_for_release() -> String {
    #[cfg(unix)]
    {
        "lima".to_string()
    }
    #[cfg(not(unix))]
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

    #[cfg(not(unix))]
    #[test]
    fn try_from_config_beta_errors_on_non_unix() {
        use super::super::{SessionHolder, VmError};

        let mut c = VmConfig::disabled();
        c.enabled = true;
        c.backend = "beta".to_string();
        c.lima_instance = "devshell-rust".to_string();
        let r = SessionHolder::try_from_config(&c);
        assert!(matches!(r, Err(VmError::BackendNotImplemented(_))));
    }
}
