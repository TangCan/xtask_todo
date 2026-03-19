# xtask-todo

[![crates.io](https://img.shields.io/crates/v/xtask-todo-lib.svg)](https://crates.io/crates/xtask-todo-lib) [![docs.rs](https://img.shields.io/docsrs/xtask-todo-lib)](https://docs.rs/xtask-todo-lib)

A Rust workspace with a todo list library and an xtask-based CLI. Tasks are stored in `.todo.json` in the current directory. The library is published as **[xtask-todo-lib](https://crates.io/crates/xtask-todo-lib)** on [crates.io](https://crates.io/crates/xtask-todo-lib).

## Usage

### Todo (data in `.todo.json`)

```bash
cargo xtask todo add "Buy milk"
cargo xtask todo list
cargo xtask todo complete <id>
cargo xtask todo delete <id>
```

**Add/update** support optional fields: `--description`, `--due-date` (YYYY-MM-DD), `--priority` (low/medium/high), `--tags` (comma-separated), `--repeat-rule` (daily, weekly, 2d, 3w, etc.), `--repeat-until` (YYYY-MM-DD), `--repeat-count`. **List** supports filters: `--status` (completed/incomplete), `--priority`, `--tags`, `--due-before`, `--due-after`, and `--sort` (created-at, due-date, priority, title). Use `cargo xtask todo --help` and `cargo xtask todo add --help` for full options.

### Other xtask commands

| Command | Description |
|---------|-------------|
| `cargo xtask fmt` | Format code (same as `cargo fmt`) |
| `cargo xtask clippy` | Lint with Clippy (pedantic + nursery, warnings as errors) |
| `cargo xtask coverage` | Run coverage per crate (cargo-tarpaulin) |
| `cargo xtask gh log` | Show log of the most recent GitHub Actions run (requires [GitHub CLI](https://cli.github.com/) in PATH; equiv. `gh run view $(gh run list --limit 1 --json databaseId -q '.[0].databaseId') --log`) |
| `cargo xtask git add` | Stage common paths (e.g. .github, xtask, docs, crates) |
| `cargo xtask git pre-commit` | Run the same checks as the pre-commit hook (fmt, clippy, .rs line limit, test) without committing |
| `cargo xtask git commit` | Commit with message "Sync" by default; use `-m "message"` for a custom message (runs pre-commit checks) |

## Tests

```bash
cargo test
```

## Cursor skill (commit message)

The project includes a Cursor skill (`.cursor/skills/git-commit-message/`) that, when you ask to commit or run `xc`/`git commit`, can generate a commit message from staged changes and run `cargo xtask git commit -m "..."` or `git commit -m "..."`. Ensure changes are staged first (e.g. `cargo xtask git add` or `git add`).

## Git hooks

A pre-commit hook runs `cargo fmt -- --check`, `cargo clippy` (pedantic + nursery), `.rs` file line limit (500), and `cargo test`. Enable it once:

```bash
git config core.hooksPath .githooks
```

## Documentation

See the [docs](docs/) folder for requirements, design, acceptance criteria, and test cases.

## Publishing

Only the **xtask-todo-lib** crate (in `crates/todo`) is published to [crates.io](https://crates.io); **xtask** is workspace tooling (`publish = false`). For steps (metadata, `cargo login`, `cargo publish -p xtask-todo-lib`) and Git-only releases, see [docs/publishing.md](docs/publishing.md).
