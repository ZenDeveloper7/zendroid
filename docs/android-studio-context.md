# Android Studio Context For ZenDroid

This document is the working memory map for building ZenDroid toward a lighter
Android Studio alternative. It summarizes how Android Studio is structured, what
major subsystems do, and what each subsystem means for ZenDroid.

It is intentionally practical: it does not try to reproduce Android Studio
internals class-for-class. Instead, it identifies the behaviors users rely on
and the lighter architecture ZenDroid should build around.

## Source baseline

Android Studio is the official IDE for Android development. It is based on the
IntelliJ Platform and adds Android-specific tooling around Gradle, emulator and
device workflows, Logcat, lint, profiling, inspection, preview, and build/run
flows.

Primary source anchors:

- Android Studio overview: https://developer.android.com/studio/intro
- Android build configuration: https://developer.android.com/build
- Android build variants: https://developer.android.com/build/build-variants
- Gradle build overview: https://developer.android.com/studio/build/gradle-build-overview
- Android Gradle Plugin overview: https://developer.android.com/build/releases/about-agp
- Layout Inspector: https://developer.android.com/studio/debug/layout-inspector
- Logcat window: https://developer.android.com/studio/debug/am-logcat
- Logcat CLI: https://developer.android.com/tools/logcat
- Device Explorer: https://developer.android.com/studio/debug/device-file-explorer
- IntelliJ Platform overview: https://plugins.jetbrains.com/docs/intellij/intellij-platform.html
- IntelliJ project model: https://plugins.jetbrains.com/docs/intellij/project-model.html
- IntelliJ VFS: https://plugins.jetbrains.com/docs/intellij/virtual-file-system.html
- IntelliJ PSI: https://plugins.jetbrains.com/docs/intellij/psi.html
- IntelliJ indexing: https://plugins.jetbrains.com/docs/intellij/indexing-and-psi-stubs.html
- IntelliJ tool windows: https://plugins.jetbrains.com/docs/intellij/tool-windows.html
- IntelliJ actions: https://plugins.jetbrains.com/docs/intellij/plugin-actions.html
- IntelliJ run configurations: https://plugins.jetbrains.com/docs/intellij/run-configurations.html

When this document says "inference", it means the behavior is inferred from
public Android Studio/IntelliJ architecture and user-visible behavior rather
than from Android Studio private source code.

## Mental model

Android Studio is best understood as two stacked products:

- IntelliJ Platform foundation: editor, projects, files, VFS, PSI, indexing,
  actions, tool windows, run/debug framework, settings, and plugin system.
- Android-specific layer: Gradle/AGP sync, Android project views, variants,
  emulator and device flows, APK/AAB packaging, Logcat, Layout Inspector,
  profiler, lint, resource tooling, Compose tooling, and previews.

ZenDroid should mirror that separation:

- Core IDE layer: panes, editor, file tree, sessions, actions, keymaps, search,
  task/process management.
- Android layer: Gradle model, variants/flavors, devices, logcat, run/install,
  preview, diagnostics, project health, and Android resources.

## Workspace and project model

Android Studio project behavior:

- A project is the top-level workspace that contains modules, libraries, SDK
  configuration, source roots, generated files, run configurations, and IDE
  settings.
- IntelliJ stores project/workspace settings under `.idea` and module metadata
  in `.iml` files for some project formats.
- Android Studio presents an "Android" project view that is not the raw disk
  tree. It groups files by modules and surfaces high-value Android locations
  such as manifests, Java/Kotlin sources, resources, and Gradle scripts.
- The actual on-disk project structure remains the source of truth for Gradle.

ZenDroid implication:

- Keep raw filesystem view as the first stable model.
- Add an Android-aware project view later that groups:
  - app/library modules
  - manifests
  - source sets
  - resources
  - Gradle scripts
  - generated/build outputs when explicitly enabled
- Maintain separate concepts for:
  - project root
  - modules
  - source sets
  - selected variant
  - open editor buffers
  - active device/process

Phase 2 target:

- Detect modules from Gradle/settings files.
- Show module grouping in the file tree or a secondary "Android view".
- Keep `.idea`, `.gradle`, and `build` ignored by default, but make them
  revealable.

## Files, VFS, documents, and editor buffers

Android Studio behavior:

- IntelliJ uses a Virtual File System (VFS) abstraction over files.
- The VFS caches file metadata/content that has been requested and tracks
  changes asynchronously.
- Editor documents are not the same as files. A file can be represented by a
  document, a PSI tree, indexes, and editor UI state.
- Edits are command/write-action driven in IntelliJ so undo/redo, save state,
  and code model updates remain coherent.

ZenDroid implication:

- ZenDroid should keep an internal file index/cache instead of repeatedly
  walking the whole tree.
- Editor buffers should remain separate from on-disk files:
  - path
  - text lines
  - dirty state
  - cursor/selection
  - scroll position
  - syntax mode
  - save status
- File watching should be added before heavy project features.

Phase 2 target:

- Add file watcher or explicit refresh strategy.
- Add unsaved-change prompts.
- Add undo/redo command history.
- Persist tab order, cursor position, and scroll position.

## PSI, syntax model, indexing, and code intelligence

Android Studio behavior:

- IntelliJ PSI parses files into syntax/semantic trees.
- PSI powers navigation, find usages, completion, inspections, quick fixes,
  refactors, code generation, and many gutter actions.
- Indexing provides fast lookup over files and symbols.
- Stub indexes store compact declaration-level data, while file-based indexes
  work over file contents.

ZenDroid implication:

- Do not try to clone IntelliJ PSI early.
- Use a staged approach:
  - syntax highlighting and text search first
  - lightweight symbol extraction next
  - LSP integration for semantic features
  - optional background indexing with memory limits
- Treat indexing as opt-in, cancelable, and bounded.

Phase 2 target:

- Add global text search.
- Add quick-open by file name.
- Add current-file symbol outline for simple formats where feasible.
- Keep heavy code intelligence for a later phase.

## Actions, commands, and keymaps

Android Studio behavior:

- IntelliJ uses an action system. Actions can be invoked from menus, toolbars,
  keyboard shortcuts, context menus, and search.
- Actions update their enabled/visible state based on context.
- Many IDE behaviors are command-based so undo/redo and event routing remain
  consistent.

ZenDroid implication:

- ZenDroid should avoid scattering behavior directly across UI widgets.
- Build an internal action registry over time:
  - id
  - label
  - shortcut
  - enabled predicate
  - handler
  - command/log metadata
- This will enable a command palette and help overlay from the same source.

Phase 2 target:

- Start centralizing key/action definitions.
- Add command palette later, but design actions now so it is easy.

## Tool windows and panes

Android Studio behavior:

- Tool windows are IDE panes for project, run, debug, logcat, device explorer,
  build, Gradle, problems, version control, profiler, layout inspector, and
  similar tools.
- Tool windows are docked around the editor and often contain multiple tabs.
- The editor is the primary center workspace.

ZenDroid implication:

- Current ZenDroid panes map well to a terminal version:
  - Files -> Project tool window
  - Editor -> central editor
  - Tasks -> Gradle/tool actions
  - Logs -> Run/Build/Logcat output
- Future panes should be modeled as tool windows, not hard-coded layout hacks.

Phase 2 target:

- Add pane registry concept.
- Add Problems pane.
- Add Devices pane.
- Let Logs switch sources: task output, logcat, app process, diagnostics.

## Gradle, AGP, sync, tasks, and variants

Android Studio behavior:

- Android Studio uses Gradle as the build foundation.
- Android Gradle Plugin adds Android-specific tasks and model information.
- Build variants are created from build types and product flavors.
- Source sets are merged by priority, with variant-specific source sets taking
  precedence over build-type/flavor/main source sets.
- Android Studio sync reads Gradle configuration and builds the IDE project
  model used for modules, dependencies, variants, tasks, and source roots.
- Android Studio can optionally build the Gradle task list during sync.

ZenDroid implication:

- Gradle is the source of truth for modules, tasks, variants, outputs, and
  dependencies.
- ZenDroid needs its own "sync" concept:
  - run lightweight Gradle/project discovery
  - parse modules
  - discover tasks
  - discover variants/flavors
  - identify APK/AAB outputs
  - cache the model
- Sync must be explicit or low-frequency to keep ZenDroid light.

Phase 2 target:

- Add `Sync Project` action.
- Add build variant/flavor detection.
- Add selected variant state.
- Group tasks by module and category.
- Add "run selected variant" and "install selected variant" actions with
  command preview and confirmation.

## Run configurations and process execution

Android Studio behavior:

- IntelliJ run configurations persist run/debug settings across sessions.
- Run configurations contain program arguments, environment, working directory,
  target device/context, and before-run tasks.
- Run/debug actions are context-aware and can be invoked from toolbar, editor,
  project view, or gutter.
- Output appears in Run/Debug tool windows.

ZenDroid implication:

- Gradle task execution is only the first version of run configurations.
- ZenDroid should introduce a lightweight run profile model:
  - name
  - command kind
  - module
  - variant
  - device target
  - before-run build/install step
  - environment
  - last status
- Execution must remain explicit and confirmable.

Phase 2 target:

- Add saved run profiles for common flows:
  - build debug
  - install debug
  - run app on selected device
  - run tests
- Show exact command before execution.
- Track run history and allow rerun.

## Devices, emulator, and running devices

Android Studio behavior:

- Android Studio integrates device selection, emulator management, running
  device windows, app deployment, device mirroring, and Layout Inspector.
- Device Explorer can browse, copy, upload, delete, and open files from a
  selected device, with limitations based on root/debuggability.

ZenDroid implication:

- Device workflows can be implemented through `adb`, emulator CLI, and Android
  CLI wrappers.
- Keep a device registry:
  - serial
  - model/name
  - API level
  - state
  - emulator/physical flag
  - selected app/process
- Device Explorer should start read-only or confirmation-first.

Phase 2 target:

- Add Devices pane.
- List connected devices/emulators.
- Select active device.
- Add basic actions: install, run, stop app, open logcat.
- Defer full file explorer until device selection is stable.

## Logcat and logs

Android Studio behavior:

- Logcat shows real-time logs from connected devices.
- Logcat supports filtering, app/process selection, scrolling behavior, and
  stack traces linked back to source lines.
- The command-line logcat tool reads structured circular buffers maintained by
  Android's logging system.

ZenDroid implication:

- Logs pane should support multiple sources:
  - Gradle/task output
  - adb logcat
  - app process logs
  - diagnostics/problems
- Logcat needs filtering by device, package, priority, tag, and text.
- Stack trace linking should become a navigation feature.

Phase 2 target:

- Add log source selector.
- Add basic logcat stream action.
- Add filter text and priority filter.
- Parse stack traces into clickable/navigable file references later.

## Problems, lint, inspections, and diagnostics

Android Studio behavior:

- Android Studio runs lint and IntelliJ inspections to catch correctness,
  security, performance, usability, accessibility, compatibility, and code
  quality issues.
- Diagnostics are surfaced in editor, Problems/Build views, and quick-fix
  flows.

ZenDroid implication:

- Start with external tool output parsing.
- Later integrate LSP diagnostics and lint XML/text reports.
- Keep diagnostics as a separate Problems model:
  - severity
  - source
  - file
  - line/column
  - message
  - suggested action if available

Phase 2 target:

- Add Problems pane.
- Parse Gradle build errors into diagnostics where feasible.
- Add lint task shortcut and report location hints.

## Layout Inspector and runtime UI inspection

Android Studio behavior:

- Layout Inspector connects to a running debuggable app process.
- It shows view hierarchy, attributes, magnified/3D views, reference overlays,
  and Compose recomposition information.
- View attribute inspection uses a device global setting and can restart the
  foreground activity.

ZenDroid implication:

- Runtime inspection is different from render preview.
- Layout inspection depends on a running app and device/process connection.
- Early ZenDroid should expose lighter substitutes first:
  - UI hierarchy dumps
  - layout tree text view
  - screenshot capture
  - selected node properties where available

Phase 2 target:

- Add "Inspect Running UI" action backed by available CLI/ADB layout tools.
- Show tree in a pane.
- Make any device-setting side effects explicit before enabling them.

## Rendering preview

Android Studio behavior:

- Android Studio provides design-time rendering for XML and Compose previews.
- Compose previews are driven by `@Preview` declarations and integrate with
  Live Edit/tooling.
- Preview is one of the heavier feature areas because it needs resources,
  themes, classpath, build artifacts, and rendering infrastructure.

ZenDroid implication:

- Preview should be optional, on-demand, and worker-based.
- Do not block the editor on preview rendering.
- Use cache keys based on:
  - file
  - selected variant
  - device preset
  - theme mode
  - density/orientation
  - relevant build/resource inputs

Phase 2 target:

- Do not implement full preview yet unless scoped separately.
- Prepare the architecture with a Preview pane placeholder and render-worker
  boundary.
- Document supported preview targets for later:
  - XML layout
  - Compose `@Preview`
  - screenshot/export

## Debugging

Android Studio behavior:

- IntelliJ platform provides run/debug infrastructure, breakpoints, call stacks,
  watches, and expression evaluation.
- Android Studio layers Android app process/device awareness on top.

ZenDroid implication:

- Debugging should be a later dedicated subsystem.
- Early steps can still prepare the model:
  - selected app process
  - run profile
  - build/install/run pipeline
  - logs and stack traces

Phase 2 target:

- Do not build full debugger yet.
- Keep run profiles structured so debug profiles can be added later.

## Profilers and app inspection

Android Studio behavior:

- Android Studio has CPU, memory, network, energy, database, and app inspection
  tooling.
- These are expensive and deeply integrated with devices/processes.

ZenDroid implication:

- Treat profiler parity as non-core for now.
- Add lighter substitutes first:
  - process list
  - memory snapshots via shell tooling where safe
  - database/file explorer through ADB
  - log-based performance markers

Phase 2 target:

- Only add simple device/process visibility.
- Keep profiler work out of the critical path.

## Settings and persistence

Android Studio behavior:

- Settings are split between IDE-level settings, project settings, workspace
  state, run configurations, keymaps, and tool window layout.

ZenDroid implication:

- Keep config layers distinct:
  - global config: theme, keymap, defaults
  - project config: selected module/variant/device preferences
  - session state: open tabs, pane sizes, selected pane, logs scroll
  - run profiles: named build/run/debug/test flows

Phase 2 target:

- Persist pane sizes and collapsed state.
- Persist selected variant/device.
- Keep generated session files outside the repo.

## Phase 2 implementation priorities

Recommended order:

1. Project sync model
2. Module/variant/task grouping
3. Device pane and selected device state
4. Run/install profile model
5. Logcat source in Logs pane
6. Problems pane with Gradle error parsing
7. Pane registry cleanup
8. Preview architecture placeholder, not full renderer

Definition of done for Phase 2:

- ZenDroid can discover modules and variants.
- ZenDroid can list devices and select an active device.
- ZenDroid can build/install/run a selected variant through explicit profiles.
- ZenDroid can stream logcat into the Logs pane.
- ZenDroid can surface basic build problems in a Problems pane.
- The architecture is ready for preview/render workers without coupling preview
  to the editor loop.

## Design rules for ZenDroid

- Prefer explicit actions over hidden background work.
- Prefer on-demand sync over always-on indexing.
- Show exact commands before process execution.
- Treat expensive integrations as optional workers.
- Keep editor responsive even when Gradle/device work is running.
- Model IDE features as panes, actions, and background services rather than
  one-off UI hacks.
- Keep Android Studio parity as a direction, not a reason to copy heavy
  implementation details blindly.
