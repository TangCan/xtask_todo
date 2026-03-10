# todo Specification

## Purpose
Todo domain in `crates/todo`: create, list, complete, and delete todo items with a public API (`TodoList`), domain types (`TodoId`, `Todo`, `TodoError`), and pluggable storage (`Store` / `InMemoryStore`).
## Requirements
### Requirement: Create todo
The system SHALL allow creating a todo item with a non-empty title and SHALL return a unique identifier (e.g. `TodoId`). The system SHALL reject empty or invalid title and SHALL return a clear error (e.g. `TodoError::InvalidInput`) without creating any todo.

#### Scenario: Valid title creates todo and returns id
- **GIVEN** a `TodoList` with in-memory storage
- **WHEN** the caller invokes `create` with a non-empty title
- **THEN** the call returns `Ok(TodoId)` and a subsequent `list()` includes one item with that title and id

#### Scenario: Empty or invalid title returns error
- **GIVEN** a `TodoList` with in-memory storage
- **WHEN** the caller invokes `create` with an empty or invalid title (per product rules)
- **THEN** the call returns `Err(TodoError::InvalidInput)` and `list()` length is unchanged

### Requirement: List todos
The system SHALL return the list of all todo items. When there are no items, the system SHALL return an empty list. When there are items, the system SHALL return them ordered by creation time (or documented order).

#### Scenario: Empty list when no todos
- **GIVEN** a newly created `TodoList`
- **WHEN** the caller invokes `list()`
- **THEN** the result is an empty list

#### Scenario: List returns all todos in creation order
- **GIVEN** a `TodoList` with one or more todos created via `create`
- **WHEN** the caller invokes `list()`
- **THEN** the result contains all todos in creation order (e.g. by `created_at`)

### Requirement: Complete todo
The system SHALL allow marking a todo as completed by its `TodoId`. If the id exists, the todo SHALL appear as completed in subsequent queries. If the id does not exist, the system SHALL return `TodoError::NotFound` and SHALL not change any other todo.

#### Scenario: Existing id is marked completed
- **GIVEN** a `TodoList` with at least one todo
- **WHEN** the caller invokes `complete(id)` for that todo's id
- **THEN** the call returns `Ok(())` and that item has `completed == true` in `list()` or `get(id)`

#### Scenario: Non-existent id returns NotFound
- **GIVEN** a `TodoList` with at least one todo
- **WHEN** the caller invokes `complete(id)` for a non-existent `TodoId`
- **THEN** the call returns `Err(TodoError::NotFound)` and existing todos are unchanged

### Requirement: Delete todo
The system SHALL allow deleting a todo by its `TodoId`. If the id exists, the todo SHALL be removed and SHALL not appear in subsequent `list()` or `get(id)`. If the id does not exist, the system SHALL return an error or SHALL succeed idempotently (per design); in either case existing data SHALL remain consistent.

#### Scenario: Existing id is removed
- **GIVEN** a `TodoList` with at least one todo
- **WHEN** the caller invokes `delete(id)` for that todo's id
- **THEN** the call succeeds and that item is no longer in `list()` or `get(id)`

#### Scenario: Non-existent id does not corrupt state
- **GIVEN** a `TodoList` with at least one todo
- **WHEN** the caller invokes `delete(id)` for a non-existent `TodoId`
- **THEN** the call returns error or idempotent Ok per design, and `list()` length and content are unchanged

