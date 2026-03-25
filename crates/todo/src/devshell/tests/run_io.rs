//! I/O tests: cat, redirects, pipe, unknown command, parse error, touch, export-readonly.

use std::io::Cursor;

use super::super::run_with;

#[test]
fn run_with_cat_file() {
    let input = "mkdir /d\necho content > /d/f\ncat /d/f\nexit\n";
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
    assert!(out.contains("content"));
}

#[test]
fn run_with_cat_multiple_files() {
    let input = "mkdir /d\necho a > /d/f1\necho b > /d/f2\ncat /d/f1 /d/f2\nexit\n";
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
    assert!(out.contains('a'));
    assert!(out.contains('b'));
}

#[test]
fn run_with_cat_stdin() {
    let input = "echo piped | cat\nexit\n";
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
    assert!(out.contains("piped"));
}

#[test]
fn run_with_unknown_command() {
    let input = "unknowncmd\n exit\n";
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
    let err = String::from_utf8(stderr).unwrap();
    assert!(err.contains("unknown command"));
}

#[test]
fn run_with_parse_error() {
    let input = "echo >\nexit\n";
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
    let err = String::from_utf8(stderr).unwrap();
    assert!(err.contains("parse error") || err.contains("redirect"));
}

#[test]
fn run_with_stdin_redirect() {
    let input = "mkdir /d\necho hi > /d/f\ncat < /d/f\nexit\n";
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
    assert!(out.contains("hi"));
}

#[test]
fn run_with_todo_list_json() {
    let input = "todo list --json\nexit\n";
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
    assert!(out.contains("[]") || out.contains('['));
}

#[test]
fn run_with_pipe() {
    let input = "echo one | echo two\nexit\n";
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
    assert!(out.contains("two"));
}

#[test]
fn run_with_stderr_redirect() {
    let input = "mkdir /d\nbadcmd 2> /d/err\ncat /d/err\nexit\n";
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
    assert!(out.contains("unknown command") || out.contains("badcmd"));
}

#[test]
fn run_with_touch() {
    let input = "mkdir /d\ntouch /d/f\nls /d\nexit\n";
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
    assert!(out.contains('f'));
}

#[test]
fn run_with_cd_invalid_path_reports_error() {
    let input = "cd /no_such_dir\npwd\nexit\n";
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
    let err = String::from_utf8(stderr).unwrap();
    assert!(
        err.contains("cd failed") || err.contains("error:"),
        "stderr: {err}"
    );
    let out = String::from_utf8(stdout).unwrap();
    assert!(
        out.contains('/'),
        "pwd should still run after cd failure: {out}"
    );
}

#[test]
fn run_with_cat_missing_file_reports_error() {
    let input = "cat /missing_file_321\nexit\n";
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
    let err = String::from_utf8(stderr).unwrap();
    assert!(
        err.contains("cat failed") || err.contains("error:"),
        "stderr: {err}"
    );
}

#[test]
fn run_with_sh_literal_is_not_executed_as_host_shell() {
    let input = "sh -c echo should_not_run\nexit\n";
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
    let err = String::from_utf8(stderr).unwrap();
    assert!(err.contains("unknown command: sh"), "stderr: {err}");
    let out = String::from_utf8(stdout).unwrap();
    assert!(
        !out.contains("should_not_run"),
        "stdout must not contain shell output: {out}"
    );
}

#[test]
fn run_with_eof_triggers_save_on_exit() {
    let input = "pwd\n";
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
    assert!(out.contains('/'));
}

#[test]
fn run_with_save_on_exit_fails_when_path_is_dir() {
    let dir = std::env::temp_dir();
    let input = "exit\n";
    let mut stdin = Cursor::new(input);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    run_with(
        &["dev_shell".to_string(), dir.display().to_string()],
        &mut stdin,
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    let err = String::from_utf8(stderr).unwrap();
    assert!(err.contains("save on exit failed"));
}

#[test]
fn run_with_export_readonly() {
    let input = "mkdir /out\necho x > /out/f\nexport-readonly /out\nexit\n";
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
    assert!(out.contains("dev_shell_export_") || out.contains("/tmp"));
}
