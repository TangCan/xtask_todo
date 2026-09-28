# Shell completions

Generate static completion scripts for the `cargo xtask` command:

```sh
cargo xtask completions bash > ~/.local/share/bash-completion/completions/xtask
cargo xtask completions zsh > ~/.zfunc/_xtask
cargo xtask completions fish > ~/.config/fish/completions/xtask.fish
```

The supported shells are `bash`, `zsh`, and `fish`. The generated scripts
cover the top-level commands, the `todo` subcommands, and the documented
static options. Completion is deliberately static: this interface does not
perform filesystem, network, or process discovery, and dynamic completion is
not part of the compatibility contract. Regenerate scripts after changing the
`argh` command definitions.
