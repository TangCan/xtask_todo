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

/// When `DEVSHELL_VM_WORKSPACE_MODE=guest` and γ is available, push/pull is disabled for rust tools.
#[cfg(unix)]
#[test]
fn gamma_session_sync_flag_follows_workspace_mode() {
    use std::sync::{Mutex, OnceLock};

    use crate::devshell::sandbox;
    use crate::devshell::vm::{
        GammaSession, VmConfig, ENV_DEVSHELL_VM, ENV_DEVSHELL_VM_BACKEND,
        ENV_DEVSHELL_VM_WORKSPACE_MODE,
    };

    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let _g = LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();

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
