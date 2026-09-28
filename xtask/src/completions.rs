//! Static shell completions for the `cargo xtask` command surface.
//!
//! The command tree is intentionally kept here as a small, reviewed list.  It
//! avoids runtime discovery and therefore keeps completion output deterministic
//! and free of diagnostics.  Dynamic completion is not promised by this API.

use argh::FromArgs;
use std::fmt::Write;

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "completions")]
/// Generate a completion script for bash, zsh, or fish.
pub struct CompletionsArgs {
    /// shell name: bash, zsh, or fish
    #[argh(positional)]
    pub shell: String,
}

pub fn cmd_completions(args: &CompletionsArgs) -> Result<(), String> {
    let script = match args.shell.as_str() {
        "bash" => bash_script(),
        "zsh" => zsh_script(),
        "fish" => fish_script(),
        other => {
            return Err(format!(
                "unsupported shell '{other}'; supported shells: bash, zsh, fish"
            ))
        }
    };
    print!("{script}");
    Ok(())
}

const COMMANDS: &str =
    "acceptance clean clippy completions coverage fmt gh ghcr git lima-todo publish release-baseline run todo";
const TODO_COMMANDS: &str =
    "add complete delete export import init-ai list search show stats update";

fn bash_script() -> String {
    format!(
        r#"# bash completion for cargo xtask (static; dynamic completion is intentionally unsupported)
_xtask() {{
    local cur="${{COMP_WORDS[COMP_CWORD]}}"
    local prev="${{COMP_WORDS[COMP_CWORD-1]}}"
    if [[ $COMP_CWORD -le 2 ]]; then
        COMPREPLY=($(compgen -W "{COMMANDS}" -- "$cur"))
        return
    fi
    if [[ ${{COMP_WORDS[2]}} == todo ]]; then
        if [[ $prev == todo || $COMP_CWORD -eq 3 ]]; then
            COMPREPLY=($(compgen -W "{TODO_COMMANDS} --json --dry-run" -- "$cur"))
        else
            COMPREPLY=($(compgen -W "--json --dry-run --due --priority --tags --status --sort --output --replace --no-next --description" -- "$cur"))
        fi
    fi
}}
complete -F _xtask cargo-xtask
complete -F _xtask xtask
"#
    )
}

fn zsh_script() -> String {
    format!(
        r"#compdef xtask
# Static completion; dynamic completion is intentionally unsupported.
_arguments '1:command:(({COMMANDS}))' '*::argument:(--json --dry-run --due --priority --tags --status --sort --output --replace --no-next {TODO_COMMANDS})'
"
    )
}

fn fish_script() -> String {
    let mut script = String::from(
        "# fish completion for xtask (static; dynamic completion is intentionally unsupported)\n",
    );
    for command in COMMANDS.split_whitespace() {
        let _ = writeln!(
            script,
            "complete -c xtask -n '__fish_use_subcommand' -a {command}"
        );
    }
    for command in TODO_COMMANDS.split_whitespace() {
        let _ = writeln!(
            script,
            "complete -c xtask -n '__fish_seen_subcommand_from todo; and __fish_use_subcommand' -a {command}"
        );
    }
    script.push_str("complete -c xtask -l json -l dry-run -l due -l priority -l tags -l status -l sort -l output -l replace -l no-next\n");
    script
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scripts_are_shell_specific_and_stdout_safe() {
        for shell in ["bash", "zsh", "fish"] {
            let output = match shell {
                "bash" => bash_script(),
                "zsh" => zsh_script(),
                _ => fish_script(),
            };
            assert!(output.contains("todo"));
            assert!(!output.contains("error:"));
        }
    }

    #[test]
    fn unsupported_shell_lists_supported_values() {
        let error = cmd_completions(&CompletionsArgs {
            shell: "pwsh".to_string(),
        })
        .expect_err("unsupported shell");
        assert!(error.contains("bash, zsh, fish"));
    }

    #[test]
    fn generated_scripts_pass_available_shell_parsers() {
        let cases = [
            ("bash", bash_script()),
            ("zsh", zsh_script()),
            ("fish", fish_script()),
        ];
        let path =
            std::env::temp_dir().join(format!("xtask-completions-{}.txt", std::process::id()));
        for (shell, script) in cases {
            std::fs::write(&path, script).expect("write completion script");
            let check = match shell {
                "bash" => std::process::Command::new("bash")
                    .args(["-n", path.to_str().unwrap()])
                    .output(),
                "zsh" => std::process::Command::new("zsh")
                    .args(["-n", path.to_str().unwrap()])
                    .output(),
                _ => std::process::Command::new("fish")
                    .args(["--no-execute", path.to_str().unwrap()])
                    .output(),
            };
            if let Ok(output) = check {
                assert!(
                    output.status.success(),
                    "{shell} parser rejected generated script"
                );
            }
        }
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn completion_catalog_covers_declared_cli_commands() {
        for command in [
            "acceptance",
            "clean",
            "clippy",
            "completions",
            "coverage",
            "fmt",
            "gh",
            "ghcr",
            "git",
            "lima-todo",
            "publish",
            "release-baseline",
            "run",
            "todo",
        ] {
            assert!(COMMANDS
                .split_whitespace()
                .any(|candidate| candidate == command));
        }
        for command in [
            "add", "complete", "delete", "export", "import", "init-ai", "list", "search", "show",
            "stats", "update",
        ] {
            assert!(TODO_COMMANDS
                .split_whitespace()
                .any(|candidate| candidate == command));
        }
    }
}
