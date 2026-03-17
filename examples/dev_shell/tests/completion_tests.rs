//! Tests for completion context parsing (command vs path), command and path completion.

use dev_shell::completion::complete_commands;
use dev_shell::completion::complete_path;
use dev_shell::completion::completion_context;
use dev_shell::completion::CompletionKind;

#[test]
fn context_at_line_start_is_command() {
    let ctx = completion_context("hel", 3).unwrap();
    assert_eq!(ctx.kind, CompletionKind::Command);
    assert_eq!(ctx.prefix, "hel");
    assert_eq!(ctx.start, 0);
}

#[test]
fn context_after_pipe_is_command() {
    let ctx = completion_context("ls | pw", 7).unwrap();
    assert_eq!(ctx.kind, CompletionKind::Command);
    assert_eq!(ctx.prefix, "pw");
}

#[test]
fn context_after_cd_is_path() {
    let ctx = completion_context("cd /fo", 6).unwrap();
    assert_eq!(ctx.kind, CompletionKind::Path);
    assert_eq!(ctx.prefix, "/fo");
}

#[test]
fn complete_commands_he_contains_help() {
    let candidates = complete_commands("he");
    assert!(candidates.contains(&"help".to_string()));
}

#[test]
fn complete_commands_empty_returns_all() {
    let candidates = complete_commands("");
    assert_eq!(candidates.len(), 13);
    assert!(candidates.contains(&"help".to_string()));
    assert!(candidates.contains(&"pwd".to_string()));
    assert!(candidates.contains(&"exit".to_string()));
}

#[test]
fn complete_path_prefix_filters_by_last_segment() {
    let parent = vec!["foo".into(), "foobar".into(), "bar".into()];
    let got = complete_path("fo", &parent);
    assert_eq!(got, &["foo", "foobar"]);
}
