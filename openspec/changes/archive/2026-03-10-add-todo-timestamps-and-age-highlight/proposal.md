# Change: Add todo timestamps and age-based highlight

## Why
Users need to track when tasks were added and when they were completed. Tasks that have been open for a long time (e.g. over a week) should be visually highlighted in the list so they are not forgotten.

## What Changes
- Add **completion timestamp** (`completed_at`) to the todo model; creation time (`created_at`) already exists. Persist both in storage (e.g. `.todo.json`).
- In **list output** (e.g. `cargo xtask todo list`): show creation and completion times; when output is to a TTY, render items that exceed an age threshold (e.g. created more than 7 days ago and still not completed) in a distinct color (e.g. yellow/red) to draw attention.
- Define the threshold (default 7 days) and that color is only applied when the list is shown in an interactive terminal (not when piped or in CI).

## Impact
- Affected specs: capability `todo` (MODIFIED + ADDED requirements)
- Affected code: `crates/todo` (Todo type + `complete()` set `completed_at`; store/JSON); `xtask` (TodoDto, list formatting, ANSI color for old open items)
