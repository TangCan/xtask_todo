# Devshell VM：Windows（β / Podman）

**γ（Lima）** 仅在 **Unix** 上可用。在 **Windows** 上，`cargo-devshell`（默认启用 **`beta-vm`**）会把 **β 后端** 设为默认，并在首次需要连接侧车时尽量 **自动**：

1. 若 **`127.0.0.1:9847`** 已有进程监听 → 直接连接。  
2. 否则若已安装 **Podman** → 在仓库根发现 **`containers/devshell-vm/Containerfile`** 时 **构建镜像**（仅首次）并 **`podman run`** 挂载当前工作区到容器 **`/workspace`**，端口 **9847**。  
3. 成功启动容器后，进程内会设置 **`DEVSHELL_VM_BETA_SESSION_STAGING=/workspace`**（若你未事先设置）。  
4. 若未安装 Podman → 尝试 **`winget install -e --id Podman.Podman`**（可能需管理员权限；失败时请手动安装 [podman.io](https://podman.io/)）。  

**一般用法**：在 **`xtask_todo` 仓库根** 打开终端，直接 **`cargo run -p xtask-todo-lib --bin cargo-devshell`**，无需再手写一长串环境变量。

**若通过 `cargo install xtask-todo-lib` 使用**：当前工作目录里通常**没有** `containers/devshell-vm/Containerfile`，自动构建会失败。请任选：**①** 在本仓库克隆目录运行；**②** 自行 `podman build` / `podman run` 侧车并设置 **`DEVSHELL_VM_SKIP_PODMAN_BOOTSTRAP=1`**；**③** 使用 **`DEVSHELL_VM_BACKEND=host`** 不连 VM。

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
