# Workflow

This repo is meant for a terminal-first Android workflow with a much smaller
memory footprint than keeping Android Studio open all day.

## Typical loop

1. Open one Android project root.
2. Browse files from the left pane.
3. Edit source in the center pane.
4. Jump directly between panes with `Alt-1..4` instead of cycling with tab.
5. Resize or collapse panes while you work.
6. Filter or inspect Gradle tasks on the right.
7. Confirm task execution only when you actually want it.
8. Watch all output in the bottom log pane.

Example:

```bash
zendroid --project ~/AndroidStudioProjects/DocSafe
```

## Recommended split today

Use `zendroid` for:

- project browsing
- quick source edits
- lightweight syntax-colored editing
- Gradle task browsing
- explicit task execution
- build/test output review
- low-memory coding sessions

Use Android Studio only when you need:

- Compose Preview
- visual inspectors
- profiler tooling
- refactors and IDE navigation
- database inspector
- APK analyzer
- deep debugging and advanced code intelligence

## Current product direction

ZenDroid `v0.1` is intentionally scoped to:

- one project root per app session
- a built-in basic editor
- one foreground task process at a time
- confirm-before-run behavior
- keyboard-first navigation

It is not trying to ship a full Android Studio clone. The goal is a fast,
usable terminal IDE for the common Android loop while the heavier IDE features
move in over later phases.
