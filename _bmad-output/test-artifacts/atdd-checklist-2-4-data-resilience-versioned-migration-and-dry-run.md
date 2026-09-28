---
story: 2-4-data-resilience-versioned-migration-and-dry-run
stepsCompleted:
  - step-01-init
  - step-02-testability
  - step-03-generate
lastStep: step-03-generate
date: 2026-09-28
generatedTestFiles: []
---

# ATDD Checklist — Story 2.4

## Red-phase scenarios

- missing `.todo.json` reads as empty without creating it.
- malformed JSON and unsupported future version return data error code 3 with path/context and preserve bytes.
- legacy array and current versioned envelope load existing tasks with defaults.
- successful and failed dry-run use normal validation but leave data, backups and migration files unchanged.
- successful writes are recoverable and preserve the established JSON/CLI contract.

## Priority

Data-loss prevention and failure classification are P0; legacy compatibility and dry-run parity are P0; migration ergonomics are P1.
