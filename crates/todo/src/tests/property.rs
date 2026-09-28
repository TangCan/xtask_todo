//! Property and snapshot tests for list filtering and sorting.

use std::time::{Duration, UNIX_EPOCH};

use insta::assert_debug_snapshot;
use proptest::prelude::*;

use crate::{InMemoryStore, ListFilter, ListOptions, ListSort, Priority, Todo, TodoId, TodoList};

fn todo(id: u64, completed: bool, tagged: bool) -> Todo {
    let created_at = UNIX_EPOCH + Duration::from_secs(id);
    Todo {
        id: TodoId::from_raw(id).expect("proptest ids start at one"),
        title: format!("todo-{id}"),
        completed,
        created_at,
        completed_at: completed.then_some(created_at),
        description: None,
        due_date: None,
        priority: None,
        tags: if tagged {
            vec!["work".to_string()]
        } else {
            Vec::new()
        },
        repeat_rule: None,
        repeat_until: None,
        repeat_count: None,
    }
}

proptest! {
    #[test]
    fn status_filter_returns_only_requested_state(states in proptest::collection::vec(any::<bool>(), 0..40)) {
        let todos: Vec<_> = states
            .iter()
            .enumerate()
            .map(|(index, completed)| todo(index as u64 + 1, *completed, false))
            .collect();
        let list = TodoList::with_store(InMemoryStore::from_todos(todos));

        for status in [false, true] {
            let filtered = list.list_with_options(&ListOptions {
                filter: Some(ListFilter {
                    status: Some(status),
                    ..ListFilter::default()
                }),
                sort: ListSort::CreatedAt,
            });
            prop_assert!(filtered.iter().all(|item| item.completed == status));
            prop_assert_eq!(filtered.len(), states.iter().filter(|value| **value == status).count());
        }
    }

    #[test]
    fn tag_filter_is_a_subset_of_tagged_items(tags in proptest::collection::vec(any::<bool>(), 0..40)) {
        let todos: Vec<_> = tags
            .iter()
            .enumerate()
            .map(|(index, tagged)| todo(index as u64 + 1, false, *tagged))
            .collect();
        let list = TodoList::with_store(InMemoryStore::from_todos(todos));
        let filtered = list.list_with_options(&ListOptions {
            filter: Some(ListFilter {
                tags_any: Some(vec!["work".to_string()]),
                ..ListFilter::default()
            }),
            sort: ListSort::CreatedAt,
        });

        prop_assert!(filtered.iter().all(|item| item.tags.iter().any(|tag| tag == "work")));
        prop_assert_eq!(filtered.len(), tags.iter().filter(|tagged| **tagged).count());
    }

    #[test]
    fn priority_filter_returns_only_requested_priority(priorities in proptest::collection::vec(any::<bool>(), 0..40)) {
        let todos: Vec<_> = priorities
            .iter()
            .enumerate()
            .map(|(index, high)| {
                let mut item = todo(index as u64 + 1, false, false);
                item.priority = Some(if *high { Priority::High } else { Priority::Low });
                item
            })
            .collect();
        let list = TodoList::with_store(InMemoryStore::from_todos(todos));

        for priority in [Priority::Low, Priority::High] {
            let filtered = list.list_with_options(&ListOptions {
                filter: Some(ListFilter {
                    priority: Some(priority),
                    ..ListFilter::default()
                }),
                sort: ListSort::CreatedAt,
            });
            prop_assert!(filtered.iter().all(|item| item.priority == Some(priority)));
        }
    }

    #[test]
    fn due_date_filters_respect_inclusive_bounds(dates in proptest::collection::vec(any::<bool>(), 0..40)) {
        let todos: Vec<_> = dates
            .iter()
            .enumerate()
            .map(|(index, before)| {
                let mut item = todo(index as u64 + 1, false, false);
                item.due_date = Some(if *before { "2025-01-01" } else { "2026-01-01" }.into());
                item
            })
            .collect();
        let list = TodoList::with_store(InMemoryStore::from_todos(todos));

        let before = list.list_with_options(&ListOptions {
            filter: Some(ListFilter {
                due_before: Some("2025-06-01".into()),
                ..ListFilter::default()
            }),
            sort: ListSort::CreatedAt,
        });
        let after = list.list_with_options(&ListOptions {
            filter: Some(ListFilter {
                due_after: Some("2025-06-01".into()),
                ..ListFilter::default()
            }),
            sort: ListSort::CreatedAt,
        });
        prop_assert_eq!(before.len(), dates.iter().filter(|date| **date).count());
        prop_assert_eq!(after.len(), dates.iter().filter(|date| !**date).count());
    }
}

#[test]
fn title_sort_snapshot_is_stable() {
    let todos = vec![
        todo(1, false, false),
        todo(2, false, false),
        todo(3, false, false),
    ];
    let mut list = TodoList::with_store(InMemoryStore::from_todos(todos));
    list.update_title(TodoId::from_raw(1).unwrap(), "zulu")
        .unwrap();
    list.update_title(TodoId::from_raw(2).unwrap(), "alpha")
        .unwrap();
    list.update_title(TodoId::from_raw(3).unwrap(), "middle")
        .unwrap();

    let titles: Vec<_> = list
        .list_with_options(&ListOptions {
            filter: None,
            sort: ListSort::Title,
        })
        .into_iter()
        .map(|item| item.title)
        .collect();
    assert_debug_snapshot!(titles);
}
