# Tasks: Implement requirements extensions

## 1. Domain: single task and update
- [x] 1.1 Add `get(id)` (or equivalent) to Store and TodoList; return `Option<Todo>` or error for non-existent id (US-T7).
- [x] 1.2 Add `update(id, patch)` to TodoList; support at least title and extend to optional fields when present (US-T8).

## 2. Domain: optional attributes and filtering
- [x] 2.1 Extend `Todo` with optional `description`, `due_date`, `priority`, `tags`; extend create/update API and serialization (US-T9).
- [x] 2.2 Extend list (or Store) to support filter/sort by status, priority, tags, due date; expose via API (US-T9).

## 3. Domain: search and stats
- [x] 3.1 Implement `search(keyword)` in TodoList (title; optionally description/tags when present) (US-T10).
- [x] 3.2 Implement `stats()` returning at least total, incomplete, complete counts (US-T11).

## 4. Domain: export and import
- [x] 4.1 Implement export (e.g. list → JSON/CSV to file) (US-T12).
- [x] 4.2 Implement import (read file, merge or replace into store per policy) (US-T12).

## 5. Domain: recurring tasks
- [x] 5.1 Add `RepeatRule` type and optional field on Todo; support daily, weekly, monthly, yearly, weekdays, custom interval (US-T13).
- [x] 5.2 On complete(id), if task has repeat_rule and no_next is false, create next instance and set due_date from rule; support no_next parameter (US-T13).

## 6. CLI: new subcommands
- [x] 6.1 Add todo subcommands: show \<id\>, update \<id\>, search \<keyword\>, stats, export \<file\>, import \<file\> (US-T7–T12).
- [x] 6.2 Add complete --no-next and show/update display and edit of repeat_rule (US-T13).

## 7. CLI: AI/CLI integration
- [ ] 7.1 Add global --json option to todo subcommands; output unified JSON structure (US-A1).
- [ ] 7.2 Use exit codes 0/1/2/3 for success, general, parameter, data errors (US-A2).
- [ ] 7.3 Add todo init-ai [--for tool] [--output dir]; generate skill files (US-A3).
- [ ] 7.4 Add --dry-run for add/update/complete/delete; no persistence when set (US-A4).

## 8. Validation and docs
- [x] 8.1 Add or extend unit and integration tests for new behavior; ensure pre-commit and CI pass.
- [ ] 8.2 Update docs/design.md, docs/tasks.md, docs/test-cases.md, docs/acceptance.md to reflect implemented features.
