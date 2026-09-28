---
title: '跨平台覆盖率能力'
type: 'feature'
created: '2026-09-28'
status: 'done'
route: 'dispatch'
baseline_commit: '84595f4'
context:
  - '/nvme_data2/richard/xtask_todo/_agile-output/implementation-artifacts/epic-1-context.md'
  - '/nvme_data2/richard/xtask_todo/_agile-output/project-context.md'
  - '/nvme_data2/richard/xtask_todo/_agile-output/specs/spec-xtask-todo-hardening/quality-tooling.md'
---

<frozen-after-approval reason="story intent derived from approved hardening Epic 1">

## Intent

Replace the Linux-oriented tarpaulin coverage runner with cargo-llvm-cov so
coverage summaries use source-based instrumentation and remain portable across
the project's supported host platforms.

## Boundaries & Constraints

- Preserve coverage summary shape, package names, error propagation, fake/fail
  test hooks, and the existing exclusion intent.
- Do not add coverage thresholds or CI gates; those belong to Story 1.4.
- Do not change runtime behavior, CLI JSON contracts, or data format.

</frozen-after-approval>

## Acceptance Criteria

- `cargo xtask coverage` invokes `cargo llvm-cov`, not cargo-tarpaulin.
- Existing exclusions are represented by `--ignore-filename-regex` patterns.
- Missing coverage tooling produces an actionable installation error.
- Unit tests cover llvm-cov parsing, success, failure, and process-start errors.

## Implementation Notes

- Replaced tarpaulin invocation with `cargo llvm-cov --json --summary-only`.
- Preserved the existing exclusion inventory and converted file globs to a
  regex that matches both `/` and `\\` path separators.
- Coverage summaries are parsed strictly from llvm-cov JSON; malformed output,
  missing tools, process-start failures, and non-zero exits cannot report a
  successful percentage.
- Added command-argument, JSON parser, Windows separator, fake process, and
  command-level failure tests.

## Review Findings

- Fixed portable path matching for Windows separators.
- Removed permissive percentage fallback that could accept unrelated output.
- Added complete ordered argv assertions and child-process failure coverage.
- Documented the remaining Unix-only shell fake as a test harness detail; the
  command builder, JSON parser, and path conversion are covered cross-platform.

## Review Findings
