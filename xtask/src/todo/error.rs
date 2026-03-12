//! CLI error types and JSON output helpers for the todo subcommand.

use xtask_todo_lib::Todo;

/// Exit codes: 0 success, 1 general, 2 parameter, 3 data (e.g. not found).
#[allow(dead_code)]
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

/// Unified JSON error payload (for future use: error output with --json).
#[allow(dead_code)]
#[derive(serde::Serialize)]
pub struct TodoJsonError {
    pub status: &'static str,
    pub error: TodoJsonErrorBody,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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
    })
}
