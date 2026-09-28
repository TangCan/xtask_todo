---
story: 1-3-cli-snapshots-and-data-properties
date: 2026-09-28
---

# 自动化测试总结 — Story 1.3

## 覆盖范围

- `xtask/tests/trycmd_list_snapshots.rs` 使用真实 `todo` 二进制执行固定文件系统场景，覆盖 status、priority、tags、due-before、due-after 与 sort。
- `crates/todo/src/tests/property.rs` 使用 proptest 验证状态、标签、优先级和日期过滤不变量，并使用 insta 固定标题排序输出。
- `xtask/tests/todo_list/list_json.rs` 验证非法日期返回退出码 2 且已有 `.todo.json` 内容保持不变。

## 验证结果

- 定向属性、insta、trycmd 和 list 集成测试通过。
- workspace 测试、doctest、clippy、fmt 和 `git diff --check` 已通过。
- 无 API、浏览器或外部服务测试；本 Story 仅涉及 Rust CLI 与数据层。
