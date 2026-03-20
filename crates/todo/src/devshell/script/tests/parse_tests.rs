use super::super::ast::ScriptStmt;
use super::super::parse::parse_script;

#[test]
fn parse_script_assign_and_command() {
    let lines = vec!["X=hello".to_string(), "echo $X".to_string()];
    let stmts = parse_script(&lines).unwrap();
    assert_eq!(stmts.len(), 2);
    match &stmts[0] {
        ScriptStmt::Assign(n, v) => {
            assert_eq!(n, "X");
            assert_eq!(v, "hello");
        }
        _ => panic!("expected Assign"),
    }
    match &stmts[1] {
        ScriptStmt::Command(c) => assert_eq!(c, "echo $X"),
        _ => panic!("expected Command"),
    }
}

#[test]
fn parse_script_set_e() {
    let lines = vec!["set -e".to_string(), "echo x".to_string()];
    let stmts = parse_script(&lines).unwrap();
    assert_eq!(stmts.len(), 2);
    assert!(matches!(stmts[0], ScriptStmt::SetE));
}

#[test]
fn parse_script_source() {
    let lines = vec!["source foo.dsh".to_string()];
    let stmts = parse_script(&lines).unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        ScriptStmt::Source(p) => assert_eq!(p, "foo.dsh"),
        _ => panic!("expected Source"),
    }
}

#[test]
fn parse_script_for_loop() {
    let lines = vec![
        "for x in a b c; do".to_string(),
        "echo $x".to_string(),
        "done".to_string(),
    ];
    let stmts = parse_script(&lines).unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        ScriptStmt::For { var, words, body } => {
            assert_eq!(var, "x");
            assert_eq!(words, &["a", "b", "c"]);
            assert_eq!(body.len(), 1);
            assert!(matches!(&body[0], ScriptStmt::Command(c) if c == "echo $x"));
        }
        _ => panic!("expected For"),
    }
}

#[test]
fn parse_script_if_then_fi() {
    let lines = vec![
        "if pwd; then".to_string(),
        "echo yes".to_string(),
        "fi".to_string(),
    ];
    let stmts = parse_script(&lines).unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        ScriptStmt::If {
            cond,
            then_body,
            else_body,
        } => {
            assert_eq!(cond, "pwd");
            assert_eq!(then_body.len(), 1);
            assert!(matches!(&then_body[0], ScriptStmt::Command(c) if c == "echo yes"));
            assert!(else_body.is_none());
        }
        _ => panic!("expected If"),
    }
}

#[test]
fn parse_script_if_missing_fi_err() {
    let lines = vec!["if pwd; then".to_string(), "echo x".to_string()];
    assert!(parse_script(&lines).is_err());
}

#[test]
fn parse_script_while_loop() {
    let lines = vec![
        "while pwd; do".to_string(),
        "echo loop".to_string(),
        "done".to_string(),
    ];
    let stmts = parse_script(&lines).unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        ScriptStmt::While { cond, body } => {
            assert_eq!(cond, "pwd");
            assert_eq!(body.len(), 1);
            assert!(matches!(&body[0], ScriptStmt::Command(c) if c == "echo loop"));
        }
        _ => panic!("expected While"),
    }
}

#[test]
fn parse_script_if_else_fi() {
    let lines = vec![
        "if false; then".to_string(),
        "echo yes".to_string(),
        "else".to_string(),
        "echo no".to_string(),
        "fi".to_string(),
    ];
    let stmts = parse_script(&lines).unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        ScriptStmt::If {
            then_body,
            else_body,
            ..
        } => {
            assert_eq!(then_body.len(), 1);
            assert!(else_body.is_some());
            let else_b = else_body.as_ref().unwrap();
            assert_eq!(else_b.len(), 1);
            assert!(matches!(&else_b[0], ScriptStmt::Command(c) if c == "echo no"));
        }
        _ => panic!("expected If"),
    }
}
