//! Binary for `cargo devshell`: delegates to the lib's devshell so all logic is test-covered.

use std::io::Write;

fn main() {
    if let Err(e) = xtask_todo_lib::devshell::run_main() {
        let _ = writeln!(std::io::stderr(), "{e}");
        std::process::exit(1);
    }
}
