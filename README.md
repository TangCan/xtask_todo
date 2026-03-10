# xtask-todo

A Rust workspace with a todo list library and an xtask-based CLI. Tasks are stored in `.todo.json` in the current directory.

## Usage

Run todo commands via cargo xtask:

```bash
cargo xtask todo add "Buy milk"
cargo xtask todo list
cargo xtask todo complete <id>
cargo xtask todo delete <id>
```

## Tests

```bash
cargo test
```

## Documentation

See the [docs](docs/) folder for requirements, design, acceptance criteria, and test cases.
