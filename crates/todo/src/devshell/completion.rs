//! Tab completion: context parsing (command vs path), command and path candidates.
//! Rustyline Helper (Completer, Hinter, Highlighter, Validator) for command and path completion.

use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;

use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::{Hint, Hinter};
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::Context;
use rustyline::Helper;

use super::vfs::Vfs;

/// 当前输入位置是命令名还是路径（用于选择补全源）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind {
    Command,
    Path,
}

/// 从 (line, pos) 解析出的补全上下文：当前词的前缀，以及是命令还是路径
#[derive(Debug)]
pub struct CompletionContext {
    pub prefix: String,
    pub kind: CompletionKind,
    /// 当前词在 line 中的起始位置（用于 rustyline 的 replace start）
    pub start: usize,
}

/// Tokenize line[..pos] by spaces and delimiters |, <, >, and "2>" as one token.
/// Returns list of (`token_string`, `start_index`).
fn tokenize(line: &str, pos: usize) -> Vec<(String, usize)> {
    let slice = line.get(..pos).unwrap_or("");
    let mut tokens = Vec::new();
    let mut i = 0;
    let bytes = slice.as_bytes();

    while i < bytes.len() {
        // Skip whitespace
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        let token_start = i;

        // Delimiter "2>" as one token
        if i + 1 < bytes.len() && bytes[i] == b'2' && bytes[i + 1] == b'>' {
            tokens.push(("2>".to_string(), token_start));
            i += 2;
            continue;
        }
        // Single-char delimiters
        if bytes[i] == b'|' || bytes[i] == b'<' || bytes[i] == b'>' {
            let ch = char::from(bytes[i]);
            tokens.push((ch.to_string(), token_start));
            i += 1;
            continue;
        }

        // Collect run of non-delimiter, non-whitespace (stop before "2>", |, <, >, or space)
        let start = i;
        while i < bytes.len() {
            if bytes[i].is_ascii_whitespace() {
                break;
            }
            if bytes[i] == b'|' || bytes[i] == b'<' || bytes[i] == b'>' {
                break;
            }
            if bytes[i] == b'2' && i + 1 < bytes.len() && bytes[i + 1] == b'>' {
                break;
            }
            i += 1;
        }
        let token = slice[start..i].to_string();
        if !token.is_empty() {
            tokens.push((token, start));
        }
    }

    tokens
}

/// Built-in command names for tab completion (must match command.rs).
const BUILTIN_COMMANDS: &[&str] = &[
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
    "exit",
    "quit",
    "help",
];

/// Command completion: case-insensitive prefix match. Returns matching command names.
#[must_use]
pub fn complete_commands(prefix: &str) -> Vec<String> {
    let prefix_lower = prefix.to_lowercase();
    BUILTIN_COMMANDS
        .iter()
        .filter(|c| c.to_lowercase().starts_with(prefix_lower.as_str()))
        .map(|s| (*s).to_string())
        .collect()
}

/// Path completion: prefix may contain slashes; only the last segment is used for matching.
/// `parent_names` is the list of names in the parent directory. Empty prefix returns all.
#[must_use]
pub fn complete_path(prefix: &str, parent_names: &[String]) -> Vec<String> {
    let last = prefix.rsplit('/').next().unwrap_or(prefix);
    parent_names
        .iter()
        .filter(|n| n.starts_with(last))
        .cloned()
        .collect()
}

#[allow(dead_code)]
const PATH_TRIGGER_TOKENS: &[&str] = &[
    "cd",
    "ls",
    "cat",
    "mkdir",
    "touch",
    "export-readonly",
    "export_readonly",
    ">",
    "2>",
    "<",
];

/// Returns the (prefix, start) for the token that contains the cursor at `pos`.
/// Prefix is the part of the token from start up to pos (what the user has typed so far).
fn token_at_cursor(line: &str, tokens: &[(String, usize)], pos: usize) -> Option<(String, usize)> {
    if pos > line.len() {
        return None;
    }
    for (token, start) in tokens {
        let end = start + token.len();
        if *start <= pos && end >= pos {
            let prefix = line.get(*start..pos).unwrap_or("").to_string();
            return Some((prefix, *start));
        }
    }
    // Cursor in trailing whitespace: prefix empty, start at pos
    if !tokens.is_empty() {
        let (last_token, last_start) = tokens.last().unwrap();
        let last_end = last_start + last_token.len();
        if pos >= last_end {
            return Some((String::new(), pos));
        }
    }
    None
}

/// Parse completion context at (line, pos). Returns None if line empty or pos out of bounds.
#[must_use]
pub fn completion_context(line: &str, pos: usize) -> Option<CompletionContext> {
    if line.is_empty() {
        return None;
    }
    let line_len = line.len();
    if pos > line_len {
        return None;
    }

    let tokens = tokenize(line, pos);
    let (prefix, start) = token_at_cursor(line, &tokens, pos)?;

    // If we got empty prefix with start == pos, we're in trailing space; still return context with kind
    let prefix = if prefix.is_empty() && start == pos && !tokens.is_empty() {
        String::new()
    } else if prefix.is_empty() && start == pos {
        return None;
    } else {
        prefix
    };

    let token_index = tokens
        .iter()
        .position(|(t, s)| *s == start && t.as_str() == prefix.as_str())
        .or({
            if prefix.is_empty() {
                Some(tokens.len())
            } else {
                None
            }
        });

    let idx = token_index.unwrap_or_else(|| tokens.iter().take_while(|(_, s)| *s < start).count());

    let kind = if idx == 0 {
        CompletionKind::Command
    } else {
        let prev = tokens.get(idx.wrapping_sub(1)).map(|(t, _)| t.as_str());
        if prev == Some("|") {
            CompletionKind::Command
        } else {
            CompletionKind::Path
        }
    };

    Some(CompletionContext {
        prefix,
        kind,
        start,
    })
}

// ---------------------------------------------------------------------------
// Rustyline Helper: command and path completion via Rc<RefCell<Vfs>>
// ---------------------------------------------------------------------------

/// Helper for rustyline: command and path tab-completion using shared Vfs.
pub struct DevShellHelper {
    pub vfs: Rc<RefCell<Vfs>>,
}

impl DevShellHelper {
    pub const fn new(vfs: Rc<RefCell<Vfs>>) -> Self {
        Self { vfs }
    }
}

/// Dummy hint type for Hinter; we never return a hint (`hint()` returns None).
#[derive(Debug)]
pub struct NoHint;
impl Hint for NoHint {
    fn display(&self) -> &'static str {
        ""
    }
    fn completion(&self) -> Option<&str> {
        None
    }
}

impl Completer for DevShellHelper {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<String>), rustyline::error::ReadlineError> {
        let Some(ctx) = completion_context(line, pos) else {
            return Ok((pos, vec![]));
        };
        let candidates = match ctx.kind {
            CompletionKind::Command => complete_commands(&ctx.prefix),
            CompletionKind::Path => {
                let parent = if ctx.prefix.contains('/') {
                    let idx = ctx.prefix.rfind('/').unwrap();
                    if idx == 0 {
                        "/".to_string()
                    } else {
                        ctx.prefix[..idx].to_string()
                    }
                } else {
                    ".".to_string()
                };
                let abs_parent = self.vfs.borrow().resolve_to_absolute(&parent);
                let names = self.vfs.borrow().list_dir(&abs_parent).unwrap_or_default();
                complete_path(&ctx.prefix, &names)
            }
        };
        Ok((ctx.start, candidates))
    }
}

impl Hinter for DevShellHelper {
    type Hint = NoHint;

    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<NoHint> {
        None
    }
}

impl Highlighter for DevShellHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        Cow::Borrowed(line)
    }
}

impl Validator for DevShellHelper {
    fn validate(
        &self,
        _ctx: &mut ValidationContext<'_>,
    ) -> Result<ValidationResult, rustyline::error::ReadlineError> {
        Ok(ValidationResult::Valid(None))
    }
}

impl Helper for DevShellHelper {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::devshell::vfs::Vfs;

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
    fn complete_path_with_prefix() {
        let names = vec!["foo".into(), "bar".into(), "food".into()];
        let c = complete_path("fo", &names);
        assert_eq!(c, vec!["foo", "food"]);
    }

    #[test]
    fn completer_complete_command() {
        use std::cell::RefCell;
        use std::rc::Rc;

        let vfs = Rc::new(RefCell::new(Vfs::new()));
        let helper = DevShellHelper::new(vfs);
        let hist = rustyline::history::MemHistory::new();
        let ctx = Context::new(&hist);
        let (start, candidates) = helper.complete("pw", 2, &ctx).unwrap();
        assert_eq!(start, 0);
        assert_eq!(candidates, vec!["pwd"]);
    }

    #[test]
    fn completer_complete_path() {
        use std::cell::RefCell;
        use std::rc::Rc;

        let vfs = Rc::new(RefCell::new(Vfs::new()));
        vfs.borrow_mut().mkdir("/a").unwrap();
        vfs.borrow_mut().mkdir("/b").unwrap();
        let helper = DevShellHelper::new(vfs);
        let hist = rustyline::history::MemHistory::new();
        let ctx = Context::new(&hist);
        let (start, candidates) = helper.complete("ls /", 4, &ctx).unwrap();
        assert!(start <= 4);
        assert!(candidates.contains(&"a".to_string()));
        assert!(candidates.contains(&"b".to_string()));
    }
}
