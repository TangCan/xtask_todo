# `devshell-vm` 容器镜像（Podman / Docker）

用于在 **任意**安装了 Podman 的环境以 **Linux 容器**运行 β 侧车，协议为 JSON 行（**可选**：通过 **`--serve-tcp`** 在容器内监听 TCP）。

## Windows 上的推荐路径（默认）

在 **Windows** 上，`cargo-devshell` 的默认实现是 **`podman machine ssh`** + **`devshell-vm --serve-stdio`**（见 **`docs/devshell-vm-windows.md`**），**不要求**在宿主机上映射 **9847** 或运行本 README 里的 `podman run -p …`。

请先在本机仓库根构建 Linux 二进制：

```bash
cargo build -p devshell-vm --release --target x86_64-unknown-linux-gnu
```

## 构建镜像

在**仓库根目录**执行：

```bash
podman build -f containers/devshell-vm/Containerfile -t devshell-vm:local .
```

## 可选：容器内 TCP 监听（调试 / 非默认 Windows 流程）

将宿主工作区挂载到 **`/workspace`** 并映射端口 **9847**（仅当你需要**本机 TCP** 连接侧车时）：

```bash
podman run --rm -p 9847:9847 -v /path/to/host/workspace:/workspace:Z devshell-vm:local
```

侧车在容器内监听 **`0.0.0.0:9847`**。宿主上的客户端需设置：

- `DEVSHELL_VM_BACKEND=beta`
- `DEVSHELL_VM_SOCKET=tcp:127.0.0.1:9847`
- `DEVSHELL_VM_BETA_SESSION_STAGING=/workspace`（与挂载点一致）
- `DEVSHELL_VM_WORKSPACE_PARENT` = 与挂载目录对应的**宿主路径**（Windows 为 `C:\…`）

详见 **`docs/devshell-vm-windows.md`**。
