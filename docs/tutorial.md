# ZenDroid Tutorial And Shortcut Reference

This guide explains how to start ZenDroid, move around the UI, and use the
current command set.

## Build and run

From the ZenDroid repo:

```bash
cargo build --release
./target/release/zendroid --project ~/AndroidStudioProjects/DocSafe
```

Install the binary locally:

```bash
mkdir -p ~/.local/bin
cp ./target/release/zendroid ~/.local/bin/zendroid
chmod +x ~/.local/bin/zendroid
```

Then run:

```bash
zendroid --project ~/AndroidStudioProjects/DocSafe
```

## CLI commands

```bash
zendroid
zendroid <project-path>
zendroid --project <project-path>
zendroid --read-only
zendroid --theme amber
zendroid --config /path/to/config.json
zendroid --help
```

Useful examples:

```bash
zendroid
zendroid ~/AndroidStudioProjects/DocSafe
zendroid --project ~/AndroidStudioProjects/DocSafe --read-only
```

## UI layout

ZenDroid opens into four main areas:

- Files: left pane for project navigation
- Editor: center pane for source editing
- Tools: right pane for Tasks, Devices, and Problems
- Logs: bottom pane for process output

The right pane has three modes:

- Tasks: Gradle tasks, variants, build/install shortcuts
- Devices: connected Android devices and Logcat launch
- Problems: parsed errors, warnings, and info messages from process output

## Global shortcuts

| Shortcut | Action |
| --- | --- |
| `Alt-1` | Focus Files pane |
| `Alt-2` | Focus Editor pane |
| `Alt-3` | Focus Tools pane |
| `Alt-4` | Focus Logs pane |
| `Tab` | Cycle to next pane |
| `Shift-Tab` | Cycle to previous pane |
| `Alt-h` | Shrink focused side pane |
| `Alt-l` | Grow focused side pane |
| `Alt-j` | Shrink Logs pane |
| `Alt-k` | Grow Logs pane |
| `Alt--` | Collapse focused non-editor pane |
| `Alt-=` | Reset focused pane size |
| `Ctrl-S` | Save current file |
| `Ctrl-W` | Close current editor tab |
| `F1` or `?` | Show in-app help |
| `q` | Quit |

## Files pane shortcuts

| Shortcut | Action |
| --- | --- |
| `Up` / `k` | Move selection up |
| `Down` / `j` | Move selection down |
| `Enter` / `Right` | Expand directory or open file |
| `Left` | Collapse directory or move to parent |
| `r` | Refresh file tree |

## Editor shortcuts

| Shortcut | Action |
| --- | --- |
| Arrow keys | Move cursor |
| Type text | Insert characters |
| `Enter` | Insert newline |
| `Backspace` | Delete previous character |
| `/` | Search current file |
| `[` | Previous editor tab |
| `]` | Next editor tab |
| `Ctrl-S` | Save current file |
| `Ctrl-W` | Close current tab |

Notes:

- The editor supports lightweight syntax highlighting.
- UTF-8 text is supported for basic movement and editing.
- Advanced IDE features like LSP, autocomplete, and refactors are roadmap work.

## Tools pane mode shortcuts

These work when the Tools pane is focused.

| Shortcut | Action |
| --- | --- |
| `t` | Show Tasks mode |
| `d` | Show Devices mode |
| `p` | Show Problems mode |

## Tasks mode shortcuts

| Shortcut | Action |
| --- | --- |
| `Up` / `k` | Move task selection up |
| `Down` / `j` | Move task selection down |
| `g` / `s` | Sync Gradle tasks, modules, and variants |
| `f` | Filter task list |
| `v` | Cycle discovered build variants |
| `b` | Build selected variant |
| `i` | Install selected variant |
| `Enter` | Prepare selected task for confirmed execution |

Task execution flow:

1. Select a task or use `b` / `i`.
2. ZenDroid shows the exact command.
3. Press `y` to run or `n` / `Esc` to cancel.

ZenDroid does not run tasks automatically just because you selected them.

## Devices mode shortcuts

| Shortcut | Action |
| --- | --- |
| `Up` / `k` | Move device selection up |
| `Down` / `j` | Move device selection down |
| `r` | Refresh devices with `adb devices -l` |
| `l` / `Enter` | Prepare Logcat stream for confirmed execution |

Device discovery is explicit. ZenDroid does not start `adb` device scanning
until you ask for it with `r`.

## Problems mode shortcuts

| Shortcut | Action |
| --- | --- |
| `Up` / `k` | Move problem selection up |
| `Down` / `j` | Move problem selection down |
| `c` | Clear captured problems |

Problems are captured from task/process output. Current parsing is lightweight
and focuses on error, warning, and info-style lines.

## Logs pane shortcuts

| Shortcut | Action |
| --- | --- |
| `Up` / `k` | Scroll logs up |
| `Down` / `j` | Scroll logs down |
| `c` | Clear logs |
| `x` | Cancel active process |

The Logs pane shows Gradle output, command output, and confirmed Logcat streams.

## Common workflows

### Open a project and edit a file

1. Run `zendroid --project <project-path>`.
2. Use `Alt-1` to focus Files.
3. Use `Enter` / `Right` to open a file.
4. Use `Alt-2` to focus Editor.
5. Edit and press `Ctrl-S` to save.

### Sync Gradle and build a variant

1. Press `Alt-3` to focus Tools.
2. Press `t` for Tasks mode.
3. Press `s` to sync Gradle tasks, modules, and variants.
4. Press `v` to choose a variant.
5. Press `b` to prepare build.
6. Press `y` to confirm.

### Install a variant

1. Press `Alt-3`.
2. Press `t`.
3. Press `s` to sync if needed.
4. Press `v` until the desired variant is selected.
5. Press `i` to prepare install.
6. Press `y` to confirm.

### Start Logcat

1. Press `Alt-3`.
2. Press `d` for Devices mode.
3. Press `r` to refresh devices.
4. Select a device.
5. Press `l` or `Enter`.
6. Press `y` to confirm Logcat.

### Review problems

1. Run a build, install, test, or Logcat command.
2. Press `Alt-3`.
3. Press `p` for Problems mode.
4. Use `Up` / `Down` to inspect captured messages.
5. Press `c` to clear the list.

## Safety model

- Selecting a task does not execute it.
- Build, install, and Logcat actions require confirmation.
- `--read-only` disables file edits and process execution.
- Only one foreground process can run at a time.
- Device discovery is explicit, not automatic.
