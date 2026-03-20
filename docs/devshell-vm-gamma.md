# Devshell γ：Lima 后端

在 **Linux / macOS** 上，可将 `rustup` / `cargo` 放到 **Lima** 虚拟机里执行，同时在内存 **VFS** 与主机工作区之间做 **增量 push / pull**（见规格 `docs/superpowers/specs/2026-03-11-devshell-microvm-session-design.md` §3）。

β 阶段会改为侧车进程 + IPC；γ 依赖本机安装的 `limactl`。

---

## 前置条件

1. 安装 [Lima](https://lima-vm.io/)，`limactl` 在 **`PATH`** 中（或通过 `DEVSHELL_VM_LIMACTL` 指定绝对路径）。
2. 已创建并至少成功启动过一次的 **Lima 实例**（默认名：`devshell-rust`，可用 `DEVSHELL_VM_LIMA_INSTANCE` 覆盖）。
3. 该实例的 YAML 中把 **cargo-devshell 使用的工作区目录** 挂载到 guest 的 **`/workspace`**（或你通过 `DEVSHELL_VM_GUEST_WORKSPACE` 指定的路径）。

---

## 工作区目录（必须挂载）

默认主机路径为：

`$DEVSHELL_EXPORT_BASE` 或 XDG 缓存下的  
**`…/cargo-devshell-exports/vm-workspace/<实例名>/`**

（与 `sandbox::devshell_export_parent_dir()` 一致，实例名中的非字母数字会变成 `_`。）

也可用 **`DEVSHELL_VM_WORKSPACE_PARENT`** 直接指定一个绝对路径。

在 Lima 实例的 **`mounts`** 里增加 **第二条**（保留模板自带的 `location: "~"` 等），例如：

```yaml
mounts:
  - location: "~"
  - location: "~/.cache/cargo-devshell-exports/vm-workspace/devshell-rust"
    mountPoint: /workspace
    writable: true
```

（若默认实例名不是 `devshell-rust`，把路径里最后一级目录改成与 **`DEVSHELL_VM_LIMA_INSTANCE`** 净化后一致；用了 **`DEVSHELL_VM_WORKSPACE_PARENT`** 时，`location` 应指向该目录的**绝对路径**。）

改完后 **必须** 重启实例：`limactl stop devshell-rust` → `limactl start -y devshell-rust`。

仓库内可复制片段：**[snippets/lima-devshell-workspace-mount.yaml](snippets/lima-devshell-workspace-mount.yaml)**。

Guest 内工程目录为 **`/workspace/<VFS cwd 最后一段>`**，与当前宿主 temp 导出布局一致（例如 VFS cwd `/proj/foo` → 主机 `…/foo/` → guest `/workspace/foo`）。

### Guest 内必须有 `cargo` / `rustup`（γ 不代装）

γ 只做 **`limactl shell … -- cargo …`**：若在 guest 里 **`cargo: command not found`**（退出码 **127**），说明 **VM 里尚未安装 Rust 工具链**，与挂载是否成功无关。

**做法一（常见）：在 guest 里装 rustup（一次性）**

```bash
limactl shell --workdir / devshell-rust
```

在 guest 提示符下执行（官方安装脚本，非交互默认接受）：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env
cargo --version
```

`limactl shell` 默认多为非登录 shell，若之后仍找不到 **`cargo`**，在 guest 的 **`~/.bashrc`** 末尾加一行：**`source "$HOME/.cargo/env"`**。

**做法二：只读挂载宿主 `~/.rustup` 与 `~/.cargo`（与宿主共用工具链）**

前提：宿主上已安装 Rust（**`~/.rustup`**、**`~/.cargo`** 目录存在）；若使用非默认路径，把下面 `location` 改成 **`$RUSTUP_HOME`** / **`$CARGO_HOME`** 的绝对路径。宿主与 guest **CPU 架构须一致**（例如均为 **x86_64**）。

在实例 **`lima.yaml`** 的 **`mounts:`** 中增加（与 **`/workspace`** 挂载并列即可）：

```yaml
  - location: "~/.rustup"
    mountPoint: /host-rustup
    writable: false
  - location: "~/.cargo"
    mountPoint: /host-cargo
    writable: false
```

在同一文件里增加 **`env:`**（写入 **`/etc/environment`**，便于 **`limactl shell … -- cargo`** 这类非登录 SSH 也能找到 **`cargo`**）：

```yaml
env:
  RUSTUP_HOME: /host-rustup
  CARGO_HOME: /host-cargo
  PATH: /host-cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
```

改完后 **`limactl stop <实例>`** → **`limactl start -y <实例>`**。验证：

```bash
limactl shell --workdir / devshell-rust -- cargo --version
```

可复制片段：**[snippets/lima-devshell-rust-toolchain-mount.yaml](snippets/lima-devshell-rust-toolchain-mount.yaml)**。

---

## 环境变量

| 变量 | 含义 |
|------|------|
| `DEVSHELL_VM` | **二进制默认：** 未设置 = **开启** VM 模式。设为 **`off`** / **`0`** / **`false`** / **`no`**（大小写不敏感）则关闭，仅用宿主临时目录 + `sandbox::run_rust_tool`。`on` / `1` / `true` / `yes` 亦为开启。**`cargo test` 编译的库**（`cfg(test)`）未设置时视为关闭，便于无 Lima 的 CI。 |
| `DEVSHELL_VM_BACKEND=lima` | 使用 γ Lima 后端（Unix）。**二进制在 Unix 上若未设置本变量，默认即为 `lima`。** |
| `DEVSHELL_VM_BACKEND=host` 或 `auto` | 强制使用宿主临时目录 + `sandbox::run_rust_tool`（不用 Lima）。 |
| `DEVSHELL_VM_LIMA_INSTANCE` | Lima 实例名（默认 `devshell-rust`）。 |
| `DEVSHELL_VM_LIMACTL` | `limactl` 可执行文件路径（可选）。 |
| `DEVSHELL_VM_WORKSPACE_PARENT` | 主机工作区根目录（可选；默认见上文）。 |
| `DEVSHELL_VM_GUEST_WORKSPACE` | Guest 挂载点（默认 `/workspace`）。 |
| `DEVSHELL_VM_STOP_ON_EXIT=1` | 会话结束（REPL exit / 脚本结束）时执行 `limactl stop`（默认不 stop，便于多终端共用实例）。 |
| `DEVSHELL_VM_LIMA_HINTS=0` | 关闭 γ 的 **Lima 配置/故障提示**（默认开启）：首次 `cargo`/`rustup` 前会做一次 guest 探测；`cargo`/`rustup` 非零退出或 **`limactl start` 失败** 时会打印与 `lima.yaml`、挂载、KVM、`cargo` PATH 相关的建议。 |

---

## 行为摘要

1. 第一次执行 `cargo` / `rustup`：`limactl start <实例>`，然后 **增量 push** VFS → 主机工作区。
2. 在 VM 内：`limactl shell --workdir <guest 工程目录> <实例> -- cargo …`
3. 每次命令后：**pull** 主机工作区 → VFS（即使命令非零退出也会尝试 pull，与实现计划一致）。
4. 退出 devshell：再 **pull** 一次；若设置了 `DEVSHELL_VM_STOP_ON_EXIT` 则 `limactl stop`。

---

## 其他宿主（概念对照，γ 未内建）

| 宿主 | γ 常见 CLI 载体 | 说明 |
|------|-----------------|------|
| **Linux** | Lima（本仓库已实现 `DEVSHELL_VM_BACKEND=lima`） | QEMU/KVM 等 |
| **macOS** | 同样可用 **Lima**（Virtualization.framework） | 挂载与工作区约定与 Linux 相同；需本机安装 Lima |
| **macOS** | **Multipass**（`multipass shell`） | 未在代码中编排；可自行套 `multipass exec` 等价于 `limactl shell` |
| **Windows** | **WSL2**（`wsl.exe -d <Distro> -- …`） | 未在代码中编排；工作区需落在 WSL 可访问路径（如 `\\wsl$\…` 或 `wslpath`） |
| **Windows** | **Hyper-V / Multipass** | 团队任选一种 CLI，γ 仅验证「一条命令进 VM」思路 |

β 阶段由 **`devshell-vm`** 侧车抽象上述差异；见 [`docs/superpowers/specs/2026-03-11-devshell-vm-ipc-draft.md`](./superpowers/specs/2026-03-11-devshell-vm-ipc-draft.md)。

---

## β 桩（Unix，需 `beta-vm` feature）

用于本地联调 **JSON-lines IPC**，不替代 γ Lima。

| 步骤 | 说明 |
|------|------|
| 编译库 | `cargo build -p xtask-todo-lib --features beta-vm`（或 `cargo devshell` 所在包带上该 feature） |
| 后端 | **`DEVSHELL_VM_BACKEND=beta`**（`DEVSHELL_VM` 二进制默认已开启，一般不必再设 `on`） |
| 套接字 | **`DEVSHELL_VM_SOCKET`**：Unix 域套接字路径（与 `ENV_DEVSHELL_VM_SOCKET` 一致） |
| 侧车 | `devshell-vm --serve-socket <path>` 在 `<path>` 上 `listen` + 逐行 JSON 处理（见 IPC 草案） |
| 测试 | 若 `exec` 的 argv 含 **`--devshell-vm-test-fail`**，桩返回退出码 **1** |

与 γ 相同：工作区父目录仍用 `DEVSHELL_VM_WORKSPACE_PARENT` / 默认缓存布局；**pull 失败**时仅警告，仍以 **guest 工具退出码** 为准。

---

## 限制与排错

- **Windows**：当前 crate 不在 Windows 上编译 Lima 后端；请用 `DEVSHELL_VM=off` 或 `DEVSHELL_VM_BACKEND=host`。
- **`template "default.yaml" not found`**：若只把 **`lima` / `limactl` 拷进 `PATH`**（例如仅解压了 `bin/`），没有把发行包里的 **`share/lima`** 装到 Lima 能找到的位置，第一次 **`limactl start <名>`**（尚无实例时会按模板创建）就会报此错。应把同一 tarball 里的 **`share/lima` 整棵目录**放到 **`$HOME/.local/share/lima`**（与 `~/.local/bin/limactl` 搭配），或按官方说明解压到 **`/usr/local`**（含 `bin` 与 `share`）。可用 **`limactl info`** 看是否已列出模板路径。
- **`Failed to open TUI` / `error=EOF`**：在 REPL 里跑 `cargo` 时，子进程往往**没有完整 TTY**，Lima 若走交互式 TUI 会读 stdin 失败。本仓库的 γ 实现已对 **`limactl start`** 传入 **`-y`**（等价 **`--tty=false`**），避免该路径；若你**手动**执行 `limactl start`，请加 **`-y`**。
- **QEMU `exit status 1` / `Driver stopped due to error`**：先看 **`~/.lima/<实例名>/ha.stderr.log`**（JSON 行里常有 **`qemu[stderr]:`**）。若出现 **`Could not access KVM kernel module: Permission denied`** 或 **`failed to initialize kvm: Permission denied`**：当前用户对 **`/dev/kvm`** 无权限。
  1. `getent group kvm` 确认存在 **`kvm`** 组；`ls -l /dev/kvm` 一般为 **`crw-rw----+ root kvm`**。
  2. 将用户加入组：`sudo usermod -aG kvm "$USER"`（把 **`$USER`** 换成你的登录名亦可）。
  3. **必须重新登录会话**（注销/重启 SSH，或开新登录 shell）；仅当前终端可临时试：`newgrp kvm`。
  4. 验证：`groups` 输出中含 **`kvm`**；再执行 `limactl start -y devshell-rust` 或在 devshell 里重试 `cargo`。
  其他常见原因：在**未开嵌套虚拟化**的虚拟机里跑 Lima；**OVMF** 路径异常。可装 **`cpu-checker`** 后运行 **`kvm-ok`**（物理机辅助检查）。
- **`limactl start` 失败**：检查 `limactl list`、实例名、KVM/虚拟化权限。
- **`cargo: command not found` / 退出码 127**：guest 内没有 **`cargo`**。见上文 **「Guest 内必须有 cargo / rustup」**：在 VM 内安装 rustup，或只读挂载宿主 **`RUSTUP_HOME`/`CARGO_HOME`** 并配置 **`PATH`**。
- **挂载路径不一致**：guest 内 `ls /workspace` 应能看到 push 过去的目录；若为空，核对 `mountPoint` 与 `DEVSHELL_VM_WORKSPACE_PARENT` / 默认缓存路径是否一致。

---

## 相关文档

- 设计：`docs/design.md`（devshell / vm）
- 规格：`docs/superpowers/specs/2026-03-11-devshell-microvm-session-design.md`
- β IPC 草案：`docs/superpowers/specs/2026-03-11-devshell-vm-ipc-draft.md`
- 计划：`docs/superpowers/plans/2026-03-11-devshell-microvm-session.md`
