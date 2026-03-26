//! Environment-driven configuration for optional devshell VM execution (`DEVSHELL_VM`, backend, Lima name).

mod constants;
mod repo;
#[cfg(test)]
mod tests;

pub use constants::*;
#[cfg(all(feature = "beta-vm", target_os = "windows"))]
pub(crate) use repo::{devshell_repo_root_from_path, devshell_repo_root_with_containerfile};

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

/// Reads [`ENV_DEVSHELL_VM_EXEC_TIMEOUT_MS`]: positive integer milliseconds for β **`exec`** JSON
/// **`timeout_ms`**, or [`None`] if unset, invalid, or zero.
#[must_use]
pub fn exec_timeout_ms_from_env() -> Option<u64> {
    std::env::var(ENV_DEVSHELL_VM_EXEC_TIMEOUT_MS)
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .filter(|&ms| ms > 0)
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
        return std::env::var(ENV_DEVSHELL_VM).is_ok_and(|s| truthy(&s));
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

        let eager_start = std::env::var(ENV_DEVSHELL_VM_EAGER).is_ok_and(|s| truthy(&s));

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
    #[allow(clippy::missing_const_for_fn)]
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
