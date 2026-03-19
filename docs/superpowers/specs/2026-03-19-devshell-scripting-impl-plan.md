# dev_shell 脚本机制 — 实现计划

**设计依据**：`docs/superpowers/specs/2026-03-19-devshell-scripting-design.md`

---

## 前置条件

- 在 `crates/todo` 下开发；devshell 位于 `src/devshell/`，现有 `parser`（pipeline）、`command`（execute_pipeline）、`repl`（process_line, run）。
- 入口为 `run_main_from_args`（当前仅处理 `[path]` 或 `[]`）。

---

## Task 1：入口与脚本文件读取

**目标**：支持 `dev_shell [-e] [-f] script.dsh` 或 `dev_shell script.dsh`，未给脚本时行为与现有一致。

**步骤**：

1. 在 `run_main_from_args` 中解析参数：识别 `-e`（set -e）、`-f`（显式脚本）、位置参数为脚本路径；允许多种顺序（如 `dev_shell -e script.dsh`、`dev_shell script.dsh`）。
2. 若存在脚本路径：从宿主 FS 或当前工作目录读取脚本文件内容（`std::fs::read_to_string`）；若失败，写 stderr 并返回错误。
3. 调用新函数 `run_script(vfs, script_src, path, set_e, stdin, stdout, stderr)`（先为 stub：仅逐行执行，不解析控制流）。
4. 若无脚本路径：保持现有逻辑（用 path 作 .bin、启动 REPL）。

**验证**：`dev_shell /nonexistent.dsh` 报错；`dev_shell script.dsh` 且 script.dsh 存在且仅含 `echo hello` 时能输出 hello（stub 可先按「按行 split、每行交 process_line」实现）。

**文件**：`crates/todo/src/devshell/mod.rs`（入口）、新建 `script.rs`（`run_script` stub，读入字符串、按行调用现有执行逻辑）。

---

## Task 2：脚本词法（续行、注释、空行）

**目标**：将脚本源码变为「逻辑行」序列（续行合并、去掉注释与空行）。

**步骤**：

1. 在 `script.rs` 或新文件 `script/lex.rs` 中实现 `logical_lines(source: &str) -> Vec<String>`：按 `\n` 切分；行尾 `\` 与下一行合并（去掉 `\` 与换行）；每行去掉 `#` 及其后内容；trim 后空行丢弃。
2. 返回的 `Vec<String>` 供后续语法解析使用。

**验证**：单元测试。如 `"echo a # comment\necho b"` → 两行；`"echo \\\nworld"` → 一行 `"echo world"`。

**文件**：`crates/todo/src/devshell/script.rs` 或 `script/lex.rs`。

---

## Task 3：变量展开

**目标**：实现 `expand_vars(s: &str, vars: &HashMap<String, String>) -> String`，支持 `$VAR`、`${VAR}`；未定义变量展开为空（或按设计约定）。

**步骤**：

1. 单遍扫描字符串，识别 `$NAME`（NAME 为字母数字下划线）和 `${NAME}`，用 vars 中值替换。
2. 单元测试：赋值与展开、未定义、边界（`$` 后无标识符等）。

**文件**：`script.rs` 或 `script/expand.rs`。

---

## Task 4：脚本 AST 与解析

**目标**：定义脚本语句的 AST，并从逻辑行解析出 AST。

**步骤**：

1. 定义类型（示例）：`ScriptStmt::Assign(name, value)`、`ScriptStmt::Command(line)`、`ScriptStmt::If { cond_line, then_body, else_body }`、`ScriptStmt::For { var, words, body }`、`ScriptStmt::While { cond_line, body }`、`ScriptStmt::Source(path)`。
2. 解析逻辑行：识别行首关键字 `set`、`if`、`for`、`while`、`source`/`.`、以及 `NAME=value`（无空格 around `=` 的赋值）；否则视为命令行。`if`/`for`/`while` 需解析到匹配的 `fi`/`done`，体为子 AST 列表。
3. 解析错误（未闭合、非法语法）返回 `Result`，写 stderr 并退出脚本路径。

**验证**：仅解析的单元测试（不执行）；多组脚本片段。

**文件**：`script/ast.rs`、`script/parse.rs` 或合并在 `script.rs`。

---

## Task 5：解释器（执行 AST）

**目标**：根据 AST 驱动执行，与现有 VFS 和 `execute_pipeline` 集成。

**步骤**：

1. 维护 `vars: HashMap<String, String>`、`set_e: bool`、`source_depth: u32`（上限如 64）。
2. 遍历 AST：`Assign` 写入 vars；`Command` 先对整行做变量展开，再 `parse_line` + `execute_pipeline`（与 repl 一致）；命令失败时若 set_e 则返回错误并终止脚本。
3. `If`：先执行 cond_line（单条命令），根据退出码决定执行 then_body 或 else_body。
4. `For`：对 words 逐项设 var 后执行 body。
5. `While`：循环执行 cond_line，为 0 则执行 body，直到 cond 非 0。
6. `Source`：source_depth+1，若超限则报错；否则读 path（VFS 或宿主），解析为 AST，递归执行（同一 vfs、同一 vars、新 depth）；执行后 depth 恢复。
7. 执行阶段错误（如 pipeline 执行失败且 set_e）时，进程退出码为非零。

**验证**：集成测试：小脚本含赋值、echo、if、for、while、source；断言 VFS 状态与 stdout/stderr。

**文件**：`script.rs` 中 `run_script` 改为「解析 → 解释执行」；或 `script/exec.rs`。

---

## Task 6：set -e 与入口 -e

**目标**：脚本内 `set -e` 与命令行 `-e` 生效。

**步骤**：

1. 解析 `set -e` 为 AST 节点或直接在解析时设标志（若首行 `set -e`）；或解释器遇到 `set -e` 命令时设 `set_e = true`。
2. 入口 `-e` 在调用 `run_script` 时传入初始 `set_e = true`。
3. 验证：脚本某条命令失败且 set_e 时脚本立即退出且退出码非零。

**文件**：`script.rs`、`mod.rs`（传 -e 到 run_script）。

---

## Task 7：REPL 内 source（可选）

**目标**：在交互 REPL 中，若用户输入 `source path` 或 `. path`，在该 VFS 或宿主路径读取文件并作为脚本执行（共享当前「会话」变量表与 source 深度限制）。

**步骤**：

1. 在 `process_line` 或 pipeline 分发前，识别首词为 `source` 或 `.` 且只有一个参数：读文件、解析为脚本 AST、用解释器执行（vfs、可选共享 vars、depth 从 0 或 1 起）。
2. REPL 下可维护一个「会话变量表」（或首期不持久，仅本次 source 内有效）；设计约定在文档说明。

**验证**：REPL 中输入 `source script.dsh`，script.dsh 含若干命令，观察输出与 VFS 变化。

**文件**：`repl.rs` 或 `command.rs`、`script.rs`。

---

## Task 8：测试与文档

**目标**：覆盖解析/解释/集成用例，并更新文档。

**步骤**：

1. 单元测试：lex、expand、parse、exec（mock 或最小 VFS）；语法错误、未闭合块、source 深度超限、set -e。
2. 集成测试：完整小脚本（变量、if、for、while、source、set -e），断言 stdout/stderr 与退出码。
3. README 或 dev_shell 文档增加「脚本」小节：语法概要、`dev_shell script.dsh` / `-e` / `-f`、`source`、示例脚本、与 REPL 的差异。

**文件**：`devshell/script/tests.rs` 或 `devshell/tests/script_*.rs`、`README.md` 或 `docs/`。

---

## 实现顺序与依赖

| 顺序 | 任务       | 依赖     |
|------|------------|----------|
| 1    | Task 1 入口 | 无       |
| 2    | Task 2 词法 | 无       |
| 3    | Task 3 变量展开 | 无   |
| 4    | Task 4 AST/解析 | Task 2 |
| 5    | Task 5 解释器 | Task 3, 4, 1 |
| 6    | Task 6 set -e | Task 5 |
| 7    | Task 7 REPL source（可选） | Task 5 |
| 8    | Task 8 测试与文档 | 全部 |

首期可实现：Task 1 → 2 → 3 → 4 → 5 → 6 → 8；Task 7 可后置。

---

*计划编写完成；可按任务顺序实现，每步通过测试与验证后再进行下一任务。*
