//! Standalone `todo` CLI — same behavior as `cargo xtask todo`.
//!
//! Build: `cargo build -p xtask --release --bin todo`
//!
//! Use in a Lima guest: mount the host `target/release` (or only `todo`) into the VM and add
//! it to `PATH` so `todo` resolves without `cargo`. See `docs/devshell-vm-gamma.md`.

use xtask::todo::args::TodoStandaloneArgs;
use xtask::todo::{print_json_error, run_standalone};

fn main() {
    let cli: TodoStandaloneArgs = argh::from_env();
    let json = cli.json;
    match run_standalone(cli) {
        Ok(()) => {}
        Err(e) => {
            if json {
                print_json_error(e.exit_code(), &e.to_string());
            } else {
                eprintln!("error: {e}");
            }
            std::process::exit(e.exit_code());
        }
    }
}
