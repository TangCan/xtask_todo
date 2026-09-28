# Agent export contract

`cargo xtask todo --json export PATH` returns the stable envelope
`{"status":"success","data":{...}}`; its `data` includes `exported`,
`schema_version: 1`, and `file`. The exported JSON file itself is documented
by [`todo-export-schema-v1.json`](todo-export-schema-v1.json): an object with
`version: 1` and a `todos` array. Legacy top-level arrays remain accepted on
import for backward compatibility. The schema permits additive properties at
both envelope and todo-item level; consumers must use the documented fields
and ignore unknown additive fields. A field removal or type change requires a
new schema version.

All todo fields are serialized by `serde_json`, so nulls, Unicode, tags,
dates, duplicate titles, and special characters remain typed and escaped
without lossy text parsing. A future breaking schema must increment `version`
and document a migration; additive fields remain compatible with consumers
that ignore unknown fields.
