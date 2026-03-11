## MODIFIED Requirements

### Requirement: Complete todo
The system SHALL allow marking a todo as completed by its `TodoId`. If the id exists, the todo SHALL appear as completed in subsequent queries and the system SHALL record the completion time (e.g. `completed_at`). If the id does not exist, the system SHALL return `TodoError::NotFound` and SHALL not change any other todo.

#### Scenario: Existing id is marked completed
- **GIVEN** a `TodoList` with at least one todo
- **WHEN** the caller invokes `complete(id)` for that todo's id
- **THEN** the call returns `Ok(())`, that item has `completed == true` in `list()` or `get(id)`, and that item has a recorded completion time (e.g. `completed_at` is set)

#### Scenario: Non-existent id returns NotFound
- **GIVEN** a `TodoList` with at least one todo
- **WHEN** the caller invokes `complete(id)` for a non-existent `TodoId`
- **THEN** the call returns `Err(TodoError::NotFound)` and existing todos are unchanged

## ADDED Requirements

### Requirement: Todo timestamps
The system SHALL record and expose creation time for each todo (e.g. `created_at`). When a todo is marked completed, the system SHALL record completion time (e.g. `completed_at`). Persistence (when used) SHALL store both so they can be displayed and used for age-based logic.

#### Scenario: Creation time is recorded and exposed
- **GIVEN** a todo created via `create`
- **WHEN** the caller inspects the todo (e.g. via `list()` or `get(id)`)
- **THEN** the todo has a creation timestamp (e.g. `created_at`) that reflects when it was created

#### Scenario: Completion time is recorded when completed
- **GIVEN** a todo that is then marked completed via `complete(id)`
- **WHEN** the caller inspects the todo after completion
- **THEN** the todo has a completion timestamp (e.g. `completed_at`) that reflects when it was completed

### Requirement: Age-based visual highlight
When listing todos in a context that supports visual distinction (e.g. CLI with a TTY), items that exceed an age threshold (e.g. created more than 7 days ago) and are still not completed SHALL be displayed in a visually distinct way (e.g. different color) to draw attention. When output is not to an interactive terminal (e.g. piped or in CI), the system SHALL NOT emit color or highlight codes so that output remains plain text.

#### Scenario: Old open items are highlighted in TTY
- **GIVEN** a list output (e.g. `cargo xtask todo list`) where stdout is a TTY and at least one todo was created more than the threshold (e.g. 7 days) ago and is not completed
- **WHEN** the list is rendered
- **THEN** that todo (or its line) is shown in a distinct style (e.g. color) compared to other items

#### Scenario: No color when not a TTY
- **GIVEN** a list output where stdout is not a TTY (e.g. piped to a file or another command)
- **WHEN** the list is rendered
- **THEN** no color or escape codes are emitted so that the output is plain text
