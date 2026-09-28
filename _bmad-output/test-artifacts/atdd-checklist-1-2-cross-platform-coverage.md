---
story: 1-2-cross-platform-coverage
stepsCompleted:
  - step-01-init
  - step-02-testability
  - step-03-generate
lastStep: step-03-generate
date: 2026-09-28
generatedTestFiles:
  - xtask/src/coverage.rs
---

# ATDD Checklist — Story 1.2

## Testability assessment

This is a backend Rust tooling story. The observable contract is the command
invocation, summary parsing, installation guidance, and error propagation.
The tests should inject a fake `CARGO` executable so they remain deterministic
and do not require cargo-llvm-cov during unit-test execution.

## Red-phase scenarios

- Parse an llvm-cov summary line into a percentage.
- Pass the expected `llvm-cov --ignore-filename-regex` arguments to the fake
  cargo executable for each package.
- Return an actionable error when the fake cargo process cannot start or exits
  unsuccessfully.
- Preserve the fake success and fake failure environment hooks.

## Priority

All scenarios are P1 quality-tooling coverage. No API or browser tests apply.
