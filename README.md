# ZenDroid

`zendroid` is a terminal UI for Android development that borrows the basic
shape of Android Studio while staying far lighter on RAM: file explorer on the
left, code editor in the middle, Gradle tasks on the right, and logs at the
bottom.

It is built in Rust with `ratatui` and `crossterm`, with Gradle used as the
project/task backend.

## Why this exists

Android Studio can become expensive in RAM-heavy environments, especially when
you mostly want:

- a real project tree
- a built-in editor
- a visible Gradle task list
- a bottom log console
- explicit task execution without hidden automation

For those cases, a terminal IDE can cover a lot of the daily loop.

## What it does

`zendroid` combines:

- Android project root discovery
- a keyboard-first file explorer
- a built-in multi-tab text editor with lightweight syntax highlighting
- Gradle task discovery and filtering
- explicit confirm-before-run task execution
- a live bottom-pane log/output console
- session restore for open files and pane state

## Requirements

- Linux or macOS shell environment with Rust
- A Gradle Android project for project-level commands

## Install

Build the binary:

```bash
cargo build --release
```

Then install it wherever you keep local binaries:

```bash
mkdir -p ~/.local/bin
cp ./target/release/zendroid ~/.local/bin/zendroid
chmod +x ~/.local/bin/zendroid
```

## Quick start

Open the current Android project:

```bash
zendroid
```

Open a specific project:

```bash
zendroid --project ~/AndroidStudioProjects/DocSafe
```

Open in safe read-only mode:

```bash
zendroid --project ~/AndroidStudioProjects/DocSafe --read-only
```

## CLI options

```bash
zendroid
zendroid <project-path>
zendroid --project <project-path>
zendroid --read-only
zendroid --theme amber
zendroid --config /path/to/config.json
```

## Default keymap

- `Alt-1` / `Alt-2` / `Alt-3` / `Alt-4`: jump directly to Files, Editor, Tasks, or Logs
- `Tab` / `Shift-Tab`: switch panes
- `Alt-h` / `Alt-l`: resize the focused side pane
- `Alt-j` / `Alt-k`: resize the logs pane
- `Alt--`: collapse the focused non-editor pane
- `Alt-=`: reset the focused pane size
- `Ctrl-S`: save current file
- `Ctrl-W`: close current tab
- `F1` or `?`: help overlay
- `q`: quit

### File explorer

- `Up` / `Down`: move selection
- `Enter` / `Right`: expand directory or open file
- `Left`: collapse directory
- `r`: refresh tree

### Editor

- `Arrows`: move cursor
- `Type`: insert text
- `Enter`: newline
- `Backspace`: delete
- `/`: search in current file
- `[` / `]`: switch tabs
- syntax colors: lightweight keyword, string, comment, type, and number highlighting

### Tasks

- `Up` / `Down`: move selection
- `Enter`: open run confirmation
- `g`: refresh Gradle task list
- `f`: filter task list

### Logs

- `Up` / `Down`: scroll output
- `c`: clear logs
- `x`: cancel active process

## Safety model

- Selecting a task never runs it immediately.
- Running a task always goes through a confirmation step.
- `--read-only` disables both file edits and task execution.
- Only one foreground task process is allowed at a time in v0.1.

## Config and session files

- Config: `~/.config/zendroid/config.json`
- Session: `~/.local/share/zendroid/session.json`

The app creates defaults automatically if they do not exist.

## Limits

This project does not try to replace:

- IDE-grade refactors
- LSP completions
- diagnostics and code actions
- Compose previews
- profilers
- APK Analyzer
- device mirroring

Those are still better in Android Studio when you actually need them.
