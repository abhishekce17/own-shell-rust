# Shell Built with Rust

A custom shell implementation written from scratch in Rust, built as part of the [CodeCrafters "Build Your Own Shell" Challenge](https://app.codecrafters.io/courses/shell/overview). This is a fully interactive, POSIX-inspired shell that runs on both **Windows** and **Unix/Linux**, featuring raw-mode keypress handling, tab autocompletion, command history, pipelines, I/O redirection, and more.

---

## What I Learned

The primary goal of this project was **learning Rust** by building something that talks directly to hardware-level terminal APIs. Some highlights:

### Raw Mode & Terminal Control

Before this project I had no idea how shells actually capture individual keypresses. Using [crossterm](https://docs.rs/crossterm), I learned about **raw mode** — where the terminal stops buffering input line-by-line and instead delivers every single keypress event to the program. This enabled me to build:

- Flicker-free rendering by queuing draw commands before flushing to stdout
- Real-time cursor movement with arrow keys
- Tab autocompletion with bell character (`\x07`) feedback
- `Ctrl+C` / `Ctrl+D` signal handling at the keystroke level

### Rust Ownership & Borrowing Rules I Internalized

Working on this project solidified the following mental model for Rust's ownership system:

| Situation                                                  | What to do                                                     |
| ---------------------------------------------------------- | -------------------------------------------------------------- |
| **Read data**                                              | Pass a reference (`&T`)                                        |
| **Read data from a collection**                            | Pass it as a slice (`&[T]`)                                    |
| **Data needed only for a function to produce something**   | Pass ownership (move, not `&mut`)                              |
| **Mutate / transform data**                                | Pass a mutable reference (`&mut T`)                            |
| **Create data inside a function and need it outside**      | Return the data itself (move it out)                           |
| **Never** return a reference to a function-scoped variable | The borrow checker won't allow it — the data would be dangling |
| **You can** return a reference to passed-in data           | Because the data lives in the caller's scope (or is `'static`) |
| **Wrap failure-prone operations**                          | Use `Option<T>` and `Result<T, E>`                             |

---

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (with `cargo`) — tested with Rust 1.92+

### Build

```sh
cargo build --release
```

The compiled binary will be located at:

- **Windows:** `target/release/codecrafters-shell.exe`
- **Linux / macOS:** `target/release/codecrafters-shell`

### Run

```sh
# Run directly after building
./target/release/codecrafters-shell

# Or run via cargo
cargo run --release
```

### History File (Optional)

Set the `HISTFILE` environment variable to persist command history across sessions:

```sh
# Unix / macOS
export HISTFILE=~/.myshell_history

# Windows (PowerShell)
$env:HISTFILE = "$HOME\.myshell_history"
```

---

## Built-in Commands

| Command   | Usage            | Description                                                                                      |
| --------- | ---------------- | ------------------------------------------------------------------------------------------------ |
| `echo`    | `echo <args...>` | Prints arguments to stdout, separated by spaces                                                  |
| `pwd`     | `pwd`            | Prints the current working directory                                                             |
| `cd`      | `cd <path>`      | Changes the working directory. Supports `~`, `..`, `.`, and absolute paths                       |
| `ls`      | `ls [path]`      | Lists files and directories (directories shown with trailing `/`). Defaults to current directory |
| `mkdir`   | `mkdir <dir...>` | Creates one or more directories (including nested paths via `create_dir_all`)                    |
| `type`    | `type <command>` | Shows whether a command is a shell builtin or an external program, and its path                  |
| `history` | `history [n]`    | Displays command history. Pass a number to show only the last N entries                          |
| `cls`     | `cls`            | Clears the terminal screen and scrollback buffer                                                 |
| `exit`    | `exit`           | Exits the shell and persists history to `HISTFILE` (if set)                                      |

### History Sub-commands

| Flag | Usage               | Description                                                             |
| ---- | ------------------- | ----------------------------------------------------------------------- |
| `-r` | `history -r <file>` | **Read** — replaces the in-memory history with the contents of `<file>` |
| `-a` | `history -a <file>` | **Append** — appends only the new (unwritten) commands to `<file>`      |
| `-w` | `history -w <file>` | **Write** — overwrites `<file>` with the full in-memory history         |

---

## Features

### External Program Execution

Any command that isn't a builtin is looked up in the system `PATH`. The shell resolves executables (including `.exe`, `.bat`, `.cmd`, `.com` on Windows) and runs them as child processes.

### I/O Redirection

| Operator      | Description                           |
| ------------- | ------------------------------------- |
| `>` or `1>`   | Redirect stdout to a file (overwrite) |
| `>>` or `1>>` | Redirect stdout to a file (append)    |
| `2>`          | Redirect stderr to a file (overwrite) |
| `2>>`         | Redirect stderr to a file (append)    |

**Example:**

```sh
echo hello > output.txt
ls /nonexistent 2> errors.log
echo world >> output.txt
```

### Pipelines

Chain multiple commands together with `|`. Builtins and external programs can be mixed freely in a pipeline.

```sh
ls | grep src
echo hello world | cat
```

### Tab Autocompletion

- **Builtin commands:** Type a partial command and press `Tab` to complete it
- **External programs:** Matches executables from `PATH`; press `Tab` twice to list all matches
- **File/directory paths:** After a space, `Tab` completes file and directory names relative to the current or specified directory
- **Longest common prefix:** When multiple matches exist, fills in as much as possible automatically

### Interactive Line Editing

| Key              | Action                                                            |
| ---------------- | ----------------------------------------------------------------- |
| `Left` / `Right` | Move cursor within the line                                       |
| `Backspace`      | Delete the character before the cursor                            |
| `Up` / `Down`    | Navigate through command history                                  |
| `Ctrl+C`         | Cancel the current input                                          |
| `Ctrl+D`         | Exit the shell                                                    |
| `Tab`            | Autocomplete (single press completes, double press lists options) |

### Quoting & Escaping

The argument parser handles single quotes (`'...'`), double quotes (`"..."`), and backslash escaping (`\`) following POSIX conventions.

### Cross-Platform Support

The shell compiles and runs on both Windows and Unix. Platform-specific behavior (executable detection, permission checks) is handled via conditional compilation (`#[cfg(windows)]` / `#[cfg(unix)]`).

---

## Project Structure

```
src/
  main.rs          — All shell logic: REPL, builtins, parsing, pipelines, I/O redirection
Cargo.toml         — Dependencies (crossterm, anyhow)
your_program.sh    — CodeCrafters runner script
```

---

## Acknowledgements

Built as part of the [CodeCrafters](https://codecrafters.io) "Build Your Own Shell" challenge.
