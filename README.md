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

### Dev shell and scripting

The **xtask-todo-lib** crate provides a small dev shell (VFS, builtins: `pwd`, `cd`, `ls`, `mkdir`, `cat`, `touch`, `echo`, `save`, `export-readonly`, `todo`, `rustup`, `cargo`, `exit`). Run the REPL:

```bash
cargo run -p xtask-todo-lib --bin cargo-devshell [path]
```

Run a **script file** (same builtins and VFS, no external commands):

```bash
cargo run -p xtask-todo-lib --bin cargo-devshell -f script.dsh
cargo run -p xtask-todo-lib --bin cargo-devshell -e -f script.dsh   # exit on first command failure
```

In the **REPL**, you can run a script with **`source path`** or **`. path`** (file is read from VFS or host). Variables and control flow in that script are independent of the next REPL line (no shared session variables).

- **`-f script.dsh`** — run the given script instead of the REPL.
- **`-e`** — enable “exit on error” (like `set -e` from the start).

**Script syntax** (logical lines; continuation with `\`; `#` comments):

- **Variables**: `NAME=value` and `$VAR` / `${VAR}` expansion.
- **Control flow**: `if command; then ... else ... fi`, `for VAR in a b c; do ... done`, `while command; do ... done`.
- **`set -e`** — subsequent failed commands abort the script.
- **`source path`** or **`. path`** — run another script (from VFS or host); max depth 64.

Example script:

```bash
X=hello
echo $X
for x in one two; do echo $x; done
if pwd; then echo ok; fi
```

Scripts only run built-in commands and use the same virtual filesystem as the REPL; they do not invoke the host shell or external programs.

**Rust toolchain** (sandboxed):

- **`rustup [args...]`** — run host `rustup` in an isolated temp dir: the current VFS subtree (cwd) is exported, `rustup` runs there, then changes are synced back into the VFS. Requires `rustup` in PATH.
- **`cargo [args...]`** — same flow for `cargo` (e.g. `cargo build`, `cargo run`, `cargo new`). Compilation outputs and new crates appear in the VFS after the command finishes.
- Isolation: one unique temp dir per run (e.g. `devshell_<pid>_<nanos>` under `$TMPDIR`), mode `0o700`; the dir is removed after sync. Stronger isolation (e.g. fd-only on Linux, or optional Docker/Podman backend) is described in `docs/superpowers/specs/2026-03-20-devshell-rust-vm-design.md`.

Example: `mkdir my_project && cd my_project && cargo new . --name my_app && cargo build`.

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
