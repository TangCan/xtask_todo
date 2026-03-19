//! Tests for todo CLI error types.

use crate::todo::error::{TodoCliError, EXIT_DATA, EXIT_GENERAL, EXIT_PARAMETER};

#[test]
fn todo_cli_error_exit_code() {
    let param = TodoCliError::Parameter("bad id".into());
    assert_eq!(param.exit_code(), EXIT_PARAMETER);
    assert_eq!(param.exit_code(), 2);

    let data = TodoCliError::Data("not found".into());
    assert_eq!(data.exit_code(), EXIT_DATA);
    assert_eq!(data.exit_code(), 3);

    let general = TodoCliError::General("io error".into());
    assert_eq!(general.exit_code(), EXIT_GENERAL);
    assert_eq!(general.exit_code(), 1);
}

#[test]
fn todo_cli_error_display() {
    assert_eq!(
        TodoCliError::Parameter("invalid id 0".into()).to_string(),
        "invalid id 0"
    );
    assert_eq!(
        TodoCliError::Data("todo not found: 99".into()).to_string(),
        "todo not found: 99"
    );
    assert_eq!(
        TodoCliError::General(Box::new(std::io::Error::other("permission denied"))).to_string(),
        "permission denied"
    );
}
