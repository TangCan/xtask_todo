//! Completion context: tokenization and `CompletionKind` from `(line, pos)`.

/// 当前输入位置是命令名、路径，或不需要补全（用于选择补全源）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind {
    Command,
    Path,
    /// 前一 token 不是管道也不是路径型参数，不提供补全
    Other,
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
pub(super) fn tokenize(line: &str, pos: usize) -> Vec<(String, usize)> {
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

/// Tokens after which the next word is completed as a path (command args or redirect target).
const PATH_TRIGGER_TOKENS: &[&str] = &[
    "cd",
    "ls",
    "cat",
    "mkdir",
    "touch",
    "export-readonly",
    "export_readonly",
    "source",
    ".",
    ">",
    "2>",
    "<",
];

/// Returns the (prefix, start) for the token that contains the cursor at `pos`.
/// Prefix is the part of the token from start up to pos (what the user has typed so far).
pub(super) fn token_at_cursor(
    line: &str,
    tokens: &[(String, usize)],
    pos: usize,
) -> Option<(String, usize)> {
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
        } else if prev.is_some_and(|p| PATH_TRIGGER_TOKENS.contains(&p)) {
            CompletionKind::Path
        } else {
            CompletionKind::Other
        }
    };

    Some(CompletionContext {
        prefix,
        kind,
        start,
    })
}
