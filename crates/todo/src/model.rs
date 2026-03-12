//! Todo item and list options (filter, sort).

use std::time::SystemTime;

use crate::priority::Priority;
use crate::repeat::RepeatRule;

/// A single todo item.
#[derive(Clone, Eq, PartialEq)]
pub struct Todo {
    pub id: crate::id::TodoId,
    pub title: String,
    pub completed: bool,
    pub created_at: SystemTime,
    /// When the todo was marked completed; None if still open.
    pub completed_at: Option<SystemTime>,
    /// Optional longer description.
    pub description: Option<String>,
    /// Optional due date (ISO 8601 date, e.g. YYYY-MM-DD).
    pub due_date: Option<String>,
    /// Optional priority.
    pub priority: Option<Priority>,
    /// Tags for grouping/filtering.
    pub tags: Vec<String>,
    /// Optional repeat rule for recurring tasks.
    pub repeat_rule: Option<RepeatRule>,
    /// Optional end date for recurrence (YYYY-MM-DD); no next instance if next due > this.
    pub repeat_until: Option<String>,
    /// Optional remaining occurrences for recurrence; no next instance when this is 0 or 1 (1 = last).
    pub repeat_count: Option<u32>,
}

/// Partial update for a todo; only `Some` fields are applied.
#[derive(Clone, Default)]
pub struct TodoPatch {
    pub title: Option<String>,
    pub description: Option<String>,
    pub due_date: Option<String>,
    pub priority: Option<Priority>,
    pub tags: Option<Vec<String>>,
    pub repeat_rule: Option<RepeatRule>,
    pub repeat_until: Option<String>,
    pub repeat_count: Option<u32>,
}

/// Filter criteria for listing todos.
#[derive(Clone, Default)]
pub struct ListFilter {
    /// If set, filter by completed status.
    pub status: Option<bool>,
    /// If set, only items with this priority.
    pub priority: Option<Priority>,
    /// If set, item must have at least one of these tags.
    pub tags_any: Option<Vec<String>>,
    /// If set, `due_date` must be <= this (YYYY-MM-DD).
    pub due_before: Option<String>,
    /// If set, `due_date` must be >= this (YYYY-MM-DD).
    pub due_after: Option<String>,
}

/// Sort order for listing.
#[derive(Clone, Copy, Default)]
pub enum ListSort {
    #[default]
    CreatedAt,
    DueDate,
    Priority,
    Title,
}

/// Options for list (filter + sort).
#[derive(Clone, Default)]
pub struct ListOptions {
    pub filter: Option<ListFilter>,
    pub sort: ListSort,
}
