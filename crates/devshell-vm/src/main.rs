//! `devshell-vm` — β sidecar (stub server).
//!
//! - Default: print stub line (stdout).
//! - `devshell-vm --serve-socket <path>` (**Unix**): listen for JSON-lines (one per line); see
//!   `docs/superpowers/specs/2026-03-11-devshell-vm-ipc-draft.md`.

use std::io::{BufRead, BufReader, Write};

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
            if let Err(e) = serve_socket(&path) {
                eprintln!("devshell-vm: {e}");
                std::process::exit(1);
            }
        }
        _ => {
            println!("devshell-vm 0.0.0 stub");
            #[cfg(unix)]
            eprintln!("β server (Unix): devshell-vm --serve-socket /path/to.sock");
        }
    }
}

#[cfg(unix)]
fn serve_socket(path: &str) -> std::io::Result<()> {
    use std::os::unix::net::UnixListener;

    let _ = std::fs::remove_file(path);
    let listener = UnixListener::bind(path)?;
    eprintln!("devshell-vm: listening on {path}");
    for incoming in listener.incoming() {
        let mut stream = match incoming {
            Ok(s) => s,
            Err(e) => {
                eprintln!("devshell-vm: accept: {e}");
                continue;
            }
        };
        let mut reader = BufReader::new(stream.try_clone()?);
        let mut line = String::new();
        while reader.read_line(&mut line)? > 0 {
            let out = handle_line(line.trim());
            writeln!(stream, "{out}")?;
            stream.flush()?;
            line.clear();
        }
    }
    Ok(())
}

fn handle_line(line: &str) -> String {
    if line.is_empty() {
        return r#"{"op":"error","code":"empty","message":"empty line"}"#.to_string();
    }
    let v: serde_json::Value = match serde_json::from_str(line) {
        Ok(v) => v,
        Err(e) => {
            return serde_json::json!({
                "op": "error",
                "code": "parse",
                "message": e.to_string(),
            })
            .to_string();
        }
    };
    let op = v.get("op").and_then(|x| x.as_str()).unwrap_or("");
    let sid = v
        .get("session_id")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    match op {
        "handshake" => serde_json::json!({
            "op": "handshake_ok",
            "version": 1u64,
            "server": "devshell-vm",
            "server_version": env!("CARGO_PKG_VERSION"),
        })
        .to_string(),
        "session_start" => serde_json::json!({
            "op": "session_ok",
            "session_id": sid,
        })
        .to_string(),
        "sync_request" => serde_json::json!({
            "op": "sync_ok",
            "session_id": sid,
        })
        .to_string(),
        "exec" => {
            let fail = v.get("argv").and_then(|a| a.as_array()).is_some_and(|a| {
                a.iter()
                    .any(|x| x.as_str() == Some("--devshell-vm-test-fail"))
            });
            let code = i64::from(fail);
            serde_json::json!({
                "op": "exec_result",
                "session_id": sid,
                "exit_code": code,
                "signal": serde_json::Value::Null,
            })
            .to_string()
        }
        "session_shutdown" => serde_json::json!({
            "op": "shutdown_ok",
            "session_id": sid,
        })
        .to_string(),
        _ => serde_json::json!({
            "op": "error",
            "code": "unknown_op",
            "message": op,
        })
        .to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::handle_line;

    #[test]
    fn handle_handshake() {
        let out = handle_line(r#"{"op":"handshake","version":1}"#);
        assert!(out.contains("handshake_ok"));
    }

    #[test]
    fn handle_exec_fail_flag() {
        let out = handle_line(
            r#"{"op":"exec","session_id":"s","argv":["cargo","--devshell-vm-test-fail"]}"#,
        );
        assert!(out.contains("\"exit_code\":1"));
    }
}
