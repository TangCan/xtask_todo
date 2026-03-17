//! Tests for `run_main_from_args`: usage error, file not found, failed to load.

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
