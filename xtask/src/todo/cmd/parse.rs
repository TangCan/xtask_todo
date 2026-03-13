//! Parsing helpers for todo CLI: YYYY-MM-DD, `TodoPatch` from add/update args, `ListOptions` from list args.

use std::str::FromStr;

use xtask_todo_lib::{ListFilter, ListOptions, ListSort, Priority, RepeatRule, TodoPatch};

use super::super::error::TodoCliError;

/// Returns true if `s` looks like YYYY-MM-DD (format only; month 01–12, day 01–31).
pub(super) fn is_yyyy_mm_dd(s: &str) -> bool {
    let s = s.trim();
    let b = s.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return false;
    }
    let year_ok = b[0..4].iter().all(u8::is_ascii_digit);
    let month_ok = b[5..7].iter().all(u8::is_ascii_digit);
    let day_ok = b[8..10].iter().all(u8::is_ascii_digit);
    if !(year_ok && month_ok && day_ok) {
        return false;
    }
    let month: u8 = (b[5] - b'0') * 10 + (b[6] - b'0');
    let day: u8 = (b[8] - b'0') * 10 + (b[9] - b'0');
    (1..=12).contains(&month) && (1..=31).contains(&day)
}

/// Build `TodoPatch` from add/update CLI optional fields (title set separately for update).
pub(super) fn patch_from_add_args(
    description: Option<&str>,
    due_date: Option<&str>,
    priority: Option<&str>,
    tags: Option<&str>,
    repeat_rule: Option<&str>,
    repeat_until: Option<&str>,
    repeat_count: Option<&str>,
) -> Result<TodoPatch, TodoCliError> {
    let priority_parsed = priority
        .filter(|s| !s.is_empty())
        .and_then(|s| Priority::from_str(s).ok());
    let repeat_parsed = repeat_rule
        .filter(|s| !s.is_empty())
        .and_then(|s| RepeatRule::from_str(s).ok());
    let repeat_count_parsed = repeat_count
        .filter(|s| !s.is_empty())
        .and_then(|s| s.trim().parse::<u32>().ok());
    if let Some(c) = repeat_count {
        if !c.is_empty() && repeat_count_parsed.is_none() {
            return Err(TodoCliError::Parameter(format!(
                "invalid repeat_count: {c} (expected positive integer)"
            )));
        }
    }
    let tags_vec = tags.map(|s| {
        s.split(',')
            .map(str::trim)
            .filter(|x| !x.is_empty())
            .map(String::from)
            .collect::<Vec<_>>()
    });
    if let Some(p) = priority {
        if !p.is_empty() && priority_parsed.is_none() {
            return Err(TodoCliError::Parameter(format!("invalid priority: {p}")));
        }
    }
    if let Some(r) = repeat_rule {
        if !r.is_empty() && repeat_parsed.is_none() {
            return Err(TodoCliError::Parameter(format!("invalid repeat_rule: {r}")));
        }
    }
    if let Some(d) = due_date {
        let d = d.trim();
        if !d.is_empty() && !is_yyyy_mm_dd(d) {
            return Err(TodoCliError::Parameter(format!(
                "invalid due_date: {d} (expected YYYY-MM-DD)"
            )));
        }
    }
    if let Some(u) = repeat_until {
        let u = u.trim();
        if !u.is_empty() && !is_yyyy_mm_dd(u) {
            return Err(TodoCliError::Parameter(format!(
                "invalid repeat_until: {u} (expected YYYY-MM-DD)"
            )));
        }
    }
    Ok(TodoPatch {
        title: None,
        description: description.map(String::from),
        due_date: due_date
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(String::from),
        priority: priority_parsed,
        tags: tags_vec,
        repeat_rule: repeat_parsed,
        repeat_until: repeat_until
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(String::from),
        repeat_count: repeat_count_parsed,
        repeat_rule_clear: false,
    })
}

/// Build `ListOptions` from list CLI optional filter/sort args.
pub(super) fn list_options_from_args(
    status: Option<&str>,
    priority: Option<&str>,
    tags: Option<&str>,
    due_before: Option<&str>,
    due_after: Option<&str>,
    sort: Option<&str>,
) -> Result<ListOptions, TodoCliError> {
    let status_parsed = match status.filter(|s| !s.is_empty()) {
        None => None,
        Some(s) => {
            let s = s.trim().to_lowercase();
            match s.as_str() {
                "completed" | "done" | "true" => Some(true),
                "incomplete" | "open" | "false" => Some(false),
                _ => {
                    return Err(TodoCliError::Parameter(format!(
                        "invalid status: {s} (use completed or incomplete)"
                    )))
                }
            }
        }
    };

    let priority_parsed = priority
        .filter(|s| !s.is_empty())
        .and_then(|s| Priority::from_str(s).ok());
    if let Some(p) = priority {
        if !p.is_empty() && priority_parsed.is_none() {
            return Err(TodoCliError::Parameter(format!("invalid priority: {p}")));
        }
    }

    let tags_any = tags.map(|s| {
        s.split(',')
            .map(str::trim)
            .filter(|x| !x.is_empty())
            .map(String::from)
            .collect::<Vec<_>>()
    });

    let sort_val = sort
        .filter(|s| !s.is_empty())
        .map(|s| match s.trim().to_lowercase().as_str() {
            "due-date" | "due_date" => ListSort::DueDate,
            "priority" => ListSort::Priority,
            "title" => ListSort::Title,
            _ => ListSort::CreatedAt,
        })
        .unwrap_or_default();

    if let Some(b) = due_before {
        let b = b.trim();
        if !b.is_empty() && !is_yyyy_mm_dd(b) {
            return Err(TodoCliError::Parameter(format!(
                "invalid due_before: {b} (expected YYYY-MM-DD)"
            )));
        }
    }
    if let Some(a) = due_after {
        let a = a.trim();
        if !a.is_empty() && !is_yyyy_mm_dd(a) {
            return Err(TodoCliError::Parameter(format!(
                "invalid due_after: {a} (expected YYYY-MM-DD)"
            )));
        }
    }

    let filter = if status_parsed.is_some()
        || priority_parsed.is_some()
        || tags_any.as_ref().is_some_and(|t| !t.is_empty())
        || due_before.is_some()
        || due_after.is_some()
    {
        Some(ListFilter {
            status: status_parsed,
            priority: priority_parsed,
            tags_any,
            due_before: due_before.map(String::from),
            due_after: due_after.map(String::from),
        })
    } else {
        None
    };

    Ok(ListOptions {
        filter,
        sort: sort_val,
    })
}
