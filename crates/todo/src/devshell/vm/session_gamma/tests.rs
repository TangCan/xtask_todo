#[test]
fn guest_dir_for_cwd_root() {
    assert_eq!(
        super::helpers::guest_dir_for_cwd_inner("/workspace", "/"),
        "/workspace"
    );
}

#[test]
fn guest_dir_for_cwd_nested() {
    assert_eq!(
        super::helpers::guest_dir_for_cwd_inner("/workspace", "/projects/hello"),
        "/workspace/hello"
    );
}

#[test]
fn sanitize_instance_replaces_dots() {
    assert_eq!(super::helpers::sanitize_instance_segment("a.b"), "a_b");
}

#[test]
fn workspace_parent_legacy_cache_when_cargo_disabled() {
    use super::workspace_parent_for_instance;
    use crate::devshell::vm::session_gamma::env::{
        ENV_DEVSHELL_VM_WORKSPACE_PARENT, ENV_DEVSHELL_VM_WORKSPACE_USE_CARGO_ROOT,
    };

    let _g = crate::test_support::vm_env_mutex();

    let old_p = std::env::var(ENV_DEVSHELL_VM_WORKSPACE_PARENT).ok();
    let old_u = std::env::var(ENV_DEVSHELL_VM_WORKSPACE_USE_CARGO_ROOT).ok();
    std::env::remove_var(ENV_DEVSHELL_VM_WORKSPACE_PARENT);
    std::env::set_var(ENV_DEVSHELL_VM_WORKSPACE_USE_CARGO_ROOT, "0");

    let wp = workspace_parent_for_instance("my.instance");
    let s = wp.to_string_lossy();
    assert!(
        s.contains("vm-workspace") && s.contains("my_instance"),
        "expected cache layout, got {wp:?}"
    );

    match old_p {
        Some(ref v) => std::env::set_var(ENV_DEVSHELL_VM_WORKSPACE_PARENT, v),
        None => std::env::remove_var(ENV_DEVSHELL_VM_WORKSPACE_PARENT),
    }
    match old_u {
        Some(ref v) => std::env::set_var(ENV_DEVSHELL_VM_WORKSPACE_USE_CARGO_ROOT, v),
        None => std::env::remove_var(ENV_DEVSHELL_VM_WORKSPACE_USE_CARGO_ROOT),
    }
}

#[test]
fn guest_dir_for_host_path_under_workspace_maps_relative() {
    let tmp = std::env::temp_dir().join(format!(
        "lima_guest_map_{}_{}",
        std::process::id(),
        std::time::SystemTime::UNIX_EPOCH
            .elapsed()
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(tmp.join("proj")).unwrap();
    let ws = tmp.canonicalize().unwrap();
    let host = ws.join("proj").canonicalize().unwrap();
    let g = super::helpers::guest_dir_for_host_path_under_workspace(&ws, "/workspace", &host);
    assert_eq!(g.as_deref(), Some("/workspace/proj"));
    let _ = std::fs::remove_dir_all(&tmp);
}

/// When `DEVSHELL_VM_WORKSPACE_MODE=guest` and γ is available, push/pull is disabled for rust tools.
#[cfg(unix)]
#[test]
fn gamma_session_sync_flag_follows_workspace_mode() {
    use crate::devshell::sandbox;
    use crate::devshell::vm::{
        GammaSession, VmConfig, ENV_DEVSHELL_VM, ENV_DEVSHELL_VM_BACKEND,
        ENV_DEVSHELL_VM_WORKSPACE_MODE,
    };

    let _g = crate::test_support::vm_env_mutex();

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
