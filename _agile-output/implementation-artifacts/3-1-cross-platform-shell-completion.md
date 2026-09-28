# Story 3.1: 跨平台 Shell 补全

Status: done

## Goal

Provide deterministic completion scripts for bash, zsh, and fish without
mixing diagnostics into stdout or changing existing todo JSON behavior.

## Acceptance criteria

- `cargo xtask completions bash|zsh|fish` emits the corresponding script only.
- Unsupported shells fail with parameter exit code 2 and list supported shells.
- Scripts include top-level commands, todo subcommands, and static options.
- No dynamic discovery is performed; the support boundary is documented.

## Implementation

- Added `xtask/src/completions.rs` and the `completions <shell>` subcommand.
- Added unit tests for shell-specific output and unsupported-shell diagnostics.
- Added `docs/completions.md` with installation examples and compatibility policy.

## Verification

- `cargo fmt --all`
- `cargo test -p xtask completions -- --nocapture`
- Manual bash generation and unsupported-shell exit-code check.
