//! Script AST: statements and parse error.

/// One script statement.
#[derive(Debug, Clone)]
pub enum ScriptStmt {
    Assign(String, String),
    Command(String),
    SetE,
    If {
        cond: String,
        then_body: Vec<Self>,
        else_body: Option<Vec<Self>>,
    },
    For {
        var: String,
        words: Vec<String>,
        body: Vec<Self>,
    },
    While {
        cond: String,
        body: Vec<Self>,
    },
    Source(String),
}

/// Parse error with message.
#[derive(Debug)]
pub struct ParseError(pub String);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ParseError {}
