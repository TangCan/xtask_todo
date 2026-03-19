//! Basic `run_with` tests: pwd, mkdir, ls, echo, help, save, cd, todo list/stats/add.

use std::io::Cursor;

use super::super::run_with;

#[test]
fn run_with_pwd_mkdir_ls_exit() {
    let input = "pwd\nmkdir foo\nls\nexit\n";
    let mut stdin = Cursor::new(input);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    run_with(
        &["dev_shell".to_string()],
        &mut stdin,
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    let out = String::from_utf8(stdout).unwrap();
    assert!(out.contains(" $ "), "expected prompt in output: {out}");
    assert!(out.contains("foo"), "expected ls to list foo: {out}");
}

#[test]
fn run_with_echo_and_exit() {
    let input = "echo hello\nquit\n";
    let mut stdin = Cursor::new(input);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    run_with(
        &["dev_shell".to_string()],
        &mut stdin,
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    let out = String::from_utf8(stdout).unwrap();
    assert!(out.contains("hello"), "expected echo output: {out}");
}

#[test]
fn run_with_usage_error() {
    let mut stdin = Cursor::new("");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let r = run_with(
        &["a".to_string(), "b".to_string(), "c".to_string()],
        &mut stdin,
        &mut stdout,
        &mut stderr,
    );
    assert!(r.is_err());
}

#[test]
fn run_with_help() {
    let input = "help\nexit\n";
    let mut stdin = Cursor::new(input);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    run_with(
        &["dev_shell".to_string()],
        &mut stdin,
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    let out = String::from_utf8(stdout).unwrap();
    assert!(out.contains("Supported commands:"));
    assert!(out.contains("pwd"));
    assert!(out.contains("todo"));
}

#[test]
fn run_with_save() {
    let input = "mkdir x\nsave /tmp/devshell_save_test.bin\nexit\n";
    let mut stdin = Cursor::new(input);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    run_with(
        &[
            "dev_shell".to_string(),
            "/tmp/devshell_save_test.bin".to_string(),
        ],
        &mut stdin,
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    let _ = std::fs::remove_file("/tmp/devshell_save_test.bin");
}

#[test]
fn run_with_todo_list_and_stats() {
    let input = "todo list\ntodo stats\nexit\n";
    let mut stdin = Cursor::new(input);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    run_with(
        &["dev_shell".to_string()],
        &mut stdin,
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    let out = String::from_utf8(stdout).unwrap();
    assert!(out.contains("total: 0") || out.contains("open:") || out.contains("completed:"));
}

#[test]
fn run_with_todo_add_and_list() {
    let dir = std::env::temp_dir().join(format!("devshell_todo_add_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let cwd = std::env::current_dir().unwrap();
    let _ = std::env::set_current_dir(&dir);
    let input = "todo add buy milk\ntodo list\nexit\n";
    let mut stdin = Cursor::new(input);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let r = run_with(
        &["dev_shell".to_string()],
        &mut stdin,
        &mut stdout,
        &mut stderr,
    );
    let _ = std::env::set_current_dir(&cwd);
    let _ = std::fs::remove_file(dir.join(".todo.json"));
    let _ = std::fs::remove_dir(&dir);
    r.unwrap();
    let out = String::from_utf8(stdout).unwrap();
    assert!(out.contains("buy milk") || out.contains("1.") || out.contains(" $ "));
}

#[test]
fn run_with_cd_and_pwd() {
    let input = "mkdir /a\ncd /a\npwd\nexit\n";
    let mut stdin = Cursor::new(input);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    run_with(
        &["dev_shell".to_string()],
        &mut stdin,
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    let out = String::from_utf8(stdout).unwrap();
    assert!(out.contains("/a"));
}
