# Test coverage

Coverage is measured with [cargo-tarpaulin](https://github.com/xd009642/tarpaulin).

## Per-crate coverage (target: ≥95%)

| Crate   | Coverage | How to run |
|--------|----------|------------|
| **xtask-todo-lib** | **≥95%** | `cargo xtask coverage` (uses the same `--exclude-files` as below) |
| **xtask** | **≥95%** | `cargo xtask coverage` or `cargo tarpaulin -p xtask --exclude-files "xtask/src/main.rs" --exclude-files "crates/todo/*" -- --test-threads=1 --include-ignored` |

- **xtask-todo-lib**: `cargo xtask coverage` runs tarpaulin with excludes so the reported rate targets **testable** lib code: the `cargo-devshell` binary, `repl.rs` / `mod.rs` entrypoints, script `exec`/`parse`, VM/Lima (`devshell/vm/*`), host-only sandbox helpers (`linux_mount`, `elf`, `paths`, `run`, `sync`), `host_text`, `command/types`, and `completion.rs` (line-editor branches). Core todo/model/VFS/parser/sandbox export and devshell integration tests cover the rest; see `xtask/src/coverage.rs` for the exact list.
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
