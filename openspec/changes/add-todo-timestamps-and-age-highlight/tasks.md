# Tasks: Add todo timestamps and age highlight

## 1. Domain and storage (crates/todo)
- [ ] 1.1 Add `completed_at: Option<SystemTime>` to `Todo`; keep `created_at` as is. Set `completed_at = Some(now)` in `TodoList::complete()` when marking completed.
- [ ] 1.2 Persist and load `completed_at` in any persistent store (e.g. extend DTO in xtask with `completed_at_secs: Option<u64>`); treat missing or zero as not completed for backward compatibility.

## 2. List display (xtask)
- [ ] 2.1 In `cargo xtask todo list`, show creation time (and completion time when completed) for each item in a compact format (e.g. date or relative).
- [ ] 2.2 When stdout is a TTY, detect items that are "old open" (created more than 7 days ago and not completed) and render them with a distinct color (e.g. ANSI yellow/red); when not a TTY, do not emit color codes.
- [ ] 2.3 Ensure existing `.todo.json` without `completed_at_secs` still loads (optional field, default None).

## 3. Validation
- [ ] 3.1 Add or extend a unit test that a completed todo has `completed_at` set; list output and color are acceptable to verify manually.
- [ ] 3.2 Run `openspec validate add-todo-timestamps-and-age-highlight --strict` and fix any issues.
