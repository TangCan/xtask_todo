use std::cell::RefCell;
use std::io::Cursor;
use std::path::PathBuf;
use std::rc::Rc;

use crate::test_support::{cwd_mutex, devshell_workspace_env_mutex, vm_env_mutex};

use super::super::vfs::Vfs;
use super::super::vm::{
    SessionHolder, VmConfig, ENV_DEVSHELL_VM, ENV_DEVSHELL_VM_BACKEND, ENV_DEVSHELL_VM_SOCKET,
    ENV_DEVSHELL_VM_WORKSPACE_MODE,
};
use super::{process_line, run, should_add_history_entry, StepResult};

fn vm_test() -> Rc<RefCell<SessionHolder>> {
    Rc::new(RefCell::new(SessionHolder::new_host()))
}

#[test]
fn process_line_empty_returns_continue() {
    let vfs = Rc::new(RefCell::new(Vfs::new()));
    let vm = vm_test();
    let mut stdin = Cursor::new(b"");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let r = process_line(&vfs, &vm, "  \n", &mut stdin, &mut stdout, &mut stderr);
    assert_eq!(r, StepResult::Continue);
}

#[test]
fn process_line_exit_returns_exit() {
    let vfs = Rc::new(RefCell::new(Vfs::new()));
    let vm = vm_test();
    let mut stdin = Cursor::new(b"");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let r = process_line(&vfs, &vm, "exit", &mut stdin, &mut stdout, &mut stderr);
    assert_eq!(r, StepResult::Exit);
}

#[test]
fn process_line_quit_returns_exit() {
    let vfs = Rc::new(RefCell::new(Vfs::new()));
    let vm = vm_test();
    let mut stdin = Cursor::new(b"");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let r = process_line(&vfs, &vm, "quit", &mut stdin, &mut stdout, &mut stderr);
    assert_eq!(r, StepResult::Exit);
}

#[test]
fn process_line_parse_error_returns_continue() {
    let vfs = Rc::new(RefCell::new(Vfs::new()));
    let vm = vm_test();
    let mut stdin = Cursor::new(b"");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let r = process_line(&vfs, &vm, "echo >", &mut stdin, &mut stdout, &mut stderr);
    assert_eq!(r, StepResult::Continue);
    let err = String::from_utf8(stderr).unwrap();
    assert!(err.contains("parse error"));
}

#[test]
fn process_line_pwd_continues_and_writes() {
    let vfs = Rc::new(RefCell::new(Vfs::new()));
    let vm = vm_test();
    let mut stdin = Cursor::new(b"");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let r = process_line(&vfs, &vm, "pwd", &mut stdin, &mut stdout, &mut stderr);
    assert_eq!(r, StepResult::Continue);
    let out = String::from_utf8(stdout).unwrap();
    assert!(out.contains('/'));
}

#[test]
fn process_line_unknown_command_continues_and_stderr() {
    let vfs = Rc::new(RefCell::new(Vfs::new()));
    let vm = vm_test();
    let mut stdin = Cursor::new(b"");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let r = process_line(
        &vfs,
        &vm,
        "unknowncmd",
        &mut stdin,
        &mut stdout,
        &mut stderr,
    );
    assert_eq!(r, StepResult::Continue);
    let err = String::from_utf8(stderr).unwrap();
    assert!(err.contains("unknown command"));
}

#[test]
fn process_line_source_runs_script_from_host() {
    let vfs = Rc::new(RefCell::new(Vfs::new()));
    let vm = vm_test();
    let dir = std::env::temp_dir().join(format!("devshell_repl_source_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let script_path = dir.join("repl_sourced.dsh");
    std::fs::write(&script_path, "echo repl_sourced\n").unwrap();
    let line = format!("source {}", script_path.display());
    let mut stdin = Cursor::new(b"");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let r = process_line(&vfs, &vm, &line, &mut stdin, &mut stdout, &mut stderr);
    assert_eq!(r, StepResult::Continue);
    let out = String::from_utf8(stdout).unwrap();
    assert!(out.contains("repl_sourced"), "stdout: {out}");
    let _ = std::fs::remove_file(&script_path);
    let _ = std::fs::remove_dir(&dir);
}

#[test]
fn process_line_dot_path_runs_script() {
    let vfs = Rc::new(RefCell::new(Vfs::new()));
    let vm = vm_test();
    let dir = std::env::temp_dir().join(format!("devshell_repl_dot_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let script_path = dir.join("dot_sourced.dsh");
    std::fs::write(&script_path, "echo dot_ok\n").unwrap();
    let line = format!(". {}", script_path.display());
    let mut stdin = Cursor::new(b"");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let r = process_line(&vfs, &vm, &line, &mut stdin, &mut stdout, &mut stderr);
    assert_eq!(r, StepResult::Continue);
    let out = String::from_utf8(stdout).unwrap();
    assert!(out.contains("dot_ok"), "stdout: {out}");
    let _ = std::fs::remove_file(&script_path);
    let _ = std::fs::remove_dir(&dir);
}

#[test]
fn should_add_history_entry_skips_empty_and_duplicate() {
    assert!(!should_add_history_entry("", None));
    assert!(!should_add_history_entry("   ", None));
    assert!(should_add_history_entry("pwd", None));
    assert!(!should_add_history_entry("pwd", Some("pwd")));
    assert!(should_add_history_entry("ls", Some("pwd")));
}

fn tmp_dir(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "xtask_devshell_repl_{name}_{}_{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ))
}

fn restore_var(key: &str, val: Option<std::ffi::OsString>) {
    match val {
        Some(v) => std::env::set_var(key, v),
        None => std::env::remove_var(key),
    }
}

#[cfg(feature = "beta-vm")]
#[test]
fn run_readline_guest_primary_eof_writes_session_json_not_legacy_bin() {
    let _cwd = cwd_mutex();
    let _vm = vm_env_mutex();
    let _ws = devshell_workspace_env_mutex();

    let dir = tmp_dir("guest_primary_eof");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let dir = dir.canonicalize().expect("canonicalize temp dir");
    let old_cwd = std::env::current_dir().expect("current_dir");
    std::env::set_current_dir(&dir).expect("chdir");

    let old_vm = std::env::var_os(ENV_DEVSHELL_VM);
    let old_backend = std::env::var_os(ENV_DEVSHELL_VM_BACKEND);
    let old_mode = std::env::var_os(ENV_DEVSHELL_VM_WORKSPACE_MODE);
    let old_sock = std::env::var_os(ENV_DEVSHELL_VM_SOCKET);
    let old_wsroot = std::env::var_os(super::session_store::ENV_DEVSHELL_WORKSPACE_ROOT);

    std::env::set_var(ENV_DEVSHELL_VM, "1");
    std::env::set_var(ENV_DEVSHELL_VM_BACKEND, "beta");
    std::env::set_var(ENV_DEVSHELL_VM_WORKSPACE_MODE, "guest");
    std::env::set_var(ENV_DEVSHELL_VM_SOCKET, "tcp:127.0.0.1:9");
    std::env::remove_var(super::session_store::ENV_DEVSHELL_WORKSPACE_ROOT);

    let cfg = VmConfig::from_env();
    let session = SessionHolder::try_from_config(&cfg).expect("beta session from config");
    assert!(
        session.is_guest_primary(),
        "mode=guest + backend=beta should be guest-primary"
    );
    let vm_session = Rc::new(RefCell::new(session));

    let vfs = Rc::new(RefCell::new(Vfs::new()));
    let bin_path = dir.join(".dev_shell.bin");
    let mut stdin = Cursor::new(Vec::<u8>::new());
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    run(
        &vfs,
        &vm_session,
        false,
        &bin_path,
        &mut stdin,
        &mut stdout,
        &mut stderr,
    )
    .expect("run readline eof");

    let session_json = dir.join(".cargo-devshell").join("session.json");
    assert!(
        !bin_path.exists(),
        "guest-primary exit should skip legacy bin save"
    );
    assert!(
        session_json.is_file(),
        "expected guest-primary session JSON"
    );
    let json = std::fs::read_to_string(&session_json).expect("read session json");
    assert!(
        json.contains("\"format\": \"devshell_session_v1\""),
        "session format should be v1: {json}"
    );

    restore_var(ENV_DEVSHELL_VM, old_vm);
    restore_var(ENV_DEVSHELL_VM_BACKEND, old_backend);
    restore_var(ENV_DEVSHELL_VM_WORKSPACE_MODE, old_mode);
    restore_var(ENV_DEVSHELL_VM_SOCKET, old_sock);
    restore_var(
        super::session_store::ENV_DEVSHELL_WORKSPACE_ROOT,
        old_wsroot,
    );
    let _ = std::env::set_current_dir(old_cwd);
    let _ = std::fs::remove_dir_all(&dir);
}
