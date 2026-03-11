# Test coverage

Coverage is measured with [cargo-tarpaulin](https://github.com/xd009642/tarpaulin).

## Per-crate coverage (target: ≥95%)

| Crate   | Coverage | How to run |
|--------|----------|------------|
| **todo** | 100%   | `cargo tarpaulin -p todo` |
| **xtask** (lib) | **96.95%** (127/131) | `cargo tarpaulin -p xtask --exclude-files "xtask/src/main.rs" -- --test-threads=1` |

- **todo**: All library and store code is covered by unit tests.
- **xtask**: Library logic is covered; `main.rs` is a 4-line entry that calls `xtask::run()` and is excluded from the coverage denominator so the reported rate reflects the testable library code. Integration test `xtask_run_exits_success` runs the binary to verify the entry point.

## Running coverage

```bash
# Install tarpaulin (once)
cargo install cargo-tarpaulin

# Per-crate summary (recommended)
cargo xtask coverage

# Todo crate only
cargo tarpaulin -p todo

# Xtask crate (lib only, single-threaded tests for stable cwd)
cargo tarpaulin -p xtask --exclude-files "xtask/src/main.rs" -- --test-threads=1

# Whole workspace (includes both crates; xtask tests use --test-threads=1)
cargo tarpaulin --exclude-files "xtask/src/main.rs" -- --test-threads=1
```

## Notes

- Xtask tests that change the process current directory use `--test-threads=1` to avoid races.
- `run()` in xtask uses `argh::from_env()` and is only exercised when the xtask binary is run (e.g. by the integration test); it is not covered by unit tests.
