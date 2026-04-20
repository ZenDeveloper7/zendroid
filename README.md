# android-dev-cli

`android-dev` is a lightweight command-line workflow for Android development.
It is not a replacement for Android Studio's editor, profiler, and inspectors,
but it does replace a lot of the heavy day-to-day actions that make Studio feel
mandatory: builds, installs, emulator control, screenshots, layout dumps, SDK
management, and log streaming.

This project is built around the local `android` CLI plus Gradle.

## Why this exists

Android Studio can become expensive in RAM-heavy environments, especially when
you mainly need to:

- build debug APKs
- install and run on an emulator
- execute unit or instrumented tests
- manage SDK packages
- inspect screens and UI layout trees
- stream logs

For those cases, a small terminal-first workflow is often enough.

## What it does

`android-dev` combines:

- `./gradlew` for builds, installs, and tests
- `android emulator` for AVD control
- `android run` for APK deployment
- `android screen capture` for screenshots
- `android layout` for layout-tree inspection
- `adb` for logcat
- `android sdk` for SDK package management

## Requirements

- Linux or macOS shell environment
- Bash
- The `android` CLI installed and working
- An Android SDK configured
- A Gradle Android project for project-level commands

## Install

Clone the repo and copy the script into a directory already on your `PATH`:

```bash
mkdir -p ~/.local/bin
cp ./bin/android-dev ~/.local/bin/android-dev
chmod +x ~/.local/bin/android-dev
```

If `~/.local/bin` is not already on your `PATH`, add it in your shell config.

## Quick start

From inside any Android Gradle project:

```bash
android-dev doctor
android-dev build
android-dev install
android-dev test unit
android-dev test android
android-dev apk
android-dev run
```

Outside a project:

```bash
android-dev emulator list
android-dev emulator start medium_phone
android-dev sdk list
```

## Commands

### Project commands

```bash
android-dev doctor
android-dev build [gradle-task]
android-dev clean
android-dev install [gradle-task]
android-dev apk [variant]
android-dev run [variant] [device-serial]
android-dev test unit [gradle-task]
android-dev test android [gradle-task]
android-dev logcat [device-serial]
```

### Device and SDK commands

```bash
android-dev emulator list
android-dev emulator start <avd-name>
android-dev emulator stop <avd-name>
android-dev sdk list
android-dev sdk install <package>...
android-dev sdk update [package]
android-dev sdk remove <package>
android-dev screen [output.png] [device-serial]
android-dev layout [device-serial]
```

## Examples

Build the debug APK:

```bash
android-dev build
```

Install the debug build using Gradle:

```bash
android-dev install
```

Build a release APK and deploy it to a specific device:

```bash
android-dev run release emulator-5554
```

Capture a screenshot from the current device:

```bash
android-dev screen
```

Print the current layout tree:

```bash
android-dev layout
```

## Notes

- `android-dev run` builds the requested variant first, then searches
  `build/outputs/apk/<variant>` for the APK.
- `android-dev doctor` tries to show a short list of Gradle tasks when it is
  run inside a project. If Gradle cannot start in the current environment, it
  prints a note and continues.
- `adb` is resolved from your shell first, then from the Android SDK's
  `platform-tools` directory.

## Limits

This project does not try to replace:

- code editing
- Layout Inspector
- Compose previews
- profilers
- lint UI
- APK Analyzer
- device mirroring

Those are still better in Android Studio when you actually need them.
