## Deferred from: code review of 1-1-create-todo-validation-no-dirty-write.md (2026-03-25)

- `handle_add`：`--dry-run` 在调用 `patch_from_add_args` 之前返回，dry-run 不会校验非法可选参数；与正常路径行为不一致，但符合 AC4「不写盘」字面；若需 CLI 一致性，可在后续 story 中把校验提前到 dry-run 分支或补充文档说明。
