## Deferred from: code review of 1-1-create-todo-validation-no-dirty-write.md (2026-03-25)

- `handle_add`：`--dry-run` 在调用 `patch_from_add_args` 之前返回，dry-run 不会校验非法可选参数；与正常路径行为不一致，但符合 AC4「不写盘」字面；若需 CLI 一致性，可在后续 story 中把校验提前到 dry-run 分支或补充文档说明。

## Deferred from: code review of 1-2-list-todos-empty-result.md (2026-03-25)

- `load_todos` / `load_todos_from_path`：JSON 解析失败时以空列表继续（`unwrap_or_default`），损坏的 `.todo.json` 在成功退出码下与「无待办」表现相同；若需区分错误与空集，应在后续 story 中显式报错或校验。

## Deferred from: code review of 1-3-filter-sort-list-browse.md (2026-03-25)

- AC1 所列多类 `list` 过滤/排序维度在本 diff 中仅部分以集成测试覆盖；`--priority`、`--due-before`/`--due-after` 等已有 `xtask-todo-lib` / `todo_cmd` 单测支撑，完整 E2E 矩阵可作为后续 story 或硬化任务再扩展。
