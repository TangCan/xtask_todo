//! Integration tests: public API and `InMemoryStore::from_todos` flow (no JSON; DTO round-trip lives in xtask).
//! Also runs the cargo-devshell binary to cover its usage error path.

use std::process::Command;
use std::time::SystemTime;

use xtask_todo_lib::{InMemoryStore, Todo, TodoId, TodoList};

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
            description: None,
            due_date: None,
            priority: None,
            tags: Vec::new(),
            repeat_rule: None,
            repeat_until: None,
            repeat_count: None,
        },
        Todo {
            id: id2,
            title: "existing two".to_string(),
            completed: false,
            created_at: now,
            completed_at: None,
            description: None,
            due_date: None,
            priority: None,
            tags: Vec::new(),
            repeat_rule: None,
            repeat_until: None,
            repeat_count: None,
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

    list.complete(id1, false).unwrap();
    let items = list.list();
    assert!(items[0].completed);
    assert!(items[0].completed_at.is_some());

    list.delete(id2).unwrap();
    let items = list.list();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].id, id1);
    assert_eq!(items[1].id, id3);
}

#[test]
fn cargo_devshell_usage_error_exits_nonzero() {
    let bin = std::env::var_os("CARGO_BIN_EXE_cargo-devshell")
        .or_else(|| std::env::var_os("CARGO_BIN_EXE_cargo_devshell"));
    let Some(bin) = bin else {
        return; // skip when not set (e.g. under tarpaulin)
    };
    let out = Command::new(bin).args(["a", "b", "c"]).output().unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("usage") || stderr.contains("dev_shell"));
}
