# Automation summary: Story 3.2

Unit coverage validates the non-TTY safety boundary for progress and the
structured logging call path. Existing import/export integration tests verify
that `--json` remains the stdout payload. The full repository pre-commit suite
is the regression gate for exit codes and JSON contracts.
