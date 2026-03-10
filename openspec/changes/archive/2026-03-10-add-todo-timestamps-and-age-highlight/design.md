# Design: Todo timestamps and age highlight

## Context
- Todo already has `created_at`. We need completion time and list display that shows timestamps and highlights "old" open tasks.
- Display is currently in `cargo xtask todo list` (CLI); color only makes sense when stdout is a TTY.

## Goals / Non-Goals
- **Goals**: Record and persist creation and completion time; show them in list; highlight items created > N days ago and still open (e.g. different color when TTY).
- **Non-Goals**: Configurable threshold in config file; GUI; other clients than xtask list.

## Decisions
- **completed_at**: Add `Option<SystemTime>` to `Todo`. Set to `Some(now)` when `complete(id)` is called. Persist as optional field in storage (e.g. `completed_at_secs: Option<u64>`); missing or 0 means not completed.
- **Age threshold**: Default 7 days. "Old" = created more than 7 days ago **and** not completed. Only these are highlighted.
- **Where to highlight**: In `cargo xtask todo list` output only. When stdout is a TTY, use ANSI escape codes (e.g. yellow or red) for the line or the title of old-open items. When not a TTY, skip color so logs/pipes stay plain.
- **Timestamp display**: In list, show creation time (and completion time if completed) in a compact format (e.g. `2025-03-10 10:00` or relative like `3d ago`). Exact format is implementation detail.

## Alternatives considered
- **Threshold in config**: Deferred; 7 days is fixed for now.
- **Color library**: Use simple ANSI codes (e.g. `\x1b[33m`) to avoid a dependency; if we need more, add a small crate later.

## Risks / Trade-offs
- **Backward compatibility**: Existing `.todo.json` without `completed_at_secs` must still load; treat missing as `None`.
- **TTY detection**: `stdout.is_terminal()` or similar; fallback to no color when false.
