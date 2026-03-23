# Devshell VM：Windows（β / Podman）

**γ（Lima）** 仅在 **Unix** 上可用。在 **Windows** 上，未设置 **`DEVSHELL_VM_BACKEND`** 时默认 **`beta`**（库默认 **`beta-vm`**）。首次需要连接侧车时会尽量 **自动检测 Podman**、尝试 **`winget install Podman.Podman`**、尝试 **`podman machine start`**，并在失败时把 **具体命令** 打到 stderr，便于你手动安装与排障。

1. 若 **`127.0.0.1:9847`** 已有进程监听 → 直接连接。  
2. 否则若 **`podman --version`** 可用 → 必要时 **`podman machine start`**，再若能从当前目录向上找到 **`containers/devshell-vm/Containerfile`** → **构建镜像**（仅首次）并 **`podman run`**。  
3. 成功启动容器后，进程内会设置 **`DEVSHELL_VM_BETA_SESSION_STAGING=/workspace`**（若你未事先设置）。  
4. 若当前目录**不在** `xtask_todo` 克隆内（找不到 Containerfile）→ 不会自动 `podman run`，stderr 会提示：**cd 到仓库根**、或**手动 build/run**、或 **`DEVSHELL_VM_SKIP_PODMAN_BOOTSTRAP=1`**、或 **`DEVSHELL_VM_BACKEND=host`**。  
5. 若 **`podman`** 仍不可用 → stderr 含 **`winget install -e --id Podman.Podman`**、**`podman version`**、文档链接与 **`DEVSHELL_VM_BACKEND=host`** 说明。

**一般用法**：在 **`xtask_todo` 仓库根** 打开终端运行 **`cargo-devshell`**，自动侧车最省事。

**若通过 `cargo install` 使用且不在本仓库目录**：请按 stderr 提示操作，或 **`set DEVSHELL_VM_BACKEND=host`** 仅用宿主沙箱。

### 默认与 Linux 对齐的语义

| 变量 | Windows 默认（β + 自动 Podman） |
|------|----------------------------------|
| **`DEVSHELL_VM_BACKEND`** | **`beta`**（未设置时） |
| **`DEVSHELL_VM_SOCKET`** | **`tcp:127.0.0.1:9847`**（未设置时） |
| **`DEVSHELL_VM_BETA_SESSION_STAGING`** | 由自动启动的容器设为 **`/workspace`**；若侧车已在本机监听且**不是**本工具拉起的容器，则仍用宿主 **`canonicalize` 路径** |

### 关闭 VM / 仅用宿主沙箱

- **`DEVSHELL_VM=off`** 或 **`DEVSHELL_VM_BACKEND=host`**：不连侧车，使用宿主临时目录沙箱（与文档其它处一致）。

### 跳过自动 Podman（测试或自管侧车）

- **`DEVSHELL_VM_SKIP_PODMAN_BOOTSTRAP=1`**：不执行 build/run；请自行保证 **`DEVSHELL_VM_SOCKET`** 可达，必要时设置 **`DEVSHELL_VM_BETA_SESSION_STAGING`**。

### `known_hosts` 被保护或损坏时

自动 Podman 启动 **`podman`** 子进程时，会设置临时 **`HOME`**（含可写的空 **`%HOME%\.ssh\known_hosts`**），避免 Podman 内嵌 SSH 去读 **`%USERPROFILE%\.ssh\known_hosts`**。若需恢复默认行为（仍读用户目录下的 `known_hosts`）：**`set DEVSHELL_VM_DISABLE_PODMAN_SSH_HOME=1`**。

### 手动脚本（可选）

仍可使用 **`scripts/windows/devshell-vm-podman.ps1`** 仅构建并前台运行容器；详见 **`containers/devshell-vm/README.md`**。

### 与 Unix β 的差异

| 项 | Unix | Windows |
|----|------|---------|
| 传输 | UDS **或** `tcp:…` | **`tcp:…`**（默认 `127.0.0.1:9847`） |
| 侧车 | `limactl` / 自启 `devshell-vm` | **推荐自动 Podman**；或本机 **`devshell-vm --serve-tcp`** |

协议与 JSON 行格式见 **`crates/devshell-vm`**、**`docs/devshell-vm-gamma.md`**。

## 交叉编译自检

Pre-commit 对 **`xtask-todo-lib`**（含默认 **`beta-vm`**）做 **`x86_64-pc-windows-msvc`** 的 **`cargo check`**。
