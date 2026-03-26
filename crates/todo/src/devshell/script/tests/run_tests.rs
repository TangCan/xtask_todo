use std::cell::RefCell;
use std::io::Cursor;
use std::rc::Rc;

use crate::devshell::vfs::Vfs;

use super::super::exec::run_script;
use super::helpers::vm_session_test;

#[test]
fn run_script_assign_and_echo() {
    let vfs = Rc::new(RefCell::new(Vfs::new()));
    let vm = vm_session_test();
    let script = "X=hello\necho $X\n";
    let mut stdin = Cursor::new(b"");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let r = run_script(
        &vfs,
        &vm,
        script,
        false,
        &mut stdin,
        &mut stdout,
        &mut stderr,
    );
    assert!(r.is_ok(), "stderr: {}", String::from_utf8_lossy(&stderr));
    let out = String::from_utf8(stdout).unwrap();
    assert!(
        out.contains("hello"),
        "stdout should contain 'hello': {out}"
    );
}

#[test]
fn run_script_for_loop() {
    let vfs = Rc::new(RefCell::new(Vfs::new()));
    let vm = vm_session_test();
    let script = "for x in one two three; do\necho $x\ndone\n";
    let mut stdin = Cursor::new(b"");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let r = run_script(
        &vfs,
        &vm,
        script,
        false,
        &mut stdin,
        &mut stdout,
        &mut stderr,
    );
    assert!(r.is_ok(), "stderr: {}", String::from_utf8_lossy(&stderr));
    let out = String::from_utf8(stdout).unwrap();
    assert!(
        out.contains("one") && out.contains("two") && out.contains("three"),
        "stdout: {out}"
    );
}

#[test]
fn run_script_if_then_else() {
    let vfs = Rc::new(RefCell::new(Vfs::new()));
    let vm = vm_session_test();
    // pwd succeeds, so then branch runs
    let script = "if pwd; then\necho then\nelse\necho else\nfi\n";
    let mut stdin = Cursor::new(b"");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let r = run_script(
        &vfs,
        &vm,
        script,
        false,
        &mut stdin,
        &mut stdout,
        &mut stderr,
    );
    assert!(r.is_ok(), "stderr: {}", String::from_utf8_lossy(&stderr));
    let out = String::from_utf8(stdout).unwrap();
    assert!(out.contains("then"), "stdout should contain 'then': {out}");
    assert!(
        !out.contains("else"),
        "stdout should not contain 'else': {out}"
    );
}

#[test]
fn run_script_if_else_branch_runs_when_cond_fails() {
    let vfs = Rc::new(RefCell::new(Vfs::new()));
    let vm = vm_session_test();
    let script = "if nosuchcommand_xy; then\necho then\nelse\necho else\nfi\n";
    let mut stdin = Cursor::new(b"");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let r = run_script(
        &vfs,
        &vm,
        script,
        false,
        &mut stdin,
        &mut stdout,
        &mut stderr,
    );
    assert!(r.is_ok(), "stderr: {}", String::from_utf8_lossy(&stderr));
    let out = String::from_utf8(stdout).unwrap();
    assert!(out.contains("else"), "stdout should contain 'else': {out}");
    assert!(
        !out.contains("then"),
        "stdout should not contain 'then': {out}"
    );
}

#[test]
fn run_script_set_e_fail_exits() {
    let vfs = Rc::new(RefCell::new(Vfs::new()));
    let vm = vm_session_test();
    // unknown command fails; with set_e script should return Err
    let script = "echo ok\nnosuchcommand_xy\necho after\n";
    let mut stdin = Cursor::new(b"");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let r = run_script(
        &vfs,
        &vm,
        script,
        true,
        &mut stdin,
        &mut stdout,
        &mut stderr,
    );
    assert!(
        r.is_err(),
        "with set_e, script should fail on unknown command"
    );
    let out = String::from_utf8(stdout).unwrap();
    assert!(
        out.contains("ok"),
        "stdout should contain first echo: {out}"
    );
    assert!(
        !out.contains("after"),
        "should not run after failing command: {out}"
    );
}

#[test]
fn run_script_set_e_script_inner() {
    let vfs = Rc::new(RefCell::new(Vfs::new()));
    let vm = vm_session_test();
    // set -e inside script turns on exit-on-error for rest of script
    let script = "echo first\nset -e\necho second\nnosuchcommand_xy\necho third\n";
    let mut stdin = Cursor::new(b"");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let r = run_script(
        &vfs,
        &vm,
        script,
        false,
        &mut stdin,
        &mut stdout,
        &mut stderr,
    );
    assert!(r.is_err());
    let out = String::from_utf8(stdout).unwrap();
    assert!(out.contains("first") && out.contains("second"));
    assert!(!out.contains("third"));
}

#[test]
fn run_script_command_parse_error_returns_failed() {
    let vfs = Rc::new(RefCell::new(Vfs::new()));
    let vm = vm_session_test();
    let script = "echo >\n"; // redirect missing path -> parse error in run_command_line
    let mut stdin = Cursor::new(b"");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let r = run_script(
        &vfs,
        &vm,
        script,
        false,
        &mut stdin,
        &mut stdout,
        &mut stderr,
    );
    assert!(r.is_ok()); // script runs, the command fails but we don't have set -e
    let err = String::from_utf8(stderr).unwrap();
    assert!(err.contains("parse") || err.contains("redirect") || !err.is_empty());
}

#[test]
fn run_script_parse_error_returns_err() {
    let vfs = Rc::new(RefCell::new(Vfs::new()));
    let vm = vm_session_test();
    let script = "if true; then\n  echo x\n"; // missing fi
    let mut stdin = Cursor::new(b"");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let r = run_script(
        &vfs,
        &vm,
        script,
        false,
        &mut stdin,
        &mut stdout,
        &mut stderr,
    );
    assert!(r.is_err());
    let err = String::from_utf8(stderr).unwrap();
    assert!(err.contains("parse") || err.contains("fi") || !err.is_empty());
}

#[test]
fn run_script_source_nonexistent_returns_err() {
    let vfs = Rc::new(RefCell::new(Vfs::new()));
    let vm = vm_session_test();
    let script = "source /nonexistent_devshell_path_xyz_123\necho ok\n";
    let mut stdin = Cursor::new(b"");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let r = run_script(
        &vfs,
        &vm,
        script,
        false,
        &mut stdin,
        &mut stdout,
        &mut stderr,
    );
    assert!(r.is_err());
    let err = String::from_utf8(stderr).unwrap();
    assert!(err.contains("source") || err.contains("cannot read") || !err.is_empty());
}

#[test]
fn run_script_source_host_file() {
    let vfs = Rc::new(RefCell::new(Vfs::new()));
    let vm = vm_session_test();
    let dir = std::env::temp_dir().join(format!("devshell_source_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let included = dir.join("included.dsh");
    std::fs::write(&included, "echo sourced\n").unwrap();
    let script = format!("echo before\nsource {}\necho after\n", included.display());
    let mut stdin = Cursor::new(b"");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let r = run_script(
        &vfs,
        &vm,
        &script,
        false,
        &mut stdin,
        &mut stdout,
        &mut stderr,
    );
    assert!(r.is_ok(), "stderr: {}", String::from_utf8_lossy(&stderr));
    let out = String::from_utf8(stdout).unwrap();
    assert!(
        out.contains("before") && out.contains("sourced") && out.contains("after"),
        "stdout: {out}"
    );
    let _ = std::fs::remove_file(&included);
    let _ = std::fs::remove_dir(&dir);
}
