# Tasks: Add code quality and CI

## 1. Formatting
- [x] 1.1 Add `rustfmt.toml` at repo root with `edition = "2021"` and `max_width = 100` (or project default).
- [x] 1.2 Run `cargo fmt` and ensure the tree is formatted; verify `cargo fmt -- --check` passes.

## 2. Linting (Clippy)
- [x] 2.1 Fix or allow Clippy warnings so `cargo clippy --all-targets -- -D warnings` passes: doc backticks, `# Errors` for `Result`-returning functions, `#[must_use]` where suggested, inline format args (e.g. `write!(f, "todo not found: {id}")`). Prefer fixing over blanket allow; document any remaining allows.
- [x] 2.2 Optionally add lint config in root `Cargo.toml` or `.cargo/config.toml` if the team agrees on a fixed set. (Documented in root `Cargo.toml`: per-crate `[lints.clippy]` in `crates/todo` and `xtask`; virtual workspace has no root `[lints]`.)

## 3. Documentation
- [x] 3.1 In `crates/todo`: add `# Errors` sections to doc comments for `create`, `complete`, `delete`; use backticks for type names (e.g. `` `TodoId` ``) in doc strings.
- [x] 3.2 Add `#[must_use]` to constructors and pure getters in `crates/todo` and `xtask` where Clippy suggests (e.g. `TodoList::new()`, `InMemoryStore::new()`, `from_todos()`, `as_u64()`).

## 4. Root README
- [x] 4.1 Add `README.md` at repo root with: project purpose (one paragraph), how to run `cargo xtask todo add/list/complete/delete`, pointer to `docs/`, and how to run tests (`cargo test`).

## 5. CI
- [x] 5.1 Add a CI workflow (e.g. `.github/workflows/ci.yml`) that runs on push/PR: `cargo build`, `cargo test`, `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings`. Optional: `cargo doc --no-deps`.

## 6. Dependencies
- [x] 6.1 Pin key dependencies to a specific minor version in `Cargo.toml` (e.g. `serde = "1.0"`, `argh = "0.1"`) if not already; ensure `cargo build` and `cargo test` still pass.

## 7. Validation
- [x] 7.1 Run `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` locally; run `openspec validate add-code-quality-and-ci --strict` and fix any issues.
