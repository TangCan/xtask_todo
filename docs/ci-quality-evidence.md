# CI quality evidence

Every matrix job writes `quality/quality-summary.md` after the quality gates
and uploads it as `quality-summary-<os>-<arch>`. The summary records the commit,
runner, ref, and the exact gates covered by the job. Coverage JSON remains in
the separate `coverage-<os>` artifact. This gives each CI run a small index that
can be attached to a release or retrospective without copying console logs.

The release optimization report is checked by the
`release_optimization_report_matches_machine_measurements` Rust test, so
`cargo test` fails if the human table drifts from
`docs/benchmarks/release-optimized.json`.
