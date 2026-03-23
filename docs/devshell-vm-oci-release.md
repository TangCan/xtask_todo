# Devshell-vm OCI 镜像发布（GHCR）

本文说明 **`containers/devshell-vm/Containerfile`** 对应的 **β 侧车镜像**如何随 **`xtask-todo-lib`** 版本发布到 **GitHub Container Registry (GHCR)**，以及它与 **`cargo-devshell`** 内默认镜像引用 **`ghcr.io/tangcan/xtask_todo/devshell-vm:v{CARGO_PKG_VERSION}`** 的对应关系。

---

## 1. 背景与目的

- Windows 上 **`cargo install xtask-todo-lib`** 的用户通常**没有**本仓库的 `target/.../devshell-vm` Linux ELF。
- 程序在找不到宿主机 ELF 时，会 **`podman pull`** 默认 OCI 镜像，再 **`podman run -i`** 执行 **`devshell-vm --serve-stdio`**（见 **`docs/devshell-vm-windows.md`**）。
- 因此每个 **`xtask-todo-lib`** 的 **crates.io 版本**最好有一个**同版本号标签**的镜像，便于 **`podman pull`** 与 `cargo install` 的 **`CARGO_PKG_VERSION`** 一致。

---

## 2. 代码中的默认镜像（与发布必须对齐）

在 **`crates/todo/src/devshell/vm/podman_machine.rs`** 中，未设置 **`DEVSHELL_VM_CONTAINER_IMAGE`** 时：

```text
ghcr.io/tangcan/xtask_todo/devshell-vm:v{CARGO_PKG_VERSION}
```

- **`CARGO_PKG_VERSION`** 来自 **`crates/todo/Cargo.toml`** 的 **`version`**（例如 `0.1.22`）。
- 镜像标签为 **`v` + 版本号**，例如 **`v0.1.22`**。
- **仓库路径**固定为 **`tangcan/xtask_todo`**（与仓库 **`repository`** 元数据一致）。若你在 **Fork** 上发布，需通过环境变量 **`DEVSHELL_VM_CONTAINER_IMAGE`** 指向你的镜像，或修改源码中的默认字符串（见下文 §8）。

---

## 3. 工作流位置与触发条件

**文件**：**`.github/workflows/release.yml`**

名称 **`Release`**，在以下情况运行：

| 触发方式 | 说明 |
|----------|------|
| **`push` 标签** | 匹配模式 **`xtask-todo-lib-v*`**（例如 **`xtask-todo-lib-v0.1.22`**） |
| **`workflow_dispatch`** | 手动运行，输入 **`tag_name`**（同上格式的完整标签名） |

同一工作流中定义 **两个** Job：

1. **`release`** — 打 GitHub Release、上传 **`.crate`** 包。
2. **`devshell-vm-oci`** — 构建并推送 GHCR 镜像（与 **`release`** **并行**，互不依赖）。

---

## 4. `devshell-vm-oci` Job 逐步说明

### 4.1 权限

```yaml
permissions:
  contents: read
  packages: write
```

- **`packages: write`**：用于向 **GHCR** 推送镜像（使用 **`GITHUB_TOKEN`** 登录）。

### 4.2 检出代码

- **`actions/checkout@v4`**，`ref` 为 **`github.event.inputs.tag_name || github.ref`**（手动运行时用手动输入的标签，否则为当前推送的标签）。
- 保证构建的 **`Containerfile`** 与 **`Cargo.lock`** 等上下文与**该标签**指向的提交一致。

### 4.3 从标签解析版本号与 Owner

```bash
REF="${{ github.event.inputs.tag_name || github.ref_name }}"
VERSION="${REF#xtask-todo-lib-v}"
OWNER=$(echo "${{ github.repository_owner }}" | tr '[:upper:]' '[:lower:]')
```

| 输入标签 | `VERSION` | 说明 |
|----------|-----------|------|
| `xtask-todo-lib-v0.1.22` | `0.1.22` | 去掉前缀 **`xtask-todo-lib-v`** |
| `xtask-todo-lib-v1.0.0` | `1.0.0` | 同上 |

- **`OWNER`**：仓库所有者的 **小写**形式（GHCR 要求），例如 **`TangCan` → `tangcan`**。

### 4.4 登录 GHCR

```yaml
uses: docker/login-action@v3
with:
  registry: ghcr.io
  username: ${{ github.actor }}
  password: ${{ secrets.GITHUB_TOKEN }}
```

- 使用 **`GITHUB_TOKEN`**，无需额外配置 **Personal Access Token**（在默认仓库权限下即可 **`packages: write`**）。

### 4.5 构建并推送

```yaml
uses: docker/build-push-action@v6
with:
  context: .
  file: containers/devshell-vm/Containerfile
  push: true
  tags: ghcr.io/${{ steps.ver.outputs.owner }}/xtask_todo/devshell-vm:v${{ steps.ver.outputs.version }}
```

**最终镜像引用示例**（上游仓库 **`TangCan/xtask_todo`**，标签 **`xtask-todo-lib-v0.1.22`**）：

```text
ghcr.io/tangcan/xtask_todo/devshell-vm:v0.1.22
```

这与 **`Cargo.toml` `version = "0.1.22"`** 时代码中的 **`CARGO_PKG_VERSION`** 一致。

### 4.6 将 GHCR 包设为 Public（自动化）

推送成功后，工作流会再执行一步 **`Set GHCR package visibility to public`**：通过 GitHub REST API 对包 **`xtask_todo/devshell-vm`** 执行 **`PATCH`**，将 **`visibility`** 设为 **`public`**。

- **组织仓库**：`PATCH /orgs/{owner}/packages/container/xtask_todo%2Fdevshell-vm`
- **个人账号仓库**：`PATCH /user/packages/container/xtask_todo%2Fdevshell-vm`（由 `gh api` 根据 `orgs/{owner}` 是否存在自动选择）

这样 **`podman pull`** 无需登录即可使用（见 **§6**）。若该步骤失败（例如权限不足），请查看 Job 日志，并仍可按 **§6.3** 在网页上手动设为 Public。

---

## 5. 版本对齐检查清单（维护者）

在推送 **`xtask-todo-lib-v*`** 标签**之前**，请确认：

1. **`crates/todo/Cargo.toml`** 的 **`version`** 与标签后缀一致（例如标签 **`...-v0.1.22`** ↔ **`version = "0.1.22"`**）。
2. **`cargo publish -p xtask-todo-lib`**（若已发布 crates.io）上的版本与上述 **相同**（否则用户 **`cargo install xtask-todo-lib@0.1.22`** 内 `CARGO_PKG_VERSION` 为 0.1.22，但镜像未构建会 **`podman pull` 失败**）。
3. 工作流成功完成后，在 GitHub 仓库 **Packages** 或 **Actions** 中确认镜像已出现。

---

## 6. GHCR 包可见性与匿名拉取

### 6.1 为何需要「可匿名 pull」

`cargo-devshell` 在 Windows 上默认会执行 **`podman pull ghcr.io/<owner>/xtask_todo/devshell-vm:v<版本>`**。  
若包为 **Private**，未登录的用户会 **pull 失败**（常见为 **403 Forbidden**）。

### 6.2 推荐：由 Release 工作流自动设为 Public

**`.github/workflows/release.yml`** 中 **`devshell-vm-oci`** Job 在 **构建并推送** 之后会执行 **「Set GHCR package visibility to public」**（见 **§4.6**），无需在网页上手点。

首次接入前若包已是 **Private**，下一次 **成功** 跑完该 Job 后也会被设为 **Public**。

### 6.3 手动操作（仅当自动化失败或需一次性修正）

1. 打开 GitHub：**仓库 → Packages**（或该镜像包页面）。
2. 将 **`devshell-vm`**（完整包名 **`xtask_todo/devshell-vm`**）对应 **Container package** 设为 **Public**（或组织策略允许匿名读取）。

这样用户 **无需** `podman login ghcr.io` 即可使用默认流程。

### 6.4 若包必须保持 Private

在 **`docs/devshell-vm-windows.md`** 或用户文档中说明：

1. 使用 **Personal Access Token**（`read:packages`）登录 GHCR：

   ```bash
   echo <TOKEN> | podman login ghcr.io -u <username> --password-stdin
   ```

2. 或设置 **`DEVSHELL_VM_CONTAINER_IMAGE`** 指向你可访问的镜像（含私有 registry 凭证由用户自行配置）。

---

## 7. 如何确认发布成功

### 7.1 GitHub Actions

1. 打开 **GitHub → Actions**，筛选 **Release** 工作流。
2. 对应 **`xtask-todo-lib-v*`** 标签的那次运行中，确认 **`devshell-vm-oci`** Job 为 **绿色**（成功）。
3. **GitHub → Packages**（或仓库右侧 **Packages**）中应出现 **`devshell-vm`** 镜像，标签含 **`v{版本}`**。

### 7.2 仓库内命令：`cargo xtask ghcr`（推荐）

在本仓库根目录（需已配置 `.cargo/config.toml` 的 `xtask` alias，见下文）：

```bash
cargo xtask ghcr
```

作用：

- **默认（`--source auto`）**：优先请求 **GitHub `releases/latest`** API，读取与 Release 一致的 **`xtask-todo-lib-vX.Y.Z`** 标签，推导镜像标签 **`vX.Y.Z`**（与 OCI 工作流使用的版本一致）。
- 若无 Release（例如仅本地测试），会 **回退到 crates.io** 的 **`max_version`**。
- 打印 **完整镜像引用** 与 **`podman pull …`** 一行命令，便于在 Linux/macOS/Windows（有 Podman）上立刻验证。

可选数据源：

| `--source` | 含义 |
|------------|------|
| **`auto`**（默认） | `releases/latest` → 失败则 **crates.io** |
| **`releases`** | 仅 **GitHub Releases** 最新标签 |
| **`crates-io`** | 仅 **crates.io** `max_version` |
| **`github-packages`** | **GitHub 包 API**（列出 GHCR 上的 `v*` 标签）；未带认证时可能 **401**，可设置环境变量 **`GITHUB_TOKEN`**（`read:packages`）后重试 |

示例输出（节选）：

```text
Resolved from: GitHub Releases (latest tag)
Latest image tag (semver): v0.1.22

Full reference:
  ghcr.io/tangcan/xtask_todo/devshell-vm:v0.1.22

Verify with Podman:
  podman pull ghcr.io/tangcan/xtask_todo/devshell-vm:v0.1.22
```

若 **`podman pull` 仍 404**：说明镜像未推上 GHCR 或包未公开，回到 **§7.1** 查 **`devshell-vm-oci`** 是否失败。

### 7.3 手动拉取（与 `xtask ghcr` 打印的命令一致）

在任意可运行 Podman 的环境：

```bash
podman pull ghcr.io/tangcan/xtask_todo/devshell-vm:v0.1.22
```

将版本号换成 **`cargo xtask ghcr`** 输出的标签，或 **`releases/latest`** 对应的 **`vX.Y.Z`**。

---

## 8. Fork 与自定义镜像名

- 工作流推送的镜像为 **`ghcr.io/<你的 owner 小写>/xtask_todo/devshell-vm:v<版本>`**。
- **上游代码**默认硬编码 **`ghcr.io/tangcan/xtask_todo/...`**，Fork 用户**若不改代码**，默认仍会拉取 **tangcan** 命名空间下的镜像（若上游已公开）。
- 若希望 Fork 自己的 **`cargo-devshell`** 默认拉自己的 GHCR：

  - 用户设置 **`DEVSHELL_VM_CONTAINER_IMAGE`**，或  
  - 在 Fork 中修改 **`default_container_image()`** 里的 `tangcan` 为你的 owner，并自行发布镜像。

---

## 9. 手动补跑镜像（`workflow_dispatch`）

若某次**仅打标签**时 **`devshell-vm-oci` 失败**（例如 CI 故障），可在 GitHub **Actions → Release → Run workflow**：

- 输入 **`tag_name`**：`xtask-todo-lib-v0.1.22`（与已有标签一致）。

会重新检出该标签并 **构建 / 推送** 镜像，**不会**重复创建 Release（除非 `release` Job 也单独重跑）。

---

## 10. 故障排查

| 现象 | 可能原因 |
|------|----------|
| **`podman pull` 404** | 标签未推送、工作流未跑、或 **`version` 与标签不一致** |
| **Pull 403 / 权限 denied** | GHCR 包为 Private，需公开或登录 |
| **镜像与本地行为不一致** | 标签指向的提交与 **`crates/todo` 版本** 不一致；检查 `Cargo.toml` 与 tag 名 |

---

## 11. 与 `docs/publishing.md` 的关系

- **`docs/publishing.md`**：描述 **crates.io** 发布 **`xtask-todo-lib`**。
- **本文**：描述 **同一版本** 的 **GHCR 侧车镜像**，与 **`cargo install`** 用户的 **默认 `podman pull`** 路径一致。

**推荐顺序**（实践上）：

1. 在 **`crates/todo/Cargo.toml`** 更新 **`version`** 并提交。
2. **`cargo publish -p xtask-todo-lib`**（若适用）。
3. 推送 **`xtask-todo-lib-vX.Y.Z`** 标签 → GitHub Release 工作流运行 **`release`** + **`devshell-vm-oci`**。

（若先打标签再 `cargo publish`，只要 **`version` 与标签一致**，镜像与 crate 仍可一一对应。）

---

## 12. 相关文件

| 文件 | 作用 |
|------|------|
| **`.github/workflows/release.yml`** | **`devshell-vm-oci` Job 定义** |
| **`containers/devshell-vm/Containerfile`** | 镜像构建定义 |
| **`crates/todo/src/devshell/vm/podman_machine.rs`** | 默认镜像 URI、`ENV_DEVSHELL_VM_CONTAINER_IMAGE` |
| **`docs/devshell-vm-windows.md`** | 终端用户行为与环境变量 |
