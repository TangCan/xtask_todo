# Devshell VM：Windows（β / Podman Machine）

**γ（Lima）** 仅在 **Unix** 上可用。在 **Windows** 上，未设置 **`DEVSHELL_VM_BACKEND`** 时默认 **`beta`**（库默认 **`beta-vm`**）。

β 的**推荐路径**是：**不在 Windows 宿主机上监听 TCP**。`cargo-devshell` 通过 **`podman machine ssh -T`** 在 Podman Machine（Linux）里执行 **`devshell-vm --serve-stdio`**，JSON 行协议走 **SSH 会话的 stdin/stdout**（与 TCP/Unix 套接字相同的一行一条 JSON）。

首次需要连接时会尽量 **自动检测 Podman**、尝试 **`winget install Podman.Podman`**、尝试 **`podman machine start`**，并确保存在 **Linux ELF** 的 `devshell-vm`（见下文），失败时把 **具体命令** 打到 stderr。

### 1. 准备 Linux 版 `devshell-vm`

在 **xtask_todo** 仓库根目录（或设置 **`DEVSHELL_VM_LINUX_BINARY`** 指向已构建好的文件）：

```bat
rustup target add x86_64-unknown-linux-gnu
cargo build -p devshell-vm --release --target x86_64-unknown-linux-gnu
```

默认查找路径：`target/x86_64-unknown-linux-gnu/release/devshell-vm`（Windows 路径）。Podman Machine 内通过 **`/mnt/<盘符>/…`** 访问该文件。

### 2. 默认环境变量（与 Linux 对齐的语义）

| 变量 | Windows 默认（β + Podman Machine stdio） |
|------|------------------------------------------|
| **`DEVSHELL_VM_BACKEND`** | **`beta`**（未设置时） |
| **`DEVSHELL_VM_SOCKET`** | **`stdio`**（未设置时） |
| **`DEVSHELL_VM_BETA_SESSION_STAGING`** | 由工具根据 **`DEVSHELL_VM_WORKSPACE_PARENT`** 映射为 Podman Machine 里的 **`/mnt/…`** 路径（与 `session_start` 的 `staging_dir` 一致） |

可选：**`DEVSHELL_VM_LINUX_BINARY`** — 显式指定 Linux `devshell-vm` 的 **Windows 路径**（覆盖默认 `target/.../release/devshell-vm`）。

### 3. 一般用法

在 **`xtask_todo` 仓库根** 打开终端：先 **`cargo build`** 出 Linux `devshell-vm`，再运行 **`cargo-devshell`**。无需手动 `podman run` 映射端口。

### 4. 可选：TCP 侧车（非默认）

若 **`DEVSHELL_VM_SOCKET=tcp:127.0.0.1:9847`**（或 `tcp://…`），则仍按 **本机 TCP** 连接；需自行在可达地址上运行 **`devshell-vm --serve-tcp`**（例如容器或 WSL）。**默认流程已改为 stdio，不再依赖宿主机 9847。**

### 5. 关闭 VM / 仅用宿主沙箱

- **`DEVSHELL_VM=off`** 或 **`DEVSHELL_VM_BACKEND=host`**：不连 β，使用宿主临时目录沙箱。

### 6. 跳过自动 Podman / 二进制检查（测试或自管）

- **`DEVSHELL_VM_SKIP_PODMAN_BOOTSTRAP=1`**：不执行 Podman 与 Linux 二进制存在性检查；请自行保证 **`DEVSHELL_VM_SOCKET`** 可达，必要时设置 **`DEVSHELL_VM_BETA_SESSION_STAGING`**。

### 7. `known_hosts` 被保护、锁定或内容损坏时

Podman 在 Windows 上走内嵌 SSH 时，**Go 的 `UserHomeDir()` 读的是 `%USERPROFILE%`，不是 `HOME`**。仅改 `HOME` 仍会访问 **`%USERPROFILE%\.ssh\known_hosts`**。

**默认行为（绕开该文件，等价于空 `known_hosts`）：** `cargo-devshell` 在拉起 **`podman`** 时会把 **`USERPROFILE` 与 `HOME`** 指到临时目录 **`%TEMP%\cargo-devshell-ssh-home`**，并在其中放置 **可写的空** `.ssh\known_hosts`。若你本机已有 Podman Machine，会尽量把 **`%USERPROFILE%\.local\share\containers\podman`** **符号链接**到该临时配置下，以免找不到已有机器（需 **Windows 开发者模式** 或具备创建符号链接的权限；失败时 stderr 会提示）。

**恢复使用真实用户目录下的 `known_hosts`：** **`set DEVSHELL_VM_DISABLE_PODMAN_SSH_HOME=1`**（此时若 `known_hosts` 仍无效，需自行修复或解锁该文件）。

**在 CMD 里手动运行 `podman`** 时不会自动套用上述隔离；可先在同一终端执行与 `scripts/windows/devshell-vm-podman.ps1` 相同的 **`USERPROFILE` 临时目录**逻辑，或修复系统上的 `known_hosts`。

### 8. 辅助脚本（可选）

**`scripts/windows/devshell-vm-podman.ps1`**：在仓库根 **构建 Linux `devshell-vm`**，并打印推荐环境变量；**不再**以前台 **`podman run -p 9847`** 作为默认流程（见 `containers/devshell-vm/README.md` 中的**可选**容器说明）。

### 9. 与 Unix β 的差异

| 项 | Unix | Windows（默认） |
|----|------|-----------------|
| 传输 | UDS **或** `tcp:…` | **`stdio`** → **`podman machine ssh`** → **`devshell-vm --serve-stdio`** |
| 侧车 | `limactl` / 自启 `devshell-vm` | **Podman Machine** 内执行 Linux `devshell-vm` |

协议与 JSON 行格式见 **`crates/devshell-vm`**、**`docs/devshell-vm-gamma.md`**。

## 交叉编译自检

Pre-commit 对 **`xtask-todo-lib`**（含默认 **`beta-vm`**）做 **`x86_64-pc-windows-msvc`** 的 **`cargo check`**。
