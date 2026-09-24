# Story 7.1：需求—设计—用例—验收追溯

Status: done

<!-- Ultimate context engine analysis completed — comprehensive developer guide created -->

## Story

作为一名维护者，  
我希望在**仓库文档体系**中对照 **FR、设计、用例 ID 与验收 ID**，  
以便变更可审计（**FR32**）。

## 映射需求

- **FR32**：维护者可追溯需求、设计、用例与验收（以 **`docs/`** 与约定编号为准）。

## Acceptance Criteria

1. **Given** 四条主线文档：**`docs/requirements.md`**、**`docs/design.md`**、**`docs/test-cases.md`**、**`docs/acceptance.md`**  
   **When** 从 **`requirements`** 某章节（如 **§3 Todo**、**§5 Devshell**）出发  
   **Then** 可在 **`test-cases.md`** 找到对应 **TC-** 或章节引用，或在 **`test-cases.md` §0「追溯索引」** 中显式映射；**无映射**的项须标 **SKIP** 或「手工」（**FR32**）。

2. **Given** **`test-cases.md`** 中一条用例（含 **ID**、**需求/设计引用** 列）  
   **When** 对照 **`acceptance.md`**  
   **Then** 存在可解释的对应关系（例如 **US-T** / **D*** 与 **T*-*** / **NF-***）；若验收表为概括性描述，在 **`acceptance.md` 前言**或 **`test-cases.md`** 中已有交叉引用说明（**FR32**）。

3. **Given** **`design.md`** 中关键决策（VM、Mode S/P、β 侧车等）  
   **When** 对照 **`requirements.md`**  
   **Then** **不**存在不可调和矛盾；若故意「设计超前于需求」，须有 **脚注或 `test-cases` 中显式 SKIP**（**FR32**）。

4. **Given** **`_bmad-output/planning-artifacts/epics.md`**（或 **FR Coverage Map**）中的 **FR1–FR34**  
   **When** 抽查至少 **N 个 FR**（**N ≥ 5**，含跨 Epic）  
   **Then** 每条可指向 **`requirements`** 段落或 **`test-cases`/`acceptance`** 中可验证表述；**无法**指向时记入 **「追溯缺口」清单**并在本故事内**修复文档**或**记为已知限制**（**FR32**）。

5. **棕地**：**`test-cases.md` §0** 已提供需求/设计 → 章节索引；**`acceptance.md`** 已链到 **`test-cases.md`**。本故事以 **审计、补链、统一编号用语** 为主，**不**要求重写全部文档。

6. **回归**：文档变更后 **`cargo xtask acceptance --stdout-only`**（或 **`cargo test`**）仍通过；若仅改 Markdown，**无**代码回归义务，但须在故事中**列出**审阅过的路径。

## Tasks / Subtasks

- [x] **走查**：自 **`requirements §3`** 起，逐段在 **`test-cases.md`** / **`acceptance.md`** 打勾或记缺口。
- [x] **§0 索引**：若 **`test-cases.md` §0** 缺 **§6 AI** 以外的新增章节，补一行映射。
- [x] **FR 抽样**：从 **`epics.md`** 选 **5+** FR，填 **「FR → 文档锚点」** 小表（可放在本故事 **Dev Agent Record** 或 **`docs/`** 附录 — **最小**新增）。
- [x] **SKIP 规范**：统一「⏸」「SKIP」「手工」在 **`acceptance.md`** 与 **`test-cases.md`** 中的用语（若当前已一致则 **no-op**）。
- [x] **验证**：若改动了与自动化相关的 **`acceptance`** 描述，运行 **`cargo xtask acceptance`** 或按 **`docs/acceptance.md` §8** 核对。

## Dev Notes

### 棕地现状（摘录）

| 文档 | 作用 |
|------|------|
| **`requirements.md`** | 能力与不承诺、章节索引 **§9** |
| **`design.md`** | 架构与关键决策 |
| **`test-cases.md`** | **TC-** ID、**§0 追溯索引**、**实现映射** 列 |
| **`acceptance.md`** | 可勾选验收、**`cargo xtask acceptance`** |

### 架构合规（摘录）

- **追溯**以 **`docs/`** 为权威；**`_bmad-output/`** 计划文档为**辅助**，冲突时以 **`requirements`** 为准。

### 前序故事

- **Epic 1–6** 已建立实现；本故事为**文档层**闭环，**不**替代功能开发。

### 参考资料

- [Source: `_bmad-output/planning-artifacts/epics.md` — Epic 7 Story 7.1]
- [Source: `docs/requirements.md` — §9 章节索引]
- [Source: `docs/test-cases.md` — §0 追溯索引]
- [Source: `docs/acceptance.md` — §1 验收说明]
- [Source: `docs/design.md`]

## Dev Agent Record

### Agent Model Used

gpt-5.3-codex

### Debug Log References

- `cargo xtask acceptance --stdout-only`

### Completion Notes List

- 已完成文档链路走查：`requirements.md §3/§5/§6/§7/§9` → `test-cases.md §0/§1~§8` → `acceptance.md §1/§3/§4/§5/§8`，均可找到对应章节或“人工/环境”说明，未发现不可调和断链。
- `test-cases.md §0` 索引已覆盖 `requirements` 主章节（含 §6 AI、§7 非功能、§1.2/§7.2 平台与 pre-commit），本故事对此项为 **no-op**，无需额外补行。
- FR 抽样（跨 Epic，N=6）：
  - `FR3` → `requirements.md §3.1/§3.2`；`test-cases.md TC-T9-*`；`acceptance.md §2.2 T2-2`
  - `FR12` → `requirements.md §4`；`test-cases.md TC-X-ACC-1`；`acceptance.md §8`
  - `FR22` → `requirements.md §5.8`；`test-cases.md TC-D-VM-1~4/6/7`；`acceptance.md §4 D9`
  - `FR27` → `requirements.md §3.2/§6 US-A4`；`test-cases.md TC-A4-1`；`acceptance.md §3.2 A4`
  - `FR31` → `requirements.md §2/§4/§7.1`；`docs/publishing.md §6`；`acceptance.md NF-3/NF-4（人工）`
  - `FR34` → `requirements.md §2（当前不承诺）`；`test-cases.md §10`；`acceptance.md §5 NF-3/NF-4（评审）`
- SKIP/手工用语核对：`acceptance.md` 已使用“⏸ 跳过（注明原因）”并在 §8 明确自动化不覆盖项；`test-cases.md` 中“手工/可选/待补充”语义与之可解释一致，本故事不做强制词形统一改写。
- 自动化核对通过：`cargo xtask acceptance --stdout-only` 全绿，报告含 `NF-1/2/5/6` 与 `AC-*` 覆盖表，同时保留人工项清单，满足 AC6。

### File List

- `_bmad-output/planning-artifacts/epics.md`（FR 抽样来源）
- `docs/requirements.md`（走查 §3/§5/§6/§7/§9）
- `docs/design.md`（与需求对照）
- `docs/test-cases.md`（§0 索引与 TC 映射）
- `docs/acceptance.md`（与用例、自动化对照）
- `docs/publishing.md`（FR31 抽样引用）
- `_bmad-output/implementation-artifacts/7-1-requirements-design-traceability.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

### Review Findings

- [x] [Review][Patch] AC6「审阅路径」清单不完整 — `File List` 原先未列出 `docs/` 下走查文件；已补全上表路径（本轮审查）。

## Change Log

- **2026-03-26**：BMad 并行审查通过；补全 **File List** 以满足 AC6；**Status → done**。

## 完成状态

- [x] 所有 AC 已验证（自动化或记录手工步骤）
- [x] 文档与实现一致或可解释差异已记录
