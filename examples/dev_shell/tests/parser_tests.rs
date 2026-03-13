use dev_shell::parser::parse_line;

#[test]
fn parse_simple() {
    let p = parse_line("echo hello").unwrap();
    assert_eq!(p.commands.len(), 1);
    assert_eq!(p.commands[0].argv, ["echo", "hello"]);
}

#[test]
fn parse_redirect_out() {
    let p = parse_line("echo hi > out").unwrap();
    assert_eq!(p.commands[0].redirects.len(), 1);
    assert_eq!(p.commands[0].redirects[0].path, "out");
    // fd for > is 1
    assert_eq!(p.commands[0].redirects[0].fd, 1);
}
