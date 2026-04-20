# ZenDroid Roadmap

ZenDroid aims to cover most day-to-day Android Studio workflows while staying
lighter, faster to launch, and easier on RAM. The goal is not to clone every
Android Studio subsystem exactly, but to deliver the highest-value Android IDE
experience in a modular terminal UI.

## Product direction

ZenDroid should eventually cover:

- project and file navigation
- solid code editing
- Gradle tasks, variants, and flavors
- device and emulator workflows
- logs and app run flows
- code intelligence and diagnostics
- rendering preview
- debugging support
- Android-specific project tooling

ZenDroid should stay lighter than Android Studio by:

- keeping heavy features modular and optional
- avoiding always-on indexing where possible
- using subprocess workers for expensive tasks
- integrating existing Android and Gradle tooling instead of reimplementing everything

## Phase 1: Core IDE foundation

Goal: make ZenDroid a credible daily driver for the common Android loop.

- stabilize the multi-pane workspace
- strengthen editor basics: undo/redo, selections, copy/paste, save-all
- add unsaved-change prompts for close and quit
- improve file tree, quick open, and global search
- harden Gradle task discovery and execution
- improve device/log panels
- persist pane layout and richer session state
- package real `v0.1.x` releases

Success criteria:

- a developer can open a project, edit code, search files, run builds, and inspect output without leaving ZenDroid for normal daily work

## Phase 2: Android workflow parity

Goal: cover the most common Android Studio project-management flows.

- build variant and flavor selection
- install/run actions for emulator or physical device
- dedicated panels for Gradle, devices, logcat, and problems
- resource-aware handling for manifests, XML, Gradle, and Kotlin files
- Android project health checks: SDK, wrapper, config, build failures
- better build error parsing and surfacing

Success criteria:

- common Android run/build/test flows are available directly from the UI and easier to understand than raw terminal output

## Phase 3: Code intelligence

Goal: make editing feel IDE-like instead of terminal-like.

- LSP integration where it stays light enough
- go to definition and references
- rename symbol
- hover docs
- diagnostics and quick fixes
- symbol outline and breadcrumbs

Success criteria:

- Kotlin and Android source navigation feels fast enough that Android Studio is no longer required for routine code navigation

## Phase 4: Rendering preview

Goal: provide visual feedback without taking on full Android Studio rendering weight.

### Preview 4.1: Static preview

- XML layout preview rendering
- resource and theme-aware rendering
- basic device presets
- light/dark mode toggle
- density and orientation switching
- preview pane or preview tab

### Preview 4.2: Compose preview

- detect `@Preview` composables
- list available previews in the current file
- render selected preview targets
- support manual refresh and refresh-on-save
- cache preview results aggressively

### Preview 4.3: Interactive preview

- component hierarchy or inspection panel
- dimensions, padding, colors, and text-style inspection
- locale, font-scale, and screen-size toggles
- screenshot export

Success criteria:

- a developer can quickly validate XML and Compose UI without opening Android Studio for every visual check

## Phase 5: Debugging and inspection

Goal: cover the most important interactive app-debugging workflows.

- debug session integration
- breakpoints
- call stack and variable inspection
- thread/process overview
- safer, lighter substitutes for heavy inspectors

Success criteria:

- ZenDroid supports routine bug investigation without forcing a switch back to Android Studio

## Phase 6: Advanced IDE features

Goal: expand power-user workflows while preserving modularity.

- split editors
- command palette
- workspace search everywhere
- plugin or extension system
- background indexing with strict memory limits
- optional advanced integrations that remain easy to disable

Success criteria:

- ZenDroid grows into a serious Android IDE without losing its lightweight identity

## Milestone framing

### v0.1.x

- harden the current TUI foundation
- improve editor safety and ergonomics
- stabilize Gradle/task execution flows
- improve docs and packaging

### v0.2.x

- Android device/run workflows
- better search and navigation
- stronger editor behavior
- first problem panel and diagnostics surfacing

### v0.3.x

- code intelligence and symbol navigation
- improved Android project awareness
- first build-variant and flavor UX

### v0.5.x

- first preview system
- richer device and inspection workflows
- stronger debugging support

### v1.0

- most common Android Studio daily workflows covered
- stable editor, task runner, navigation, preview, and debugging foundations
- clear performance and memory advantage over Android Studio for day-to-day usage

## Non-goals

ZenDroid should avoid blindly copying:

- Android Studio's full profiler stack
- every inspector exactly as-is
- massive always-on indexing
- heavyweight always-running background services

When a feature is expensive, ZenDroid should prefer:

- on-demand workers
- cached results
- optional activation
- external tooling integration behind a clean UI

## Practical north star

ZenDroid succeeds when a developer can do most normal Android work in it:

- open projects
- edit code
- search and navigate
- build and run
- inspect logs
- preview UI
- debug common issues

without needing Android Studio for everyday development.
