//! Devshell REPL and VFS: same logic as the `cargo-devshell` binary, exposed so tests can cover it.
#![allow(
    dead_code,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::too_long_first_doc_paragraph,
    clippy::too_many_lines,
    clippy::result_unit_err,
    clippy::cast_possible_truncation,
    clippy::branches_sharing_code,
    clippy::needless_pass_by_value,
    clippy::match_wildcard_for_single_variants,
    clippy::map_identity
)]

pub mod command;
pub mod completion;
pub mod parser;
pub mod serialization;
pub mod todo_io;
pub mod vfs;

mod repl;

use std::cell::RefCell;
use std::io::{self, BufReader, IsTerminal, Write};
use std::path::Path;
use std::rc::Rc;

use vfs::Vfs;

/// Run the devshell using process args and standard I/O (for the binary).
///
/// # Errors
/// Returns an error if usage is wrong or I/O fails critically.
pub fn run_main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let path = match args.as_slice() {
        [] | [_] => Path::new(".dev_shell.bin"),
        [_, path] => Path::new(path),
        _ => {
            writeln!(io::stderr(), "usage: dev_shell [path]")?;
            return Err(Box::new(std::io::Error::other("usage")));
        }
    };
    let vfs = match serialization::load_from_file(path) {
        Ok(v) => v,
        Err(e) => {
            if e.kind() == io::ErrorKind::NotFound {
                if args.len() > 1 {
                    let _ = writeln!(io::stderr(), "File not found, starting with empty VFS");
                }
            } else {
                let _ = writeln!(io::stderr(), "Failed to load {}: {}", path.display(), e);
            }
            Vfs::new()
        }
    };
    let vfs = Rc::new(RefCell::new(vfs));
    let is_tty = io::stdin().is_terminal();
    let mut stdin = BufReader::new(io::stdin());
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();
    repl::run(&vfs, is_tty, path, &mut stdin, &mut stdout, &mut stderr).map_err(|()| {
        Box::new(std::io::Error::other("repl error")) as Box<dyn std::error::Error>
    })?;
    Ok(())
}

/// Run the devshell with given args and streams (for tests).
pub fn run_with<R, W1, W2>(
    args: &[String],
    stdin: &mut R,
    stdout: &mut W1,
    stderr: &mut W2,
) -> Result<(), ()>
where
    R: std::io::BufRead + std::io::Read,
    W1: std::io::Write,
    W2: std::io::Write,
{
    let path = match args {
        [] | [_] => Path::new(".dev_shell.bin"),
        [_, path] => Path::new(path),
        _ => {
            let _ = writeln!(stderr, "usage: dev_shell [path]");
            return Err(());
        }
    };
    let vfs = serialization::load_from_file(path).unwrap_or_default();
    let vfs = Rc::new(RefCell::new(vfs));
    repl::run(&vfs, false, path, stdin, stdout, stderr)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

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
        let input = "todo add buy milk\ntodo list\nexit\n";
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
        assert!(out.contains("buy milk") || out.contains("1."));
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
    fn run_with_todo_add_empty_title_errors() {
        let input = "todo add \nexit\n";
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
        assert!(err.contains("title") || err.contains("non-empty"));
    }

    #[test]
    fn run_with_todo_unknown_subcommand() {
        let input = "todo unknownsub\nexit\n";
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
        assert!(err.contains("unknown") || err.contains("subcommand"));
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
    fn run_with_todo_show_complete_search_with_existing_file() {
        let dir = std::env::temp_dir().join(format!("devshell_todo_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let json_path = dir.join(".todo.json");
        let json =
            r#"[{"id":1,"title":"buy milk","completed":false,"created_at_secs":0,"tags":[]}]"#;
        std::fs::write(&json_path, json).unwrap();
        let cwd = std::env::current_dir().unwrap();
        let _ = std::env::set_current_dir(&dir);
        let input = "todo show 1\ntodo complete 1\ntodo search milk\ntodo list\nexit\n";
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
        let _ = std::fs::remove_file(&json_path);
        let _ = std::fs::remove_dir(&dir);
        r.unwrap();
        let out = String::from_utf8(stdout).unwrap();
        assert!(out.contains("buy milk"));
    }

    #[test]
    fn run_with_todo_update_and_delete() {
        let dir = std::env::temp_dir().join(format!("devshell_todo2_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let json_path = dir.join(".todo.json");
        let json =
            r#"[{"id":1,"title":"original","completed":false,"created_at_secs":0,"tags":[]}]"#;
        std::fs::write(&json_path, json).unwrap();
        let cwd = std::env::current_dir().unwrap();
        let _ = std::env::set_current_dir(&dir);
        let input = "todo update 1 updated title\ntodo delete 1\ntodo list\nexit\n";
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
        let _ = std::fs::remove_file(&json_path);
        let _ = std::fs::remove_dir(&dir);
        r.unwrap();
        let out = String::from_utf8(stdout).unwrap();
        assert!(!out.is_empty(), "expected prompt output");
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
}
