//! Tests for `run_main_from_args`: usage error, file not found, failed to load, script mode.

use std::io::Cursor;

use super::super::run_main_from_args;

#[test]
fn run_main_from_args_usage_error() {
    let args = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let mut stdin = Cursor::new("");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let r = run_main_from_args(&args, false, &mut stdin, &mut stdout, &mut stderr);
    assert!(r.is_err());
    let err = String::from_utf8(stderr).unwrap();
    assert!(err.contains("usage"));
}

#[test]
fn run_main_from_args_file_not_found() {
    let path = std::env::temp_dir().join(format!("devshell_nonexist_{}", std::process::id()));
    let args = vec!["dev_shell".to_string(), path.display().to_string()];
    let mut stdin = Cursor::new("exit\n");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let r = run_main_from_args(&args, false, &mut stdin, &mut stdout, &mut stderr);
    r.unwrap();
    let err = String::from_utf8(stderr).unwrap();
    assert!(err.contains("File not found") || err.is_empty());
}

#[test]
fn run_main_from_args_failed_to_load() {
    let dir = std::env::temp_dir().join(format!("devshell_bad_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let bad = dir.join("bad.bin");
    std::fs::write(&bad, b"not valid vfs bytes").unwrap();
    let args = vec!["dev_shell".to_string(), bad.display().to_string()];
    let mut stdin = Cursor::new("exit\n");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let r = run_main_from_args(&args, false, &mut stdin, &mut stdout, &mut stderr);
    r.unwrap();
    let err = String::from_utf8(stderr).unwrap();
    assert!(err.contains("Failed to load"));
    let _ = std::fs::remove_file(&bad);
    let _ = std::fs::remove_dir(&dir);
}

#[test]
fn run_main_from_args_script_mode() {
    let dir = std::env::temp_dir().join(format!("devshell_script_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let script = dir.join("s.dsh");
    std::fs::write(&script, "echo hello\npwd\nexit\n").unwrap();
    let args = vec![
        "dev_shell".to_string(),
        "-f".to_string(),
        script.display().to_string(),
    ];
    let mut stdin = Cursor::new("");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let r = run_main_from_args(&args, false, &mut stdin, &mut stdout, &mut stderr);
    r.unwrap();
    let out = String::from_utf8(stdout).unwrap();
    assert!(
        out.contains("hello"),
        "stdout should contain 'hello': {out}"
    );
    assert!(out.contains('/'), "stdout should contain pwd output");
    let _ = std::fs::remove_file(&script);
    let _ = std::fs::remove_dir(&dir);
}

#[test]
fn run_main_from_args_script_mode_set_e() {
    let dir = std::env::temp_dir().join(format!("devshell_script_e_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let script = dir.join("s.dsh");
    std::fs::write(&script, "echo ok\nnosuchcommand_xy\necho after\n").unwrap();
    let args = vec![
        "dev_shell".to_string(),
        "-e".to_string(),
        "-f".to_string(),
        script.display().to_string(),
    ];
    let mut stdin = Cursor::new("");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let r = run_main_from_args(&args, false, &mut stdin, &mut stdout, &mut stderr);
    assert!(
        r.is_err(),
        "with -e, script should return error when command fails"
    );
    let out = String::from_utf8(stdout).unwrap();
    assert!(out.contains("ok"));
    assert!(!out.contains("after"));
    let _ = std::fs::remove_file(&script);
    let _ = std::fs::remove_dir(&dir);
}

#[test]
fn run_main_from_args_script_mode_requires_script_path() {
    let args = vec!["dev_shell".to_string(), "-f".to_string()];
    let mut stdin = Cursor::new("");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let r = run_main_from_args(&args, false, &mut stdin, &mut stdout, &mut stderr);
    assert!(r.is_err(), "missing script path should be usage error");
    let err = String::from_utf8(stderr).unwrap();
    assert!(err.contains("usage: dev_shell [-e] -f script.dsh"));
}

#[test]
fn run_main_from_args_script_mode_rejects_extra_positionals() {
    let args = vec![
        "dev_shell".to_string(),
        "-f".to_string(),
        "a.dsh".to_string(),
        "extra".to_string(),
    ];
    let mut stdin = Cursor::new("");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let r = run_main_from_args(&args, false, &mut stdin, &mut stdout, &mut stderr);
    assert!(r.is_err(), "extra positional args should be usage error");
    let err = String::from_utf8(stderr).unwrap();
    assert!(err.contains("usage: dev_shell [-e] -f script.dsh"));
}

#[test]
fn run_main_from_args_script_mode_missing_file_reports_path() {
    let missing = std::env::temp_dir().join(format!(
        "devshell_missing_script_{}.dsh",
        std::process::id()
    ));
    let args = vec![
        "dev_shell".to_string(),
        "-f".to_string(),
        missing.display().to_string(),
    ];
    let mut stdin = Cursor::new("");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let r = run_main_from_args(&args, false, &mut stdin, &mut stdout, &mut stderr);
    assert!(r.is_err(), "missing script should return error");
    let err = String::from_utf8(stderr).unwrap();
    assert!(err.contains("dev_shell:"));
    assert!(err.contains(&missing.display().to_string()));
}

#[test]
fn run_main_from_args_repl_branch_executes_until_exit() {
    let args = vec!["dev_shell".to_string()];
    let mut stdin = Cursor::new("echo repl_ok\nexit\n");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let r = run_main_from_args(&args, false, &mut stdin, &mut stdout, &mut stderr);
    assert!(r.is_ok(), "repl branch should execute and exit cleanly");
    let out = String::from_utf8(stdout).unwrap();
    assert!(out.contains("repl_ok"), "stdout: {out}");
}

#[test]
fn run_main_from_args_repl_source() {
    let dir = std::env::temp_dir().join(format!("devshell_repl_src_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let to_source = dir.join("repl_src.dsh");
    std::fs::write(&to_source, "echo from_sourced\n").unwrap();
    let bin_path = dir.join("vfs.bin");
    let args = vec!["dev_shell".to_string(), bin_path.display().to_string()];
    let stdin_content = format!("source {}\npwd\nexit\n", to_source.display());
    let mut stdin = Cursor::new(stdin_content);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let r = run_main_from_args(&args, false, &mut stdin, &mut stdout, &mut stderr);
    r.unwrap();
    let out = String::from_utf8(stdout).unwrap();
    assert!(
        out.contains("from_sourced"),
        "REPL source should run script: {out}"
    );
    assert!(out.contains('/'), "pwd after source");
    let _ = std::fs::remove_file(&to_source);
    let _ = std::fs::remove_dir(&dir);
}
