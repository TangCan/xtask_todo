use super::*;

fn restore_var(key: &str, val: Option<std::ffi::OsString>) {
    match val {
        Some(v) => std::env::set_var(key, v),
        None => std::env::remove_var(key),
    }
}

#[test]
fn try_from_config_vm_off_returns_host_session() {
    let cfg = VmConfig {
        enabled: false,
        backend: "lima".to_string(),
        eager_start: false,
        lima_instance: "devshell-rust".to_string(),
    };
    let session = SessionHolder::try_from_config(&cfg).expect("vm off should not fail");
    assert!(matches!(session, SessionHolder::Host(_)));
}

#[test]
fn try_from_config_host_like_backends_return_host_session() {
    for backend in ["host", "auto", ""] {
        let cfg = VmConfig {
            enabled: true,
            backend: backend.to_string(),
            eager_start: false,
            lima_instance: "devshell-rust".to_string(),
        };
        let session =
            SessionHolder::try_from_config(&cfg).expect("host-like backend should not fail");
        assert!(
            matches!(session, SessionHolder::Host(_)),
            "backend={backend:?} should resolve to host session"
        );
    }
}

#[cfg(unix)]
#[test]
fn try_session_rc_reports_lima_missing_when_backend_lima() {
    let _g = crate::test_support::vm_env_mutex();
    let old_vm = std::env::var_os(ENV_DEVSHELL_VM);
    let old_backend = std::env::var_os(ENV_DEVSHELL_VM_BACKEND);
    let old_limactl = std::env::var_os(ENV_DEVSHELL_VM_LIMACTL);
    let old_path = std::env::var_os("PATH");

    std::env::set_var(ENV_DEVSHELL_VM, "1");
    std::env::set_var(ENV_DEVSHELL_VM_BACKEND, "lima");
    std::env::remove_var(ENV_DEVSHELL_VM_LIMACTL);
    std::env::set_var("PATH", "/nonexistent_devshell_path_404");

    let mut stderr = Vec::new();
    let result = try_session_rc(&mut stderr);

    restore_var(ENV_DEVSHELL_VM, old_vm);
    restore_var(ENV_DEVSHELL_VM_BACKEND, old_backend);
    restore_var(ENV_DEVSHELL_VM_LIMACTL, old_limactl);
    restore_var("PATH", old_path);

    assert!(result.is_err(), "lima backend should fail without limactl");
    let err = String::from_utf8(stderr).expect("stderr UTF-8");
    assert!(
        err.contains("limactl not found in PATH"),
        "expected limactl diagnostic, got: {err}"
    );
}

#[cfg(unix)]
#[test]
fn try_session_rc_or_host_falls_back_to_host_when_lima_unavailable() {
    let _g = crate::test_support::vm_env_mutex();
    let old_vm = std::env::var_os(ENV_DEVSHELL_VM);
    let old_backend = std::env::var_os(ENV_DEVSHELL_VM_BACKEND);
    let old_limactl = std::env::var_os(ENV_DEVSHELL_VM_LIMACTL);
    let old_path = std::env::var_os("PATH");

    std::env::set_var(ENV_DEVSHELL_VM, "1");
    std::env::set_var(ENV_DEVSHELL_VM_BACKEND, "lima");
    std::env::remove_var(ENV_DEVSHELL_VM_LIMACTL);
    std::env::set_var("PATH", "/nonexistent_devshell_path_404");

    let mut stderr = Vec::new();
    let session = try_session_rc_or_host(&mut stderr);

    restore_var(ENV_DEVSHELL_VM, old_vm);
    restore_var(ENV_DEVSHELL_VM_BACKEND, old_backend);
    restore_var(ENV_DEVSHELL_VM_LIMACTL, old_limactl);
    restore_var("PATH", old_path);

    assert!(session.borrow().is_host_only(), "should degrade to host");
    let err = String::from_utf8(stderr).expect("stderr UTF-8");
    assert!(
        err.contains("VM unavailable"),
        "expected host fallback message, got: {err}"
    );
}
