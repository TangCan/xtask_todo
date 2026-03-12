//! Basic CRUD and core type tests.

use crate::{InMemoryStore, TodoError, TodoId, TodoList};

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
    list.complete(id, false).unwrap();
    let items = list.list();
    assert!(items[0].completed);
    assert!(items[0].completed_at.is_some());
}

#[test]
fn us_t3_complete_nonexistent_returns_not_found() {
    let mut list = TodoList::new();
    list.create("only").unwrap();
    let bad_id = TodoId::from_raw(999).unwrap();
    let err = list.complete(bad_id, false).unwrap_err();
    match &err {
        TodoError::NotFound(x) => assert_eq!(*x, bad_id),
        TodoError::InvalidInput => panic!("expected NotFound"),
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
        TodoError::InvalidInput => panic!("expected NotFound"),
    }
    assert_eq!(list.list().len(), 1);
}

#[test]
fn todo_id_as_u64_and_display() {
    let id = TodoId::from_raw(42).unwrap();
    assert_eq!(id.as_u64(), 42);
    assert_eq!(format!("{id}"), "42");
}

#[test]
fn todo_error_display() {
    assert_eq!(
        format!("{}", TodoError::InvalidInput),
        "invalid input: title must be non-empty"
    );
    let id = TodoId::from_raw(1).unwrap();
    assert_eq!(format!("{}", TodoError::NotFound(id)), "todo not found: 1");
}

#[test]
fn default_todo_list() {
    let list = TodoList::default();
    assert!(list.list().is_empty());
}

#[test]
fn with_store() {
    let store = InMemoryStore::new();
    let mut list = TodoList::with_store(store);
    let id = list.create("task").unwrap();
    assert_eq!(list.list().len(), 1);
    assert_eq!(list.list()[0].id.as_u64(), id.as_u64());
}

#[test]
fn get_existing_returns_todo() {
    let mut list = TodoList::new();
    let id = list.create("item").unwrap();
    let t = list.get(id).unwrap();
    assert_eq!(t.id, id);
    assert_eq!(t.title, "item");
    assert!(!t.completed);
}

#[test]
fn get_nonexistent_returns_none() {
    let list = TodoList::new();
    let bad_id = TodoId::from_raw(99).unwrap();
    assert!(list.get(bad_id).is_none());
}

#[test]
fn update_title_success() {
    let mut list = TodoList::new();
    let id = list.create("old").unwrap();
    list.update_title(id, "new").unwrap();
    let t = list.get(id).unwrap();
    assert_eq!(t.title, "new");
}

#[test]
fn update_title_nonexistent_returns_err() {
    let mut list = TodoList::new();
    list.create("x").unwrap();
    let bad_id = TodoId::from_raw(99).unwrap();
    assert!(list.update_title(bad_id, "y").is_err());
}

#[test]
fn update_title_empty_returns_err() {
    let mut list = TodoList::new();
    let id = list.create("x").unwrap();
    assert!(list.update_title(id, "").is_err());
    assert_eq!(list.get(id).unwrap().title, "x");
}

#[test]
fn todo_id_from_raw_zero_returns_none() {
    assert!(TodoId::from_raw(0).is_none());
}
