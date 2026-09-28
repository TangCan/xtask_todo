# Automation Summary — Story 2.1

## Implemented

- Added `cargo xtask release-baseline` with `--output`, `--runs`, and `--profile`.
- Added unit coverage for stable target ordering, percentile calculation, and JSON schema fields.
- Added a documentation contract and a generated `docs/benchmarks/release-baseline.json` artifact.
- The command captures build metadata and refuses to overwrite an existing output.

## Verification

- `cargo xtask release-baseline --help`
- `cargo xtask release-baseline --runs 0` returns non-zero with a parameter diagnostic.
- A real `--runs 1` measurement discovered all workspace binary targets and parsed as JSON.
- `cargo fmt --all -- --check`
- `cargo test -p xtask release_baseline --lib`
- `cargo clippy -p xtask --all-targets --all-features -- -D warnings`
