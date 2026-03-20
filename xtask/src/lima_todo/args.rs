//! CLI args and Lima instance / `lima.yaml` path helpers.

use std::path::PathBuf;

use argh::FromArgs;
use xtask_todo_lib::devshell::vm::ENV_DEVSHELL_VM_LIMA_INSTANCE;

/// Default Lima instance name (same as γ / `VmConfig`).
pub(super) const DEFAULT_LIMA_INSTANCE: &str = "devshell-rust";

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "lima-todo")]
/// Build `target/release/todo` and merge `mounts` + `env.PATH` into `~/.lima/<instance>/lima.yaml`
pub struct LimaTodoArgs {
    /// only print a YAML fragment; do not modify `lima.yaml` (legacy behavior)
    #[argh(switch)]
    pub print_only: bool,
    /// skip `cargo build -p xtask --release --bin todo`
    #[argh(switch)]
    pub no_build: bool,
    /// write the same fragment as `--print-only` to this file (UTF-8)
    #[argh(option)]
    pub write: Option<PathBuf>,
    /// guest mountPoint for the host `target/release` directory (default: /host-todo-bin)
    #[argh(option, default = "default_guest_mount()")]
    pub guest_mount: String,
    /// override instance name (default: `$DEVSHELL_VM_LIMA_INSTANCE` or devshell-rust)
    #[argh(option)]
    pub instance: Option<String>,
    /// path to `lima.yaml` (default: ~/.lima/<instance>/lima.yaml)
    #[argh(option)]
    pub lima_yaml: Option<PathBuf>,
    /// after a successful merge, do not run `limactl stop` / `limactl start -y`
    #[argh(switch)]
    pub no_restart: bool,
}

pub(super) fn default_guest_mount() -> String {
    "/host-todo-bin".to_string()
}

pub(super) fn lima_instance_name(args: &LimaTodoArgs) -> String {
    args.instance
        .clone()
        .or_else(|| {
            std::env::var(ENV_DEVSHELL_VM_LIMA_INSTANCE)
                .ok()
                .map(|s| s.trim().to_string())
        })
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_LIMA_INSTANCE.to_string())
}

pub(super) fn default_lima_yaml_path(instance: &str) -> Option<PathBuf> {
    #[cfg(unix)]
    {
        std::env::var_os("HOME").map(|h| {
            PathBuf::from(h)
                .join(".lima")
                .join(instance)
                .join("lima.yaml")
        })
    }
    #[cfg(not(unix))]
    {
        let _ = instance;
        None
    }
}
