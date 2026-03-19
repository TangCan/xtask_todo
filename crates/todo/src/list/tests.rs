//! Unit tests for `TodoList`.

use std::time::SystemTime;

use crate::model::{ListFilter, ListOptions, ListSort, Todo, TodoPatch};
use crate::store::InMemoryStore;
use crate::{RepeatRule, TodoId, TodoList};

#[test]
fn create_inserts_into_store() {
    let mut list = TodoList::new();
    let id = list.create("task").unwrap();
    assert!(id.as_u64() >= 1);
}

#[test]
fn add_todo_inserts_with_new_id() {
    let mut list = TodoList::new();
    let t = Todo {
        id: TodoId::from_raw(1).unwrap(),
        title: "imported".into(),
        completed: false,
        created_at: SystemTime::now(),
        completed_at: None,
        description: None,
        due_date: None,
        priority: None,
        tags: vec![],
        repeat_rule: None,
        repeat_until: None,
        repeat_count: None,
    };
    let id = list.add_todo(&t);
    assert_eq!(id.as_u64(), 1);
    assert_eq!(list.list().len(), 1);
    assert_eq!(list.list()[0].title, "imported");
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

/// `due_before` filter excludes todos with no `due_date` (covers `list_with_options` branch).
#[test]
fn list_with_options_due_before_excludes_no_due_date() {
    let now = SystemTime::now();
    let todos = vec![
        Todo {
            id: TodoId::from_raw(1).unwrap(),
            title: "no due".into(),
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
            title: "with due".into(),
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
    ];
    let list = TodoList::with_store(InMemoryStore::from_todos(todos));
    let filtered = list.list_with_options(&ListOptions {
        filter: Some(ListFilter {
            due_before: Some("2025-07-01".into()),
            ..ListFilter::default()
        }),
        sort: ListSort::CreatedAt,
    });
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].title, "with due");
}

/// `due_after` filter excludes todos with no `due_date` (covers `list_with_options` branch).
#[test]
fn list_with_options_due_after_excludes_no_due_date() {
    let now = SystemTime::now();
    let todos = vec![
        Todo {
            id: TodoId::from_raw(1).unwrap(),
            title: "no due".into(),
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
            title: "with due".into(),
            completed: false,
            created_at: now,
            completed_at: None,
            description: None,
            due_date: Some("2025-08-01".into()),
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
            due_after: Some("2025-07-01".into()),
            ..ListFilter::default()
        }),
        sort: ListSort::CreatedAt,
    });
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].title, "with due");
}

/// update with `repeat_count` and search with empty keyword.
#[test]
fn update_repeat_count_and_search_empty_returns_all() {
    let mut list = TodoList::new();
    let _ = list.create("a").unwrap();
    let _ = list.create("b").unwrap();
    list.update(
        list.list().first().map(|t| t.id).unwrap(),
        TodoPatch {
            repeat_count: Some(2),
            ..TodoPatch::default()
        },
    )
    .unwrap();
    let empty_search = list.search("");
    assert_eq!(empty_search.len(), 2);
    let whitespace_search = list.search("   ");
    assert_eq!(whitespace_search.len(), 2);
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
