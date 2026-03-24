//! Per-connection state and JSON-line handling for the β sidecar.

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;

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

fn exec_status_fields(status: std::process::ExitStatus) -> (i64, serde_json::Value) {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(sig) = status.signal() {
            let code = i64::from(128 + sig);
            return (code, serde_json::json!(sig));
        }
    }
    let code = status.code().map_or(-1, i64::from);
    (code, serde_json::Value::Null)
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

/// Same JSON-lines protocol as [`serve_socket`], over **stdin/stdout** (one client, line-delimited).
/// Used with **`podman machine ssh`** on Windows so the host does not need a local TCP listener.
pub fn serve_stdio() -> std::io::Result<()> {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut line = String::new();
    let mut state = ServerState::default();
    while reader.read_line(&mut line)? > 0 {
        let out = handle_line(line.trim(), &mut state);
        writeln!(stdout, "{out}")?;
        stdout.flush()?;
        line.clear();
    }
    Ok(())
}

/// Same JSON-lines protocol as [`serve_socket`], over TCP (for Windows and portable testing).
pub fn serve_tcp(bind_addr: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(bind_addr)?;
    eprintln!("devshell-vm: listening on tcp {bind_addr}");
    for incoming in listener.incoming() {
        let stream = match incoming {
            Ok(s) => s,
            Err(e) => {
                eprintln!("devshell-vm: accept: {e}");
                continue;
            }
        };
        if let Err(e) = run_one_tcp_connection(stream) {
            eprintln!("devshell-vm: connection: {e}");
        }
    }
    Ok(())
}

/// One TCP client: read JSON lines until EOF, reply on the same stream (used by [`serve_tcp`] and tests).
pub fn run_one_tcp_connection(mut stream: TcpStream) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut line = String::new();
    let mut state = ServerState::default();
    while reader.read_line(&mut line)? > 0 {
        let out = handle_line(line.trim(), &mut state);
        writeln!(stream, "{out}")?;
        stream.flush()?;
        line.clear();
    }
    Ok(())
}

/// Runs `argv` in `host_cwd`, pipes child stdout/stderr to our stderr, returns JSON `exec_result` or error.
fn run_exec_command(
    argv: &[String],
    host_cwd: &Path,
    v: &serde_json::Value,
    sid: &serde_json::Value,
) -> String {
    let mut cmd = Command::new(&argv[0]);
    cmd.args(&argv[1..]);
    cmd.current_dir(host_cwd);
    // Stdio transport uses this process's **stdout** for JSON lines only. Child processes must not
    // inherit that fd — e.g. `cargo run` would print program output to stdout and break the next
    // `read_json_line` on the host. Pipe child stdout/stderr and forward both to **our stderr**
    // (typically still visible in the user's terminal; IPC stays clean).
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    if let Some(env_obj) = v.get("env").and_then(|e| e.as_object()) {
        for (k, val) in env_obj {
            if let Some(s) = val.as_str() {
                cmd.env(k, s);
            }
        }
    }

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            return serde_json::json!({
                "op": "error",
                "code": "exec_spawn",
                "message": e.to_string(),
            })
            .to_string();
        }
    };

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let drain_out = thread::spawn(move || {
        if let Some(mut out) = stdout {
            let mut err = std::io::stderr().lock();
            let _ = std::io::copy(&mut out, &mut err);
        }
    });
    let drain_err = thread::spawn(move || {
        if let Some(mut err_in) = stderr {
            let mut err = std::io::stderr().lock();
            let _ = std::io::copy(&mut err_in, &mut err);
        }
    });

    let status = child.wait();
    let _ = drain_out.join();
    let _ = drain_err.join();

    match status {
        Ok(status) => {
            let (exit_code, signal) = exec_status_fields(status);
            serde_json::json!({
                "op": "exec_result",
                "session_id": sid,
                "exit_code": exit_code,
                "signal": signal,
            })
            .to_string()
        }
        Err(e) => serde_json::json!({
            "op": "error",
            "code": "exec_wait",
            "message": e.to_string(),
        })
        .to_string(),
    }
}

fn handle_exec(v: &serde_json::Value, state: &ServerState, sid: &serde_json::Value) -> String {
    let argv: Vec<String> = v
        .get("argv")
        .and_then(|a| a.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    let fail = argv.iter().any(|x| x.as_str() == "--devshell-vm-test-fail");
    if fail {
        return serde_json::json!({
            "op": "exec_result",
            "session_id": sid,
            "exit_code": 1i64,
            "signal": serde_json::Value::Null,
        })
        .to_string();
    }

    if argv.is_empty() {
        return serde_json::json!({
            "op": "error",
            "code": "exec",
            "message": "empty argv",
        })
        .to_string();
    }

    let Some(ref ctx) = state.session else {
        return serde_json::json!({
            "op": "error",
            "code": "no_session",
            "message": "exec requires session_start",
        })
        .to_string();
    };

    let guest_cwd = v
        .get("guest_cwd")
        .and_then(|x| x.as_str())
        .unwrap_or(ctx.guest_mount.as_str());

    let host_cwd = match ctx.map_guest_to_host(guest_cwd) {
        Ok(p) => p,
        Err(e) => {
            return serde_json::json!({
                "op": "error",
                "code": "bad_guest_cwd",
                "message": e,
            })
            .to_string();
        }
    };

    if !host_cwd.is_dir() {
        return serde_json::json!({
            "op": "error",
            "code": "bad_guest_cwd",
            "message": "guest cwd is not an existing directory on the host",
        })
        .to_string();
    }

    run_exec_command(&argv, &host_cwd, v, sid)
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
        "exec" => handle_exec(&v, state, &sid),
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
