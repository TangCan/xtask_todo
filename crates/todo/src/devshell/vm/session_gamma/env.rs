//! Environment variable names for γ (Lima) session.

/// Override path to `limactl` (default: `PATH`).
pub const ENV_DEVSHELL_VM_LIMACTL: &str = "DEVSHELL_VM_LIMACTL";

/// Host directory we push/pull (must be mounted at [`super::GammaSession::guest_mount`] in the Lima VM).
pub const ENV_DEVSHELL_VM_WORKSPACE_PARENT: &str = "DEVSHELL_VM_WORKSPACE_PARENT";

/// When unset or truthy (default): if [`ENV_DEVSHELL_VM_WORKSPACE_PARENT`] is unset, resolve the host
/// workspace directory from `cargo metadata` in the **current directory** (Cargo workspace root), so
/// `cargo-devshell` started from a checkout maps `cwd` ↔ guest under the same mount prefix.
/// Set to `0`/`false`/`no`/`off` to use the legacy default (`…/vm-workspace/<instance>/` under the devshell export cache).
pub const ENV_DEVSHELL_VM_WORKSPACE_USE_CARGO_ROOT: &str = "DEVSHELL_VM_WORKSPACE_USE_CARGO_ROOT";

/// Guest mount point for that directory (default `/workspace`).
pub const ENV_DEVSHELL_VM_GUEST_WORKSPACE: &str = "DEVSHELL_VM_GUEST_WORKSPACE";

/// When set truthy, run `limactl stop` on session shutdown.
pub const ENV_DEVSHELL_VM_STOP_ON_EXIT: &str = "DEVSHELL_VM_STOP_ON_EXIT";

/// When unset or truthy (default): after the VM is running, probe for `gcc` in the guest and, if
/// missing, try `apt-get install -y build-essential` via non-interactive `sudo` (requires NOPASSWD
/// or equivalent). Set to `0`/`false`/`no`/`off` to skip.
pub const ENV_DEVSHELL_VM_AUTO_BUILD_ESSENTIAL: &str = "DEVSHELL_VM_AUTO_BUILD_ESSENTIAL";

/// When unset or truthy (default): if `target/release` from `cargo metadata` (host cwd) lies under the
/// Lima workspace mount, prepend that path in the guest to `PATH` when `exec limactl shell` so `todo`
/// works without an extra mounts entry. Set to `0`/`false`/`no`/`off` to skip.
pub const ENV_DEVSHELL_VM_AUTO_TODO_PATH: &str = "DEVSHELL_VM_AUTO_TODO_PATH";

/// When unset or truthy (default): before `limactl shell`, probe guest for `todo` and print install hints if missing.
/// Set to `0`/`false`/`no`/`off` to skip.
pub const ENV_DEVSHELL_VM_GUEST_TODO_HINT: &str = "DEVSHELL_VM_GUEST_TODO_HINT";

/// When `1`/`true`/`yes`: if guest lacks `todo` but Cargo workspace is under the Lima mount, run
/// `cargo build -p xtask --release --bin todo` in the guest (slow; needs `cargo` + toolchain). Default: off.
pub const ENV_DEVSHELL_VM_AUTO_BUILD_TODO_GUEST: &str = "DEVSHELL_VM_AUTO_BUILD_TODO_GUEST";

/// Name of a symlink in the guest `$HOME` pointing at the host `current_dir` project (default: `host_dir`).
/// Set to `0`/`false`/`off`/`no` to skip `$HOME/<name>` and `~/.todo.json` symlinks (still `cd` into the guest project when under the workspace mount).
pub const ENV_DEVSHELL_VM_GUEST_HOST_DIR: &str = "DEVSHELL_VM_GUEST_HOST_DIR";
