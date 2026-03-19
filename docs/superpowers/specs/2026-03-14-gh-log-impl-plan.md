# cargo xtask gh log — 实现计划

**设计依据**：`docs/superpowers/specs/2026-03-14-gh-log-design.md`

| 顺序 | 任务 | 验证 |
|------|------|------|
| 1 | 新增 `xtask/src/gh.rs`：GhArgs、GhSub::Log、cmd_gh；先 `gh run list --limit 1 --json databaseId -q '.[0].databaseId'` 取 id，再 `gh run view <id> --log` 透传；错误时 stderr 说明、返回 Err | `cargo build -p xtask` |
| 2 | 在 `lib.rs` 中 `mod gh`、`XtaskSub::Gh(GhArgs)`、run_with 分支调用 `gh::cmd_gh` | `cargo xtask gh log`（需 gh 在 PATH） |
| 3 | README 或 docs 中补充 `cargo xtask gh log` 说明 | 人工确认 |
