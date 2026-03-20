//! Tests for `add_todo`, update (patch), and `list_with_options`.

use std::time::SystemTime;

use crate::model::{ListFilter, ListOptions, ListSort, TodoPatch};
use crate::{InMemoryStore, Priority, RepeatRule, Todo, TodoId, TodoList};

fn make_todo(
    id: u64,
    title: &str,
    completed: bool,
    due_date: Option<&str>,
    priority: Option<Priority>,
    tags: Vec<&str>,
) -> Todo {
    let created = SystemTime::now();
    Todo {
        id: TodoId::from_raw(id).unwrap(),
        title: title.to_string(),
        completed,
        created_at: created,
        completed_at: if completed { Some(created) } else { None },
        description: None,
        due_date: due_date.map(str::to_string),
        priority,
        tags: tags.into_iter().map(str::to_string).collect(),
        repeat_rule: None,
        repeat_until: None,
        repeat_count: None,
    }
}

#[test]
fn add_todo_creates_new_id() {
    let mut list = TodoList::new();
    let t = make_todo(1, "original", false, None, None, vec![]);
    let id = list.add_todo(&t);
    assert_eq!(id.as_u64(), 1);
    let items = list.list();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].title, "original");
    let t2 = make_todo(2, "second", true, None, Some(Priority::High), vec!["a"]);
    let id2 = list.add_todo(&t2);
    assert_eq!(id2.as_u64(), 2);
    assert_eq!(list.list().len(), 2);
}

#[test]
fn update_patch_description_due_priority_tags_repeat() {
    let mut list = TodoList::new();
    let id = list.create("title").unwrap();
    list.update(
        id,
        TodoPatch {
            description: Some("desc".into()),
            due_date: Some("2026-01-01".into()),
            priority: Some(Priority::High),
            tags: Some(vec!["work".into()]),
            repeat_rule: Some(RepeatRule::Daily),
            ..TodoPatch::default()
        },
    )
    .unwrap();
    let t = list.get(id).unwrap();
    assert_eq!(t.description.as_deref(), Some("desc"));
    assert_eq!(t.due_date.as_deref(), Some("2026-01-01"));
    assert_eq!(t.priority, Some(Priority::High));
    assert_eq!(t.tags, vec!["work"]);
    assert_eq!(t.repeat_rule, Some(RepeatRule::Daily));
}

#[test]
fn update_patch_invalid_title_returns_err() {
    let mut list = TodoList::new();
    let id = list.create("ok").unwrap();
    let err = list
        .update(
            id,
            TodoPatch {
                title: Some(String::new()),
                ..TodoPatch::default()
            },
        )
        .unwrap_err();
    assert!(matches!(err, crate::TodoError::InvalidInput));
}

#[test]
fn list_with_options_filter_status() {
    let created = SystemTime::now();
    let todos = vec![
        Todo {
            id: TodoId::from_raw(1).unwrap(),
            title: "a".into(),
            completed: false,
            created_at: created,
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
            title: "b".into(),
            completed: true,
            created_at: created,
            completed_at: Some(created),
            description: None,
            due_date: None,
            priority: None,
            tags: vec![],
            repeat_rule: None,
            repeat_until: None,
            repeat_count: None,
        },
    ];
    let store = InMemoryStore::from_todos(todos);
    let list = TodoList::with_store(store);
    let incomplete = list.list_with_options(&ListOptions {
        filter: Some(ListFilter {
            status: Some(false),
            ..ListFilter::default()
        }),
        sort: ListSort::CreatedAt,
    });
    assert_eq!(incomplete.len(), 1);
    assert_eq!(incomplete[0].title, "a");
    let complete = list.list_with_options(&ListOptions {
        filter: Some(ListFilter {
            status: Some(true),
            ..ListFilter::default()
        }),
        sort: ListSort::CreatedAt,
    });
    assert_eq!(complete.len(), 1);
    assert_eq!(complete[0].title, "b");
}

#[test]
fn list_with_options_filter_priority_and_tags() {
    let created = SystemTime::now();
    let todos = vec![
        Todo {
            id: TodoId::from_raw(1).unwrap(),
            title: "low".into(),
            completed: false,
            created_at: created,
            completed_at: None,
            description: None,
            due_date: None,
            priority: Some(Priority::Low),
            tags: vec!["x".into()],
            repeat_rule: None,
            repeat_until: None,
            repeat_count: None,
        },
        Todo {
            id: TodoId::from_raw(2).unwrap(),
            title: "high".into(),
            completed: false,
            created_at: created,
            completed_at: None,
            description: None,
            due_date: None,
            priority: Some(Priority::High),
            tags: vec!["y".into()],
            repeat_rule: None,
            repeat_until: None,
            repeat_count: None,
        },
    ];
    let store = InMemoryStore::from_todos(todos);
    let list = TodoList::with_store(store);
    let high = list.list_with_options(&ListOptions {
        filter: Some(ListFilter {
            priority: Some(Priority::High),
            ..ListFilter::default()
        }),
        sort: ListSort::CreatedAt,
    });
    assert_eq!(high.len(), 1);
    assert_eq!(high[0].title, "high");
    let with_tag = list.list_with_options(&ListOptions {
        filter: Some(ListFilter {
            tags_any: Some(vec!["x".into()]),
            ..ListFilter::default()
        }),
        sort: ListSort::CreatedAt,
    });
    assert_eq!(with_tag.len(), 1);
    assert_eq!(with_tag[0].tags, vec!["x"]);
}

#[test]
fn list_with_options_filter_tags_any_empty() {
    let created = SystemTime::now();
    let todos = vec![Todo {
        id: TodoId::from_raw(1).unwrap(),
        title: "any".into(),
        completed: false,
        created_at: created,
        completed_at: None,
        description: None,
        due_date: None,
        priority: None,
        tags: vec![],
        repeat_rule: None,
        repeat_until: None,
        repeat_count: None,
    }];
    let store = InMemoryStore::from_todos(todos);
    let list = TodoList::with_store(store);
    let items = list.list_with_options(&ListOptions {
        filter: Some(ListFilter {
            tags_any: Some(vec![]),
            ..ListFilter::default()
        }),
        sort: ListSort::CreatedAt,
    });
    assert_eq!(items.len(), 1);
}

#[test]
fn list_with_options_filter_due_before_after() {
    let created = SystemTime::now();
    let todos = vec![
        Todo {
            id: TodoId::from_raw(1).unwrap(),
            title: "early".into(),
            completed: false,
            created_at: created,
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
            created_at: created,
            completed_at: None,
            description: None,
            due_date: Some("2025-12-01".into()),
            priority: None,
            tags: vec![],
            repeat_rule: None,
            repeat_until: None,
            repeat_count: None,
        },
        Todo {
            id: TodoId::from_raw(3).unwrap(),
            title: "no due".into(),
            completed: false,
            created_at: created,
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
    let store = InMemoryStore::from_todos(todos);
    let list = TodoList::with_store(store);
    let before = list.list_with_options(&ListOptions {
        filter: Some(ListFilter {
            due_before: Some("2025-07-01".into()),
            ..ListFilter::default()
        }),
        sort: ListSort::CreatedAt,
    });
    assert_eq!(before.len(), 1);
    assert_eq!(before[0].title, "early");
    let after = list.list_with_options(&ListOptions {
        filter: Some(ListFilter {
            due_after: Some("2025-07-01".into()),
            ..ListFilter::default()
        }),
        sort: ListSort::CreatedAt,
    });
    assert_eq!(after.len(), 1);
    assert_eq!(after[0].title, "late");
}

/// `due_after` excludes todos whose due date is strictly before the cutoff (`due < d` branch).
#[test]
fn list_with_options_due_after_excludes_todos_before_cutoff() {
    let todos = vec![
        make_todo(1, "before", false, Some("2025-05-01"), None, vec![]),
        make_todo(2, "on_cutoff", false, Some("2025-07-01"), None, vec![]),
        make_todo(3, "after", false, Some("2025-08-01"), None, vec![]),
    ];
    let list = TodoList::with_store(InMemoryStore::from_todos(todos));
    let filtered = list.list_with_options(&ListOptions {
        filter: Some(ListFilter {
            due_after: Some("2025-07-01".into()),
            ..ListFilter::default()
        }),
        sort: ListSort::CreatedAt,
    });
    assert_eq!(filtered.len(), 2);
    let titles: Vec<_> = filtered.iter().map(|t| t.title.as_str()).collect();
    assert!(titles.contains(&"on_cutoff"));
    assert!(titles.contains(&"after"));
    assert!(!titles.contains(&"before"));
}

#[test]
fn list_with_options_sort_due_date_priority_title() {
    let created = SystemTime::now();
    let todos = vec![
        Todo {
            id: TodoId::from_raw(1).unwrap(),
            title: "z".into(),
            completed: false,
            created_at: created,
            completed_at: None,
            description: None,
            due_date: Some("2025-02-01".into()),
            priority: Some(Priority::Low),
            tags: vec![],
            repeat_rule: None,
            repeat_until: None,
            repeat_count: None,
        },
        Todo {
            id: TodoId::from_raw(2).unwrap(),
            title: "a".into(),
            completed: false,
            created_at: created,
            completed_at: None,
            description: None,
            due_date: Some("2025-01-01".into()),
            priority: Some(Priority::High),
            tags: vec![],
            repeat_rule: None,
            repeat_until: None,
            repeat_count: None,
        },
    ];
    let store = InMemoryStore::from_todos(todos);
    let list = TodoList::with_store(store);
    let by_due = list.list_with_options(&ListOptions {
        filter: None,
        sort: ListSort::DueDate,
    });
    assert_eq!(by_due[0].title, "a");
    assert_eq!(by_due[1].title, "z");
    let by_priority = list.list_with_options(&ListOptions {
        filter: None,
        sort: ListSort::Priority,
    });
    assert_eq!(by_priority[0].title, "z");
    assert_eq!(by_priority[1].title, "a");
    let by_title = list.list_with_options(&ListOptions {
        filter: None,
        sort: ListSort::Title,
    });
    assert_eq!(by_title[0].title, "a");
    assert_eq!(by_title[1].title, "z");
}
