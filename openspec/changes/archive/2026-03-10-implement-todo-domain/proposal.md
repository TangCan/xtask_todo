# Change: Implement Todo domain and satisfy acceptance

## Why
The project has requirements, design, tasks, test cases, and acceptance criteria documented, but `crates/todo` is still a stub and tests are missing. This change implements the full Todo domain (create, list, complete, delete) with domain types, storage abstraction, public API, and automated tests so that all acceptance criteria in `docs/acceptance.md` for the Todo scope and xtask workflow can be verified.

## What Changes
- Implement `crates/todo`: domain types (`TodoId`, `Todo`, `TodoError`), title validation, `Store` trait, `InMemoryStore`, `TodoList` with `create` / `list` / `complete` / `delete`.
- Add unit and/or integration tests in `crates/todo` covering US-T1–US-T4 (per `docs/test-cases.md`).
- Ensure `cargo xtask run` and `cargo xtask --help` behave per US-X1/US-X2 (already scaffolded; verify exit codes and help output).
- Introduce OpenSpec capability `todo` so the implemented behavior is specified and traceable.

## Impact
- Affected specs: new capability `todo`
- Affected code: `crates/todo/src/` (new modules and tests), optionally `xtask/src/main.rs` if run is wired to a demo or kept as placeholder)
