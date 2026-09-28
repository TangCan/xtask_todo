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

/// Broad category used by CLI adapters and other callers to map domain failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TodoErrorKind {
    Input,
    Business,
}

impl TodoError {
    /// Classify this domain error without parsing its display text.
    #[must_use]
    pub const fn kind(&self) -> TodoErrorKind {
        match self {
            Self::InvalidInput => TodoErrorKind::Input,
            Self::NotFound(_) => TodoErrorKind::Business,
        }
    }
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

#[cfg(test)]
mod tests {
    use super::{TodoError, TodoErrorKind};
    use crate::TodoId;

    #[test]
    fn errors_have_machine_readable_categories() {
        assert_eq!(TodoError::InvalidInput.kind(), TodoErrorKind::Input);
        assert_eq!(
            TodoError::NotFound(TodoId::from_raw(1).unwrap()).kind(),
            TodoErrorKind::Business
        );
    }
}
