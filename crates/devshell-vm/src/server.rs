//! Per-connection state and JSON-line handling for the β sidecar.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::PathBuf;

use crate::guest_fs::{guest_fs_on_host, guest_fs_stub};

/// Per-connection state: after `session_start`, `guest_fs` can use the host staging tree.
#[derive(Debug, Default)]
pub struct ServerState {
    pub session: Option<SessionCtx>,
}

#[derive(Debug, Clone)]
pub struct SessionCtx {
    pub staging_root: PathBuf,
    pub guest_mount: String,
}

impl SessionCtx {
    pub fn map_guest_to_host(&self, guest_path: &str) -> Result<PathBuf, String> {
        let mount = self.guest_mount.trim_end_matches('/');
        let gp = guest_path.trim();
        if !gp.starts_with('/') {
            return Err("guest path must be absolute".into());
        }
        if gp != mount && !gp.starts_with(&format!("{mount}/")) {
            return Err("path outside guest mount".into());
        }
        let rel = if gp == mount {
            ""
        } else {
            gp[mount.len()..].trim_start_matches('/')
        };
        let mut host = self.staging_root.clone();
        for part in rel.split('/').filter(|s| !s.is_empty()) {
            if part == ".." || part == "." {
                return Err("invalid path component".into());
            }
            host.push(part);
        }
        if host.strip_prefix(&self.staging_root).is_err() {
            return Err("path escapes staging dir".into());
        }
        Ok(host)
    }
}

#[cfg(unix)]
pub fn serve_socket(path: &str) -> std::io::Result<()> {
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
        let mut state = ServerState::default();
        while reader.read_line(&mut line)? > 0 {
            let out = handle_line(line.trim(), &mut state);
            writeln!(stream, "{out}")?;
            stream.flush()?;
            line.clear();
        }
    }
    Ok(())
}

/// Same JSON-lines protocol as [`serve_socket`], over TCP (for Windows and portable testing).
pub fn serve_tcp(bind_addr: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(bind_addr)?;
    eprintln!("devshell-vm: listening on tcp {bind_addr}");
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
        let mut state = ServerState::default();
        while reader.read_line(&mut line)? > 0 {
            let out = handle_line(line.trim(), &mut state);
            writeln!(stream, "{out}")?;
            stream.flush()?;
            line.clear();
        }
    }
    Ok(())
}

pub fn handle_line(line: &str, state: &mut ServerState) -> String {
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
        "session_start" => {
            let staging = v.get("staging_dir").and_then(|x| x.as_str()).unwrap_or("");
            let gw = v
                .get("guest_workspace")
                .and_then(|x| x.as_str())
                .unwrap_or("/workspace");
            if staging.is_empty() {
                state.session = None;
            } else {
                state.session = Some(SessionCtx {
                    staging_root: PathBuf::from(staging),
                    guest_mount: gw.to_string(),
                });
            }
            serde_json::json!({
                "op": "session_ok",
                "session_id": sid,
            })
            .to_string()
        }
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
        "session_shutdown" => {
            state.session = None;
            serde_json::json!({
                "op": "shutdown_ok",
                "session_id": sid,
            })
            .to_string()
        }
        "guest_fs" => {
            if let Some(ref ctx) = state.session {
                return guest_fs_on_host(ctx, &v, &sid);
            }
            guest_fs_stub(&sid, &v)
        }
        _ => serde_json::json!({
            "op": "error",
            "code": "unknown_op",
            "message": op,
        })
        .to_string(),
    }
}
