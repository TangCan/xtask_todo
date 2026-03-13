# dev_shell

A development shell with a virtual filesystem, cross-platform (Linux, macOS, Windows), written in Rust.

## Features

- **Virtual filesystem** — In-memory directory tree; no host filesystem writes except via explicit commands.
- **.bin persistence** — Save and load the virtual FS to/from a single `.bin` file. The shell **auto-saves** to the current session’s .bin path when you exit (exit/quit or EOF), so you don’t need to run `save` before leaving.
- **Built-in commands only** — No host process execution; all commands operate on the virtual FS.
- **export-readonly** — Export a subtree to a host temp directory; path is printed to stdout so you can inspect files on the host.
- **Tab completion** — In interactive mode (TTY), press Tab to complete command names and path/file names in the virtual FS.

## Build & Run

```bash
cargo build
cargo run [path]
```

`path` is optional. If omitted, the shell uses `.dev_shell.bin` in the current directory. If given, it should be the path to a `.bin` file (created with `save` or used as a new file).

The prompt shows the current working directory in the virtual FS, e.g. `"/ $ "` or `"/foo $ "`. Use `help` to list all supported commands.

## Built-in commands

| Command | Description |
|--------|-------------|
| `pwd` | Print working directory |
| `cd` *path* | Change directory |
| `ls` [*path*] | List directory (default: current) |
| `mkdir` *path* | Create directory (and parents) |
| `cat` [*path* ...] | Print file contents (or stdin) |
| `touch` *path* | Create empty file |
| `echo` *args* | Print arguments to stdout |
| `save` [*path*] | Save VFS to .bin file (default: `.dev_shell.bin`) |
| `export-readonly` [*path*] | Export subtree to host temp dir; prints path to stdout |
| `exit` / `quit` | Exit the shell |
| `help` | List all supported commands |

## Usage example

Start the shell, create a directory, then exit. The VFS is auto-saved to the session’s .bin path; next run loads it.

```bash
# First session: create /foo and exit (auto-saved to .dev_shell.bin)
cargo run
/ $ mkdir /foo
/ $ exit

# Second session: same .bin is loaded by default; list root
cargo run
/ $ ls /
foo
/ $ quit
```

To use a different .bin file: `cargo run -- my.bin` (loads/saves `my.bin` for that session).

### export-readonly example

Export the current (or given) path to a temporary directory on the host; the command prints the absolute path so you can open it in your file manager or editor:

```bash
/ $ mkdir /out
/ $ echo hello > /out/greeting.txt
/ $ export-readonly /out
/tmp/dev_shell_export_12345_1710000000000
```

Then on the host: open that path (e.g. `ls /tmp/dev_shell_export_12345_...`) to inspect the exported files.

## License

See repository for license terms.
