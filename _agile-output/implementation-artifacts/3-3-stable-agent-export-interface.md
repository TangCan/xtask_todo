# Story 3.3: 稳定的 Agent/外部工具导出接口

Status: done

## Acceptance criteria

- Export JSON has a versioned envelope and a documented field schema.
- `--json export` remains a single stable JSON response and exposes the schema version.
- Nulls, Unicode, special characters, tags, dates, and duplicate titles retain their types.
- Legacy array imports remain supported; future breaking changes have a migration rule.
- Invalid input continues to use the existing parameter/data/general exit contracts.

## Implementation

- Versioned JSON file exports with `version: 1` and `todos`.
- Legacy array and versioned envelope imports are both accepted.
- Added `docs/todo-export-schema-v1.json` and `docs/agent-export.md`.
- Added `schema_version` to the existing JSON export result.

## Verification

- Existing import/export integration tests pass through the full pre-commit suite.
- JSON serialization remains delegated to `serde_json`.
