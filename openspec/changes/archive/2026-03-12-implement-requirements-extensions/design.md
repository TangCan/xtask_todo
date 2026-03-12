# Design: Implement requirements extensions

## Context
- Current todo: create, list, complete, delete; timestamps and age-based highlight. Data in `.todo.json` via xtask; `Todo` has id, title, completed, created_at, completed_at.
- Requirements doc adds: show/update, optional attributes, search/stats, export/import, recurring tasks, and CLI/AI (--json, exit codes, init-ai, --dry-run).

## Goals / Non-Goals
- Goals: Implement US-T7..US-T13 and US-A1..US-A4 in line with `docs/requirements.md` and `docs/design.md`; keep backward compatibility for existing API and CLI.
- Non-Goals: HTTP API, multi-user, reminder/calendar integration, skill marketplace.

## Decisions
- **Optional fields**: Add to `Todo` and DTOs: `description: Option<String>`, `due_date: Option<...>` (e.g. `SystemTime` or date type), `priority: Option<Priority>` (enum High/Medium/Low), `tags: Vec<String>`, `repeat_rule: Option<RepeatRule>`. Defaults: None / empty; add/update accept optional args.
- **RepeatRule**: In-memory struct (e.g. type: daily|weekly|monthly|yearly|weekdays|custom, interval, until/remaining). On `complete(id, no_next: bool)`, if task has repeat_rule and no_next is false, create next instance and compute due_date from rule; persist in same store.
- **Store trait**: Extend with `get(id)` if not present; `update(todo)` or equivalent for partial update. InMemoryStore and .todo.json serialization extended for new fields.
- **CLI --json**: All todo subcommands accept optional `--json`; output `{ "status": "success", "data": ... }` or `{ "status": "error", "error": { "code": n, "message": "..." } }`. Without --json, keep current human-readable output.
- **Exit codes**: 0 success; 1 general; 2 parameter (missing/invalid args); 3 data (e.g. id not found). Propagate from library errors where possible.
- **init-ai**: Subcommand writes markdown (or target-format) skill files under `.cursor/commands/` (or --output); content describes commands and --json usage so AI can invoke todo correctly.
- **--dry-run**: For add/update/complete/delete, when --dry-run is set, log or print intended operation and do not call store write or save .todo.json.

## Risks / Trade-offs
- Data model change: existing .todo.json without new fields SHALL be read with defaults (None/empty) to avoid migration burden.
- Scope creep: Implement in order of tasks.md; P2 items (e.g. stats details, --no-next) can follow P0/P1.

## Migration Plan
- No schema version bump required; new fields optional. Old .todo.json files load with new code; new fields omitted when saving if not set, or always written as null/[] for clarity.

## Open Questions
- Date type for due_date: `SystemTime` vs. date-only (e.g. NaiveDate) vs. string (YYYY-MM-DD) in API—align with docs/design.md and serialization.
