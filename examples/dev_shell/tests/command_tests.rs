//! Tests for builtin command execution (pwd, cd, ls, mkdir, cat, touch, echo, export-readonly).

use dev_shell::command::{execute_pipeline, run_builtin, ExecContext, RunResult};
use std::fs;
use std::path::Path;
use dev_shell::parser::{parse_line, SimpleCommand};
use dev_shell::vfs::Vfs;
use std::io;

fn run_builtin_with_buffers(
    vfs: &mut Vfs,
    argv: Vec<&str>,
    redirects: Vec<(u8, String)>,
) -> (Vec<u8>, Vec<u8>, Result<(), dev_shell::command::BuiltinError>) {
    let mut stdin = io::empty();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let redirects: Vec<_> = redirects
        .into_iter()
        .map(|(fd, path)| dev_shell::parser::Redirect { fd, path })
        .collect();
    let cmd = SimpleCommand {
        argv: argv.into_iter().map(String::from).collect(),
        redirects,
    };
    let mut ctx = ExecContext {
        vfs,
        stdin: &mut stdin,
        stdout: &mut stdout,
        stderr: &mut stderr,
    };
    let result = run_builtin(&mut ctx, &cmd);
    (stdout, stderr, result)
}

#[test]
fn builtin_pwd_writes_cwd_to_stdout() {
    let mut vfs = Vfs::new();
    let (stdout, _, result) = run_builtin_with_buffers(&mut vfs, vec!["pwd"], vec![]);
    result.unwrap();
    let out = String::from_utf8(stdout).unwrap();
    assert!(out.contains("/"), "stdout should contain cwd; got {:?}", out);
    assert!(out.trim_end().ends_with('/') || out == "/\n", "stdout should be cwd + newline; got {:?}", out);
}

#[test]
fn builtin_cd_changes_cwd() {
    let mut vfs = Vfs::new();
    vfs.mkdir("/foo").unwrap();
    let (_, _, result) = run_builtin_with_buffers(&mut vfs, vec!["cd", "/foo"], vec![]);
    result.unwrap();
    assert_eq!(vfs.cwd(), "/foo");
}

#[test]
fn builtin_ls_lists_directory() {
    let mut vfs = Vfs::new();
    vfs.mkdir("/foo").unwrap();
    vfs.mkdir("/foo/bar").unwrap();
    let (stdout, _, result) = run_builtin_with_buffers(&mut vfs, vec!["ls", "/foo"], vec![]);
    result.unwrap();
    let out = String::from_utf8(stdout).unwrap();
    assert!(out.contains("bar"), "ls output should contain 'bar'; got {:?}", out);
}

#[test]
fn builtin_mkdir_creates_directory() {
    let mut vfs = Vfs::new();
    let (_, _, result) = run_builtin_with_buffers(&mut vfs, vec!["mkdir", "/baz"], vec![]);
    result.unwrap();
    let names = vfs.list_dir("/").unwrap();
    assert!(names.contains(&"baz".to_string()), "ls / should contain 'baz'; got {:?}", names);
}

#[test]
fn builtin_cat_writes_file_content_to_stdout() {
    let mut vfs = Vfs::new();
    vfs.mkdir("/dir").unwrap();
    vfs.write_file("/dir/f", b"hello world").unwrap();
    let (stdout, _, result) = run_builtin_with_buffers(&mut vfs, vec!["cat", "/dir/f"], vec![]);
    result.unwrap();
    assert_eq!(stdout, b"hello world");
}

#[test]
fn builtin_touch_creates_file() {
    let mut vfs = Vfs::new();
    vfs.mkdir("/t").unwrap();
    let (_, _, result) = run_builtin_with_buffers(&mut vfs, vec!["touch", "/t/f"], vec![]);
    result.unwrap();
    let content = vfs.read_file("/t/f").unwrap();
    assert_eq!(content, b"");
}

#[test]
fn builtin_echo_writes_args_joined_by_space() {
    let mut vfs = Vfs::new();
    let (stdout, _, result) = run_builtin_with_buffers(&mut vfs, vec!["echo", "hello", "world"], vec![]);
    result.unwrap();
    let out = String::from_utf8(stdout).unwrap();
    assert_eq!(out.trim_end(), "hello world");
}

#[test]
fn builtin_help_lists_commands() {
    let mut vfs = Vfs::new();
    let (stdout, _, result) = run_builtin_with_buffers(&mut vfs, vec!["help"], vec![]);
    result.unwrap();
    let out = String::from_utf8(stdout).unwrap();
    assert!(out.contains("Supported commands"), "help should list header; got {:?}", out);
    assert!(out.contains("pwd"), "help should list pwd");
    assert!(out.contains("help"), "help should list help");
}

#[test]
fn builtin_unknown_command_returns_err_and_writes_stderr() {
    let mut vfs = Vfs::new();
    let (_, stderr, result) = run_builtin_with_buffers(&mut vfs, vec!["nonexistent_cmd"], vec![]);
    assert!(result.is_err());
    let err_out = String::from_utf8(stderr).unwrap();
    assert!(err_out.contains("unknown command"), "stderr should mention unknown command; got {:?}", err_out);
}

#[test]
fn builtin_export_readonly_copies_vfs_to_temp_dir() {
    let mut vfs = Vfs::new();
    vfs.mkdir("/x").unwrap();
    vfs.write_file("/x/f", b"hi").unwrap();
    let (stdout, _, result) = run_builtin_with_buffers(&mut vfs, vec!["export-readonly", "/"], vec![]);
    result.unwrap();
    let out = String::from_utf8(stdout).unwrap();
    let host_path = out.trim_end();
    assert!(!host_path.is_empty(), "stdout should be the temp dir path");
    assert!(Path::new(host_path).exists(), "export dir should exist: {}", host_path);
    let f_path = Path::new(host_path).join("x").join("f");
    assert!(f_path.exists(), "x/f should exist: {:?}", f_path);
    let content = fs::read_to_string(&f_path).unwrap();
    assert_eq!(content, "hi", "x/f content should be 'hi'");
}

#[test]
fn pipeline_echo_a_pipe_cat_produces_a_newline() {
    let mut vfs = Vfs::new();
    let pipeline = parse_line("echo a | cat").unwrap();
    assert_eq!(pipeline.commands.len(), 2);
    let mut stdin = io::empty();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let mut ctx = ExecContext {
        vfs: &mut vfs,
        stdin: &mut stdin,
        stdout: &mut stdout,
        stderr: &mut stderr,
    };
    let result = execute_pipeline(&mut ctx, &pipeline);
    assert_eq!(result.unwrap(), RunResult::Continue);
    assert_eq!(stdout, b"a\n", "final stdout should be 'a\\n'; got {:?}", stdout);
}

#[test]
fn builtin_save_writes_bin_default_path() {
    let mut vfs = Vfs::new();
    vfs.mkdir("/foo").unwrap();
    let dir = std::env::temp_dir().join("dev_shell_save_test");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("saved.bin");
    let path_str = path.to_string_lossy().to_string();
    let (_, _, result) = run_builtin_with_buffers(&mut vfs, vec!["save", &path_str], vec![]);
    result.unwrap();
    assert!(path.exists(), "saved.bin should exist at {:?}", path);
}

#[test]
fn execute_pipeline_exit_returns_exit_without_running() {
    let mut vfs = Vfs::new();
    let pipeline = parse_line("exit").unwrap();
    let mut stdin = io::empty();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let mut ctx = ExecContext {
        vfs: &mut vfs,
        stdin: &mut stdin,
        stdout: &mut stdout,
        stderr: &mut stderr,
    };
    let result = execute_pipeline(&mut ctx, &pipeline);
    assert_eq!(result.unwrap(), RunResult::Exit);
    assert!(stdout.is_empty());
}

#[test]
fn execute_pipeline_quit_returns_exit_without_running() {
    let mut vfs = Vfs::new();
    let pipeline = parse_line("quit").unwrap();
    let mut stdin = io::empty();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let mut ctx = ExecContext {
        vfs: &mut vfs,
        stdin: &mut stdin,
        stdout: &mut stdout,
        stderr: &mut stderr,
    };
    let result = execute_pipeline(&mut ctx, &pipeline);
    assert_eq!(result.unwrap(), RunResult::Exit);
}
