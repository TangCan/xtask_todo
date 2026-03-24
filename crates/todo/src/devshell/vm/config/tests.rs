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
fn exec_timeout_ms_from_env_unset_is_none() {
    let _g = vm_env_lock();
    let old = std::env::var(ENV_DEVSHELL_VM_EXEC_TIMEOUT_MS).ok();
    set_env(ENV_DEVSHELL_VM_EXEC_TIMEOUT_MS, None);
    assert_eq!(exec_timeout_ms_from_env(), None);
    set_env(ENV_DEVSHELL_VM_EXEC_TIMEOUT_MS, old.as_deref());
}

#[test]
fn exec_timeout_ms_from_env_positive() {
    let _g = vm_env_lock();
    let old = std::env::var(ENV_DEVSHELL_VM_EXEC_TIMEOUT_MS).ok();
    set_env(ENV_DEVSHELL_VM_EXEC_TIMEOUT_MS, Some(" 600000 "));
    assert_eq!(exec_timeout_ms_from_env(), Some(600_000));
    set_env(ENV_DEVSHELL_VM_EXEC_TIMEOUT_MS, old.as_deref());
}

#[test]
fn exec_timeout_ms_from_env_zero_or_invalid_is_none() {
    let _g = vm_env_lock();
    let old = std::env::var(ENV_DEVSHELL_VM_EXEC_TIMEOUT_MS).ok();
    set_env(ENV_DEVSHELL_VM_EXEC_TIMEOUT_MS, Some("0"));
    assert_eq!(exec_timeout_ms_from_env(), None);
    set_env(ENV_DEVSHELL_VM_EXEC_TIMEOUT_MS, Some("not_a_number"));
    assert_eq!(exec_timeout_ms_from_env(), None);
    set_env(ENV_DEVSHELL_VM_EXEC_TIMEOUT_MS, old.as_deref());
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
    use crate::devshell::sandbox;
    use crate::devshell::vm::{SessionHolder, VmError};

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
    use crate::devshell::vm::{SessionHolder, VmError};

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
    use crate::devshell::vm::{SessionHolder, VmError};

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
    use crate::devshell::vm::{SessionHolder, VmError};

    let mut c = VmConfig::disabled();
    c.enabled = true;
    c.backend = "beta".to_string();
    c.lima_instance = "devshell-rust".to_string();
    let r = SessionHolder::try_from_config(&c);
    assert!(matches!(r, Err(VmError::BackendNotImplemented(_))));
}
