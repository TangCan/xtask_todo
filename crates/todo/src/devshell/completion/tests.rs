use std::cell::RefCell;
use std::rc::Rc;

use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::{Hint, Hinter};
use rustyline::Context;

use crate::devshell::vfs::Vfs;

use super::candidates::{complete_commands, complete_path, list_dir_names_for_completion};
use super::context::{completion_context, CompletionKind};
use super::helper::{DevShellHelper, NoHint};

fn host_session() -> Rc<RefCell<super::super::vm::SessionHolder>> {
    Rc::new(RefCell::new(super::super::vm::SessionHolder::new_host()))
}

#[test]
fn complete_commands_prefix() {
    let c = complete_commands("pw");
    assert_eq!(c, vec!["pwd"]);
    let c = complete_commands("ex");
    assert!(c.iter().any(|s| s == "exit"));
    let c = complete_commands("");
    assert!(c.len() > 5);
}

#[test]
fn complete_commands_contains_builtin_and_aliases() {
    let c = complete_commands("");
    for cmd in [
        "pwd",
        "cd",
        "ls",
        "mkdir",
        "cat",
        "touch",
        "echo",
        "save",
        "export-readonly",
        "export_readonly",
        "todo",
        "rustup",
        "cargo",
        "exit",
        "quit",
        "help",
    ] {
        assert!(
            c.iter().any(|s| s == cmd),
            "completion list should include {cmd}: {c:?}"
        );
    }
}

#[test]
fn complete_path_empty_prefix() {
    let names = vec!["a".into(), "b".into()];
    assert_eq!(complete_path("", &names), vec!["a", "b"]);
}

#[test]
fn completion_context_first_token() {
    let ctx = completion_context("pwd", 3).unwrap();
    assert_eq!(ctx.prefix, "pwd");
    assert_eq!(ctx.kind, CompletionKind::Command);
}

#[test]
fn completion_context_after_pipe_is_command() {
    let ctx = completion_context("echo x | pw", 10).unwrap();
    assert_eq!(ctx.kind, CompletionKind::Command);
}

#[test]
fn completion_context_path_token() {
    let ctx = completion_context("cat /a/b", 8).unwrap();
    assert_eq!(ctx.kind, CompletionKind::Path);
}

#[test]
fn completion_context_with_2_redirect() {
    let ctx = completion_context("echo x 2> ", 10).unwrap();
    assert_eq!(ctx.kind, CompletionKind::Path);
}

#[test]
fn completion_context_trailing_space_after_command() {
    let ctx = completion_context("pwd ", 4).unwrap();
    assert_eq!(ctx.prefix, "");
}

#[test]
fn completion_context_pos_past_line_len_returns_none() {
    assert!(completion_context("pwd", 10).is_none());
}

#[test]
fn complete_path_with_prefix() {
    let names = vec!["foo".into(), "bar".into(), "food".into()];
    let c = complete_path("fo", &names);
    assert_eq!(c, vec!["foo", "food"]);
}

#[test]
fn complete_path_trailing_slash_keeps_parent_in_candidate() {
    let names = vec!["main.rs".into(), "lib.rs".into()];
    let mut c = complete_path("src/", &names);
    c.sort();
    assert_eq!(c, vec!["src/lib.rs", "src/main.rs"]);
}

#[test]
fn complete_path_partial_under_subdir() {
    let names = vec!["main.rs".into(), "mod.rs".into()];
    let c = complete_path("src/ma", &names);
    assert_eq!(c, vec!["src/main.rs"]);
}

#[test]
fn completer_complete_command() {
    let vfs = Rc::new(RefCell::new(Vfs::new()));
    let helper = DevShellHelper::new(vfs, host_session());
    let hist = rustyline::history::MemHistory::new();
    let ctx = Context::new(&hist);
    let (start, candidates) = helper.complete("pw", 2, &ctx).unwrap();
    assert_eq!(start, 0);
    assert_eq!(candidates, vec!["pwd"]);
}

#[test]
fn completer_complete_path() {
    let vfs = Rc::new(RefCell::new(Vfs::new()));
    vfs.borrow_mut().mkdir("/a").unwrap();
    vfs.borrow_mut().mkdir("/b").unwrap();
    let helper = DevShellHelper::new(vfs, host_session());
    let hist = rustyline::history::MemHistory::new();
    let ctx = Context::new(&hist);
    let (start, candidates) = helper.complete("ls /", 4, &ctx).unwrap();
    assert!(start <= 4);
    assert!(candidates.contains(&"/a".to_string()));
    assert!(candidates.contains(&"/b".to_string()));
}

#[test]
fn completer_complete_when_context_none_returns_empty() {
    let vfs = Rc::new(RefCell::new(Vfs::new()));
    let helper = DevShellHelper::new(vfs, host_session());
    let hist = rustyline::history::MemHistory::new();
    let ctx = Context::new(&hist);
    let (pos, candidates) = helper.complete("pwd", 10, &ctx).unwrap();
    assert_eq!(pos, 10);
    assert!(candidates.is_empty());
}

#[test]
fn no_hint_display_and_completion() {
    let h = NoHint;
    assert_eq!(h.display(), "");
    assert!(h.completion().is_none());
}

#[test]
fn hinter_returns_none_or_no_hint() {
    let vfs = Rc::new(RefCell::new(Vfs::new()));
    let helper = DevShellHelper::new(vfs, host_session());
    let history = rustyline::history::MemHistory::new();
    let ctx = Context::new(&history);
    let hint = helper.hint("pwd", 4, &ctx);
    if let Some(h) = hint {
        assert_eq!(h.display(), "");
    }
}

#[test]
fn highlighter_returns_borrowed() {
    let vfs = Rc::new(RefCell::new(Vfs::new()));
    let helper = DevShellHelper::new(vfs, host_session());
    let out = helper.highlight("echo x", 6);
    assert_eq!(out.as_ref(), "echo x");
}

#[test]
fn list_dir_names_for_completion_host_matches_vfs() {
    let vfs = Rc::new(RefCell::new(Vfs::new()));
    vfs.borrow_mut().mkdir("/a").unwrap();
    let names = list_dir_names_for_completion(&vfs, &host_session(), "/");
    assert!(names.contains(&"a".to_string()));
}
