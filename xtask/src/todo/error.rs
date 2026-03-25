//! CLI error types and JSON output helpers for the todo subcommand.

use xtask_todo_lib::Todo;

/// Exit codes: 0 success, 1 general, 2 parameter, 3 data (e.g. not found).
pub const EXIT_GENERAL: i32 = 1;
pub const EXIT_PARAMETER: i32 = 2;
pub const EXIT_DATA: i32 = 3;

/// CLI error with exit code for todo subcommand.
#[derive(Debug)]
pub enum TodoCliError {
    General(Box<dyn std::error::Error>),
    Parameter(String),
    Data(String),
}

impl TodoCliError {
    #[must_use]
    pub const fn exit_code(&self) -> i32 {
        match self {
            Self::Parameter(_) => EXIT_PARAMETER,
            Self::Data(_) => EXIT_DATA,
            Self::General(_) => EXIT_GENERAL,
        }
    }
}

impl std::fmt::Display for TodoCliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::General(e) => write!(f, "{e}"),
            Self::Parameter(s) | Self::Data(s) => write!(f, "{s}"),
        }
    }
}

impl std::error::Error for TodoCliError {}

/// Unified JSON success payload.
#[derive(serde::Serialize)]
pub struct TodoJsonSuccess<T> {
    pub status: &'static str,
    pub data: T,
}

/// Unified JSON error payload (error output with `--json`).
#[derive(serde::Serialize)]
pub struct TodoJsonError {
    pub status: &'static str,
    pub error: TodoJsonErrorBody,
}

#[derive(serde::Serialize)]
pub struct TodoJsonErrorBody {
    pub code: i32,
    pub message: String,
}

pub(super) fn print_json_success<T: serde::Serialize>(data: &T) {
    let out = TodoJsonSuccess {
        status: "success",
        data,
    };
    println!("{}", serde_json::to_string(&out).expect("serialize"));
}

/// Print JSON error to stdout (used when `cmd_todo` fails with `--json`).
pub(super) fn print_json_error(code: i32, message: &str) {
    let out = TodoJsonError {
        status: "error",
        error: TodoJsonErrorBody {
            code,
            message: message.to_string(),
        },
    };
    println!("{}", serde_json::to_string(&out).expect("serialize"));
}

/// JSON `data` payload for `list` / `search` when `--json`: `items`, `empty`, and `message` when empty.
#[must_use]
pub(super) fn todo_list_json_payload(items: &[Todo]) -> serde_json::Value {
    let data: Vec<serde_json::Value> = items.iter().map(todo_to_json).collect();
    let empty = data.is_empty();
    if empty {
        serde_json::json!({
            "items": data,
            "empty": true,
            "message": super::format::EMPTY_LIST_MESSAGE,
        })
    } else {
        serde_json::json!({
            "items": data,
            "empty": false,
        })
    }
}

pub(super) fn todo_to_json(t: &Todo) -> serde_json::Value {
    serde_json::json!({
        "id": t.id.as_u64(),
        "title": t.title,
        "completed": t.completed,
        "description": t.description,
        "due_date": t.due_date,
        "priority": t.priority.as_ref().map(ToString::to_string),
        "tags": t.tags,
        "repeat_rule": t.repeat_rule.as_ref().map(ToString::to_string),
        "repeat_until": t.repeat_until,
        "repeat_count": t.repeat_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_json_error_prints_valid_json() {
        print_json_error(2, "title must be non-empty");
    }

    #[test]
    fn todo_list_json_payload_empty_has_message_and_empty_flag() {
        let v = super::todo_list_json_payload(&[]);
        assert_eq!(v["empty"], true);
        assert_eq!(v["message"], "No tasks.");
        assert_eq!(v["items"], serde_json::json!([]));
    }

    #[test]
    fn todo_list_json_payload_nonempty_has_empty_false() {
        let id = xtask_todo_lib::TodoId::from_raw(1).unwrap();
        let t = Todo {
            id,
            title: "x".into(),
            completed: false,
            created_at: std::time::SystemTime::UNIX_EPOCH,
            completed_at: None,
            description: None,
            due_date: None,
            priority: None,
            tags: vec![],
            repeat_rule: None,
            repeat_until: None,
            repeat_count: None,
        };
        let v = super::todo_list_json_payload(std::slice::from_ref(&t));
        assert_eq!(v["empty"], false);
        assert!(v.get("message").is_none());
        assert_eq!(v["items"].as_array().unwrap().len(), 1);
    }
}
