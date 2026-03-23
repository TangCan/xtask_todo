# Devshell VM：Windows（β / Podman）

**γ（Lima）** 仅在 **Unix** 上可用。在 **Windows** 上，未设置 **`DEVSHELL_VM_BACKEND`** 时默认 **`beta`**（库默认 **`beta-vm`**）。

β 使用 **JSON 行**协议，**不在 Windows 宿主机上监听 TCP**。默认 **`DEVSHELL_VM_SOCKET=stdio`**，有两种实现（**自动选择**）：

| 方式 | 何时使用 | 侧车看到的工程目录 |
|------|----------|-------------------|
| **A. `podman machine ssh` + 宿主机上的 Linux ELF** | 在克隆里交叉编译过 `devshell-vm`，或设置了 **`DEVSHELL_VM_LINUX_BINARY`** | Podman Machine 内 **`/mnt/<盘符>/…`**（与宿主路径对应） |
| **B. `podman run -i` + OCI 镜像（自动回退）** | **找不到**宿主机 ELF 时（例如仅 **`cargo install xtask-todo-lib`**、没有仓库） | 容器内 **`/workspace`**（由 `-v` 挂载宿主工作区） |

首次连接前会尽量 **检测 Podman**、**`podman machine start`**，并在使用 **B** 时 **`podman pull`** 默认镜像（见下文）。

### 1. `cargo install` 用户（推荐依赖自动回退 B）

**`cargo install xtask-todo-lib` 不会附带源码树或 Linux ELF。** 默认行为是：若未找到宿主机上的 `devshell-vm`，则 **`podman pull`** 并使用：

**`ghcr.io/tangcan/xtask_todo/devshell-vm:v{与 xtask-todo-lib 相同的版本号}`**

（镜像由 **`release.yml`** 的 **`devshell-vm-oci`** Job 在打 **`xtask-todo-lib-v*`** 标签时构建并推送；GHCR 可见性与版本对齐见 **[devshell-vm-oci-release.md](./devshell-vm-oci-release.md)**。）然后执行：

`podman run --rm -i -v <宿主工作区>:/workspace:Z -w /workspace <镜像> /usr/local/bin/devshell-vm --serve-stdio`

**一般无需设置环境变量。** 若 **`podman pull`** 失败（离线、镜像尚未发布、私有仓库），可任选：

- 临时 **`DEVSHELL_VM_BACKEND=host`** 使用宿主沙箱；或  
- 从本仓库 **交叉编译** 出 ELF 后设置 **`DEVSHELL_VM_LINUX_BINARY`**；或  
- **`DEVSHELL_VM_CONTAINER_IMAGE`** 指向你可拉取的镜像。

### 2. 有克隆的开发者（优先方式 A）

在 **xtask_todo** 根目录：

```bat
rustup target add x86_64-unknown-linux-gnu
cargo build -p devshell-vm --release --target x86_64-unknown-linux-gnu
```

若 **`target/x86_64-unknown-linux-gnu/release/devshell-vm`** 存在，工具会优先走 **方式 A**（不经容器镜像）。

可选：**`DEVSHELL_VM_LINUX_BINARY`**、**`DEVSHELL_VM_REPO_ROOT`**（见下表）。

### 3. 环境变量摘要

| 变量 | 含义 |
|------|------|
| **`DEVSHELL_VM_BACKEND`** | 未设置时 **`beta`** |
| **`DEVSHELL_VM_SOCKET`** | 未设置时 **`stdio`** |
| **`DEVSHELL_VM_LINUX_BINARY`** | 显式指定 Linux `devshell-vm` ELF 的 Windows 路径（强制走方式 A） |
| **`DEVSHELL_VM_REPO_ROOT`** | 含 `containers/devshell-vm/Containerfile` 的仓库根，用于在磁盘上查找 `target/.../devshell-vm` |
| **`DEVSHELL_VM_CONTAINER_IMAGE`** | 覆盖方式 B 的镜像（默认 `ghcr.io/tangcan/xtask_todo/devshell-vm:v{CARGO_PKG_VERSION}`） |
| **`DEVSHELL_VM_STDIO_TRANSPORT`** | **`auto`**（默认）、**`machine-ssh`**（仅方式 A，缺 ELF 则报错）、**`podman-run`**（仅方式 B） |
| **`DEVSHELL_VM_BETA_SESSION_STAGING`** | 覆盖发给侧车的 `staging_dir`；未设置时由工具根据方式 A（`/mnt/…`）或 B（`/workspace`）填写 |

### 4. 可选：TCP 侧车

**`DEVSHELL_VM_SOCKET=tcp:127.0.0.1:9847`** 时仍走本机 TCP（需自行运行 **`devshell-vm --serve-tcp`**）。默认 stdio 不占用宿主机端口。

### 5. 关闭 VM / 仅用宿主沙箱

- **`DEVSHELL_VM=off`** 或 **`DEVSHELL_VM_BACKEND=host`**

### 6. 跳过自动 Podman / 预拉镜像

- **`DEVSHELL_VM_SKIP_PODMAN_BOOTSTRAP=1`**：不检查 Podman、不 **`podman pull`**、不要求宿主机 ELF（测试或完全自管 β）。

### 7. `known_hosts` 与 SSH

Podman 在 Windows 上使用 **`%USERPROFILE%\.ssh\known_hosts`**。默认将 **`USERPROFILE`/`HOME`** 指到 **`%TEMP%\cargo-devshell-ssh-home`** 并生成可写空 **`known_hosts`**；必要时符号链接已有 Podman Machine 目录。详见此前文档版本或源码注释。

**`set DEVSHELL_VM_DISABLE_PODMAN_SSH_HOME=1`** 可改回真实用户目录。

### 8. 与 Unix β 的差异

| 项 | Unix | Windows（默认） |
|----|------|-----------------|
| 传输 | UDS 或 `tcp:…` | **`stdio`** → 方式 A 或 B |
| 侧车 | `limactl` 等 | Podman Machine +（可选）OCI |

协议见 **`crates/devshell-vm`**。

## 交叉编译自检

Pre-commit 对 **`xtask-todo-lib`** 做 **`x86_64-pc-windows-msvc`** 的 **`cargo check`**。
