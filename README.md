# xtask-todo

A Rust workspace with a todo list library and an xtask-based CLI. Tasks are stored in `.todo.json` in the current directory.

## Usage

### Todo (data in `.todo.json`)

```bash
cargo xtask todo add "Buy milk"
cargo xtask todo list
cargo xtask todo complete <id>
cargo xtask todo delete <id>
```

### Other xtask commands

| Command | Description |
|---------|-------------|
| `cargo xtask fmt` | Format code (same as `cargo fmt`) |
| `cargo xtask clippy` | Lint with Clippy (pedantic + nursery, warnings as errors) |
| `cargo xtask coverage` | Run coverage per crate (cargo-tarpaulin) |
| `cargo xtask git add` | Stage common paths (e.g. .github, xtask, docs, crates) |
| `cargo xtask git commit` | Commit with message "Sync" (runs pre-commit checks) |

## Tests

```bash
cargo test
```

## Git hooks

A pre-commit hook runs `cargo fmt -- --check`, `cargo clippy` (pedantic + nursery), `.rs` file line limit (500), and `cargo test`. Enable it once:

```bash
git config core.hooksPath .githooks
```

## Documentation

See the [docs](docs/) folder for requirements, design, acceptance criteria, and test cases.

## Publishing

Only the **todo** crate is published to [crates.io](https://crates.io); **xtask** is workspace tooling (`publish = false`). For steps (metadata, `cargo login`, `cargo publish -p todo`) and Git-only releases, see [docs/publishing.md](docs/publishing.md).
