//! Time/duration formatting and "old open" hint for todo list display.

use std::time::{Duration, SystemTime};

use xtask_todo_lib::Todo;

pub const AGE_THRESHOLD_DAYS: u64 = 7;

#[must_use]
pub fn format_time_ago(when: SystemTime) -> String {
    let now = SystemTime::now();
    let d = now.duration_since(when).unwrap_or(Duration::ZERO);
    let s = d.as_secs();
    if s < 60 {
        "just now".into()
    } else if s < 3600 {
        format!("{}m ago", s / 60)
    } else if s < 86400 {
        format!("{}h ago", s / 3600)
    } else {
        format!("{}d ago", s / 86400)
    }
}

#[must_use]
pub fn format_duration(d: Duration) -> String {
    let s = d.as_secs();
    if s < 60 {
        format!("{s}s")
    } else if s < 3600 {
        format!("{}m", s / 60)
    } else if s < 86400 {
        format!("{}h", s / 3600)
    } else {
        format!("{}d", s / 86400)
    }
}

#[must_use]
pub fn is_old_open(t: &Todo, now: SystemTime) -> bool {
    if t.completed {
        return false;
    }
    let age = now.duration_since(t.created_at).unwrap_or(Duration::ZERO);
    age.as_secs() >= AGE_THRESHOLD_DAYS * 86400
}

/// Prints todo list items to stdout. Used by list subcommand and tests.
///
/// # Panics
/// May panic if writing to stdout fails (e.g. broken pipe).
pub fn print_todo_list_items(items: &[Todo], use_color: bool) {
    let now = SystemTime::now();
    if items.is_empty() {
        println!("No tasks.");
    } else {
        for t in items {
            let mark = if t.completed { "✓" } else { " " };
            let created = format_time_ago(t.created_at);
            let time_info = t.completed_at.as_ref().map_or_else(
                || format!("  创建 {created}"),
                |cat| {
                    let completed = format_time_ago(*cat);
                    let took = cat
                        .duration_since(t.created_at)
                        .ok()
                        .map(format_duration)
                        .map(|s| format!("  用时 {s}"))
                        .unwrap_or_default();
                    format!("  创建 {created}  完成 {completed}{took}")
                },
            );
            let line = format!("  [{}] {} {}  {}", t.id, mark, t.title, time_info);
            if use_color && is_old_open(t, now) {
                println!("\x1b[33m{line}\x1b[0m");
            } else {
                println!("{line}");
            }
        }
    }
}
