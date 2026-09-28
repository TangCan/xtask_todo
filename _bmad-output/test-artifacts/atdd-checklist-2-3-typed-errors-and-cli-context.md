---
story: 2-3-typed-errors-and-cli-context
stepsCompleted:
  - step-01-init
  - step-02-testability
  - step-03-generate
lastStep: step-03-generate
date: 2026-09-28
generatedTestFiles: []
---

# ATDD Checklist — Story 2.3

## Red-phase scenarios

- library errors classify invalid input, not found/business, data/parse, storage and external failures.
- displayed errors include operation/path context while preserving a source error.
- parameter failures return code 2; data/business failures code 3; general failures code 1.
- `--json` failures emit exactly one valid error envelope without human text on stdout.
- successful JSON and existing CLI snapshots remain unchanged.

## Priority

Exit-code and JSON protocol stability are P0; source/context preservation is P0; exhaustive error taxonomy is P1.
