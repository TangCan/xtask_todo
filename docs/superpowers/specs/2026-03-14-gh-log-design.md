# cargo xtask gh log — 设计说明

**日期**：2026-03-14  
**范围**：新增 `cargo xtask gh log`，显示最近一次 GitHub Actions run 的 job log；等价于  
`gh run view $(gh run list --limit 1 --json databaseId -q '.[0].databaseId') --log`。  
**状态**：设计已获用户逐节通过（§1 范围与依赖、§2 实现要点与错误处理、§3 文档与测试）。

---

## 1. 设计目标

- 提供一条 xtask 命令，在本地快速查看当前仓库最近一次 GitHub Actions run 的 log，无需手写 `gh run list` + `gh run view`。
- 无 run 或 `gh` 不可用时：退出码 1，并在 stderr 输出简短说明（用户选择 **A**）。

---

## 2. 范围与依赖（§1 已通过）

- **命令形态**：`cargo xtask gh log`（顶层子命令 `gh`，子命令 `log`；首期不实现其他 `gh *`）。
- **依赖**：仅使用标准库与现有 `std::process::Command`；不新增第三方 crate。
- **前提**：本机已安装 [GitHub CLI](https://cli.github.com/) 且 `gh` 在 PATH 中（与「需安装 git」类似）。
- **行为等价**：与  
  `gh run view $(gh run list --limit 1 --json databaseId -q '.[0].databaseId') --log`  
  一致；失败时退出码 1 + stderr 简短说明（**A**）。

---

## 3. 实现要点与错误处理（§2 已通过）

- **结构**：新增 `xtask/src/gh.rs`（必要时可拆为 `gh/mod.rs` + `gh/log.rs`）；在 `lib.rs` 中注册 `GhArgs`、`XtaskSub::Gh`，分发到 `gh::cmd_gh`。`cmd_gh` 仅处理 `gh log` 子命令。
- **流程**：
  1. 执行 `gh run list --limit 1 --json databaseId -q '.[0].databaseId'`，捕获 stdout。
  2. 若命令执行失败（如 `gh` 未找到）、或 stdout 为空、或无法解析为合法 run id → 视作「无 run」或「gh 不可用」：stderr 输出简短说明，返回退出码 1。
  3. 若有 run id：执行 `gh run view <id> --log`，将 stdout/stderr 透传到当前进程；若该命令退出非零，则 stderr 转发或归纳错误信息，返回退出码 1。
- **错误信息约定**：
  - `gh` 未找到或 spawn 失败 → stderr "gh: command not found"（或系统错误信息）。
  - list 无结果或解析失败 → stderr "no runs found"（或等价说明）。
  - `gh run view` 失败 → 将 `gh` 的 stderr 转发或简短归纳。  
  所有错误路径：退出码 1 + stderr 说明（符合 **A**）。

---

## 4. 文档与测试（§3 已通过）

- **文档**：在 README 或 xtask 相关文档中增加一句说明：`cargo xtask gh log` 用于显示最近一次 GitHub Actions run 的 log，等价于上述 `gh run view ...` 命令；前提为已安装并配置 `gh`。
- **测试**：不强制在 CI 中安装 `gh`。单元测试可 mock `Command` 或在不具备 `gh` 的环境下跳过；若有集成测试，仅在 `gh` 可用时运行，断言有 run 时能输出 log、无 run 时退出 1 且 stderr 含预期说明。

---

## 5. 实现时要点摘要

1. **xtask/src/gh.rs**：定义 `GhArgs`（argh 子命令 `gh`）、`GhSub::Log`，实现 `cmd_gh`；在 `cmd_gh` 中先 `gh run list ...` 取 id，再 `gh run view <id> --log`，透传 stdout/stderr，按 §3 处理错误。
2. **xtask/src/lib.rs**：`mod gh`；`use crate::gh::GhArgs`；`XtaskSub` 增加 `Gh(GhArgs)`；`run_with` 中增加分支 `XtaskSub::Gh(args) => gh::cmd_gh(&args).map_err(|e| to_run_failure(&*e))`。
3. **文档**：README 或 docs 中补充 `gh log` 的说明与前提。
4. **不实现**：其他 `gh` 子命令（如 `gh list`、`gh view` 等）；本期仅 `gh log`。

---

*设计三节均已通过；可据此进入 spec review（可选）与 writing-plans，再开始编码。*
