# Workflow

This repo is meant for a terminal-first Android workflow with a smaller memory
footprint than keeping Android Studio open all day.

## Typical loop

1. Start an emulator.
2. Build or install a debug build.
3. Run the app.
4. Watch logcat.
5. Capture screenshots or inspect layout when needed.

Example:

```bash
android-dev emulator start medium_phone
android-dev build
android-dev install
android-dev logcat
```

## Recommended split

Use `android-dev` for:

- build and install loops
- SDK management
- emulator management
- screenshots
- layout tree inspection
- test execution
- CI-friendly workflows

Use Android Studio only when you need:

- Compose Preview
- visual inspectors
- profiler tooling
- refactors and IDE navigation
- database inspector
- APK analyzer

## Pairing with an editor

This works well with:

- VS Code
- Zed
- Neovim
- Helix

That gives you a setup where the editor handles code and the terminal handles
Android-specific tasks.
