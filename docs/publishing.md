# 发布说明 (Publishing)

本仓库为 Cargo workspace，仅 **`crates/todo`**（包名 **`xtask-todo-lib`**）适合发布到 [crates.io](https://crates.io)；**xtask** 已设置 `publish = false`，仅作为工作区内部工具使用，不发布。

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

---

## 三、与 GitHub 对应（Releases 与 crates.io 版本一致）

**说明**：GitHub 的 **Packages** 不支持 Cargo/Rust 包（仅支持 npm、Maven、Docker、NuGet、RubyGems 等），因此无法把同一 crate 发布到 GitHub Packages。要让 GitHub 上也有与 crates.io 一致的版本信息，可用 **GitHub Releases**：

1. 发布到 crates.io 后，打 tag 并推送，例如：
   ```bash
   git tag xtask-todo-lib-v0.1.2
   git push origin xtask-todo-lib-v0.1.2
   ```
2. 仓库中的 **Release** 工作流（`.github/workflows/release.yml`）会在推送 tag `xtask-todo-lib-v*` 时自动创建 GitHub Release，并附带对应的 `.crate` 文件。
3. 这样 **crates.io** 与 **GitHub → Releases** 的版本一一对应（如 0.1.0、0.1.1、0.1.2），用户可在 Releases 页下载 `.crate` 或通过链接跳转到 crates.io / docs.rs。

**已有历史版本**：若 0.1.0、0.1.1 已发布但未打过 tag，可补打 tag 并推送，再在 GitHub 上对该 tag 手动创建 Release（或重新推送同一 tag 触发工作流，需先删除远程 tag）。

**为何已有 tag 却没有自动 Release？** 工作流只在「推送 tag」时触发，且运行的是 **该 tag 指向的提交** 里的 workflow。若 tag 是在加入 `release.yml` 之前就推送的，当时不会跑工作流，之后也不会补跑。处理方式：（1）在 GitHub → Actions 查看是否有对应 tag 的 Run 或失败记录；（2）需要自动 Release 时，可删除远程 tag 后重新推送同一 tag（`git push origin :refs/tags/xtask-todo-lib-v0.1.0`，再 `git push origin xtask-todo-lib-v0.1.0`），会重新触发工作流；（3）或在该 tag 下于 GitHub 网页端手动创建 Release。

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
