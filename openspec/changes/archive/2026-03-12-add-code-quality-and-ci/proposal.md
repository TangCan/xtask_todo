# Change: Add code quality tooling and CI

## Why
Per [docs/improvements-and-rust-style.md](../../improvements-and-rust-style.md), the project should adopt Rust coding conventions (rustfmt, Clippy), a root README for onboarding, and CI to prevent regressions. This change implements the high-priority and selected medium-priority improvements without changing product behavior.

## What Changes
- Add **rustfmt.toml** at repo root (edition, max_width) and ensure code passes `cargo fmt -- --check`.
- Resolve or configure **Clippy** so `cargo clippy --all-targets -- -D warnings` passes (fix doc backticks, `# Errors`, `#[must_use]`, inline format args, or documented allow).
- Add **root README** with project purpose, `cargo xtask todo` usage, docs pointer, and how to run tests.
- Add **CI workflow** (e.g. GitHub Actions) that runs `cargo build`, `cargo test`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`; optional `cargo doc --no-deps`.
- Improve **API docs** in `crates/todo`: add `# Errors` for functions returning `Result`; use backticks for types in doc comments.
- Add **`#[must_use]`** where Clippy suggests (e.g. `new()`, `from_todos()`, `as_u64()`).
- **Dependency versions**: pin key deps with minor version in Cargo.toml where missing.

## Impact
- Affected specs: new capability `development-conventions`
- Affected code/docs: root (README, rustfmt.toml, .github/workflows or similar), crates/todo (doc comments, attributes), xtask (format/clippy fixes)
