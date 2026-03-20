use std::sync::{Mutex, OnceLock, PoisonError};

use super::super::vfs::Vfs;
use super::{
    export_vfs_to_temp_dir, find_in_path, run_in_export_dir, run_rust_tool, sync_host_dir_to_vfs,
    SandboxError,
};
use crate::test_support::cwd_mutex;

fn path_env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}

fn export_base_env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}

#[cfg(unix)]
#[test]
fn restore_execute_bits_makes_elf_under_target_executable() {
    use std::os::unix::fs::PermissionsExt;

    let root = std::env::temp_dir().join(format!(
        "devshell_elf_chmod_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&root);
    let dbg = root.join("target").join("debug");
    std::fs::create_dir_all(&dbg).unwrap();
    let bin = dbg.join("fake_hello");
    std::fs::write(&bin, [0x7F, b'E', b'L', b'F', 2]).unwrap();
    let mut perms = std::fs::metadata(&bin).unwrap().permissions();
    perms.set_mode(0o644);
    std::fs::set_permissions(&bin, perms).unwrap();
    assert_eq!(
        std::fs::metadata(&bin).unwrap().permissions().mode() & 0o777,
        0o644
    );

    super::restore_execute_bits_for_build_artifacts(&root).unwrap();
    assert_eq!(
        std::fs::metadata(&bin).unwrap().permissions().mode() & 0o777,
        0o755
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn devshell_export_parent_dir_respects_devshell_export_base() {
    let _g = export_base_env_lock();
    let tmp = std::env::temp_dir().join(format!(
        "devshell_base_env_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    let old = std::env::var(super::ENV_EXPORT_BASE).ok();
    std::env::set_var(super::ENV_EXPORT_BASE, &tmp);
    assert_eq!(super::devshell_export_parent_dir(), tmp);
    match old {
        Some(v) => std::env::set_var(super::ENV_EXPORT_BASE, v),
        None => std::env::remove_var(super::ENV_EXPORT_BASE),
    }
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn sandbox_error_display() {
    let e = SandboxError::ExportFailed(std::io::Error::other("e"));
    assert!(e.to_string().contains("export failed"));
    let e = SandboxError::CopyFailed(super::super::vfs::VfsError::InvalidPath);
    assert!(e.to_string().contains("copy to host"));
    let e = SandboxError::SyncBackFailed(std::io::Error::other("s"));
    assert!(e.to_string().contains("sync back"));
}

/// Needs `unshare(CLONE_NEWNS)` + `mount` (often blocked in sandboxes / CI).
#[cfg(target_os = "linux")]
#[test]
#[ignore = "requires unshare+mount (EPERM in some environments); run: cargo test -p xtask-todo-lib run_in_export_dir_true_with_mount_namespace -- --ignored"]
fn run_in_export_dir_true_with_mount_namespace() {
    let _g = path_env_lock();
    let old = std::env::var("DEVSHELL_RUST_MOUNT_NAMESPACE").ok();
    std::env::set_var("DEVSHELL_RUST_MOUNT_NAMESPACE", "1");
    let dir = std::env::temp_dir().join(format!(
        "devshell_run_ns_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let true_path = find_in_path("true").expect("true in PATH");
    let status = run_in_export_dir(&dir, true_path, &[]).unwrap();
    assert!(status.success());
    let _ = std::fs::remove_dir_all(&dir);
    match old {
        Some(v) => std::env::set_var("DEVSHELL_RUST_MOUNT_NAMESPACE", v),
        None => std::env::remove_var("DEVSHELL_RUST_MOUNT_NAMESPACE"),
    }
}

#[test]
fn find_in_path_returns_none_when_missing() {
    let _g = path_env_lock();
    let old = std::env::var_os("PATH");
    std::env::set_var("PATH", "/nonexistent_devshell_path_999");
    assert!(find_in_path("no_such_program_xyz").is_none());
    match old {
        Some(p) => std::env::set_var("PATH", p),
        None => std::env::remove_var("PATH"),
    }
}

#[test]
fn run_in_export_dir_runs_true_on_unix() {
    #[cfg(unix)]
    {
        let _g = path_env_lock();
        let dir = std::env::temp_dir().join(format!(
            "devshell_run_in_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let true_path = find_in_path("true").expect("true in PATH");
        let status = run_in_export_dir(&dir, true_path, &[]).unwrap();
        assert!(status.success());
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[test]
fn run_rust_tool_true_on_unix() {
    #[cfg(unix)]
    {
        let _g = path_env_lock();
        let mut vfs = Vfs::new();
        let status = run_rust_tool(&mut vfs, "/", "true", &[]).unwrap();
        assert!(status.success());
    }
}

#[test]
fn run_rust_tool_program_not_found() {
    let _g = path_env_lock();
    let mut vfs = Vfs::new();
    let r = run_rust_tool(&mut vfs, "/", "nonexistent_program_devshell_xyz", &[]);
    assert!(r.is_err());
    let err = r.unwrap_err().to_string();
    assert!(err.contains("not found") || err.contains("PATH"));
}

#[test]
fn sync_skips_when_host_export_leaf_missing() {
    let mut vfs = Vfs::new();
    let base = std::env::temp_dir().join(format!("devshell_sync_skip_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir(&base).unwrap();
    sync_host_dir_to_vfs(&base, "/no_such_exported_node", &mut vfs).unwrap();
    let _ = std::fs::remove_dir(&base);
}

#[test]
fn export_empty_cwd_creates_dir() {
    let vfs = Vfs::new();
    let path = export_vfs_to_temp_dir(&vfs, "/").unwrap();
    assert!(path.is_dir());
    assert!(path
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .starts_with("devshell_"));
    let _ = std::fs::remove_dir(path);
}

#[test]
fn export_with_files_and_dirs() {
    let _cwd = cwd_mutex();
    let mut vfs = Vfs::new();
    vfs.mkdir("/proj").unwrap();
    vfs.set_cwd("/proj").unwrap();
    vfs.write_file("/proj/Cargo.toml", b"[package]\nname = \"foo\"\n")
        .unwrap();
    vfs.mkdir("/proj/src").unwrap();
    vfs.write_file("/proj/src/main.rs", b"fn main() {}\n")
        .unwrap();
    let path = export_vfs_to_temp_dir(&vfs, "/proj").unwrap();
    assert!(path.is_dir());
    // copy_tree_to_host exports the node at /proj into path, so content is path/proj/...
    let proj = path.join("proj");
    let cargo = proj.join("Cargo.toml");
    let main = proj.join("src/main.rs");
    assert!(cargo.is_file(), "proj/Cargo.toml should exist");
    assert!(main.is_file(), "proj/src/main.rs should exist");
    let content = std::fs::read_to_string(&cargo).unwrap();
    assert!(content.contains("foo"));
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn sync_host_to_vfs_adds_files() {
    let mut vfs = Vfs::new();
    vfs.mkdir("/proj").unwrap();
    let path = export_vfs_to_temp_dir(&vfs, "/proj").unwrap();
    let proj_host = path.join("proj");
    std::fs::write(proj_host.join("new.txt"), b"hello").unwrap();
    sync_host_dir_to_vfs(&path, "/proj", &mut vfs).unwrap();
    let content = vfs.read_file("/proj/new.txt").unwrap();
    assert_eq!(content, b"hello");
    let _ = std::fs::remove_dir_all(path);
}

/// Regression: cwd for cargo must be `export_dir/<last-segment>`, not `export_dir/<full-vfs-path>`.
/// Otherwise `cargo run` fails with ENOENT on `current_dir` (misreported as `CargoNotFound`).
#[test]
fn nested_vfs_path_host_uses_leaf_dir_not_full_path() {
    let mut vfs = Vfs::new();
    vfs.mkdir("/projects").unwrap();
    vfs.mkdir("/projects/hello").unwrap();
    vfs.write_file(
        "/projects/hello/Cargo.toml",
        b"[package]\nname = \"hello\"\n",
    )
    .unwrap();

    let export = export_vfs_to_temp_dir(&vfs, "/projects/hello").unwrap();
    let hello = export.join("hello");
    let wrong = export.join("projects").join("hello");
    assert!(
        hello.join("Cargo.toml").is_file(),
        "expected export at export_dir/hello/, matching copy_tree_to_host"
    );
    assert!(
        !wrong.join("Cargo.toml").is_file(),
        "must not use export_dir/projects/hello (that path is empty)"
    );

    std::fs::write(hello.join("synced.txt"), b"ok").unwrap();
    sync_host_dir_to_vfs(&export, "/projects/hello", &mut vfs).unwrap();
    assert_eq!(vfs.read_file("/projects/hello/synced.txt").unwrap(), b"ok");

    let _ = std::fs::remove_dir_all(export);
}
