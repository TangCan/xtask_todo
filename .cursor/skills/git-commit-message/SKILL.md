---
name: git-commit-message
description: >-
  Generates a git commit message from staged changes and runs git commit -m or
  cargo xtask git commit -m. Use when the user wants to commit, run xc, git
  commit, 提交, or asks to generate a commit message and commit.
---

# Git commit with generated message

## When to use

- User says: commit, 提交, xc, git commit, `cargo xtask git commit`, or "generate commit message and commit".
- User wants to commit staged changes and expects a sensible `-m` message.

## Instructions

1. **Inspect staged changes**
   - Run `git status` and `git diff --cached` (or `git diff --cached --stat`) to see what is staged.
   - If nothing is staged, suggest staging first: `git add ...` or `cargo xtask git add`, then retry.

2. **Generate the commit message**
   - Write a short message (one line, ideally ≤72 chars) that describes the change.
   - Prefer [Conventional Commits](https://www.conventionalcommits.org/): `type(scope): description`.
   - Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`, etc.
   - Base the message on file names and diff content (e.g. "feat(todo): add optional -m to git commit", "docs: update requirements").

3. **Run commit with the generated message**
   - Prefer project convention: **`cargo xtask git commit -m "<message>"`** (runs pre-commit checks).
   - Alternatively: `git commit -m "<message>"`.
   - Use the exact generated message inside quotes; escape any inner quotes if needed.

4. **If the user only asked for a message**
   - Output the suggested message and the command they can run, e.g.:
   - `cargo xtask git commit -m "feat(xtask): allow custom commit message via -m"`

## Project context

- This repo uses `cargo xtask git commit`; optional `-m <message>` overrides the default message "Sync".
- Without `-m`, the default is `"Sync"`. With `-m`, use the provided (or generated) message.

## Example

User: "commit my changes"

1. Run `git diff --cached --stat` → see `xtask/src/git.rs`, `xtask/src/tests/git.rs` changed.
2. Generate: `feat(xtask): add -m/--message to git commit`
3. Run: `cargo xtask git commit -m "feat(xtask): add -m/--message to git commit"`
