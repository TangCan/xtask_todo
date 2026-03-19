//! Script parser: logical lines → AST.

use super::ast::{ParseError, ScriptStmt};

const fn is_identifier_char(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

fn is_identifier(s: &str) -> bool {
    let b = s.as_bytes();
    if b.is_empty() {
        return false;
    }
    if b[0] != b'_' && !b[0].is_ascii_alphabetic() {
        return false;
    }
    b.iter().all(|&c| is_identifier_char(c))
}

/// Returns (name, value) if line is NAME=value (no space before =); else None.
fn parse_assign(line: &str) -> Option<(String, String)> {
    let line = line.trim();
    let eq_pos = line.find('=')?;
    if eq_pos == 0 {
        return None;
    }
    let (left, right) = line.split_at(eq_pos);
    let name = left.trim_end();
    if name != left {
        return None;
    }
    let value = right[1..].trim_start().to_string();
    if name.is_empty() || !is_identifier(name) {
        return None;
    }
    Some((name.to_string(), value))
}

/// Parse logical lines into a list of script statements.
///
/// # Errors
/// Returns `ParseError` on unclosed if/for/while or invalid syntax.
pub fn parse_script(lines: &[String]) -> Result<Vec<ScriptStmt>, ParseError> {
    let mut stmts = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let (stmt, consumed) = parse_one(lines, i)?;
        stmts.push(stmt);
        i += consumed;
    }
    Ok(stmts)
}

/// Parse one or more lines (one statement); returns (stmt, number of lines consumed).
fn parse_one(lines: &[String], start: usize) -> Result<(ScriptStmt, usize), ParseError> {
    let line = lines
        .get(start)
        .ok_or_else(|| ParseError("unexpected end".to_string()))?;
    let line = line.trim();

    if line == "set -e" {
        return Ok((ScriptStmt::SetE, 1));
    }
    if let Some((name, value)) = parse_assign(line) {
        return Ok((ScriptStmt::Assign(name, value), 1));
    }
    if let Some(rest) = line.strip_prefix("source ") {
        let path = rest.trim();
        if path.is_empty() {
            return Err(ParseError("source: missing path".to_string()));
        }
        return Ok((ScriptStmt::Source(path.to_string()), 1));
    }
    if let Some(path) = line.strip_prefix(". ") {
        let path = path.trim();
        if path.is_empty() {
            return Err(ParseError(".: missing path".to_string()));
        }
        return Ok((ScriptStmt::Source(path.to_string()), 1));
    }
    if let Some(rest) = line.strip_prefix("if ") {
        return parse_if_block(lines, start, rest.trim());
    }
    if let Some(rest) = line.strip_prefix("for ") {
        return parse_for_block(lines, start, rest.trim());
    }
    if let Some(rest) = line.strip_prefix("while ") {
        return parse_while_block(lines, start, rest.trim());
    }
    Ok((ScriptStmt::Command(line.to_string()), 1))
}

fn parse_if_block(
    lines: &[String],
    start: usize,
    rest: &str,
) -> Result<(ScriptStmt, usize), ParseError> {
    let (cond, _has_then) = if let Some(pos) = rest.find("; then") {
        (rest[..pos].trim().to_string(), true)
    } else if rest.contains("then") {
        let pos = rest.find("then").unwrap();
        (rest[..pos].trim().trim_end_matches(';').to_string(), true)
    } else {
        return Err(ParseError("if: missing 'then'".to_string()));
    };
    let mut then_body = Vec::new();
    let mut else_body = None;
    let mut i = start + 1;
    while i < lines.len() {
        let l = lines[i].trim();
        if l == "fi" {
            return Ok((
                ScriptStmt::If {
                    cond,
                    then_body,
                    else_body,
                },
                i - start + 1,
            ));
        }
        if l == "else" {
            else_body = Some(Vec::new());
            i += 1;
            continue;
        }
        let (stmt, n) = parse_one(lines, i)?;
        if let Some(else_b) = &mut else_body {
            else_b.push(stmt);
        } else {
            then_body.push(stmt);
        }
        i += n;
    }
    Err(ParseError("if: missing 'fi'".to_string()))
}

fn parse_for_block(
    lines: &[String],
    start: usize,
    rest: &str,
) -> Result<(ScriptStmt, usize), ParseError> {
    let (var, words) = if let Some(pos) = rest.find(" in ") {
        let var = rest[..pos].trim();
        let after_in = rest[pos + " in ".len()..].trim();
        let words = if let Some(semi) = after_in.find("; do") {
            split_words(&after_in[..semi])
        } else {
            return Err(ParseError("for: missing '; do'".to_string()));
        };
        (var.to_string(), words)
    } else {
        return Err(ParseError("for: missing 'in'".to_string()));
    };
    if !is_identifier(&var) {
        return Err(ParseError("for: invalid variable name".to_string()));
    }
    let mut body = Vec::new();
    let mut i = start + 1;
    while i < lines.len() {
        if lines[i].trim() == "done" {
            return Ok((ScriptStmt::For { var, words, body }, i - start + 1));
        }
        let (stmt, n) = parse_one(lines, i)?;
        body.push(stmt);
        i += n;
    }
    Err(ParseError("for: missing 'done'".to_string()))
}

fn parse_while_block(
    lines: &[String],
    start: usize,
    rest: &str,
) -> Result<(ScriptStmt, usize), ParseError> {
    let cond = if let Some(pos) = rest.find("; do") {
        rest[..pos].trim().to_string()
    } else {
        return Err(ParseError("while: missing '; do'".to_string()));
    };
    let mut body = Vec::new();
    let mut i = start + 1;
    while i < lines.len() {
        if lines[i].trim() == "done" {
            return Ok((ScriptStmt::While { cond, body }, i - start + 1));
        }
        let (stmt, n) = parse_one(lines, i)?;
        body.push(stmt);
        i += n;
    }
    Err(ParseError("while: missing 'done'".to_string()))
}

pub(super) fn split_words(s: &str) -> Vec<String> {
    s.split_whitespace().map(String::from).collect()
}
