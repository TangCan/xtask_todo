# Automation summary: Story 3.1

Automated coverage is in `xtask/src/completions.rs`:

- shell output smoke checks for bash, zsh, and fish;
- stdout-safety assertion;
- unsupported-shell error contract assertion.

Verification command: `cargo test -p xtask completions -- --nocapture`.
