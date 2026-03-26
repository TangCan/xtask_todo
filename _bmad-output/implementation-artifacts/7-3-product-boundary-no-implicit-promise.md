# Story 7.3：产品能力边界（不默认承诺）

Status: ready-for-dev

<!-- Ultimate context engine analysis completed — comprehensive developer guide created -->

## Story

作为一名产品/工程角色，  
我希望系统与文档**不把** HTTP API、多租户、**`.todo.json` 自动迁移**等**未列入 PRD** 的能力当作默认承诺，  
以免范围 creep（**FR34**）。

## 映射需求

- **FR34**：**不得**将 PRD 未列能力**默认**承诺为 **HTTP API**、**多用户权限**、**`.todo.json` 自动迁移流水线** 等；若未来纳入须**显式增改 FR**（与 **`requirements §2`**「当前不承诺」、**`prd.md`** 一致）。

## Acceptance Criteria

1. **Given** **`docs/requirements.md` §2**「当前不承诺」列表  
   **When** 对照 **`_bmad-output/planning-artifacts/prd.md`** 中 **FR34** 表述  
   **Then** **语义一致**（终端/库优先、无隐式网络服务为产品边界）；若一方更窄或更宽，在本故事内**修订**并交叉引用（**FR34**）。

2. **Given** 对外入口：**根 `README.md`**、**`crates/todo/README.md`**（crates.io 用户可见）  
   **When** 全文检索 **API** / **server** / **REST** / **multi-tenant** / **sync service** / **migrate** 等营销用语  
   **Then** **不**暗示「安装即得托管服务 / 自动迁移」；若提及 **GitHub / crates.io / gh / Podman** 等，仅为**可选工具链**，与 **`requirements §2`** 不冲突（**FR34**）。

3. **Given** **`docs/design.md`** 或 **`architecture.md`** 中「迁移」「并发」「安全」相关段落  
   **When** 阅读  
   **Then** 与 **「未承诺自动迁移流水线」** 一致（**`architecture.md`** 已有迁移决策时可**引用**而非重复矛盾）（**FR34**）。

4. **Given** 代码面**抽查**（**非**全仓库形式化证明）：**无**默认监听 TCP/HTTP 服务端点作为产品核心路径（**`devshell-vm`** 侧车、**`gh`** 调用等**显式**、可选）  
   **When** 与 **FR34** 对照  
   **Then** 若发现 README/帮助文案暗示「内置云 API」，**最小**修正文案或增加 **§2** 链接（**FR34**）。

5. **棕地**：本故事以**文档审计与最小文案修正**为主；**不**在本故事中启动新功能（HTTP API 等）。

6. **回归**：若仅改 Markdown，**无**强制 **`cargo test`**；若触及 **`xtask`** 帮助字符串，运行 **`cargo test -p xtask`** 相关用例。

## Tasks / Subtasks

- [ ] **清单**：从 **FR34** + **§2** 抽出「禁止默认承诺」关键词表，用于 README/文档检索。
- [ ] **README 双文件**：根 **`README.md`** + **`crates/todo/README.md`** 各通读一节「边界」是否足够（不足则加**短**段落或链接 **§2**）。
- [ ] **PRD 对齐**：确认 **`prd.md` FR34** 与 **`requirements §2`** 无漂移。
- [ ] **记录**：在故事 **Completion Notes** 或 **`requirements §8`** 维护说明中记一次审计日期（**可选**）。

## Dev Notes

### 棕地现状（摘录）

| 文档 | 作用 |
|------|------|
| **`requirements.md` §2** | **当前不承诺** 权威列表 |
| **`prd.md`** | **FR34** 原文 |
| **`architecture.md`** | 边界与「无 HTTP API」架构立场 |

### 架构合规（摘录）

- **FR34** 是**产品/文档**约束；实现上保持 **CLI/库** 交付，**不**引入未立项网络服务。

### 前序故事

- **7.1**：追溯链；本故事确保**边界**在对外文档中可审计。

### 参考资料

- [Source: `_bmad-output/planning-artifacts/epics.md` — Epic 7 Story 7.3]
- [Source: `docs/requirements.md` — §2]
- [Source: `_bmad-output/planning-artifacts/prd.md` — FR34]
- [Source: `_bmad-output/planning-artifacts/architecture.md` — 边界/迁移]

## Dev Agent Record

### Agent Model Used

（实现时填写）

### Debug Log References

### Completion Notes List

### File List

## 完成状态

- [ ] 所有 AC 已验证（自动化或记录手工步骤）
- [ ] 文档与实现一致或可解释差异已记录
