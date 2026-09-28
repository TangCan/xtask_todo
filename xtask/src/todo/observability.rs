//! Stderr-only progress and opt-in structured logging for long todo actions.

use std::io::{self, IsTerminal, Write};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn log_event(level: &str, operation: &str, message: &str, context: Option<&str>) {
    let configured = std::env::var("XTASK_LOG")
        .unwrap_or_default()
        .to_ascii_lowercase();
    if configured.is_empty() || (configured == "info" && level == "debug") {
        return;
    }
    let event = serde_json::json!({
        "time_unix_ms": SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis()),
        "level": level,
        "operation": operation,
        "message": message,
        "context": context,
    });
    let _ = writeln!(io::stderr(), "{event}");
}

pub fn progress(operation: &str, processed: usize, total: usize) {
    let mut stderr = io::stderr();
    if !stderr.is_terminal() {
        return;
    }
    let _ = writeln!(stderr, "{operation}: processed {processed}/{total}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_is_safe_without_terminal() {
        progress("import", 1, 2);
        log_event("info", "import", "started", Some("file.json"));
    }
}
