//! Environment variable names for γ (Lima) session.

/// Override path to `limactl` (default: `PATH`).
pub const ENV_DEVSHELL_VM_LIMACTL: &str = "DEVSHELL_VM_LIMACTL";

/// Host directory we push/pull (must be mounted at [`super::GammaSession::guest_mount`] in the Lima VM).
pub const ENV_DEVSHELL_VM_WORKSPACE_PARENT: &str = "DEVSHELL_VM_WORKSPACE_PARENT";

/// Guest mount point for that directory (default `/workspace`).
pub const ENV_DEVSHELL_VM_GUEST_WORKSPACE: &str = "DEVSHELL_VM_GUEST_WORKSPACE";

/// When set truthy, run `limactl stop` on session shutdown.
pub const ENV_DEVSHELL_VM_STOP_ON_EXIT: &str = "DEVSHELL_VM_STOP_ON_EXIT";

/// When unset or truthy (default): after the VM is running, probe for `gcc` in the guest and, if
/// missing, try `apt-get install -y build-essential` via non-interactive `sudo` (requires NOPASSWD
/// or equivalent). Set to `0`/`false`/`no`/`off` to skip.
pub const ENV_DEVSHELL_VM_AUTO_BUILD_ESSENTIAL: &str = "DEVSHELL_VM_AUTO_BUILD_ESSENTIAL";
