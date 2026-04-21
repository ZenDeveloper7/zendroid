# ZenDroid

`zendroid` is a terminal UI for Android development that borrows the basic
shape of Android Studio while staying far lighter on RAM: file explorer on the
left, code editor in the middle, Gradle tasks on the right, and logs at the
bottom.

It is built in Rust with `ratatui` and `crossterm`, with Gradle used as the
project/task backend.

See [docs/roadmap.md](docs/roadmap.md) for the longer-term plan toward a
lighter Android Studio alternative.
See [docs/workflow.md](docs/workflow.md) for the intended day-to-day workflow.
See [docs/android-studio-context.md](docs/android-studio-context.md) for the
Android Studio architecture notes that guide future ZenDroid phases.

## Why this exists

Android Studio can become expensive in RAM-heavy environments, especially when
you mostly want:

- a real project tree
- a built-in editor
- a visible Gradle task list
- a bottom log console
- explicit task execution without hidden automation

For those cases, a terminal IDE can cover a lot of the daily loop.

## Current status

ZenDroid is currently a strong `v0.1` foundation:

- a single-project Android workspace
- a file tree, editor, tasks pane, and logs pane
- session restore for open tabs and focus
- safe Gradle task execution with confirmation
- lightweight syntax highlighting
- keyboard-first navigation and pane management

It is already useful for low-memory Android development sessions, but it is
not yet trying to fully replace Android Studio's previews, debugging, or code
intelligence stack.

## What it does today

`zendroid` combines:

- Android project root discovery
- a keyboard-first file explorer
- a built-in multi-tab text editor with lightweight syntax highlighting
- Gradle task discovery and filtering
- module and variant discovery from Gradle sync
- Android device discovery through `adb`
- Problems capture from build/process output
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

## Typical workflow

1. Open an Android project root in ZenDroid.
2. Browse files in the left pane and edit code in the center pane.
3. Jump directly to Tasks or Logs with `Alt-3` and `Alt-4`.
4. Filter or inspect Gradle tasks before running anything.
5. Confirm the exact command before execution.
6. Review build and test output in the bottom pane.

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
- `t` / `d` / `p`: switch the right pane between Tasks, Devices, and Problems
- `g` / `s`: sync Gradle tasks, modules, and variants
- `f`: filter task list
- `v`: cycle discovered variants
- `b` / `i`: build or install the selected variant

### Devices

- `Up` / `Down`: move device selection
- `r`: refresh `adb devices -l`
- `l` / `Enter`: start a confirmed Logcat stream for the selected device

### Problems

- `Up` / `Down`: move problem selection
- `c`: clear captured problems

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

## Current limits

This project does not try to replace:

- IDE-grade refactors
- LSP completions
- diagnostics and code actions
- Compose previews
- profilers
- APK Analyzer
- device mirroring

Those workflows are still better in Android Studio today. They are part of the
longer-term roadmap, not the current `v0.1` contract.
