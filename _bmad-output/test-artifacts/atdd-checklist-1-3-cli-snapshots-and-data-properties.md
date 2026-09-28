---
story: 1-3-cli-snapshots-and-data-properties
stepsCompleted:
  - step-01-init
  - step-02-testability
  - step-03-generate
lastStep: step-03-generate
date: 2026-09-28
generatedTestFiles:
  - xtask/tests/trycmd_list_snapshots.rs
  - xtask/tests/trycmd/list_filters.trycmd
  - crates/todo/src/tests/property.rs
---

# ATDD Checklist — Story 1.3

## Testability assessment

This is a backend Rust testing story with two observable surfaces: the real
`todo` binary's JSON snapshots and the data-layer list invariants. Fixtures use
fixed timestamps and a static `.todo.json` so output is deterministic under
parallel test execution.

## Red-phase scenarios

- Each list filter and sort dimension returns the expected matching titles.
- Each dimension has a no-match snapshot with the established empty payload.
- Randomly generated lists preserve filter membership and never duplicate ids.
- Representative title sorting remains stable under insta.
- Invalid list arguments return code 2 without changing an existing store.

## Priority

CLI snapshot and data invariant scenarios are P0/P1. No API or browser tests
apply.
