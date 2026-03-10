# Project Context

## Purpose
- **xtask_todo** provides reusable Todo (待办) capability as a library and optional entrypoints, with development and build automation via `cargo xtask`.
- Scope: (1) Todo domain in `crates/todo` (create, list, complete, delete); (2) xtask workflow for developers (run, extend subcommands). See `docs/requirements.md` for user stories and acceptance criteria.

## Tech Stack
- **Language**: Rust, edition 2021
- **Workspace**: Cargo workspace, resolver `"2"`, members `crates/todo` and `xtask`
- **CLI (xtask)**: `argh` for subcommand and argument parsing
- **Entrypoint**: `cargo xtask` via `.cargo/config.toml` alias (`xtask = "run -p xtask --"`), no global install required

## Project Conventions

### Code Style
- Rust standard formatting: `cargo fmt`; follow default style (4 spaces, etc.).
- Naming: `TodoId`, `Todo`, `TodoList`, `TodoError`; crate names `todo` (library), `xtask` (binary).
- Layout: no `src/` at repo root; each crate has its own `src/` (e.g. `crates/todo/src/`, `xtask/src/`).

### Architecture Patterns
- **crates/todo**: Three layers — Public API (e.g. `TodoList` with `create`/`list`/`complete`/`delete`), Domain (types and validation), Storage (trait + `InMemoryStore`). See `docs/design.md`.
- **xtask**: Orchestration only; no domain logic. Subcommands implemented in `xtask/src/main.rs` with `argh`; new subcommands added there without changing Cargo config.
- Storage is abstracted behind a `Store` trait so implementations can be swapped (e.g. file/DB later) without changing the public API.

### Testing Strategy
- **Todo (US-T*)**: Unit and/or integration tests in `crates/todo` (e.g. `crates/todo/tests/` or in-tree tests). Verify create (valid/invalid input), list (empty and ordered), complete, delete per `docs/acceptance.md`.
- **Xtask (US-X*)**: Manual or CI: `cargo xtask --help`, `cargo xtask run` exit codes and output. No domain logic in xtask, so no dedicated xtask unit tests required for business rules.

### Git Workflow
- Not formally specified in repo docs. Prefer conventional commits and a single main branch unless the team defines otherwise.

## Domain Context
- **Todo**: A single item with `id` (`TodoId`), `title`, `completed`; optional `created_at`. Create (with non-empty title), list (ordered e.g. by creation), complete (by id), delete (by id).
- **Errors**: `TodoError` with at least `InvalidInput` (e.g. empty title) and `NotFound(TodoId)` for missing id in complete/delete.
- **Storage**: Default in-memory; persistence can be added via new `Store` implementations without changing the public API.

## Important Constraints
- Implementation must stay in Rust; workspace must include `crates/todo` and `xtask`.
- `cargo xtask` must work without installing extra tools globally (alias in `.cargo/config.toml`).
- Requirements and design are authoritative; see `docs/requirements.md` and `docs/design.md`. Task breakdown and acceptance checklist: `docs/tasks.md`, `docs/acceptance.md`.

## External Dependencies
- **Crates**: `argh` (xtask only). No external services or APIs; storage is in-process by default.
