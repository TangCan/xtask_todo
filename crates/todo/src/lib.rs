//! todo - workspace library
//!
//! Todo domain: create, list, complete, delete items with in-memory or pluggable storage.

mod store;

use std::fmt;
use std::time::SystemTime;

pub use store::{InMemoryStore, Store};

/// Unique identifier for a todo item. Opaque; use for completion and deletion.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TodoId(std::num::NonZeroU64);

impl TodoId {
    /// Creates a TodoId from a raw u64 (e.g. when loading from storage). Returns None if n is 0.
    pub fn from_raw(n: u64) -> Option<Self> {
        std::num::NonZeroU64::new(n).map(TodoId)
    }

    /// Returns the raw numeric id (e.g. for serialization).
    pub fn as_u64(self) -> u64 {
        self.0.get()
    }
}

impl fmt::Display for TodoId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A single todo item.
#[derive(Clone, Eq, PartialEq)]
pub struct Todo {
    pub id: TodoId,
    pub title: String,
    pub completed: bool,
    pub created_at: SystemTime,
}

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
            TodoError::InvalidInput => f.write_str("invalid input: title must be non-empty"),
            TodoError::NotFound(id) => write!(f, "todo not found: {}", id),
        }
    }
}

impl std::error::Error for TodoError {}

/// Validates title: after trim, must be non-empty. Returns `Err(TodoError::InvalidInput)` otherwise.
fn validate_title(title: &str) -> Result<String, TodoError> {
    let t = title.trim();
    if t.is_empty() {
        return Err(TodoError::InvalidInput);
    }
    Ok(t.to_string())
}

/// Facade for todo operations. Holds a store (default: in-memory).
pub struct TodoList<S> {
    store: S,
}

impl TodoList<InMemoryStore> {
    /// Creates a new list with in-memory storage.
    pub fn new() -> Self {
        Self {
            store: InMemoryStore::new(),
        }
    }
}

impl Default for TodoList<InMemoryStore> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: Store> TodoList<S> {
    /// Builds a list with the given store (e.g. for testing or custom backends).
    pub fn with_store(store: S) -> Self {
        Self { store }
    }

    /// Creates a todo with the given title. Returns its id or an error if title is invalid.
    pub fn create(&mut self, title: impl AsRef<str>) -> Result<TodoId, TodoError> {
        let title = validate_title(title.as_ref())?;
        let id = self.store.next_id();
        let todo = Todo {
            id,
            title,
            completed: false,
            created_at: SystemTime::now(),
        };
        self.store.insert(todo);
        Ok(id)
    }

    /// Returns all todos in creation order.
    pub fn list(&self) -> Vec<Todo> {
        self.store.list()
    }

    /// Marks the todo with the given id as completed. Returns `NotFound` if id does not exist.
    pub fn complete(&mut self, id: TodoId) -> Result<(), TodoError> {
        let mut todo = self.store.get(id).ok_or(TodoError::NotFound(id))?;
        todo.completed = true;
        self.store.update(todo);
        Ok(())
    }

    /// Removes the todo with the given id. Returns `NotFound` if id does not exist.
    pub fn delete(&mut self, id: TodoId) -> Result<(), TodoError> {
        if self.store.get(id).is_none() {
            return Err(TodoError::NotFound(id));
        }
        self.store.remove(id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn us_t1_valid_title_creates_and_appears_in_list() {
        let mut list = TodoList::new();
        let id = list.create("  buy milk  ").unwrap();
        let items = list.list();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, id);
        assert_eq!(items[0].title, "buy milk");
        assert!(!items[0].completed);
    }

    #[test]
    fn us_t1_empty_title_returns_err_and_list_unchanged() {
        let mut list = TodoList::new();
        assert!(list.create("").is_err());
        assert!(list.create("   ").is_err());
        assert!(list.list().is_empty());
    }

    #[test]
    fn us_t2_empty_list() {
        let list = TodoList::new();
        assert!(list.list().is_empty());
    }

    #[test]
    fn us_t2_list_order_by_creation() {
        let mut list = TodoList::new();
        list.create("first").unwrap();
        list.create("second").unwrap();
        list.create("third").unwrap();
        let items = list.list();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].title, "first");
        assert_eq!(items[1].title, "second");
        assert_eq!(items[2].title, "third");
    }

    #[test]
    fn us_t3_complete_existing() {
        let mut list = TodoList::new();
        let id = list.create("task").unwrap();
        list.complete(id).unwrap();
        let items = list.list();
        assert!(items[0].completed);
    }

    #[test]
    fn us_t3_complete_nonexistent_returns_not_found() {
        let mut list = TodoList::new();
        list.create("only").unwrap();
        let bad_id = TodoId::from_raw(999).unwrap();
        let err = list.complete(bad_id).unwrap_err();
        match &err {
            TodoError::NotFound(x) => assert_eq!(*x, bad_id),
            _ => panic!("expected NotFound"),
        }
        assert_eq!(list.list().len(), 1);
        assert!(!list.list()[0].completed);
    }

    #[test]
    fn us_t4_delete_existing() {
        let mut list = TodoList::new();
        let id = list.create("gone").unwrap();
        list.delete(id).unwrap();
        assert!(list.list().is_empty());
    }

    #[test]
    fn us_t4_delete_nonexistent_returns_not_found() {
        let mut list = TodoList::new();
        list.create("stay").unwrap();
        let bad_id = TodoId::from_raw(999).unwrap();
        let err = list.delete(bad_id).unwrap_err();
        match &err {
            TodoError::NotFound(x) => assert_eq!(*x, bad_id),
            _ => panic!("expected NotFound"),
        }
        assert_eq!(list.list().len(), 1);
    }
}
