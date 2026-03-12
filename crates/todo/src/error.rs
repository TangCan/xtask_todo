//! Errors from todo operations.

use std::fmt;

use crate::id::TodoId;

/// Errors from todo operations.
#[derive(Debug)]
pub enum TodoError {
    /// Title was empty or invalid.
    InvalidInput,
    /// No todo with the given id.
    NotFound(TodoId),
}

impl fmt::Display for TodoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput => f.write_str("invalid input: title must be non-empty"),
            Self::NotFound(id) => write!(f, "todo not found: {id}"),
        }
    }
}

impl std::error::Error for TodoError {}
