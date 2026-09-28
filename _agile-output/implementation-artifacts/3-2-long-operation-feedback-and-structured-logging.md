# Story 3.2: 长操作反馈与结构化日志

Status: done

## Acceptance criteria

- Import/export progress is count-based, rate-limited, and stderr-only on a TTY.
- JSON stdout remains valid and free of progress or diagnostics.
- `XTASK_LOG=info|debug` enables JSON events with time, level, operation, and context.
- Failures emit error context without changing existing exit-code contracts.

## Implementation

- Added `todo::observability` with stderr-only progress and opt-in JSON logging.
- Instrumented import/export start, completion, progress, and failure paths.
- Documented the environment variable and output boundary in `docs/observability.md`.

## Verification

- `cargo fmt --all`
- `cargo test` via repository pre-commit checks
- Existing integration tests continue to validate JSON stdout contracts.
