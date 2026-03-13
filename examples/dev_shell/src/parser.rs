use std::fmt;

/// Redirect: fd 0=stdin, 1=stdout, 2=stderr
pub struct Redirect {
    pub fd: u8,
    pub path: String,
}

pub struct SimpleCommand {
    pub argv: Vec<String>,
    pub redirects: Vec<Redirect>,
}

pub struct Pipeline {
    pub commands: Vec<SimpleCommand>,
}

#[derive(Debug)]
pub struct ParseError(pub String);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ParseError {}

/// Tokenize line: split on whitespace, treat `>`, `2>`, `<`, `|` as separate tokens.
fn tokenize(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        if c.is_whitespace() {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
            continue;
        }
        if c == '2' && chars.peek() == Some(&'>') {
            chars.next();
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
            tokens.push("2>".to_string());
            continue;
        }
        if c == '>' || c == '<' || c == '|' {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
            tokens.push(c.to_string());
            continue;
        }
        current.push(c);
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

/// Split token list by `|` into command token lists.
fn split_by_pipe(tokens: Vec<String>) -> Vec<Vec<String>> {
    let mut commands = Vec::new();
    let mut current = Vec::new();
    for t in tokens {
        if t == "|" {
            if !current.is_empty() {
                commands.push(std::mem::take(&mut current));
            }
        } else {
            current.push(t);
        }
    }
    if !current.is_empty() {
        commands.push(current);
    }
    if commands.is_empty() {
        commands.push(Vec::new());
    }
    commands
}

/// Parse one command's tokens into SimpleCommand (argv + redirects).
fn parse_simple_command(tokens: Vec<String>) -> Result<SimpleCommand, ParseError> {
    let mut argv = Vec::new();
    let mut redirects = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        let t = &tokens[i];
        if t == ">" {
            i += 1;
            let path = tokens.get(i).ok_or_else(|| ParseError("redirect '>' missing path".to_string()))?;
            redirects.push(Redirect { fd: 1, path: path.clone() });
            i += 1;
        } else if t == "2>" {
            i += 1;
            let path = tokens.get(i).ok_or_else(|| ParseError("redirect '2>' missing path".to_string()))?;
            redirects.push(Redirect { fd: 2, path: path.clone() });
            i += 1;
        } else if t == "<" {
            i += 1;
            let path = tokens.get(i).ok_or_else(|| ParseError("redirect '<' missing path".to_string()))?;
            redirects.push(Redirect { fd: 0, path: path.clone() });
            i += 1;
        } else {
            argv.push(t.clone());
            i += 1;
        }
    }
    Ok(SimpleCommand { argv, redirects })
}

pub fn parse_line(line: &str) -> Result<Pipeline, ParseError> {
    let tokens = tokenize(line.trim());
    let command_tokens_list = split_by_pipe(tokens);
    let mut commands = Vec::new();
    for ct in command_tokens_list {
        commands.push(parse_simple_command(ct)?);
    }
    Ok(Pipeline { commands })
}
