//! Unique identifier for a todo item.

use std::fmt;

/// Unique identifier for a todo item. Opaque; use for completion and deletion.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TodoId(std::num::NonZeroU64);

impl TodoId {
    /// Creates a `TodoId` from a raw u64 (e.g. when loading from storage). Returns `None` if n is 0.
    #[must_use]
    pub fn from_raw(n: u64) -> Option<Self> {
        std::num::NonZeroU64::new(n).map(TodoId)
    }

    /// Returns the raw numeric id (e.g. for serialization).
    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0.get()
    }
}

impl fmt::Display for TodoId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
