//! `DEVSHELL_VM*` environment variable names (documented in module root).

/// `DEVSHELL_VM` — **Release / binary default:** unset means **on** (use VM backend per `ENV_DEVSHELL_VM_BACKEND`).
///
/// Set to `off` / `0` / `false` / `no` (case-insensitive) to use **only** the host temp sandbox.
/// `on` / `1` / `true` / `yes` also enable VM mode.
///
/// **Unit tests** (`cfg(test)`): unset defaults to **off** so `cargo test` works without Lima.
pub const ENV_DEVSHELL_VM: &str = "DEVSHELL_VM";

/// Backend selector: `host`, `auto`, `lima`, `beta`, …
///
/// **Release / binary default on Unix:** `lima` (γ) when this variable is unset.
/// **Windows** default: **`beta`** (with **`beta-vm`** feature). Use **`DEVSHELL_VM_BACKEND=host`** for host-only sandbox.
/// **Other non-Unix (non-Windows):** `host`.
/// **`cfg(test)`:** unset → `auto` (host sandbox) for the same reason as `ENV_DEVSHELL_VM`.
pub const ENV_DEVSHELL_VM_BACKEND: &str = "DEVSHELL_VM_BACKEND";

/// When `1`/`true`/`yes`, start the VM session eagerly (future γ); default is lazy start on first rust tool.
pub const ENV_DEVSHELL_VM_EAGER: &str = "DEVSHELL_VM_EAGER";

/// Lima instance name for γ (`limactl shell <name>`).
pub const ENV_DEVSHELL_VM_LIMA_INSTANCE: &str = "DEVSHELL_VM_LIMA_INSTANCE";

/// Unix socket path for β client ↔ `devshell-vm --serve-socket` (see IPC draft).
pub const ENV_DEVSHELL_VM_SOCKET: &str = "DEVSHELL_VM_SOCKET";

/// When set (non-empty), β **`session_start`** sends this string as **`staging_dir`** to the sidecar instead of
///
/// `canonicalize(DEVSHELL_VM_WORKSPACE_PARENT / …)`. Use a **POSIX path** visible to the sidecar process
/// (e.g. **`/workspace`** inside a Podman/WSL Linux container) while **`DEVSHELL_VM_WORKSPACE_PARENT`** on the
/// host remains the real Windows path for push/pull. See **`docs/devshell-vm-windows.md`** (Podman).
///
/// On Windows, **`stdio`** (default) maps the host workspace to **`/mnt/<drive>/…`** inside Podman Machine for
/// `session_start` **`staging_dir`** unless you set this explicitly.
pub const ENV_DEVSHELL_VM_BETA_SESSION_STAGING: &str = "DEVSHELL_VM_BETA_SESSION_STAGING";

/// When set (any value), skip **`podman machine ssh`** bootstrap on Windows: no Podman check / no requirement
/// that the Linux `devshell-vm` binary exists (tests or fully manual β setup).
pub const ENV_DEVSHELL_VM_SKIP_PODMAN_BOOTSTRAP: &str = "DEVSHELL_VM_SKIP_PODMAN_BOOTSTRAP";

/// **Windows β:** optional full **Windows** path to the **Linux** `devshell-vm` binary
/// (`x86_64-unknown-linux-gnu` / ELF) for **`podman machine ssh`** transport.
///
/// If unset, the binary is searched under **`$repo_root/target/x86_64-unknown-linux-gnu/release/devshell-vm`**
/// where `repo_root` is discovered from cwd, [`ENV_DEVSHELL_VM_REPO_ROOT`], or walking up from the workspace
/// parent — **not** the ephemeral `cargo-devshell-exports` tree. If still not found, **automatic fallback
/// uses [`ENV_DEVSHELL_VM_CONTAINER_IMAGE`] with `podman run -i`** (see `podman_machine.rs`).
pub const ENV_DEVSHELL_VM_LINUX_BINARY: &str = "DEVSHELL_VM_LINUX_BINARY";

/// **Windows β:** optional **Windows** path to an **`xtask_todo`** repository root (directory containing
///
/// **`containers/devshell-vm/Containerfile`**). Locates **`target/x86_64-unknown-linux-gnu/release/devshell-vm`**
/// when [`ENV_DEVSHELL_VM_LINUX_BINARY`] is unset. Useful if you keep a checkout for building the sidecar but run
/// **`cargo devshell`** from other directories; **not** applicable when you only have a crates.io install and no clone.
pub const ENV_DEVSHELL_VM_REPO_ROOT: &str = "DEVSHELL_VM_REPO_ROOT";

/// **Windows β:** OCI image used when **no** host Linux `devshell-vm` ELF is found: `podman run -i` with
///
/// **`--serve-stdio`** and the workspace mounted at **`/workspace`** (no host TCP).
/// Default: **`ghcr.io/tangcan/xtask_todo/devshell-vm:v{CARGO_PKG_VERSION}`** (published by CI on release).
pub const ENV_DEVSHELL_VM_CONTAINER_IMAGE: &str = "DEVSHELL_VM_CONTAINER_IMAGE";

/// **Windows β:** timeout (seconds) for each `podman pull` attempt in OCI fallback mode.
/// When unset/invalid/0, default timeout is applied by implementation.
#[cfg(windows)]
pub const ENV_DEVSHELL_VM_PULL_TIMEOUT_SECS: &str = "DEVSHELL_VM_PULL_TIMEOUT_SECS";

/// **Windows β:** stdio transport for `DEVSHELL_VM_SOCKET=stdio`: **`auto`** (default), **`machine-ssh`**
/// (host ELF + `podman machine ssh`), or **`podman-run`** (OCI image + `podman run -i`).
pub const ENV_DEVSHELL_VM_STDIO_TRANSPORT: &str = "DEVSHELL_VM_STDIO_TRANSPORT";

/// When set (any value), do **not** isolate **`USERPROFILE` / `HOME`** for `podman` subprocesses (Windows).
///
/// By default we point **`USERPROFILE`** (Go’s `UserHomeDir()` on Windows — not only `HOME`) at a writable
/// temp “profile” with an **empty default** `.ssh/known_hosts`, so a **locked, protected, or invalid**
/// **`%USERPROFILE%\.ssh\known_hosts`** is not read. An existing Podman Machine dir is **symlinked** in when
/// possible (see `podman_machine.rs`).
pub const ENV_DEVSHELL_VM_DISABLE_PODMAN_SSH_HOME: &str = "DEVSHELL_VM_DISABLE_PODMAN_SSH_HOME";

/// `DEVSHELL_VM_WORKSPACE_MODE` — **`sync`** (default) or **`guest`** (Mode P; guest filesystem as source of truth).
///
/// **`guest`** is effective only when the VM is enabled and the backend is **`lima`** or **`beta`**; otherwise
/// `VmConfig::workspace_mode_effective` returns `WorkspaceMode::Sync` (design `2026-03-20-devshell-guest-primary-design.md` §6).
///
/// **Unset** (including **`cfg(test)`**): `WorkspaceMode::Sync`.
pub const ENV_DEVSHELL_VM_WORKSPACE_MODE: &str = "DEVSHELL_VM_WORKSPACE_MODE";

/// **β 侧车 `exec`：** 若设为正整数，作为毫秒超时（与 JSON **`timeout_ms`** 二选一/叠加见 **`devshell-vm`**）。**`0`** 或未设置表示不限制（仅受宿主/客户端是否在 JSON 里传 **`timeout_ms`** 约束）。
pub const ENV_DEVSHELL_VM_EXEC_TIMEOUT_MS: &str = "DEVSHELL_VM_EXEC_TIMEOUT_MS";
