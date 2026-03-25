# 发布说明 (Publishing)

本仓库为 Cargo workspace，仅 **`crates/todo`**（包名 **`xtask-todo-lib`**）适合发布到 [crates.io](https://crates.io)。该包**同一 Cargo.toml** 内包含 **库**（`[lib]`）和 **二进制**（`[[bin]] name = "cargo-devshell"`），一次 `cargo publish` 同时发布库与 `cargo devshell` 子命令。**xtask** 与 **`crates/devshell-vm`**（β 侧车，随仓库构建；OCI 镜像见 **`docs/devshell-vm-oci-release.md`**）已设置 `publish = false`，仅工作区内使用，不发布到 crates.io。

---

## 一、发布前准备

### 1.1 完善 `crates/todo/Cargo.toml` 元数据

crates.io 要求至少包含：

- **license** 或 **license-file**（必填）
- **repository**（推荐，便于用户找到源码）
- **homepage**、**documentation**、**keywords**、**categories**（可选，利于检索）

示例（按需修改）：

```toml
[package]
name = "xtask-todo-lib"
version = "0.1.0"
edition = "2021"
description = "Todo workspace library"
license = "MIT OR Apache-2.0"
# repository = "https://github.com/你的用户名/xtask_todo"  # 发布前取消注释并替换
# keywords = ["todo", "task", "cli"]
# categories = ["development-tools"]
```

若使用双许可，可写 `license-file = "LICENSE"` 并在仓库根目录放置 `LICENSE` 文件。

### 1.2 确认可发布内容

- 在项目根执行：`cargo publish -p xtask-todo-lib --dry-run`（若使用镜像 registry，可加 `--registry crates-io`）。
- 会执行构建、打包并检查元数据，但不会真正上传；根据报错补全或修正 `Cargo.toml`。

### 1.3 版本号

- 首次发布可用 `0.1.0`。
- 之后按 [语义化版本](https://semver.org/lang/zh-CN/) 在 `crates/todo/Cargo.toml` 中更新 `version`，再执行发布。

---

## 二、发布到 crates.io

### 2.1 账号与登录

1. 在 [crates.io](https://crates.io) 注册账号（可用 GitHub 登录）。
2. 在「Account Settings」中创建 **API Token**。
3. 本地登录（仅需一次）：
   ```bash
   cargo login
   ```
   按提示粘贴 API Token 并回车。

### 2.2 执行发布

在仓库根目录执行：

```bash
cargo publish -p xtask-todo-lib
```

若本地配置了 registry 镜像（替换了 crates.io），需显式指定：

```bash
cargo publish -p xtask-todo-lib --registry crates-io
```

- 会打包 `xtask-todo-lib` 并上传到 crates.io。
- 若未登录或 Token 无效，会提示先执行 `cargo login`。

### 2.3 发布后

- 页面：`https://crates.io/crates/xtask-todo-lib`。
- 其他项目可通过在 `Cargo.toml` 中写 `xtask-todo-lib = "0.1"` 依赖，代码中 `use xtask_todo_lib::{TodoList, TodoId, ...};` 使用。

### 2.4 一键发布（推荐）

在仓库根目录执行：

```bash
cargo xtask publish
```

仅执行发布前预检（不改版本、不提交、不打 tag、不推送）：

```bash
cargo xtask publish --dry-run
```

该命令会依次：将 **patch 版本号 +1**（如 0.1.2 → 0.1.3）、**提交** `crates/todo/Cargo.toml`、**发布到 crates.io**（同一包含库与 `cargo-devshell` 二进制）、**打 tag**（`xtask-todo-lib-vX.Y.Z`）、**推送当前分支与 tag** 到 GitHub。推送 tag 后，Release 工作流会自动创建 GitHub Release 并附带 `.crate` 文件。

**前置条件**：已执行过 `cargo login`；当前在要推送的分支（如 `main`）且工作区无未提交改动（仅允许为即将提交的版本变更）。

---

## 三、与 GitHub 对应（Releases 与 crates.io 版本一致）

**说明**：GitHub 的 **Packages** 不支持 Cargo/Rust 包（仅支持 npm、Maven、Docker、NuGet、RubyGems 等），因此无法把同一 crate 发布到 GitHub Packages。要让 GitHub 上也有与 crates.io 一致的版本信息，可用 **GitHub Releases**：

1. 发布到 crates.io 后，打 tag 并推送，例如：
   ```bash
   git tag xtask-todo-lib-v0.1.2
   git push origin xtask-todo-lib-v0.1.2
   ```
2. 仓库中的 **Release** 工作流（`.github/workflows/release.yml`）会在推送 tag `xtask-todo-lib-v*` 时自动创建 GitHub Release，并附带对应的 `.crate` 文件；**同一工作流**还会并行运行 **`devshell-vm-oci`** Job，构建并推送 **Windows β 默认 OCI 侧车镜像**到 GHCR（与 `xtask-todo-lib` 版本对齐）。详见 **[devshell-vm-oci-release.md](./devshell-vm-oci-release.md)**。
3. 这样 **crates.io** 与 **GitHub → Releases** 的版本一一对应（如 0.1.0、0.1.1、0.1.2），用户可在 Releases 页下载 `.crate` 或通过链接跳转到 crates.io / docs.rs。

**已有历史版本**：若 0.1.0、0.1.1 已发布但未打过 tag，可补打 tag 并推送，再在 GitHub 上对该 tag 手动创建 Release（或重新推送同一 tag 触发工作流，需先删除远程 tag）。

**为何已有 tag 却没有自动 Release？** 工作流只在「推送 tag」时触发，且 GitHub 执行的是 **该 tag 指向的提交** 里的 workflow 文件。若 tag 指向的是加入 `release.yml` 之前的旧提交，该提交里没有 Release 工作流，所以不会触发，重推同一 tag 也不会（因为用的仍是同一旧提交）。处理方式：（1）**推荐**：在 GitHub → Actions 中打开 **Release** 工作流，点击 “Run workflow”；**分支/标签选择器保持为 `main`**（不要选成 tag，否则会报 “Workflow does not exist or does not have a workflow_dispatch trigger in this tag”）；在输入框 “Tag to release” 中填入该 tag（如 `xtask-todo-lib-v0.1.0`），再点 “Run workflow”，即可为已有 tag 创建 Release；（2）或在该 tag 下于 GitHub 网页端手动创建 Release。今后新版本请在包含 `release.yml` 的提交上打 tag 再推送，即可自动创建 Release。

---

## 四、仅做 Git 发布（不发布到 crates.io）

若只希望打版本标签、做 GitHub/GitLab Release，而不发布到 crates.io：

1. 在 `crates/todo/Cargo.toml` 中设置 **`publish = false`**，则该 crate 不会被 `cargo publish` 上传。
2. 按团队习惯打 tag（例如 `git tag v0.1.0`）并推送。
3. 在 GitHub/GitLab 上基于该 tag 创建 Release，附上变更说明或 CHANGELOG 即可。

---

## 五、简要对照

| 目标                 | 操作 |
|----------------------|------|
| 发布 **xtask-todo-lib** 到 crates.io | 补全 `crates/todo` 的 repository 等元数据 → `cargo login` → `cargo publish -p xtask-todo-lib` |
| **xtask** 不发布     | 已设置 `publish = false`，无需改动 |
| 仅 Git tag / Release | 给仓库打 tag，在托管平台上创建 Release，可不执行 `cargo publish` |

如有 CI（如 GitHub Actions），可在工作流中增加：在 tag 推送时自动执行 `cargo publish -p xtask-todo-lib`（需在 CI 中配置 `CARGO_REGISTRY_TOKEN` 等），实现发布自动化。

---

## 六、cargo devshell 子命令（与库同一包发布）

**`cargo-devshell`** 与 **xtask-todo-lib** 共用**同一 Cargo.toml**（`crates/todo`）：该包内配置了 `[lib]` 与 `[[bin]] name = "cargo-devshell"`，一次 `cargo publish -p xtask-todo-lib` 会同时发布库和该二进制。Cargo 约定：安装的二进制若名为 `cargo-<名字>`，即可通过 `cargo <名字>` 作为子命令调用。

### 6.1 当前配置

- **crates/todo/Cargo.toml** 中：
  ```toml
  [lib]
  name = "xtask_todo_lib"
  path = "src/lib.rs"

  [[bin]]
  name = "cargo-devshell"
  path = "src/bin/cargo_devshell/main.rs"
  ```
- 发布后，用户安装 **xtask-todo-lib** 即可同时获得库（供依赖）和 **`cargo devshell`** 子命令。

### 6.2 用户安装与使用

发布后，用户执行：

```bash
cargo install xtask-todo-lib
```

安装完成后，在任意目录执行：

```bash
cargo devshell [path]
```

即可启动内嵌的 devshell（`path` 可选，为持久化 VFS 的文件路径，默认为 `.dev_shell.bin`）。若仅需库而不需要子命令，在 `Cargo.toml` 中依赖 `xtask-todo-lib = "x.y"` 即可。

**VM 默认行为（与源码 tree 中 `cargo run --bin cargo-devshell` 一致）：** 在 **Linux / macOS** 上，未设置 **`DEVSHELL_VM`** 时视为 **开启**；未设置 **`DEVSHELL_VM_BACKEND`** 时默认为 **`lima`**。因此需本机已安装 **Lima**（`limactl` 在 `PATH`）且实例与工作区挂载按 **`docs/devshell-vm-gamma.md`** 配置，否则启动 devshell 时可能在创建会话阶段失败。若只想用宿主临时目录跑 `cargo`（无 VM），请设置 **`DEVSHELL_VM=off`** 或 **`DEVSHELL_VM_BACKEND=host`**。自动化/CI 无 Lima 时务必设置其一。

### 6.3 可选 feature：`beta-vm`（β IPC 后端）

默认发布的 **`cargo devshell` / 库** 不包含 **`DEVSHELL_VM_BACKEND=beta`** 相关代码路径；需显式打开 Cargo feature **`beta-vm`**（见 `crates/todo/Cargo.toml`）。

**安装带 β 后端的 `cargo devshell`：**

```bash
cargo install xtask-todo-lib --features beta-vm
```

**其他 crate 依赖本库并启用 β：**

```toml
[dependencies]
xtask-todo-lib = { version = "x.y", features = ["beta-vm"] }
```

启用后：**Unix** 可设置 **`DEVSHELL_VM_BACKEND=beta`** 与 **`DEVSHELL_VM_SOCKET`**（Unix 域路径或 **`tcp:`**）；**Windows** 默认 **`beta`**，通常 **`DEVSHELL_VM_SOCKET=stdio`**，由 **`podman`** 运行 OCI 侧车或宿主编译的 Linux ELF（**`DEVSHELL_VM`** 开启 VM 时一般无需再设 `on`）。侧车 **`devshell-vm`** 为 workspace 内 crate（**`publish = false`**），不随 `cargo install` 安装；可从**本仓库**构建，或使用 **GHCR** 镜像（见 **`docs/devshell-vm-windows.md`**），例如：

```bash
cargo build -p devshell-vm --release
# 可执行文件通常在 target/release/devshell-vm
```

侧车用法与 γ/β 环境变量摘要见 **`docs/devshell-vm-gamma.md`**（§「β 侧车」）、**`docs/devshell-vm-windows.md`**。

### 6.4 小结

| 项目 | 说明 |
|------|------|
| 子命令名 | **`cargo devshell`**（由二进制名 `cargo-devshell` 决定） |
| 发布方式 | 与库**同一包**发布，一次 `cargo publish -p xtask-todo-lib` 同时发布 lib 与 bin |
| 安装命令 | `cargo install xtask-todo-lib`（同时得到库与 `cargo devshell`）；需 β IPC 时加 **`--features beta-vm`** |
