//! Tests for script parse, expand, logical lines, and `run_script`.

use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Cursor;
use std::path::Path;
use std::rc::Rc;

use crate::devshell::vfs::Vfs;

use super::ast::ScriptStmt;
use super::exec::{expand_vars, logical_lines, run_script};
use super::parse::parse_script;

#[test]
fn expand_vars_empty_map() {
    assert_eq!(expand_vars("echo $X", &HashMap::new()), "echo ");
    assert_eq!(expand_vars("a${Y}b", &HashMap::new()), "ab");
}

#[test]
fn expand_vars_dollar_name() {
    let mut v = HashMap::new();
    v.insert("X".to_string(), "hello".to_string());
    v.insert("AB".to_string(), "world".to_string());
    v.insert("X_".to_string(), "suffix".to_string());
    assert_eq!(expand_vars("echo $X", &v), "echo hello");
    assert_eq!(expand_vars("$X $AB", &v), "hello world");
    assert_eq!(expand_vars("$X_", &v), "suffix");
    assert_eq!(expand_vars("$X._", &v), "hello._");
}

#[test]
fn expand_vars_brace() {
    let mut v = HashMap::new();
    v.insert("VAR".to_string(), "val".to_string());
    assert_eq!(expand_vars("${VAR}", &v), "val");
    assert_eq!(expand_vars("a${VAR}b", &v), "avalb");
}

#[test]
fn expand_vars_no_match_stays() {
    let v = HashMap::new();
    assert_eq!(expand_vars("$", &v), "$");
    assert_eq!(expand_vars("a$ b", &v), "a$ b");
    assert_eq!(expand_vars("${", &v), "${");
}

#[test]
fn logical_lines_comments_and_blank() {
    let src = "echo a # comment\necho b\n  \n";
    let got = logical_lines(src);
    assert_eq!(got, ["echo a", "echo b"]);
}

#[test]
fn logical_lines_continuation() {
    let src = "echo \\\nworld";
    let got = logical_lines(src);
    assert_eq!(got.len(), 1);
    assert_eq!(got[0], "echo world");
}

#[test]
fn logical_lines_continuation_then_comment() {
    let src = "echo \\\nhello # rest";
    let got = logical_lines(src);
    assert_eq!(got.len(), 1);
    assert_eq!(got[0], "echo hello");
}

#[test]
fn logical_lines_empty_source() {
    assert!(logical_lines("").is_empty());
    assert!(logical_lines("\n\n  # only comment\n").is_empty());
}

#[test]
fn parse_script_assign_and_command() {
    let lines = vec!["X=hello".to_string(), "echo $X".to_string()];
    let stmts = parse_script(&lines).unwrap();
    assert_eq!(stmts.len(), 2);
    match &stmts[0] {
        ScriptStmt::Assign(n, v) => {
            assert_eq!(n, "X");
            assert_eq!(v, "hello");
        }
        _ => panic!("expected Assign"),
    }
    match &stmts[1] {
        ScriptStmt::Command(c) => assert_eq!(c, "echo $X"),
        _ => panic!("expected Command"),
    }
}

#[test]
fn parse_script_set_e() {
    let lines = vec!["set -e".to_string(), "echo x".to_string()];
    let stmts = parse_script(&lines).unwrap();
    assert_eq!(stmts.len(), 2);
    assert!(matches!(stmts[0], ScriptStmt::SetE));
}

#[test]
fn parse_script_source() {
    let lines = vec!["source foo.dsh".to_string()];
    let stmts = parse_script(&lines).unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        ScriptStmt::Source(p) => assert_eq!(p, "foo.dsh"),
        _ => panic!("expected Source"),
    }
}

#[test]
fn parse_script_for_loop() {
    let lines = vec![
        "for x in a b c; do".to_string(),
        "echo $x".to_string(),
        "done".to_string(),
    ];
    let stmts = parse_script(&lines).unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        ScriptStmt::For { var, words, body } => {
            assert_eq!(var, "x");
            assert_eq!(words, &["a", "b", "c"]);
            assert_eq!(body.len(), 1);
            assert!(matches!(&body[0], ScriptStmt::Command(c) if c == "echo $x"));
        }
        _ => panic!("expected For"),
    }
}

#[test]
fn parse_script_if_then_fi() {
    let lines = vec![
        "if pwd; then".to_string(),
        "echo yes".to_string(),
        "fi".to_string(),
    ];
    let stmts = parse_script(&lines).unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        ScriptStmt::If {
            cond,
            then_body,
            else_body,
        } => {
            assert_eq!(cond, "pwd");
            assert_eq!(then_body.len(), 1);
            assert!(matches!(&then_body[0], ScriptStmt::Command(c) if c == "echo yes"));
            assert!(else_body.is_none());
        }
        _ => panic!("expected If"),
    }
}

#[test]
fn parse_script_if_missing_fi_err() {
    let lines = vec!["if pwd; then".to_string(), "echo x".to_string()];
    assert!(parse_script(&lines).is_err());
}

#[test]
fn parse_script_while_loop() {
    let lines = vec![
        "while pwd; do".to_string(),
        "echo loop".to_string(),
        "done".to_string(),
    ];
    let stmts = parse_script(&lines).unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        ScriptStmt::While { cond, body } => {
            assert_eq!(cond, "pwd");
            assert_eq!(body.len(), 1);
            assert!(matches!(&body[0], ScriptStmt::Command(c) if c == "echo loop"));
        }
        _ => panic!("expected While"),
    }
}

#[test]
fn parse_script_if_else_fi() {
    let lines = vec![
        "if false; then".to_string(),
        "echo yes".to_string(),
        "else".to_string(),
        "echo no".to_string(),
        "fi".to_string(),
    ];
    let stmts = parse_script(&lines).unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        ScriptStmt::If {
            then_body,
            else_body,
            ..
        } => {
            assert_eq!(then_body.len(), 1);
            assert!(else_body.is_some());
            let else_b = else_body.as_ref().unwrap();
            assert_eq!(else_b.len(), 1);
            assert!(matches!(&else_b[0], ScriptStmt::Command(c) if c == "echo no"));
        }
        _ => panic!("expected If"),
    }
}

#[test]
fn run_script_assign_and_echo() {
    let vfs = Rc::new(RefCell::new(Vfs::new()));
    let script = "X=hello\necho $X\n";
    let mut stdin = Cursor::new(b"");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let r = run_script(
        &vfs,
        script,
        Path::new(""),
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
    let script = "for x in one two three; do\necho $x\ndone\n";
    let mut stdin = Cursor::new(b"");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let r = run_script(
        &vfs,
        script,
        Path::new(""),
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
    // pwd succeeds, so then branch runs
    let script = "if pwd; then\necho then\nelse\necho else\nfi\n";
    let mut stdin = Cursor::new(b"");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let r = run_script(
        &vfs,
        script,
        Path::new(""),
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
fn run_script_set_e_fail_exits() {
    let vfs = Rc::new(RefCell::new(Vfs::new()));
    // unknown command fails; with set_e script should return Err
    let script = "echo ok\nnosuchcommand_xy\necho after\n";
    let mut stdin = Cursor::new(b"");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let r = run_script(
        &vfs,
        script,
        Path::new(""),
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
    // set -e inside script turns on exit-on-error for rest of script
    let script = "echo first\nset -e\necho second\nnosuchcommand_xy\necho third\n";
    let mut stdin = Cursor::new(b"");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let r = run_script(
        &vfs,
        script,
        Path::new(""),
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
fn run_script_source_host_file() {
    let vfs = Rc::new(RefCell::new(Vfs::new()));
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
        &script,
        Path::new(""),
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
