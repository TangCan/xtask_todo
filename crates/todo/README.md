# xtask-todo-lib

Todo list library: create, list, complete, and delete items with in-memory or pluggable storage.

## Product boundary (FR34)

This crate focuses on local library/CLI capabilities. It does **not** imply a hosted HTTP API, multi-tenant service model, or automatic `.todo.json` migration pipeline by default. For the authoritative product boundary, see [../../docs/requirements.md §2](../../docs/requirements.md).

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

## Devshell VM on Windows

The crate **defaults** to the **`beta-vm`** feature: on Windows, `cargo-devshell` uses the **beta** backend and JSON-lines **`devshell-vm --serve-stdio`** over stdio (see **`docs/devshell-vm-windows.md`**). To depend without it: `xtask-todo-lib = { version = "0.1", default-features = false }`.

**`cargo install` does not include a Linux `devshell-vm` on the host disk**, but the default flow **falls back** to an **OCI image** (`podman pull` + `podman run -i` with the workspace at `/workspace`) when no host ELF is found — **no mandatory env vars** for typical use. Optional **`DEVSHELL_VM_LINUX_BINARY`** / **`DEVSHELL_VM_CONTAINER_IMAGE`** / **`DEVSHELL_VM_STDIO_TRANSPORT`** tune behavior.

## `cargo install` (Windows)

Use **`xtask-todo-lib` 0.1.16+** for `cargo install xtask-todo-lib` on Windows. Older releases (e.g. 0.1.15) could fail to compile: `rustyline` was only listed under Linux target deps, `WorkspaceBackendError` was imported under `cfg(unix)` while used unconditionally, and `vm_workspace_host_root` was Unix-only while still referenced in type-checked branches.

For **beta VM after install**, see **`docs/devshell-vm-windows.md`** (automatic OCI fallback vs host ELF).

## License

MIT OR Apache-2.0
