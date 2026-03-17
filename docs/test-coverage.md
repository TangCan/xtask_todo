# Test coverage

Coverage is measured with [cargo-tarpaulin](https://github.com/xd009642/tarpaulin).

## Per-crate coverage (target: ≥95%)

| Crate   | Coverage | How to run |
|--------|----------|------------|
| **xtask-todo-lib** | — | `cargo xtask coverage` or `cargo tarpaulin -p xtask-todo-lib` |
| **xtask** | **≥95%** | `cargo xtask coverage` or `cargo tarpaulin -p xtask --exclude-files "xtask/src/main.rs" --exclude-files "crates/todo/*" -- --test-threads=1 --include-ignored` |

- **xtask-todo-lib**: Includes the library and the `cargo-devshell` binary. Devshell REPL/VFS/command logic lives in the lib (`src/devshell/`) so unit and integration tests can cover it; the binary is a thin wrapper. Coverage is the ratio of covered lines to all package source lines (currently ~90.5%). **Refactor for testability**: REPL loop body is in `repl::process_line()` so it can be unit-tested without a TTY; `run_main_from_args()` accepts args and streams so all “main” logic is testable. Remaining uncovered: (1) TTY branch in `repl.rs` (rustyline `Editor::readline`), (2) binary `main.rs` and `run_main()` when run from the process, (3) some branches in command/completion/list/vfs. Reaching ≥95% would require PTY-based tests for the TTY repl or excluding the binary/TTY-only path.
- **xtask**: Library logic is covered; `main.rs` is a 4-line entry that calls `xtask::run()` and is excluded from the coverage denominator so the reported rate reflects the testable library code. Integration test `xtask_run_exits_success` runs the binary to verify the entry point.

## Running coverage

```bash
# Install tarpaulin (once)
cargo install cargo-tarpaulin

# Per-crate summary (recommended)
cargo xtask coverage

# Todo package (lib + cargo-devshell binary)
cargo tarpaulin -p xtask-todo-lib

# Xtask crate (lib only, single-threaded tests for stable cwd)
cargo tarpaulin -p xtask --exclude-files "xtask/src/main.rs" -- --test-threads=1

# Whole workspace (includes both crates; xtask tests use --test-threads=1)
cargo tarpaulin --exclude-files "xtask/src/main.rs" -- --test-threads=1
```

## Notes

- Xtask tests that change the process current directory use `--test-threads=1` to avoid races.
- `run()` in xtask uses `argh::from_env()` and is only exercised when the xtask binary is run (e.g. by the integration test); it is not covered by unit tests.
