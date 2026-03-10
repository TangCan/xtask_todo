# Tasks: Implement Todo domain

## 1. Domain layer (crates/todo)
- [ ] 1.1 Define `TodoId` (e.g. `NonZeroU64` or newtype); implement `Clone`, `Eq`, `Hash`, `Display` as needed.
- [ ] 1.2 Define `Todo` with `id: TodoId`, `title: String`, `completed: bool`, and optional `created_at`; implement needed traits.
- [ ] 1.3 Define `TodoError` with `InvalidInput` and `NotFound(TodoId)`; implement `std::error::Error` and `Display`.
- [ ] 1.4 Implement title validation: reject empty (and optionally trim); expose a validation helper used by `create`.

## 2. Storage layer (crates/todo)
- [ ] 2.1 Define `Store` trait with `insert`, `get`, `list`, `update`, `remove` (or equivalent) in a private or internal module.
- [ ] 2.2 Implement `InMemoryStore` using `HashMap<TodoId, Todo>` (or equivalent), implementing `Store`; `list` returns items ordered by creation.

## 3. Public API (crates/todo)
- [ ] 3.1 Implement `TodoList` holding a `Store` (generic or `dyn Store`); provide constructor (e.g. default with `InMemoryStore`).
- [ ] 3.2 Implement `TodoList::create(title)` using validation, then `store.insert`, return `Result<TodoId, TodoError>`.
- [ ] 3.3 Implement `TodoList::list()` returning `Vec<Todo>` from store, sorted by creation.
- [ ] 3.4 Implement `TodoList::complete(id)`: if present set `completed = true` and update; else return `NotFound`.
- [ ] 3.5 Implement `TodoList::delete(id)`: if present remove; else return error or idempotent Ok per design.

## 4. Tests (crates/todo)
- [ ] 4.1 US-T1 tests: create with valid title → `Ok(id)` and item in list; create with empty title → `Err` and list unchanged.
- [ ] 4.2 US-T2 tests: new list → `list()` empty; after several `create` → `list()` length and order correct.
- [ ] 4.3 US-T3 tests: `complete` existing id → item completed; `complete` non-existent id → `Err(NotFound)`, list unchanged.
- [ ] 4.4 US-T4 tests: `delete` existing id → item gone; `delete` non-existent id → design behavior, list unchanged.

## 5. Xtask and validation
- [ ] 5.1 Ensure `cargo xtask --help` shows usage and subcommands (e.g. `run`); ensure `cargo xtask run` exits 0 on success.
- [ ] 5.2 Run `cargo test -p todo` and confirm all tests pass; run `openspec validate implement-todo-domain --strict` and fix any issues.
