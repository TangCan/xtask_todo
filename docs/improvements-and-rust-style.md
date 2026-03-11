# 项目改进建议与 Rust 编码规范

## 一、项目分析概览

当前项目结构清晰：workspace（`crates/todo` + `xtask`）、文档齐全（requirements / design / tasks / test-cases / acceptance）、OpenSpec 与实现一致。下面从工程实践与代码质量角度给出改进建议，并说明 Rust 项目常用的编码规范。

---

## 二、改进建议

### 2.1 代码质量与规范

| 建议 | 说明 | 优先级 |
|------|------|--------|
| **启用 Clippy 并修复告警** | 当前 `cargo clippy -W clippy::pedantic` 会报 doc 反引号、`#[must_use]`、`# Errors`、`format!` 内联等；建议在 CI 中 `cargo clippy -- -D warnings`，并逐步修复或按需 `allow`。 | 高 |
| **统一格式化** | 使用 `cargo fmt` 并提交前检查（或 pre-commit）；可选在根目录添加 `rustfmt.toml` 固定行宽等。 | 高 |
| **公开 API 文档** | 为 `create` / `complete` / `delete` 等返回 `Result` 的接口补充 `# Errors` 小节；类型名在 doc 中用反引号（如 `` `TodoId` ``）。 | 中 |
| **`#[must_use]`** | 对 `new()`、`from_todos()`、`as_u64()` 等纯返回值、无副作用的函数加上 `#[must_use]`，避免忽略返回值。 | 中 |

### 2.2 工程与协作

| 建议 | 说明 | 优先级 |
|------|------|--------|
| **根目录 README** | 增加简短 README：项目目的、`cargo xtask todo add/list/complete/delete` 用法、文档入口（docs/）、如何跑测试。 | 高 |
| **CI 流水线** | 使用 GitHub Actions 或 GitLab CI：`cargo build`、`cargo test`、`cargo fmt -- --check`、`cargo clippy -- -D warnings`；可选 `cargo doc --no-deps`。 | 高 |
| **依赖版本** | 在根或各 crate 的 `Cargo.toml` 中为关键依赖写清版本（含 minor），便于复现与安全审计。 | 中 |
| **pre-commit** | 可选配置 pre-commit：`cargo fmt`、`cargo clippy`（或仅 `cargo check`），保证提交前通过。 | 低 |

### 2.3 测试与可维护性

| 建议 | 说明 | 优先级 |
|------|------|--------|
| **集成测试** | 在 `crates/todo/tests/` 增加少量集成测试（如从文件加载、保存、再加载），覆盖 xtask 使用的 DTO 与 `from_todos` 路径。 | 中 |
| **xtask 测试** | 对 `cargo xtask todo` 的 add/list/complete/delete 可写简单端到端测试（临时目录 + 调用二进制），或至少在 CI 中跑一遍命令。 | 低 |
| **错误处理** | 考虑为 `TodoError` 实现 `From` 或与 `thiserror`/`anyhow` 集成，便于在 xtask 中统一错误输出与日志。 | 低 |

### 2.4 安全与健壮性

| 建议 | 说明 | 优先级 |
|------|------|--------|
| **.todo.json 路径** | 当前使用 `current_dir()`；若需多工作区隔离，可约定“项目根”为含 `Cargo.toml` 的目录，避免误写其他目录。 | 低 |
| **JSON 大小** | 若列表极大，可考虑流式或分页；当前规模一般无需处理。 | 低 |

---

## 三、Rust 编码规范（参考）

Rust 官方与社区普遍采用以下工具与约定，本项目可逐步对齐。

### 3.1 官方风格与工具

- **Rust Style Guide**（[doc.rust-lang.org/style-guide](https://doc.rust-lang.org/style-guide/)）  
  - 行宽 100 字符、缩进 4 空格、尾逗号、无行尾空格等。  
  - **rustfmt** 为该风格的自动格式化实现，建议所有 Rust 项目使用。

- **rustfmt**  
  - 命令：`cargo fmt`（格式化）、`cargo fmt -- --check`（仅检查，适合 CI）。  
  - 可选在项目根添加 `rustfmt.toml`，例如：
    ```toml
    edition = "2021"
    max_width = 100
    ```

- **Clippy**  
  - 命令：`cargo clippy`（默认 lints）、`cargo clippy -- -D warnings`（把警告当错误，适合 CI）。  
  - 常用分组：`correctness`（deny）、`suspicious`/`style`/`complexity`/`perf`（warn）、`pedantic`（按需开启）。  
  - 可在根 `Cargo.toml` 或 `.cargo/config.toml` 中配置 lints，或在代码中用 `#![allow(clippy::xxx)]` 局部放宽。

### 3.2 建议的本项目配置

1. **根目录 `rustfmt.toml`**（可选）  
   - 统一 `edition`、`max_width` 等，便于多人协作。

2. **CI 中必跑**  
   - `cargo fmt -- --check`  
   - `cargo clippy --all-targets -- -D warnings`  
   - `cargo test`

3. **文档与 API**  
   - 公开类型与函数写 doc comment；返回 `Result` 的要有 `# Errors`。  
   - 文档中的类型/函数名用反引号（`` `TodoId` ``）。  
   - 运行 `cargo doc --no-deps --open` 检查生成效果。

4. **依赖与版本**  
   - 关键依赖写清版本；可定期 `cargo update` 并跑测试。

### 3.3 单文件行数约定（本项目）

- **约定**：单个 `.rs` 文件不宜超过 **500 行**（不含空行与注释可按需放宽，但建议同一量级）。
- **超出时**：按职责拆分为子模块（`mod`）或拆成多个 `.rs` 文件，保持单文件可读、易导航、职责单一。
- **说明**：Rust 官方风格未规定文件行数上限；本约定为项目内可维护性考虑，便于评审与重构。

上述规范与改进可与 `openspec/project.md` 中的 Code Style、Testing Strategy 合并，形成团队统一的 Rust 编码约定。

---

## 四、与本项目文档的对应

- **需求/设计/任务/测试/验收**：已覆盖功能与验收；改进项多为工程与质量（CI、fmt、clippy、README）。  
- **OpenSpec project.md**：可在 “Code Style” 中增加“提交前需通过 `cargo fmt` 与 `cargo clippy`（或 CI 强制）”；“Testing Strategy” 中可注明 CI 跑 `cargo test` 与可选 xtask 端到端检查。

文档与实现不一致时，以实际代码与本文档的“改进建议”为准，并同步更新相关 doc。
