## ADDED Requirements

### Requirement: Show single todo
The system SHALL allow retrieving a single todo by id (e.g. `get(id)` or equivalent). When the id exists, the system SHALL return the full todo (id, title, created_at, completed_at, state, and any optional fields). When the id does not exist, the system SHALL return an error or None and SHALL NOT modify state. The CLI SHALL provide a subcommand (e.g. `todo show <id>`) that outputs the task or an explicit error and SHALL use a non-zero exit code when the id is missing.

#### Scenario: Valid id returns full todo
- **GIVEN** a TodoList containing at least one todo
- **WHEN** the caller invokes get(id) for an existing id
- **THEN** the result is that todo with all stored fields (id, title, timestamps, optional description, due_date, priority, tags, repeat_rule)

#### Scenario: Non-existent id returns error
- **GIVEN** a TodoList
- **WHEN** the caller invokes get(id) for a non-existent id
- **THEN** the call returns an error or None and the CLI exits with a non-zero exit code

### Requirement: Update todo
The system SHALL allow updating an existing todo by id (e.g. `update(id, patch)`). Updates SHALL support at least title and SHALL support optional fields (description, due_date, priority, tags, repeat_rule) when implemented. When the id exists and the patch is valid, the system SHALL persist the updated todo and SHALL reflect it in list and get. When the id does not exist or the patch is invalid, the system SHALL return an error and SHALL NOT change other todos. The CLI SHALL provide a subcommand (e.g. `todo update <id>`) that applies the given options.

#### Scenario: Valid update persists
- **GIVEN** a TodoList with at least one todo
- **WHEN** the caller invokes update(id, patch) with valid fields
- **THEN** the todo is updated in storage and a subsequent get(id) or list() shows the updated values

#### Scenario: Non-existent id or invalid patch returns error
- **GIVEN** a TodoList
- **WHEN** the caller invokes update(id, patch) for a non-existent id or with invalid data
- **THEN** the call returns an error and no other todos are modified

### Requirement: Optional task attributes
The system SHALL support optional attributes on a todo: description, due date, priority (e.g. high/medium/low), and tags (list of strings). Create and update APIs SHALL accept these optionally. List and get SHALL expose them when present. The system SHALL support filtering and sorting the list by status, priority, tags, and due date (scope MAY be phased).

#### Scenario: Create and list with optional attributes
- **GIVEN** create (or update) is called with optional description, due_date, priority, tags
- **WHEN** the caller invokes list() or get(id)
- **THEN** the returned todo(s) include those attributes where supplied

#### Scenario: List supports filter and sort
- **GIVEN** a TodoList with todos that have priority and due date
- **WHEN** the caller invokes list() with filter or sort options (e.g. by status, priority, due date)
- **THEN** the result is filtered and/or ordered according to the options

### Requirement: Search todos
The system SHALL provide search by keyword (e.g. `search(keyword)`). Search SHALL match at least against todo titles and SHALL include description and tags when those fields exist. The system SHALL return a list of matching todos; when there are no matches, the system SHALL return an empty list. The CLI SHALL provide a subcommand (e.g. `todo search <keyword>`).

#### Scenario: Keyword matches return list
- **GIVEN** todos with titles (and optionally descriptions/tags) containing a keyword
- **WHEN** the caller invokes search(keyword)
- **THEN** the result is a list of todos that match the keyword in title (or description/tags)

#### Scenario: No match returns empty list
- **GIVEN** a TodoList
- **WHEN** the caller invokes search(keyword) and no todo matches
- **THEN** the result is an empty list

### Requirement: Todo statistics
The system SHALL provide statistics (e.g. `stats()`) that include at least: total number of todos, number of incomplete todos, and number of completed todos. The CLI SHALL provide a subcommand (e.g. `todo stats`) that outputs these values. Additional metrics (e.g. overdue count, by priority) MAY be included.

#### Scenario: Stats reflect current list
- **GIVEN** a TodoList with some todos, some completed and some not
- **WHEN** the caller invokes stats()
- **THEN** the result includes total count, incomplete count, and completed count consistent with list()

### Requirement: Export and import todos
The system SHALL support exporting the current list of todos to a file (e.g. JSON or CSV by path or option) and SHALL support importing from a file into the store. Export SHALL write the current list in the chosen format. Import SHALL merge or replace according to a defined policy (e.g. merge by id or replace all) and SHALL persist the result. The CLI SHALL provide subcommands (e.g. `todo export <file>`, `todo import <file>`).

#### Scenario: Export writes current list to file
- **GIVEN** a TodoList with one or more todos
- **WHEN** the caller invokes export(path, format)
- **THEN** the file at path contains the serialized list in the specified format

#### Scenario: Import updates store from file
- **GIVEN** a file containing serialized todos
- **WHEN** the caller invokes import(path)
- **THEN** the store is updated according to the import policy and list() reflects the imported data

### Requirement: Recurring tasks
The system SHALL support an optional repeat rule on a todo (e.g. daily, weekly, monthly, yearly, weekdays, or custom interval). When a todo with a repeat rule is marked completed, the system SHALL create the next instance according to the rule and SHALL set the new instance's due date when applicable. The system SHALL support an option (e.g. no_next) to complete the current instance without creating the next. Task detail (show) and update SHALL expose and allow editing or clearing the repeat rule. The CLI SHALL support complete with --no-next and SHALL show repeat_rule in show and update.

#### Scenario: Complete with repeat creates next instance
- **GIVEN** a todo with a repeat rule (e.g. daily) and optionally a due date
- **WHEN** the caller invokes complete(id) without no_next
- **THEN** the todo is marked completed and a new todo is created with the next due date per the rule

#### Scenario: Complete with no_next does not create next
- **GIVEN** a todo with a repeat rule
- **WHEN** the caller invokes complete(id, no_next: true) or CLI with --no-next
- **THEN** the todo is marked completed and no new instance is created

### Requirement: Structured JSON output
The CLI SHALL support an optional flag (e.g. --json) on todo subcommands. When --json is set, the output SHALL be valid JSON. On success, the output SHALL include a status field (e.g. "success") and a data field with the result. On failure, the output SHALL include a status field (e.g. "error") and an error object with at least a code and message. When --json is not set, the CLI SHALL retain the existing human-readable output behavior.

#### Scenario: Success response structure
- **GIVEN** a todo subcommand that succeeds (e.g. list, add, show)
- **WHEN** the subcommand is run with --json
- **THEN** stdout is valid JSON with status and data; no extra non-JSON text

#### Scenario: Error response structure
- **GIVEN** a todo subcommand that fails (e.g. invalid id, missing arg)
- **WHEN** the subcommand is run with --json
- **THEN** stdout (or stderr per design) is valid JSON with status and error (code, message)

### Requirement: Standard exit codes
The CLI SHALL use the following exit codes: 0 for success; 1 for general error; 2 for parameter error (e.g. missing or invalid arguments); 3 for data error (e.g. todo id not found, invalid data operation). The implementation SHALL map library and CLI errors to these codes consistently.

#### Scenario: Success exits 0
- **GIVEN** a todo subcommand that completes successfully
- **WHEN** the process exits
- **THEN** the exit code is 0

#### Scenario: Parameter error exits 2
- **GIVEN** a todo subcommand invoked with missing or invalid arguments
- **WHEN** the process exits with failure
- **THEN** the exit code is 2

#### Scenario: Data error exits 3
- **GIVEN** a todo subcommand that fails due to data (e.g. id not found)
- **WHEN** the process exits with failure
- **THEN** the exit code is 3

### Requirement: AI skill generation
The system SHALL provide a command (e.g. `todo init-ai` or `cargo xtask todo init-ai`) that generates skill or instruction files for target AI assistants (e.g. Cursor, Claude Code). The command SHALL accept an option to specify the target (e.g. --for cursor) and an optional output directory (e.g. --output dir). Generated files SHALL be placed in the assistant's expected directory (e.g. .cursor/commands/) when applicable. The content SHALL include metadata (name, description, trigger) and natural-language instructions so the AI can parse user intent and construct commands, including use of --json where appropriate.

#### Scenario: init-ai creates files for target
- **GIVEN** the user runs todo init-ai --for cursor (and optionally --output dir)
- **WHEN** the command completes successfully
- **THEN** the expected directory contains skill/instruction files with metadata and usage instructions

### Requirement: Dry-run for mutating commands
The CLI SHALL support an optional flag (e.g. --dry-run) for mutating todo subcommands (add, update, complete, delete). When --dry-run is set, the CLI SHALL output a description of the operation that would be performed and SHALL NOT persist any change (e.g. SHALL NOT write to .todo.json or update in-memory store for persistence). Other subcommands (e.g. list, show, search, stats) MAY ignore or no-op --dry-run.

#### Scenario: Dry-run does not persist
- **GIVEN** the user runs a mutating command (e.g. todo add "x" --dry-run or todo complete 1 --dry-run)
- **WHEN** the command completes
- **THEN** the intended operation is shown but .todo.json and stored state are unchanged
