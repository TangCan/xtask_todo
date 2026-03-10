//! Storage abstraction for todo items.

use std::collections::HashMap;

use crate::{Todo, TodoId};

/// Backing store for todo items. Implementations may be in-memory or persistent.
pub trait Store {
    /// Returns the next available id and advances the counter.
    fn next_id(&mut self) -> TodoId;
    /// Inserts a todo; the id must have been obtained from `next_id`.
    fn insert(&mut self, todo: Todo);
    /// Returns the todo with the given id, if any.
    fn get(&self, id: TodoId) -> Option<Todo>;
    /// Returns all todos in creation order (by created_at).
    fn list(&self) -> Vec<Todo>;
    /// Updates an existing todo (e.g. after marking completed).
    fn update(&mut self, todo: Todo);
    /// Removes the todo with the given id.
    fn remove(&mut self, id: TodoId);
}

/// In-memory store using a map and a monotonic id counter.
#[derive(Default)]
pub struct InMemoryStore {
    next_id: u64,
    items: HashMap<TodoId, Todo>,
}

impl InMemoryStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Store for InMemoryStore {
    fn next_id(&mut self) -> TodoId {
        self.next_id += 1;
        TodoId::from_raw(self.next_id).expect("id overflow")
    }

    fn insert(&mut self, todo: Todo) {
        self.items.insert(todo.id, todo);
    }

    fn get(&self, id: TodoId) -> Option<Todo> {
        self.items.get(&id).cloned()
    }

    fn list(&self) -> Vec<Todo> {
        let mut out: Vec<Todo> = self.items.values().cloned().collect();
        out.sort_by_key(|t| t.id);
        out
    }

    fn update(&mut self, todo: Todo) {
        if self.items.contains_key(&todo.id) {
            self.items.insert(todo.id, todo);
        }
    }

    fn remove(&mut self, id: TodoId) {
        self.items.remove(&id);
    }
}
