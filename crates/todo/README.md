# xtask-todo-lib

Todo list library: create, list, complete, and delete items with in-memory or pluggable storage.

## Usage

Add to `Cargo.toml`:

```toml
[dependencies]
xtask-todo-lib = "0.1"
```

## Example

```rust
use xtask_todo_lib::{InMemoryStore, TodoId, TodoList};

let store = InMemoryStore::new();
let mut list = TodoList::with_store(store);

let id = list.create("Buy milk".into()).unwrap();
list.complete(id).unwrap();
let items = list.list();
```

## Main types

- **`TodoList<S>`** – facade over a store `S`; use `TodoList::new()` for in-memory or `TodoList::with_store(store)` for a custom store.
- **`TodoId`** – opaque id for a todo; use for `complete` / `delete`.
- **`Todo`** – single item (`id`, `title`, `completed`, `created_at`, `completed_at`).
- **`InMemoryStore`** – default in-memory store; implement **`Store`** for your own backend.

## `cargo install` (Windows)

Use **`xtask-todo-lib` 0.1.16+** for `cargo install xtask-todo-lib` on Windows. Older releases (e.g. 0.1.15) could fail to compile: `rustyline` was only listed under Linux target deps, `WorkspaceBackendError` was imported under `cfg(unix)` while used unconditionally, and `vm_workspace_host_root` was Unix-only while still referenced in type-checked branches.

## License

MIT OR Apache-2.0
