# Change: Implement requirements document extensions

## Why
The project requirements document (`docs/requirements.md`) defines extension user stories (US-T7 through US-T13, US-A1 through US-A4) that are not yet implemented. These cover: single-task view and update, optional task attributes (description, due date, priority, tags), search and stats, import/export, recurring tasks, and AI/CLI integration (JSON output, standard exit codes, init-ai skill generation, dry-run). Implementing them delivers the full scope agreed in the requirements doc.

## What Changes
- **Todo domain (crates/todo)**: Add `get(id)` (or equivalent) for single-task view; add `update(id, patch)`; extend `Todo` with optional `description`, `due_date`, `priority`, `tags`, and `repeat_rule`; add `search(keyword)`, `stats()`, export/import (serialization + file I/O); add `RepeatRule` and logic to create next instance on complete (with optional `no_next`).
- **Todo CLI (xtask todo)**: New subcommands `show <id>`, `update <id>`, `search <keyword>`, `stats`, `export <file>`, `import <file>`, and `init-ai [--for tool] [--output dir]`; `complete` gains `--no-next`. Global options: `--json` for all subcommands (unified JSON output), `--dry-run` for mutating commands (add/update/complete/delete). Exit codes: 0 success, 1 general error, 2 parameter error, 3 data error.
- **Docs/tests**: Update `docs/design.md`, `docs/tasks.md`, `docs/test-cases.md`, `docs/acceptance.md` as implementation progresses; add tests for new behavior per acceptance criteria.

## Impact
- Affected specs: `todo`
- Affected code: `crates/todo/src/` (lib, store), `xtask/src/todo.rs`, `xtask/src/main.rs` (subcommand dispatch), `docs/`
