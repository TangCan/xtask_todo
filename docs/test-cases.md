# 测试用例（Test Cases）

本文档与 **[requirements.md](./requirements.md)**、**[design.md](./design.md)** 对齐，用于需求 / 设计 → 用例 → 验证的追溯与回归。若与 [acceptance.md](./acceptance.md) 有验收项，可通过「需求引用」列对应。

**列说明**：每条用例含 **ID**、**需求/设计引用**、**描述**、**步骤要点**、**验证方式**、**预期结果**、**实现映射**（主要自动化测试或源码位置；无则标「手工」）。

**执行**：`cargo test`（workspace）；pre-commit / CI 应全量通过。部分用例依赖宿主工具（`gh`、PATH 中的 `cargo`/`rustup`），在映射列注明。

---

## 0. 追溯索引

| 需求/设计 | 本文档章节 |
|-----------|------------|
| requirements §1 概述（含 Mode S/P） | §5 Devshell |
| requirements §2 能力与不承诺 | §8 非功能；概述性用例 |
| requirements §3 Todo | §2 |
| requirements §4 其他 xtask | §4 |
| requirements §5 Devshell | §5 |
| requirements §5.2 / §5.8（工作区路径、Windows β） | §5.10 |
| requirements §6 AI / 退出码 | §1 |
| requirements §7 非功能 | §7 |
| requirements §1.2、§7.1～§7.2（平台、pre-commit、MSVC 检查） | §7 |
| design §1～§3 | §4～§6 |
| design §1.4 VM / β 侧车 | §5.10 |
| design §4 关键决策 | §5、§6 |

---

## 1. AI / 可编程接口（requirements §6）

### US-A1：结构化 JSON

| ID | 需求/设计 | 描述 | 步骤要点 | 验证方式 | 预期结果 | 实现映射 |
|----|-----------|------|----------|----------|----------|----------|
| TC-A1-1 | US-A1 | 成功路径 JSON 可解析 | 各 todo 子命令加 `--json` | 单元/集成 | 合法 JSON，`status`+`data` | `xtask/src/tests/todo/todo_cmd/json_dry_init.rs` |
| TC-A1-2 | US-A1 | 失败路径 JSON | 触发参数/数据错误且 `--json` | 集成 | `status`+`error`（含 message/code） | `xtask::todo::print_json_error`；`lib.rs` todo 分支 |

### US-A2：退出码

| ID | 需求/设计 | 描述 | 步骤要点 | 验证方式 | 预期结果 | 实现映射 |
|----|-----------|------|----------|----------|----------|----------|
| TC-A2-1 | US-A2 / design §2.2 | 0/2/3 约定 | 成功、缺参、不存在 id | CLI/单元 | 0 / 2 / 3 | `xtask/src/tests/todo/todo_error.rs`、`todo_cmd/crud.rs` |
| TC-A2-2 | US-A2 | id 0 → 参数错误 2 | complete/delete/update/show 对 id `0` | 集成 | `exit_code == 2` | `todo_cmd/crud.rs`（`*_id_zero_errors`） |
| TC-A2-3 | US-A2 | 不存在 id → 数据错误 3 | complete/delete/update/show | 集成 | `exit_code == 3` | `todo_cmd/crud.rs`（`*_nonexistent`） |
| TC-A2-4 | US-A2 / design §2.2 | 非 todo 子命令失败多为 1 | fmt/clippy 失败场景（若可模拟） | 手工/CI | 退出码 1 | 视 CI 策略 |

### US-A3：init-ai

| ID | 需求/设计 | 描述 | 步骤要点 | 验证方式 | 预期结果 | 实现映射 |
|----|-----------|------|----------|----------|----------|----------|
| TC-A3-1 | US-A3 | 生成技能/命令文件 | `todo init-ai`，可选 `--for-tool`、`--output` | CLI/集成 | 目标目录生成文件，含子命令与 `--json`/退出码说明 | `xtask/src/tests/todo/todo_cmd/json_dry_init.rs`（init-ai 相关） |

### US-A4：dry-run

| ID | 需求/设计 | 描述 | 步骤要点 | 验证方式 | 预期结果 | 实现映射 |
|----|-----------|------|----------|----------|----------|----------|
| TC-A4-1 | US-A4 / design §2.1 | 修改类不写盘、不改内存列表 | add/update/complete/delete + `--dry-run` 后查 `.todo.json` 与再次 list | 集成 | 文件与列表未变；输出含拟执行说明 | `todo_cmd/json_dry_init.rs` |

---

## 2. Todo 领域（requirements §3）

### US-T1 创建

| ID | 需求 | 描述 | 验证方式 | 预期结果 | 实现映射 |
|----|------|------|----------|----------|----------|
| TC-T1-1 | US-T1 | 非空标题创建成功 | 单元/集成 | `Ok(TodoId)`，list 含该项 | `crates/todo/src/tests/crud.rs` |
| TC-T1-2 | US-T1 | 空标题拒绝 | 单元 | `InvalidInput`，条数不变 | `crates/todo/src/tests/crud.rs` |
| TC-T1-3 | US-T1 / §3.2 | CLI 空标题退出码 2 | `cargo xtask todo add ""` | 集成 | 退出码 2 | `xtask` todo_cmd 测试 |

### US-T2 列表

| ID | 需求 | 描述 | 验证方式 | 预期结果 | 实现映射 |
|----|------|------|----------|----------|----------|
| TC-T2-1 | US-T2 | 空列表 | 单元 | 空 `Vec` / 提示 | `crates/todo/src/tests/crud.rs` |
| TC-T2-2 | US-T2 | 多条顺序 | 单元 | 与创建顺序一致 | `crates/todo/src/tests/crud.rs`、`list_options.rs` |

### US-T3 / US-T4 完成与删除

| ID | 需求 | 描述 | 验证方式 | 预期结果 | 实现映射 |
|----|------|------|----------|----------|----------|
| TC-T3-1 | US-T3 | complete 后 completed | 单元 | `completed` true，`completed_at` Some | `crates/todo/src/tests/crud.rs` |
| TC-T3-2 | US-T3 | 不存在 id | 单元 | `NotFound` | `crates/todo/src/tests/crud.rs` |
| TC-T4-1 | US-T4 | delete 后消失 | 单元 | `get` None | `crates/todo/src/tests/crud.rs` |
| TC-T4-2 | US-T4 | 不存在 id | 单元 | 与设计一致（Err） | `crates/todo/src/tests/crud.rs` |

### US-T5 时间戳与展示

| ID | 需求 | 描述 | 验证方式 | 预期结果 | 实现映射 |
|----|------|------|----------|----------|----------|
| TC-T5-1 | US-T5 | created_at / completed_at | 单元 | 字段符合模型 | `crates/todo/src/tests/crud.rs` |
| TC-T5-2 | US-T5 / design §2.4 | list 人类可读含创建/完成/用时 | CLI/手工 | 输出含相对时间与已完成用时 | `xtask/src/todo/format.rs` 测试或手工 |

### US-T6 长期未完成着色

| ID | 需求 | 描述 | 验证方式 | 预期结果 | 实现映射 |
|----|------|------|----------|----------|----------|
| TC-T6-1 | US-T6 / design §2.4 | TTY 超 7 天未完成着色 | 手工 | ANSI 黄色 | `format.rs` 中 `AGE_THRESHOLD_DAYS`、`is_old_open` |
| TC-T6-2 | US-T6 | 非 TTY 无颜色 | `list \| cat` | 无 `\x1b[` | 手工 |

### US-T7～T11 扩展

| ID | 需求 | 描述 | 实现映射 |
|----|------|------|----------|
| TC-T7-1 | US-T7 | show 有效 id | `xtask/todo_cmd/crud.rs`、`crates/todo/src/tests/advanced.rs` |
| TC-T7-2 | US-T7 | show 无效 id | 同上 |
| TC-T8-1 | US-T8 | update 字段与持久化 | `advanced.rs`、`xtask/todo_cmd` |
| TC-T8-2 | US-T8 | `--clear-repeat-rule` | `priority_repeat.rs`、`advanced.rs` |
| TC-T9-1 | US-T9 | add/update 可选字段 | `list_options.rs`、`todo_cmd_io.rs` |
| TC-T9-2 | US-T9 | list 过滤排序 | `crates/todo/src/tests/list_options.rs`、`xtask/.../list_options.rs` |
| TC-T9-3 | US-T9 / §3.2 | 非法 `--status` / `--due-before` 等 | 集成 | 退出码 2 | `xtask/.../list_options.rs`（`cmd_todo_list_invalid_status_*`、`invalid_due_before_*`） |
| TC-T10-1 | US-T10 | search 命中 | `advanced.rs`、`xtask` |
| TC-T10-2 | US-T10 | search 空 | 同上 |
| TC-T11-1 | US-T11 | stats 计数 | `advanced.rs`、`xtask` |

### US-T12 导入导出

| ID | 需求 | 描述 | 实现映射 |
|----|------|------|----------|
| TC-T12-1 | US-T12 | export JSON/CSV | `xtask/todo_cmd_io.rs` 或 `json_dry_init` |
| TC-T12-2 | US-T12 | import 合并 | `todo_cmd_io.rs` |
| TC-T12-3 | US-T12 / §3.2 | `import --replace` 替换列表 | 集成 | 仅导入文件中的任务 | `xtask/.../todo_cmd_io.rs`（`cmd_todo_export_and_import_merge_replace`） |

### US-T13 重复任务

| ID | 需求 | 描述 | 实现映射 |
|----|------|------|----------|
| TC-T13-1 | US-T13 | 完成后生成下一实例 | `crates/todo/src/tests/priority_repeat.rs` |
| TC-T13-2 | US-T13 | `--no-next` | `priority_repeat.rs`、`xtask/todo_cmd` |
| TC-T13-3 | US-T13 | 2d/3w/custom:N | `priority_repeat.rs`、`repeat.rs` 测试 |
| TC-T13-4 | US-T13 | repeat_until / repeat_count 终止 | `priority_repeat.rs` |
| TC-T13-5 | US-T13 | show/update 规则与 clear | `advanced.rs`、CLI |
| TC-T13-6 | US-T13 | 非法 repeat_count | CLI 退出码 2 | `todo_error.rs` |
| TC-T13-7 | US-T13 | add 带 repeat 端到端 | `xtask/tests/integration.rs`（`xtask_todo_add_with_repeat_options_then_list`） |

### 日期与参数校验（requirements §3.2）

| ID | 需求 | 描述 | 实现映射 |
|----|------|------|----------|
| TC-DATE-1 | §3.2 | 非法 `--due-date` / `--repeat-until` | 退出码 2 | `todo_cmd_io.rs`、`todo_error.rs` |
| TC-DATE-2 | §3.2 | 非法 `--due-before` / `--due-after` | 退出码 2 | `list_options.rs` |
| TC-PRI-1 | §3.2 | 非法 `--priority` | 退出码 2 | `todo_error.rs` |

### 库层集成（design §2.1 / §3.1）

| ID | 设计 | 描述 | 实现映射 |
|----|------|------|----------|
| TC-LIB-1 | design §2.1 | `InMemoryStore::from_todos` 后 CRUD | `crates/todo/tests/integration.rs` |
| TC-LIB-2 | design §3.1 | `TodoId` 0 非法 | `id`/`list` 测试 | `crates/todo/src/tests/crud.rs` |

---

## 3. `cargo xtask todo` 端到端（requirements §3 + US-X4）

| ID | 需求 | 描述 | 实现映射 |
|----|------|------|----------|
| TC-X4-1 | US-X4 | add + list 持久化 | `xtask/tests/integration.rs`（`xtask_todo_add_then_list_shows_task`） |
| TC-X4-2 | US-X4 | complete/delete 写回 `.todo.json` | `xtask/todo_cmd/crud.rs` + 临时目录 |
| TC-X4-3 | US-X4 | 数据文件 `.todo.json` | 集成测试 `current_dir` + 文件存在 | `integration.rs`、`todo_cmd` |

---

## 4. 其他 `cargo xtask` 子命令（requirements §4 + design §3.2）

### US-X1～X3

| ID | 需求 | 描述 | 实现映射 |
|----|------|------|----------|
| TC-X1-1 | US-X1 | `--help` 列出子命令 | 手工：`cargo xtask --help` |
| TC-X1-2 | US-X1 | 子命令退出码 | 各集成测试 | `xtask/tests/integration.rs` 等 |
| TC-X2-1 | US-X2 | `run` 成功 | `xtask/tests/integration.rs`（`xtask_run_exits_success`） |
| TC-X2-2 | US-X2 | run 失败场景 | 手工/视项目约定 | N/A 可标 |
| TC-X3-1 | US-X3 | 新子命令注册模式 | 代码评审 | 新增时补集成测试 |

### 工具子命令（与设计 `XtaskSub` 一致）

| ID | 设计/需求 | 描述 | 验证方式 | 实现映射 |
|----|-----------|------|----------|----------|
| TC-X-CLIPPY-1 | design §3.2 | `clippy` 调用可测试逻辑 | 单元（mock/稀疏） | `xtask/src/tests/clippy.rs` |
| TC-X-CLEAN-1 | design §3.2 | `clean` | 单元 | `xtask/src/tests/clean.rs` |
| TC-X-GIT-1 | design §3.2 | `git` 子命令解析/行为 | 单元 | `xtask/src/tests/git.rs` |
| TC-X-GIT-2 | requirements §4 / design §1.6 | **`.githooks/pre-commit`** 与 **`cargo xtask git pre-commit`** 含 **`cargo check -p xtask-todo-lib --target x86_64-pc-windows-msvc`**；需 **`rustup target`** | 手工 / CI 跑完整 pre-commit；或单独 **`cargo check -p xtask-todo-lib --target x86_64-pc-windows-msvc`** | `.githooks/pre-commit`；`xtask/src/git.rs` |
| TC-X-ACC-1 | requirements §4 / acceptance §8 | **`cargo xtask acceptance`** 生成报告且退出码 **0**（失败检查为 **1**） | 仓库根执行；或 **`--stdout-only`** | `xtask/src/acceptance/` |
| TC-X-GH-1 | design §3.2 | `gh` 相关 | 单元（无 gh 时可跳过） | `xtask/src/tests/gh.rs` |
| TC-X-FMT-1 | design §3.2 | `fmt` | CI / 手工 | **待补充** 专用测试或依赖 `cargo fmt --check` |
| TC-X-COV-1 | design §3.2 | `coverage` | CI | 手工或 CI 脚本 |
| TC-X-PUB-1 | design §3.2 | `publish` | 手工 | `docs/publishing.md`；不自动发版 |

---

## 5. Devshell（requirements §1.1 + §5 + design §1.4～§2.5）

### 5.1 启动、持久化、CLI（requirements §5.3）

| ID | 需求/设计 | 描述 | 实现映射 |
|----|-----------|------|----------|
| TC-D0-1 | §5.3 | 非法 CLI 参数非零退出 | `crates/todo/tests/integration.rs`（`cargo_devshell_usage_error_exits_nonzero`） |
| TC-D0-2 | §1.1 / design §2.3 | 会话与工作区：VFS 序列化、`session_store`（**`GuestPrimarySessionV1`** / **`devshell_session_v1`**）、**工作区内** `session.json` 路径 | `session_store` 测试、`run_main.rs`、`serialization.rs` |
| TC-D0-3 | §5.3 | `-f script` / `-e` | `run_main.rs` |
| TC-D0-4 | §1.2 / §7.1 | **Windows**：**`xtask-todo-lib`** 库目标可编译（**rustyline**、桩 **`vm_workspace_host_root`** 等）；**不**要求在本仓库 CI 中启动真实 Podman/Windows 进程。全链路 **`cargo devshell` + β + `cargo new`/`cargo run`** 见 **§5.10**（手工）。交叉 **`cargo check --target x86_64-pc-windows-msvc`** 或 **TC-NF-5** | `crates/todo` **`cfg`** / **`vm/mod.rs`** |

### 5.2 VFS（requirements §5.4 + design vfs）

| ID | 需求 | 描述 | 实现映射 |
|----|------|------|----------|
| TC-D-VFS-1 | §5.4 | mkdir / cd / pwd / ls | `devshell/tests/run_basic.rs`、`vfs/tests.rs` |
| TC-D-VFS-2 | §5.4 | 读写文件、路径解析 | `vfs/tests.rs`、`run_basic.rs` |

### 5.3 内置命令（requirements §5.4）

| ID | 需求 | 描述 | 实现映射 |
|----|------|------|----------|
| TC-D-BUILTIN-1 | §5.4 | cat/touch/echo/save/help | `run_basic.rs`、`run_io.rs` |
| TC-D-BUILTIN-2 | §5.4 | export-readonly | `devshell/tests/run_io.rs`（`run_with_export_readonly`） |
| TC-D-BUILTIN-3 | §5.4 | 管道 `\|` | `run_io.rs`、`parser.rs` 测试 |
| TC-D-BUILTIN-4 | §5.4 | 重定向 `<` `>` `2>` | `run_io.rs` |
| TC-D-BUILTIN-5 | §5.4 | 解析错误可读 | `parser.rs` 测试 |

### 5.4 内置 `todo` 子集

| ID | 需求 | 描述 | 实现映射 |
|----|------|------|----------|
| TC-D-TODO-1 | §5.4 | list/add/show/update/complete/delete/search/stats | `devshell/tests/run_todo.rs` |
| TC-D-TODO-2 | §5.4 | 与 `.todo.json` 约定一致 | `run_todo.rs`、`todo_io.rs` |

### 5.5 脚本（requirements §5.5）

| ID | 需求 | 描述 | 实现映射 |
|----|------|------|----------|
| TC-D-SCRIPT-1 | §5.5 | 变量、if/for/while | `devshell/script/tests.rs` |
| TC-D-SCRIPT-2 | §5.5 | `set -e`、续行、注释 | `script/tests.rs` |
| TC-D-SCRIPT-3 | §5.5 | source / 嵌套深度 | `script/tests.rs`、`run_basic.rs`（source） |

### 5.6 REPL：`source` / `.`（requirements §5.5）

| ID | 需求 | 描述 | 实现映射 |
|----|------|------|----------|
| TC-D-REPL-1 | §5.5 | `source path`、`. path` | `repl.rs` 测试、`run_basic.rs` |

### 5.7 Tab 补全（requirements §5.6 + design §4.3）

| ID | 需求/设计 | 描述 | 实现映射 |
|----|-----------|------|----------|
| TC-D-COMP-1 | §5.6 | 命令名补全 | `devshell/completion.rs` 测试 |
| TC-D-COMP-2 | §5.6 / design §4.3 | 路径补全；`src/` 前缀保留 | `complete_path_trailing_slash_keeps_parent_in_candidate` 等 | `completion.rs` |
| TC-D-COMP-3 | design §4.3 | `CompletionType::List`（非 Circular 回退） | 手工 REPL 或配置断言 | `repl.rs`（集成配置）；**行为**以代码为准 |

### 5.8 Rust 沙箱（requirements §5.4 + design §2.5）

| ID | 需求/设计 | 描述 | 实现映射 |
|----|-----------|------|----------|
| TC-D-SBX-1 | §5.4 / §2.5 | 导出→临时目录→同步 | `devshell/sandbox.rs` 单元测试（export/sync） |
| TC-D-SBX-2 | design §2.5 | 嵌套 VFS 路径 `host_export_root` 与 `copy_tree_to_host` 一致 | `nested_vfs_path_host_uses_leaf_dir_not_full_path` | `sandbox.rs` |
| TC-D-SBX-3 | §5.4 | PATH 无 cargo/rustup 时错误提示 | 手工或隔离 PATH | **可选** |
| TC-D-SBX-4 | dev-container.md | Linux `DEVSHELL_RUST_MOUNT_NAMESPACE=1` 下 `run_in_export_dir` 可启动子进程 | `run_in_export_dir_true_with_mount_namespace`（Linux，`#[ignore]`，需特权环境：`cargo test … -- --ignored`） | `sandbox.rs` |

### 5.9 序列化（design §1.4）

| ID | 设计 | 描述 | 实现映射 |
|----|------|------|----------|
| TC-D-SER-1 | serialization | Vfs ↔ 快照 round-trip | `devshell/serialization.rs` 内测试 |

### 5.10 VM / β 侧车（`devshell-vm` + requirements §5.8）

与 **Lima（γ）** 的 **`limactl`** 集成测试不同，β 侧车为 **独立 crate**（**`crates/devshell-vm`**），在 **Linux** 上跑单元测试；**Windows** 上 **`cargo devshell` + Podman + OCI** 以 **手工 / 环境** 验证为主（见 **TC-D-VM-4**）。

| ID | 需求/设计 | 描述 | 验证方式 | 预期结果 | 实现映射 |
|----|-----------|------|----------|----------|----------|
| TC-D-VM-1 | design §1.4 / requirements §5.8 | **`exec`** 含 **`--devshell-vm-test-fail`** 时返回 **`exit_code: 1`**（不依赖真实 `cargo`） | 单元 | JSON 一行可解析，`exec_result` | `crates/devshell-vm/src/tests.rs`（`handle_exec_fail_flag`） |
| TC-D-VM-2 | requirements §5.8 | **`session_start`** 后 **`exec`** 在 **`staging_dir`** 上执行子进程并写宿主文件 | 单元（Unix：`sh` 写 `marker.txt`） | `exec_result` 成功，宿主路径存在文件 | `crates/devshell-vm/src/tests.rs`（`handle_exec_runs_subprocess_in_staging_dir`） |
| TC-D-VM-3 | design §1.4 | **`guest_fs`** 在 **`session_start`** 后读宿主 **`staging_dir`** 内文件 | 单元 | `guest_fs_ok`、内容与基64 一致 | `crates/devshell-vm/src/tests.rs`（`guest_fs_reads_host_file_after_session_start`） |
| TC-D-VM-4 | requirements §5.8 / §7.1 | **Windows + Podman**：Mode P、**`DEVSHELL_VM_BACKEND=beta`**（默认），OCI 或 ELF 侧车；**`cargo new --bin hello`** 后 **`cd hello`**，**`cargo run`** 成功，宿主工作区出现工程；**无**「sidecar response is not JSON」类错误（子进程 stdout 不得污染侧车协议 stdout） | **手工**（需 Podman Machine、网络拉镜像等） | 与 **requirements §5.8**「stdio 与程序输出」一致 | 手工；日志样例见仓库内 **`006_win.log`** 类记录 |
| TC-D-VM-5 | design §1.1 图 | **`devshell-vm`** crate 与 **`handle_line`** 可编译、**`cargo test -p devshell-vm`** 通过 | CI / 本地 | 全绿 | `crates/devshell-vm`；**`acceptance`** 编排含该包测试（见 **TC-X-ACC-1**） |
| TC-D-VM-6 | design §1.4 | **TCP**：子进程 **`devshell-vm --serve-tcp`** + **`TcpStream`** 做 **handshake → session_start → exec**（**Unix**，**`true`**）；避免同进程 **`accept`/`connect`** 死锁 | 集成测试 **`cargo test -p devshell-vm`** | 通过 | **`crates/devshell-vm/tests/tcp_subprocess.rs`** |

---

## 6. 设计决策专项验证（design §4）

| ID | 设计 | 描述 | 实现映射 |
|----|------|------|----------|
| TC-DES-4.1 | §4.1 持久化分离 | 库无内嵌 `.todo.json` I/O；xtask/io 负责 | 架构/代码评审；`crates/todo` 无 std::fs todo 文件 |
| TC-DES-4.2 | §4.2 devshell 与 xtask | xtask 二进制不内嵌 REPL | `xtask` crate 依赖不包含 repl 入口 |
| TC-DES-4.3 | §4.3 补全 | 见 TC-D-COMP-* | `completion.rs`、`repl.rs` |
| TC-DES-4.4 | §4.4 脚本变量作用域 | 脚本不污染下一条 REPL 行 | `script/tests.rs` 或手工 |
| TC-DES-4.5 | §1.4 β 侧车 | **`exec`** 子进程 stdout/stderr 与 JSON 协议分离（见 **TC-D-VM-2**、**TC-D-VM-4**） | `crates/devshell-vm/src/server.rs`；回归 **TC-D-VM-4** |

---

## 7. 非功能（requirements §7）

| ID | 需求 | 描述 | 验证方式 | 实现映射 |
|----|------|------|----------|----------|
| TC-NF-1 | §7 | Workspace members | 检查根 `Cargo.toml` | 手工 |
| TC-NF-2 | §7 | 无需全局 cargo-xtask | `.cargo/config.toml` alias | 手工 |
| TC-NF-3 | §7 | 人类错误 stderr；json 错误结构 | 抽样 CLI | `todo_cmd` 测试 |
| TC-NF-4 | §7 | 列表颜色仅 TTY | 见 TC-T6 | `format.rs` |
| TC-NF-5 | §1.2 / §7.2 | **`xtask-todo-lib`** 对 **`x86_64-pc-windows-msvc`** 可 **`cargo check`**（交叉编译） | 开发者：`rustup target add x86_64-pc-windows-msvc` 后执行命令 | 与 **TC-X-GIT-2**、pre-commit 最后一步一致 |
| TC-NF-6 | §7.1 | **`cargo install xtask-todo-lib`** 在 Windows 上可用（版本以 **README** 为准） | 发布前验证或用户环境 | `crates/todo/README.md`；crates.io 版本 |

---

## 8. 代码 ↔ 用例映射（主索引）

| 路径 | 覆盖用例范围 |
|------|----------------|
| `crates/todo/src/tests/crud.rs` | TC-T1～T5、T3、T4、TC-LIB-2 |
| `crates/todo/src/tests/list_options.rs` | TC-T9-2、过滤排序 |
| `crates/todo/src/tests/advanced.rs` | TC-T7～T11、T8、T13 部分 |
| `crates/todo/src/tests/priority_repeat.rs` | TC-T13-* |
| `crates/todo/tests/integration.rs` | TC-LIB-1、TC-D0-1（devshell usage） |
| `xtask/src/tests/todo/todo_cmd/crud.rs` | TC-X4、TC-A2、TC-T7～T8 CLI |
| `xtask/src/tests/todo/todo_cmd/list_options.rs` | TC-T9-2、TC-DATE-2 |
| `xtask/src/tests/todo/todo_cmd/json_dry_init.rs` | TC-A1、TC-A3、TC-A4 |
| `xtask/src/tests/todo/todo_cmd_io.rs` | TC-T12、TC-DATE-1、export/import |
| `xtask/src/tests/todo/todo_error.rs` | TC-A2、TC-PRI-1、TC-T13-6 |
| `xtask/tests/integration.rs` | TC-X2-1、TC-X4-1、TC-T13-7 |
| `xtask/src/tests/clippy.rs`、**clean.rs**、**git.rs**、**gh.rs** | TC-X-CLIPPY-1、TC-X-GIT-1 等 |
| **`.githooks/pre-commit`**（非 Rust 测试） | TC-X-GIT-2、TC-NF-5（完整 pre-commit / MSVC 检查） |
| **`cargo xtask acceptance`** | TC-X-ACC-1 |
| `crates/todo/src/devshell/tests/run_basic.rs` | TC-D-VFS、TC-D-BUILTIN、管道基础 |
| `crates/todo/src/devshell/tests/run_io.rs` | TC-D-BUILTIN-3/4/5 |
| `crates/todo/src/devshell/tests/run_todo.rs` | TC-D-TODO-* |
| `crates/todo/src/devshell/tests/run_main.rs` | TC-D0-2/3 |
| `crates/todo/src/devshell/vfs/tests.rs` | TC-D-VFS-* |
| `crates/todo/src/devshell/parser.rs`（#[test]） | 解析、管道 token |
| `crates/todo/src/devshell/script/tests.rs` | TC-D-SCRIPT-* |
| `crates/todo/src/devshell/completion.rs`（#[test]） | TC-D-COMP-* |
| `crates/todo/src/devshell/sandbox.rs`（#[test]） | TC-D-SBX-* |
| `crates/todo/src/devshell/serialization.rs`（#[test]） | TC-D-SER-1 |
| `crates/devshell-vm/src/server.rs`、`crates/devshell-vm/src/tests.rs` | TC-D-VM-1～3、TC-DES-4.5 |
| **`crates/devshell-vm/tests/tcp_subprocess.rs`** | TC-D-VM-6 |
| **Windows 手工**（Podman + `cargo devshell`） | TC-D-VM-4 |
| `crates/todo/src/devshell/repl.rs`（#[test]） | process_line、脚本入口 |
| `xtask/src/todo/format.rs` | TC-T5-2、TC-T6、TC-NF-4（逻辑） |

---

## 9. 维护说明

1. **新增需求**（requirements.md）：在本文档增加至少一条用例，或标注 **N/A / 待补充** 及原因。  
2. **新增设计决策**（design.md）：在 **design §4～§5** 增加 **TC-DES-*** 或 devshell/xtask 用例。  
3. **新增自动化测试**：更新 §8 映射表。  
4. **与 acceptance.md**：若验收表中有独立 ID，可在用例表「需求引用」列双向注明。

---

## 10. 明确不覆盖（requirements §2「当前不承诺」）

以下不在当前测试用例范围内（按需补充）：HTTP API、多用户权限、`.todo.json` 自动迁移流水线、并发写同一文件的强保证等。
