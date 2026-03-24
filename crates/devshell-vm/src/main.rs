//! `devshell-vm` — β sidecar: JSON-lines IPC (Unix socket, **TCP**, or **stdio**).
//!
//! - No args: print version and usage hints (**stderr**).
//! - **`--serve-stdio`** / **`--serve-tcp`** / (**Unix**) **`--serve-socket`**: protocol in
//!   `docs/superpowers/specs/2026-03-11-devshell-vm-ipc-draft.md` and `docs/requirements.md` §5.8.
//!
//! **`guest_fs`**: after **`session_start`** with **`staging_dir`**, paths under **`guest_workspace`**
//! map to the host staging tree. **Without** **`session_start`**, `guest_fs` returns **fixed canned
//! responses** (for unit tests only — not a real filesystem).
//!
//! **`exec`**: runs a real child process on the mapped host directory; child **stdout/stderr** are
//! piped to this process’s **stderr** so **stdout** stays JSON-only (stdio transport).

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
        Some("--serve-stdio") => {
            if let Err(e) = server::serve_stdio() {
                eprintln!("devshell-vm: {e}");
                std::process::exit(1);
            }
        }
        _ => {
            println!("devshell-vm {}", env!("CARGO_PKG_VERSION"));
            eprintln!(
                "β server (stdio): devshell-vm --serve-stdio   # e.g. via podman machine ssh"
            );
            eprintln!("β server (TCP): devshell-vm --serve-tcp 127.0.0.1:9847");
            #[cfg(unix)]
            eprintln!("β server (Unix socket): devshell-vm --serve-socket /path/to.sock");
        }
    }
}
