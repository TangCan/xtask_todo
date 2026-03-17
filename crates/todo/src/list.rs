//! Facade for todo operations (create, list, complete, delete).

use std::time::SystemTime;

use crate::error::TodoError;
use crate::id::TodoId;
use crate::model::{ListOptions, ListSort, Todo, TodoPatch};
use crate::priority::Priority;
use crate::store::{InMemoryStore, Store};

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
    #[must_use]
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
    #[must_use]
    pub const fn with_store(store: S) -> Self {
        Self { store }
    }

    /// Creates a todo with the given title. Returns its `TodoId` or an error if title is invalid.
    ///
    /// # Errors
    /// Returns `TodoError::InvalidInput` if the title is empty or only whitespace after trim.
    pub fn create(&mut self, title: impl AsRef<str>) -> Result<TodoId, TodoError> {
        let title = validate_title(title.as_ref())?;
        let id = self.store.next_id();
        let todo = Todo {
            id,
            title,
            completed: false,
            created_at: SystemTime::now(),
            completed_at: None,
            description: None,
            due_date: None,
            priority: None,
            tags: Vec::new(),
            repeat_rule: None,
            repeat_until: None,
            repeat_count: None,
        };
        self.store.insert(todo);
        Ok(id)
    }

    /// Inserts an existing todo's content with a new id (e.g. for import/merge). Returns the new `TodoId`.
    pub fn add_todo(&mut self, todo: &Todo) -> TodoId {
        let id = self.store.next_id();
        let new_todo = Todo {
            id,
            title: todo.title.clone(),
            completed: todo.completed,
            created_at: todo.created_at,
            completed_at: todo.completed_at,
            description: todo.description.clone(),
            due_date: todo.due_date.clone(),
            priority: todo.priority,
            tags: todo.tags.clone(),
            repeat_rule: todo.repeat_rule.clone(),
            repeat_until: todo.repeat_until.clone(),
            repeat_count: todo.repeat_count,
        };
        self.store.insert(new_todo);
        id
    }

    /// Returns the todo with the given id, if it exists.
    #[must_use]
    pub fn get(&self, id: TodoId) -> Option<Todo> {
        self.store.get(id)
    }

    /// Returns all todos in creation order.
    #[must_use]
    pub fn list(&self) -> Vec<Todo> {
        self.store.list()
    }

    /// Returns todos filtered and sorted according to `options`.
    #[must_use]
    pub fn list_with_options(&self, options: &ListOptions) -> Vec<Todo> {
        let mut items = self.store.list();
        if let Some(ref f) = options.filter {
            items.retain(|t| {
                if let Some(s) = f.status {
                    if t.completed != s {
                        return false;
                    }
                }
                if let Some(p) = f.priority {
                    if t.priority != Some(p) {
                        return false;
                    }
                }
                if let Some(ref tags) = f.tags_any {
                    if tags.is_empty() {
                        return true;
                    }
                    if !t.tags.iter().any(|tag| tags.contains(tag)) {
                        return false;
                    }
                }
                if let Some(ref d) = f.due_before {
                    if let Some(ref due) = t.due_date {
                        if due > d {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                if let Some(ref d) = f.due_after {
                    if let Some(ref due) = t.due_date {
                        if due < d {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                true
            });
        }
        match options.sort {
            ListSort::CreatedAt => items.sort_by_key(|t| t.created_at),
            ListSort::DueDate => items.sort_by(|a, b| {
                a.due_date
                    .as_ref()
                    .cmp(&b.due_date.as_ref())
                    .then_with(|| a.id.cmp(&b.id))
            }),
            ListSort::Priority => items.sort_by(|a, b| {
                let pa = a.priority.map_or(0, Priority::as_u8);
                let pb = b.priority.map_or(0, Priority::as_u8);
                pa.cmp(&pb).then_with(|| a.id.cmp(&b.id))
            }),
            ListSort::Title => {
                items.sort_by(|a, b| a.title.cmp(&b.title).then_with(|| a.id.cmp(&b.id)));
            }
        }
        items
    }

    /// Updates the title of the todo with the given id.
    ///
    /// # Errors
    /// Returns `TodoError::NotFound(id)` if no todo with that id exists.
    /// Returns `TodoError::InvalidInput` if the new title is empty or only whitespace.
    pub fn update_title(&mut self, id: TodoId, title: impl AsRef<str>) -> Result<(), TodoError> {
        self.update(
            id,
            TodoPatch {
                title: Some(validate_title(title.as_ref())?),
                ..TodoPatch::default()
            },
        )
    }

    /// Applies a partial update to the todo with the given id. Only fields set in `patch` are updated.
    ///
    /// # Errors
    /// Returns `TodoError::NotFound(id)` if no todo with that id exists.
    /// Returns `TodoError::InvalidInput` if `patch.title` is Some and empty/whitespace.
    pub fn update(&mut self, id: TodoId, patch: TodoPatch) -> Result<(), TodoError> {
        let mut todo = self.store.get(id).ok_or(TodoError::NotFound(id))?;
        if let Some(ref t) = patch.title {
            todo.title = validate_title(t)?;
        }
        if patch.description.is_some() {
            todo.description = patch.description;
        }
        if patch.due_date.is_some() {
            todo.due_date = patch.due_date;
        }
        if patch.priority.is_some() {
            todo.priority = patch.priority;
        }
        if patch.tags.is_some() {
            todo.tags = patch.tags.unwrap_or_default();
        }
        if patch.repeat_rule.is_some() {
            todo.repeat_rule = patch.repeat_rule;
        }
        if patch.repeat_until.is_some() {
            todo.repeat_until = patch.repeat_until;
        }
        if patch.repeat_count.is_some() {
            todo.repeat_count = patch.repeat_count;
        }
        if patch.repeat_rule_clear {
            todo.repeat_rule = None;
        }
        self.store.update(todo);
        Ok(())
    }

    /// Marks the todo with the given `TodoId` as completed.
    ///
    /// # Errors
    /// Returns `TodoError::NotFound(id)` if no todo with that id exists.
    /// Marks the todo as completed. If it has a repeat rule and `no_next` is false, creates the next instance.
    pub fn complete(&mut self, id: TodoId, no_next: bool) -> Result<(), TodoError> {
        let mut todo = self.store.get(id).ok_or(TodoError::NotFound(id))?;
        let repeat_rule = todo.repeat_rule.clone();
        let due_date = todo.due_date.clone();
        let repeat_until = todo.repeat_until.clone();
        let repeat_count = todo.repeat_count;
        let title = todo.title.clone();
        let description = todo.description.clone();
        let priority = todo.priority;
        let tags = todo.tags.clone();
        todo.completed = true;
        todo.completed_at = Some(SystemTime::now());
        self.store.update(todo);
        if !no_next {
            if let (Some(rule), Some(ref from)) = (repeat_rule, &due_date) {
                if repeat_count == Some(0) || repeat_count == Some(1) {
                    // last occurrence, do not create next
                } else if let Some(next_due) = rule.next_due_date(from) {
                    let past_until = repeat_until
                        .as_ref()
                        .is_some_and(|until| next_due.as_str() > until);
                    if past_until {
                        // next would be after end date
                    } else {
                        let next_count = repeat_count.and_then(|n| n.checked_sub(1));
                        let next_id = self.store.next_id();
                        let next_todo = Todo {
                            id: next_id,
                            title,
                            completed: false,
                            created_at: SystemTime::now(),
                            completed_at: None,
                            description,
                            due_date: Some(next_due),
                            priority,
                            tags,
                            repeat_rule: Some(rule),
                            repeat_until,
                            repeat_count: next_count,
                        };
                        self.store.insert(next_todo);
                    }
                }
            }
        }
        Ok(())
    }

    /// Removes the todo with the given `TodoId`.
    ///
    /// # Errors
    /// Returns `TodoError::NotFound(id)` if no todo with that id exists.
    pub fn delete(&mut self, id: TodoId) -> Result<(), TodoError> {
        if self.store.get(id).is_none() {
            return Err(TodoError::NotFound(id));
        }
        self.store.remove(id);
        Ok(())
    }

    /// Search todos by keyword (matches title; optionally description and tags when present).
    #[must_use]
    pub fn search(&self, keyword: &str) -> Vec<Todo> {
        let k = keyword.trim().to_lowercase();
        if k.is_empty() {
            return self.store.list();
        }
        self.store
            .list()
            .into_iter()
            .filter(|t| {
                t.title.to_lowercase().contains(&k)
                    || t.description
                        .as_ref()
                        .is_some_and(|d| d.to_lowercase().contains(&k))
                    || t.tags.iter().any(|tag| tag.to_lowercase().contains(&k))
            })
            .collect()
    }

    /// Returns counts: total, incomplete, complete.
    #[must_use]
    pub fn stats(&self) -> (usize, usize, usize) {
        let items = self.store.list();
        let total = items.len();
        let complete = items.iter().filter(|t| t.completed).count();
        let incomplete = total - complete;
        (total, incomplete, complete)
    }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use crate::model::{ListFilter, ListOptions, ListSort, Todo, TodoPatch};
    use crate::store::InMemoryStore;
    use crate::{RepeatRule, TodoId};

    use super::TodoList;

    #[test]
    fn create_inserts_into_store() {
        let mut list = TodoList::new();
        let id = list.create("task").unwrap();
        assert!(id.as_u64() >= 1);
    }

    #[test]
    fn list_with_options_sort_title() {
        let now = SystemTime::now();
        let todos = vec![
            Todo {
                id: TodoId::from_raw(1).unwrap(),
                title: "z".into(),
                completed: false,
                created_at: now,
                completed_at: None,
                description: None,
                due_date: None,
                priority: None,
                tags: vec![],
                repeat_rule: None,
                repeat_until: None,
                repeat_count: None,
            },
            Todo {
                id: TodoId::from_raw(2).unwrap(),
                title: "a".into(),
                completed: false,
                created_at: now,
                completed_at: None,
                description: None,
                due_date: None,
                priority: None,
                tags: vec![],
                repeat_rule: None,
                repeat_until: None,
                repeat_count: None,
            },
        ];
        let list = TodoList::with_store(InMemoryStore::from_todos(todos));
        let by_title = list.list_with_options(&ListOptions {
            filter: None,
            sort: ListSort::Title,
        });
        assert_eq!(by_title[0].title, "a");
    }

    #[test]
    fn list_with_options_filter_due_after() {
        let now = SystemTime::now();
        let todos = vec![
            Todo {
                id: TodoId::from_raw(1).unwrap(),
                title: "early".into(),
                completed: false,
                created_at: now,
                completed_at: None,
                description: None,
                due_date: Some("2025-06-01".into()),
                priority: None,
                tags: vec![],
                repeat_rule: None,
                repeat_until: None,
                repeat_count: None,
            },
            Todo {
                id: TodoId::from_raw(2).unwrap(),
                title: "late".into(),
                completed: false,
                created_at: now,
                completed_at: None,
                description: None,
                due_date: Some("2025-07-01".into()),
                priority: None,
                tags: vec![],
                repeat_rule: None,
                repeat_until: None,
                repeat_count: None,
            },
        ];
        let list = TodoList::with_store(InMemoryStore::from_todos(todos));
        let filtered = list.list_with_options(&ListOptions {
            filter: Some(ListFilter {
                due_after: Some("2025-06-15".into()),
                ..ListFilter::default()
            }),
            sort: ListSort::CreatedAt,
        });
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].title, "late");
    }

    #[test]
    fn update_title_and_patch_repeat_clear() {
        let mut list = TodoList::new();
        let id = list.create("t").unwrap();
        list.update_title(id, "updated").unwrap();
        assert_eq!(list.get(id).unwrap().title, "updated");
        list.update(
            id,
            TodoPatch {
                repeat_rule: Some(RepeatRule::Daily),
                repeat_until: Some("2025-12-31".into()),
                ..TodoPatch::default()
            },
        )
        .unwrap();
        list.update(
            id,
            TodoPatch {
                repeat_rule_clear: true,
                ..TodoPatch::default()
            },
        )
        .unwrap();
        assert!(list.get(id).unwrap().repeat_rule.is_none());
    }

    #[test]
    fn search_matches_description_and_tags() {
        let mut list = TodoList::new();
        let _ = list.create("x").unwrap();
        let id2 = list.create("y").unwrap();
        list.update(
            id2,
            TodoPatch {
                description: Some("secret".into()),
                tags: Some(vec!["tag".into()]),
                ..TodoPatch::default()
            },
        )
        .unwrap();
        let by_desc = list.search("secret");
        assert_eq!(by_desc.len(), 1);
        let by_tag = list.search("tag");
        assert_eq!(by_tag.len(), 1);
    }
}
