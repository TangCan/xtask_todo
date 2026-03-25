//! Tests for `complete_with_repeat`, search, and stats.

use crate::{InMemoryStore, RepeatRule, Todo, TodoId, TodoList};

#[test]
fn complete_with_repeat_creates_next() {
    let created = std::time::SystemTime::now();
    let id1 = TodoId::from_raw(1).unwrap();
    let todos = vec![Todo {
        id: id1,
        title: "recur".into(),
        completed: false,
        created_at: created,
        completed_at: None,
        description: None,
        due_date: Some("2025-01-10".into()),
        priority: None,
        tags: vec![],
        repeat_rule: Some(RepeatRule::Daily),
        repeat_until: None,
        repeat_count: None,
    }];
    let store = InMemoryStore::from_todos(todos);
    let mut list = TodoList::with_store(store);
    list.complete(id1, false).unwrap();
    let items = list.list();
    assert_eq!(items.len(), 2);
    assert!(items[0].completed);
    assert!(!items[1].completed);
    assert_eq!(items[1].due_date.as_deref(), Some("2025-01-11"));
}

#[test]
fn complete_with_repeat_no_next_does_not_create() {
    let created = std::time::SystemTime::now();
    let id1 = TodoId::from_raw(1).unwrap();
    let todos = vec![Todo {
        id: id1,
        title: "once".into(),
        completed: false,
        created_at: created,
        completed_at: None,
        description: None,
        due_date: Some("2025-01-10".into()),
        priority: None,
        tags: vec![],
        repeat_rule: Some(RepeatRule::Daily),
        repeat_until: None,
        repeat_count: None,
    }];
    let store = InMemoryStore::from_todos(todos);
    let mut list = TodoList::with_store(store);
    list.complete(id1, true).unwrap();
    assert_eq!(list.list().len(), 1);
}

#[test]
fn complete_with_repeat_count_one_does_not_create_next() {
    let created = std::time::SystemTime::now();
    let id1 = TodoId::from_raw(1).unwrap();
    let todos = vec![Todo {
        id: id1,
        title: "last".into(),
        completed: false,
        created_at: created,
        completed_at: None,
        description: None,
        due_date: Some("2025-01-10".into()),
        priority: None,
        tags: vec![],
        repeat_rule: Some(RepeatRule::Daily),
        repeat_until: None,
        repeat_count: Some(1),
    }];
    let store = InMemoryStore::from_todos(todos);
    let mut list = TodoList::with_store(store);
    list.complete(id1, false).unwrap();
    assert_eq!(list.list().len(), 1);
}

#[test]
fn complete_with_repeat_until_past_does_not_create_next() {
    let created = std::time::SystemTime::now();
    let id1 = TodoId::from_raw(1).unwrap();
    let todos = vec![Todo {
        id: id1,
        title: "until".into(),
        completed: false,
        created_at: created,
        completed_at: None,
        description: None,
        due_date: Some("2025-01-10".into()),
        priority: None,
        tags: vec![],
        repeat_rule: Some(RepeatRule::Daily),
        repeat_until: Some("2025-01-10".into()),
        repeat_count: None,
    }];
    let store = InMemoryStore::from_todos(todos);
    let mut list = TodoList::with_store(store);
    list.complete(id1, false).unwrap();
    assert_eq!(list.list().len(), 1);
}

#[test]
fn search_by_title_description_tags() {
    let created = std::time::SystemTime::now();
    let todos = vec![
        Todo {
            id: TodoId::from_raw(1).unwrap(),
            title: "Buy milk".into(),
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
            title: "other".into(),
            completed: false,
            created_at: created,
            completed_at: None,
            description: Some("milk recipe".into()),
            due_date: None,
            priority: None,
            tags: vec![],
            repeat_rule: None,
            repeat_until: None,
            repeat_count: None,
        },
        Todo {
            id: TodoId::from_raw(3).unwrap(),
            title: "x".into(),
            completed: false,
            created_at: created,
            completed_at: None,
            description: None,
            due_date: None,
            priority: None,
            tags: vec!["milk".into()],
            repeat_rule: None,
            repeat_until: None,
            repeat_count: None,
        },
    ];
    let store = InMemoryStore::from_todos(todos);
    let list = TodoList::with_store(store);
    let by_title = list.search("Buy");
    assert_eq!(by_title.len(), 1);
    assert_eq!(by_title[0].title, "Buy milk");
    let by_desc = list.search("recipe");
    assert_eq!(by_desc.len(), 1);
    assert_eq!(by_desc[0].description.as_deref(), Some("milk recipe"));
    let by_tag = list.search("milk");
    assert_eq!(by_tag.len(), 3);
    assert!(by_tag.iter().any(|t| t.title == "Buy milk"));
    assert!(by_tag
        .iter()
        .any(|t| t.description.as_deref() == Some("milk recipe")));
    assert!(by_tag.iter().any(|t| t.tags.contains(&"milk".to_string())));
    let empty = list.search("   ");
    assert_eq!(empty.len(), 3);
}

#[test]
fn search_is_case_insensitive() {
    let created = std::time::SystemTime::now();
    let todos = vec![Todo {
        id: TodoId::from_raw(1).unwrap(),
        title: "CamelCase Title".into(),
        completed: false,
        created_at: created,
        completed_at: None,
        description: Some("UPPER desc".into()),
        due_date: None,
        priority: None,
        tags: vec!["MiXeD".into()],
        repeat_rule: None,
        repeat_until: None,
        repeat_count: None,
    }];
    let store = InMemoryStore::from_todos(todos);
    let list = TodoList::with_store(store);
    assert_eq!(list.search("camelcase").len(), 1);
    assert_eq!(list.search("upper").len(), 1);
    assert_eq!(list.search("mixed").len(), 1);
}

#[test]
fn stats_counts_total_incomplete_complete() {
    let mut list = TodoList::new();
    list.create("a").unwrap();
    list.create("b").unwrap();
    let id = list.create("c").unwrap();
    list.complete(id, false).unwrap();
    let (total, incomplete, complete) = list.stats();
    assert_eq!(total, 3);
    assert_eq!(incomplete, 2);
    assert_eq!(complete, 1);
}
