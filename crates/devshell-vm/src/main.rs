//! `devshell-vm` — β sidecar: JSON-lines IPC over a Unix socket.
//!
//! - Default: print stub line (stdout).
//! - `devshell-vm --serve-socket <path>`: listen for JSON-lines; see
//!   `docs/superpowers/specs/2026-03-11-devshell-vm-ipc-draft.md`.
//!
//! After a **`session_start`** with **`staging_dir`**, **`guest_fs`** maps **`guest_path`** under
//! **`guest_workspace`** (e.g. `/workspace`) to files under that host directory (development /
//! local testing). Without a session, **`guest_fs`** falls back to canned stub responses (unit tests).

mod guest_fs;
mod server;

#[cfg(test)]
mod tests;

fn main() {
    let mut args = std::env::args();
    let _exe = args.next();
    match args.next().as_deref() {
        #[cfg(unix)]
        Some("--serve-socket") => {
            let path = args.next().unwrap_or_default();
            if path.is_empty() {
                eprintln!("usage: devshell-vm --serve-socket <path>");
                std::process::exit(2);
            }
            if let Err(e) = server::serve_socket(&path) {
                eprintln!("devshell-vm: {e}");
                std::process::exit(1);
            }
        }
        Some("--serve-tcp") => {
            let addr = args.next().unwrap_or_default();
            if addr.is_empty() {
                eprintln!("usage: devshell-vm --serve-tcp <host:port>");
                std::process::exit(2);
            }
            if let Err(e) = server::serve_tcp(&addr) {
                eprintln!("devshell-vm: {e}");
                std::process::exit(1);
            }
        }
        _ => {
            println!("devshell-vm {}", env!("CARGO_PKG_VERSION"));
            eprintln!("β server (TCP): devshell-vm --serve-tcp 127.0.0.1:9847");
            #[cfg(unix)]
            eprintln!("β server (Unix socket): devshell-vm --serve-socket /path/to.sock");
        }
    }
}
