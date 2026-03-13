# Tab 补全（命令 + 路径）设计文档

**日期**：2026-03-13  
**范围**：交互式 Tab 按键补全；补全命令名与虚拟 FS 路径/文件名；基于 rustyline 集成

---

## 1. 目标与方案选择

### 1.1 目标

- **交互式 Tab**：用户输入到一半按 Tab，当前行立即补全并刷新显示（类似 bash）。
- **补全范围**：既补全**命令名**（pwd、cd、help、export-readonly 等），也补全**路径/文件名**（基于虚拟 FS 的当前目录及已输入路径前缀）。

### 1.2 方案选择

- **采用 rustyline**：使用现成行编辑库实现 raw 模式与补全回调，跨平台（Linux/macOS/Windows），减少手写终端与平台相关代码。首版在 Linux/macOS 验证，再在 Windows 上验证。
- **不手写 raw 模式**：手写 termios/Windows Console 工作量大且易踩坑，不采用。

### 1.3 整体流程

1. REPL 不再使用 `stdin.read_line()`，在交互式模式下改为调用 rustyline 的 `Editor::readline(prompt)`。
2. 用户按 Tab 时，rustyline 调用我们注册的 **completion 回调**。
3. 回调根据「当前行 + 光标位置」解析出当前 token 与上下文（命令位置 vs 路径位置）。
4. **命令位置**：返回以当前 token 为前缀的内置命令名候选。
5. **路径位置**：用 Vfs 的 `list_dir`（结合 `resolve_to_absolute` 解析父路径），返回以当前 token 为前缀的文件/目录名；若唯一且为目录可加 `/`。
6. rustyline 将补全结果插入行并刷新显示。

---

## 2. 补全上下文解析

### 2.1 命令位置（只做命令补全）

以下情况视为**命令名**输入，仅做命令补全：

- **行首**：从行首到第一个空格或 `|`、`<`、`>`、`2>` 之间的片段。
- **管道后**：紧接在 `|` 之后的第一个 token（到下一个空格或重定向符为止）。

实现：从左到右按空格与 `|`、`<`、`>` 分词；若光标所在 token 是「行首 token」或「紧跟在 `|` 后的 token」，则补命令。

### 2.2 路径位置（只做路径/文件名补全）

以下情况视为**路径**输入，仅做路径补全：

- **命令后的参数**：前一个 token 为 `cd`、`ls`、`cat`、`mkdir`、`touch`、`export-readonly` 等时，当前 token 做路径补全。
- **重定向目标**：前一个 token 为 `>`、`2>`、`<` 时，当前 token 为路径。

实现：根据「当前是第几个 token」与「前一个 token」判断；若前 token 为上述命令或重定向符，则当前 token 做路径补全。

### 2.3 当前 token 与前缀

- 用光标位置将行分为「光标前」「光标后」；光标前的部分按上述规则切出**最后一个词**作为 prefix。
- 补全时返回「以 prefix 为前缀」的候选（命令名或路径名）。

### 2.4 路径补全的 VFS 解析

- prefix 可能是相对路径（如 `foo`、`foo/ba`）或绝对路径（如 `/`、`/foo/ba`）。
- 用 Vfs 的 `resolve_to_absolute` 解析 prefix 的**父路径**；对父路径调用 `list_dir`，再对结果做前缀匹配（匹配 prefix 的最后一段）。
- 若 prefix 以 `/` 结尾或解析后为目录，则在该目录下 list_dir。
- 若唯一候选且为目录，补全后可加 `/`（由 rustyline 或返回的 replacement 控制）。

### 2.5 边界

- 空行或光标在行首：视为命令位置，可列出所有命令。
- 未知上下文：首版采用「若前 token 在命令名集合或重定向符中则按上规则，否则仅做路径补全」，使 `cat foo<Tab>` 等仍能补路径。

---

## 3. rustyline 集成与 REPL 改造

### 3.1 依赖

- `Cargo.toml` 增加 `rustyline`（如 `rustyline = "15"` 或当前稳定版）。
- 通过 `Editor::set_helper(Some(helper))` 设置实现 `Helper` trait 的补全逻辑。

### 3.2 Completer / Helper

- 实现一个 `Helper`（如 `DevShellHelper`），在 Tab 时由 rustyline 调用 `completion`。
- **数据**：补全需要只读访问 Vfs（list_dir、resolve_to_absolute）。为避免长期持有 `&mut Vfs`，采用「每轮 readline 前由 REPL 更新 Helper 内上下文」：Helper 持有当前 cwd 与必要时的一次 list_dir 结果快照，REPL 每轮根据当前 `vfs` 更新该上下文后再调用 `editor.readline(prompt)`。

### 3.3 REPL 改造

- `repl::run` 保留 `(vfs, stdin, stdout, stderr)` 或改为 `(vfs, stdout, stderr)`（见实现）。
- **交互式**（stdin 为 TTY）：创建 `Editor`，设置 `set_helper(DevShellHelper::new(completion_ctx))`；每轮先更新 Helper 的上下文，再 `editor.readline(&prompt)`；读到的行照旧交给 `parse_line` 与 `execute_pipeline`。
- **非交互式**（管道/重定向）：仍用 `stdin.read_line()`，保证脚本与管道行为不变。判断方式：`stdin.is_terminal()`（rustyline 或 `is-terminal` crate）或等价方式。

### 3.4 Prompt 与 EOF

- Prompt 仍为 `format!("{} $ ", vfs.cwd())`，每次 readline 前生成。
- `readline` 返回 `Ok(None)` 时视为 EOF，退出循环。

---

## 4. 错误处理、测试与文档

### 4.1 错误处理

- **补全回调**：路径解析或 list_dir 失败时返回空候选列表，不向用户报错。
- **readline 错误**：`readline` 返回 `Err(e)` 时，写简短信息到 stderr，视情况 `continue` 或 `return Err(())`。
- **非交互**：沿用现有 EOF 与错误处理。

### 4.2 测试

- **单元测试**：为「补全上下文解析」写独立函数（如 `split_tokens(line, pos)` 或 `completion_context(line, pos)`），对多组 `(line, pos)` 测试（行首、`|` 后、`cd ` 后、`> out` 等）。路径补全的「前缀 + list_dir 过滤」用固定 Vfs 状态做单元测试。
- **交互**：依赖 TTY，CI 中不跑；在 README 中说明手动验证方式。

### 4.3 文档

- README 增加：支持 Tab 补全命令与路径，交互式下按 Tab 触发。
- 本设计文档保存于 `docs/plans/2026-03-13-tab-completion-design.md`。

---

## 5. 后续可选

- 多候选时展示菜单或二次 Tab 循环（若 rustyline 支持）。
- 路径补全时对目录候选自动追加 `/`。
- Windows 上若遇兼容问题，可条件编译或更换库。
