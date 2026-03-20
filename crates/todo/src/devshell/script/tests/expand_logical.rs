use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Cursor;
use std::path::Path;
use std::rc::Rc;

use crate::devshell::vfs::Vfs;

use super::super::ast::ParseError;
use super::super::exec::{expand_vars, logical_lines, run_script};
use super::helpers::vm_session_test;

#[test]
fn parse_error_display_and_error_trait() {
    let e = ParseError("bad syntax".to_string());
    assert!(e.to_string().contains("bad syntax"));
    assert!(std::error::Error::source(&e).is_none());
}

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
fn run_script_empty_command_line_succeeds() {
    let vfs = Rc::new(RefCell::new(Vfs::new()));
    let vm = vm_session_test();
    let script = "   \n"; // logical line "  " -> Command("  ") -> after trim empty, returns Success
    let mut stdin = Cursor::new(b"");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let r = run_script(
        &vfs,
        &vm,
        script,
        Path::new(""),
        false,
        &mut stdin,
        &mut stdout,
        &mut stderr,
    );
    assert!(r.is_ok());
}

#[test]
fn logical_lines_empty_source() {
    assert!(logical_lines("").is_empty());
    assert!(logical_lines("\n\n  # only comment\n").is_empty());
}
