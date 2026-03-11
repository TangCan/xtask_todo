//! Integration tests: public API and `InMemoryStore::from_todos` flow (no JSON; DTO round-trip lives in xtask).

use std::time::SystemTime;

use todo::{InMemoryStore, Todo, TodoId, TodoList};

#[test]
fn from_todos_then_create_list_complete_delete() {
    let now = SystemTime::now();
    let id1 = TodoId::from_raw(1).unwrap();
    let id2 = TodoId::from_raw(2).unwrap();
    let existing = vec![
        Todo {
            id: id1,
            title: "existing one".to_string(),
            completed: false,
            created_at: now,
            completed_at: None,
        },
        Todo {
            id: id2,
            title: "existing two".to_string(),
            completed: false,
            created_at: now,
            completed_at: None,
        },
    ];
    let store = InMemoryStore::from_todos(existing);
    let mut list = TodoList::with_store(store);

    let id3 = list.create("new item").unwrap();
    assert_eq!(id3.as_u64(), 3);

    let items = list.list();
    assert_eq!(items.len(), 3);
    assert_eq!(items[0].title, "existing one");
    assert_eq!(items[1].title, "existing two");
    assert_eq!(items[2].title, "new item");

    list.complete(id1).unwrap();
    let items = list.list();
    assert!(items[0].completed);
    assert!(items[0].completed_at.is_some());

    list.delete(id2).unwrap();
    let items = list.list();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].id, id1);
    assert_eq!(items[1].id, id3);
}
