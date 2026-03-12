//! todo - workspace library
//!
//! Todo domain: create, list, complete, delete items with in-memory or pluggable storage.

mod error;
mod id;
mod list;
mod model;
mod priority;
mod repeat;
mod store;

#[cfg(test)]
mod tests;

pub use error::TodoError;
pub use id::TodoId;
pub use list::TodoList;
pub use model::{ListFilter, ListOptions, ListSort, Todo, TodoPatch};
pub use priority::Priority;
pub use repeat::RepeatRule;
pub use store::{InMemoryStore, Store};
