# `devshell-vm` 容器镜像（Podman / Docker）

用于在 **Windows**（或任意安装了 Podman 的环境）中以 **Linux 容器**运行 β 侧车，协议仍为 JSON 行 over TCP。

## 构建

在**仓库根目录**执行：

```bash
podman build -f containers/devshell-vm/Containerfile -t devshell-vm:local .
```

## 运行示例

将宿主工作区目录挂载到容器内 **`/workspace`**，并映射端口 **9847**：

```bash
podman run --rm -p 9847:9847 -v /path/to/host/workspace:/workspace:Z devshell-vm:local
```

侧车监听 **`0.0.0.0:9847`**。宿主上的 **`cargo-devshell`** 应设置：

- `DEVSHELL_VM_BACKEND=beta`
- `DEVSHELL_VM_SOCKET=tcp:127.0.0.1:9847`
- `DEVSHELL_VM_BETA_SESSION_STAGING=/workspace`（与挂载点一致）
- `DEVSHELL_VM_WORKSPACE_PARENT` = 与挂载目录对应的**宿主路径**（Windows 为 `C:\…`）

详见 **`docs/devshell-vm-windows.md`**。
