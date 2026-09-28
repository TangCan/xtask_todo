# Todo operation feedback

`todo import` and `todo export` keep stdout reserved for their documented
human output or `--json` payload. When stderr is an interactive terminal,
they print a small count-based progress update there. Redirecting stdout for a
pipeline therefore remains safe.

Structured logs are opt-in through `XTASK_LOG=info` (or `debug`). Each line is
JSON with `time_unix_ms`, `level`, `operation`, `message`, and `context`.
Failures emit an error event and retain the existing exit code and JSON error
contract. Progress and logs are never written to JSON stdout.
