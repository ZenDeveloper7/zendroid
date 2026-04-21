use crate::app::{App, FocusPane, InputMode, RightPaneMode};
use crate::editor::highlight_line;
use crate::gradle::TaskCategory;
use crate::problems::ProblemSeverity;
use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs, Wrap};
use ratatui::{Frame, layout::Position};

pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(10),
            if app.layout.logs_collapsed {
                Constraint::Length(0)
            } else {
                Constraint::Length(app.layout.logs_height)
            },
            Constraint::Length(1),
        ])
        .split(area);

    draw_top_bar(frame, app, root[0]);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            if app.layout.explorer_collapsed {
                Constraint::Length(0)
            } else {
                Constraint::Percentage(app.layout.explorer_width)
            },
            Constraint::Min(24),
            if app.layout.tasks_collapsed {
                Constraint::Length(0)
            } else {
                Constraint::Percentage(app.layout.tasks_width)
            },
        ])
        .split(root[1]);

    if !app.layout.explorer_collapsed {
        draw_explorer(frame, app, body[0]);
    }
    let editor_cursor = draw_editor(frame, app, body[1]);
    if !app.layout.tasks_collapsed {
        draw_right_pane(frame, app, body[2]);
    }
    if !app.layout.logs_collapsed {
        draw_logs(frame, app, root[2]);
    }
    draw_status(frame, app, root[3]);

    if let Some((x, y)) = editor_cursor {
        frame.set_cursor_position(Position::new(x, y));
    }

    match &app.input_mode {
        InputMode::Help => draw_help(frame, app, area),
        InputMode::Search { query } => draw_prompt(frame, area, "Search", query),
        InputMode::TaskFilter { query } => draw_prompt(frame, area, "Task Filter", query),
        InputMode::ConfirmRun { command } => draw_confirm(frame, area, command),
        InputMode::Normal => {}
    }
}

fn draw_top_bar(frame: &mut Frame, app: &App, area: Rect) {
    let project_name = app
        .project_root
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| app.project_root.display().to_string());
    let file_path = app
        .editor
        .active_path()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "No file open".to_string());
    let dirty = if app.editor.active_dirty() { " *" } else { "" };
    let pane = format!("{:?}", app.focus);
    let mode = if app.read_only { "READ ONLY" } else { "EDIT" };
    let layout_hint = format!(
        "E:{}% T:{}% L:{}{}{}",
        app.layout.explorer_width,
        app.layout.tasks_width,
        app.layout.logs_height,
        if app.layout.explorer_collapsed {
            " e-"
        } else {
            ""
        },
        if app.layout.tasks_collapsed || app.layout.logs_collapsed {
            if app.layout.tasks_collapsed && app.layout.logs_collapsed {
                " t- l-"
            } else if app.layout.tasks_collapsed {
                " t-"
            } else {
                " l-"
            }
        } else {
            ""
        }
    );

    let text = Line::from(vec![
        Span::styled(
            format!(" {} ", project_name),
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(file_path, Style::default().fg(Color::Cyan)),
        Span::styled(dirty.to_string(), Style::default().fg(Color::LightRed)),
        Span::raw("  "),
        Span::styled(format!("Pane: {pane}"), Style::default().fg(Color::Gray)),
        Span::raw("  "),
        Span::styled(layout_hint, Style::default().fg(Color::DarkGray)),
        Span::raw("  "),
        Span::styled(mode, Style::default().fg(Color::Green)),
    ]);
    frame.render_widget(Paragraph::new(text), area);
}

fn draw_explorer(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .explorer
        .entries
        .iter()
        .map(|entry| {
            let indent = "  ".repeat(entry.depth);
            let icon = if entry.is_dir { "▸" } else { "•" };
            let style = if entry.path == app.project_root {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(vec![Span::styled(
                format!("{indent}{icon} {}", entry.name),
                style,
            )]))
        })
        .collect();

    let block = pane_block("Files", app.focus == FocusPane::Explorer);
    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::White));
    let mut state = ListState::default().with_selected(Some(app.explorer.selected));
    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_editor(frame: &mut Frame, app: &mut App, area: Rect) -> Option<(u16, u16)> {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(3)])
        .split(area);

    let titles: Vec<Line> = if app.editor.buffers.is_empty() {
        vec![Line::from("No File")]
    } else {
        app.editor
            .buffers
            .iter()
            .map(|buffer| {
                let title = buffer
                    .path
                    .file_name()
                    .map(|value| value.to_string_lossy().to_string())
                    .unwrap_or_else(|| "file".to_string());
                let suffix = if buffer.dirty { "*" } else { "" };
                Line::from(format!("{title}{suffix}"))
            })
            .collect()
    };
    let tabs = Tabs::new(titles)
        .block(pane_block("Editor", app.focus == FocusPane::Editor))
        .select(app.editor.active.unwrap_or(0))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    frame.render_widget(tabs, chunks[0]);

    let inner = chunks[1];
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style(app.focus == FocusPane::Editor));
    let editor_area = inner.inner(Margin {
        vertical: 1,
        horizontal: 1,
    });
    frame.render_widget(block, inner);

    let Some(_buffer) = app.editor.current() else {
        frame.render_widget(
            Paragraph::new("Open a file from the explorer to start editing.")
                .wrap(Wrap { trim: false }),
            editor_area,
        );
        return None;
    };

    let viewport_height = editor_area.height.max(1) as usize;
    app.editor.ensure_cursor_visible(viewport_height);
    let buffer = app.editor.current().unwrap();
    let visible = buffer
        .lines
        .iter()
        .enumerate()
        .skip(buffer.scroll_row)
        .take(viewport_height)
        .map(|(index, line)| {
            let mut spans = vec![Span::styled(
                format!("{:>4} ", index + 1),
                Style::default().fg(Color::DarkGray),
            )];
            spans.extend(highlight_line(&buffer.path, line).spans);
            Line::from(spans)
        })
        .collect::<Vec<_>>();
    frame.render_widget(
        Paragraph::new(visible).wrap(Wrap { trim: false }),
        editor_area,
    );

    if app.focus != FocusPane::Editor || !matches!(app.input_mode, InputMode::Normal) {
        return None;
    }

    let cursor_x = editor_area.x + 5 + buffer.cursor_col as u16;
    let cursor_y = editor_area.y + buffer.cursor_row.saturating_sub(buffer.scroll_row) as u16;
    let max_x = editor_area.x + editor_area.width.saturating_sub(1);
    let max_y = editor_area.y + editor_area.height.saturating_sub(1);
    Some((cursor_x.min(max_x), cursor_y.min(max_y)))
}

fn draw_tasks(frame: &mut Frame, app: &App, area: Rect) {
    let title = match &app.tasks.state {
        crate::gradle::TaskDiscoveryState::Discovering => "Tasks (scanning)",
        crate::gradle::TaskDiscoveryState::Failed(error) if !error.is_empty() => "Tasks (failed)",
        crate::gradle::TaskDiscoveryState::Failed(_) => "Tasks",
        _ => "Tasks",
    };

    let items: Vec<ListItem> = app
        .tasks
        .filtered_tasks()
        .into_iter()
        .map(|task| {
            let category = match task.category {
                TaskCategory::Build => "build",
                TaskCategory::Install => "install",
                TaskCategory::Test => "test",
                TaskCategory::Lint => "lint",
                TaskCategory::Clean => "clean",
                TaskCategory::Other => "other",
            };
            let module = task.module.as_deref().unwrap_or("root");
            ListItem::new(format!("{} [{}/{category}]", task.name, module))
        })
        .collect();
    let list = List::new(items)
        .block(pane_block(title, app.focus == FocusPane::Tasks))
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::White));
    let mut state = ListState::default().with_selected(Some(app.tasks.selected));
    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_right_pane(frame: &mut Frame, app: &App, area: Rect) {
    match app.right_pane {
        RightPaneMode::Tasks => draw_tasks(frame, app, area),
        RightPaneMode::Devices => draw_devices(frame, app, area),
        RightPaneMode::Problems => draw_problems(frame, app, area),
    }
}

fn draw_devices(frame: &mut Frame, app: &App, area: Rect) {
    let items = if app.devices.is_empty() {
        vec![ListItem::new("No devices. Press r to refresh.")]
    } else {
        app.devices
            .iter()
            .map(|device| {
                let description = if device.description.is_empty() {
                    "".to_string()
                } else {
                    format!(" {}", device.description)
                };
                ListItem::new(format!(
                    "{} [{}]{}",
                    device.serial, device.state, description
                ))
            })
            .collect()
    };
    let list = List::new(items)
        .block(pane_block("Devices", app.focus == FocusPane::Tasks))
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::White));
    let mut state = ListState::default().with_selected(Some(app.selected_device));
    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_problems(frame: &mut Frame, app: &App, area: Rect) {
    let items = if app.problems.problems.is_empty() {
        vec![ListItem::new("No problems captured yet.")]
    } else {
        app.problems
            .problems
            .iter()
            .map(|problem| {
                let severity = match problem.severity {
                    ProblemSeverity::Error => "E",
                    ProblemSeverity::Warning => "W",
                    ProblemSeverity::Info => "I",
                };
                let location = match (&problem.file, problem.line) {
                    (Some(file), Some(line)) => format!("{file}:{line}: "),
                    (Some(file), None) => format!("{file}: "),
                    _ => String::new(),
                };
                ListItem::new(format!("{severity} {location}{}", problem.message))
            })
            .collect()
    };
    let title = format!("Problems ({})", app.problems.problems.len());
    let list = List::new(items)
        .block(pane_block(&title, app.focus == FocusPane::Tasks))
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::White));
    let mut state = ListState::default().with_selected(Some(app.problems.selected));
    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_logs(frame: &mut Frame, app: &App, area: Rect) {
    let viewport_height = area.height.saturating_sub(2) as usize;
    let len = app.logs.lines.len();
    let start = len.saturating_sub(viewport_height + app.logs.scroll);
    let end = len.saturating_sub(app.logs.scroll);
    let lines = app.logs.lines[start..end]
        .iter()
        .cloned()
        .map(Line::from)
        .collect::<Vec<_>>();
    let title = if app.process.is_running() {
        "Logs (running)"
    } else {
        "Logs"
    };
    frame.render_widget(
        Paragraph::new(lines)
            .block(pane_block(title, app.focus == FocusPane::Logs))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn draw_status(frame: &mut Frame, app: &App, area: Rect) {
    let variant = app
        .selected_variant
        .as_deref()
        .map(|variant| format!(" | variant {variant}"))
        .unwrap_or_default();
    let hint = format!(
        "Alt-1..4 panes | right: t tasks d devices p problems | b build i install v variant | Ctrl-S save | F1 help{variant}"
    );
    let message = if app.status.is_empty() {
        &hint
    } else {
        &app.status
    };
    frame.render_widget(
        Paragraph::new(message.to_string()).style(Style::default().fg(Color::Gray)),
        area,
    );
}

fn draw_prompt(frame: &mut Frame, area: Rect, title: &str, query: &str) {
    let popup = centered_rect(60, 20, area);
    frame.render_widget(Clear, popup);
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    frame.render_widget(Paragraph::new(query.to_string()).block(block), popup);
}

fn draw_confirm(frame: &mut Frame, area: Rect, command: &str) {
    let popup = centered_rect(70, 28, area);
    frame.render_widget(Clear, popup);
    let text = format!("Run this command?\n\n{command}\n\nPress y to run, n or Esc to cancel.");
    frame.render_widget(
        Paragraph::new(text)
            .block(
                Block::default()
                    .title("Confirm Task Run")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .wrap(Wrap { trim: false }),
        popup,
    );
}

fn draw_help(frame: &mut Frame, app: &App, area: Rect) {
    let popup = centered_rect(84, 84, area);
    frame.render_widget(Clear, popup);
    let viewport_height = popup.height.saturating_sub(2) as usize;
    let help_lines = HELP_TEXT.lines().count();
    let max_scroll = help_lines.saturating_sub(viewport_height);
    let scroll = app.help_scroll.min(max_scroll);
    let visible = HELP_TEXT
        .lines()
        .skip(scroll)
        .take(viewport_height)
        .map(Line::from)
        .collect::<Vec<_>>();
    let title = format!(
        "Help / Tutorial ({}/{}) - Up/Down scroll, Esc close",
        scroll.saturating_add(1),
        help_lines.max(1)
    );

    frame.render_widget(
        Paragraph::new(visible)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .wrap(Wrap { trim: false }),
        popup,
    );
}

const HELP_TEXT: &str = "\
Zendroid in-app tutorial

Build and run
  cargo build --release
  ./target/release/zendroid --project ~/AndroidStudioProjects/MyApp
  zendroid --project ~/AndroidStudioProjects/MyApp

CLI commands
  zendroid                         Open current directory
  zendroid <project-path>          Open a project directly
  zendroid --project <path>        Open an explicit project path
  zendroid --read-only             Disable file edits and command execution
  zendroid --theme amber           Use a named theme
  zendroid --config <path>         Use a custom config file
  zendroid --help                  Show CLI help

UI layout
  Files                            Left project explorer
  Editor                           Center code editor
  Tools                            Right pane: Tasks, Devices, Problems
  Logs                             Bottom process output

Global shortcuts
  Alt-1 / Alt-2 / Alt-3 / Alt-4    Focus Files, Editor, Tools, Logs
  Tab / Shift-Tab                  Cycle panes
  Alt-h / Alt-l                    Shrink or grow focused side pane
  Alt-j / Alt-k                    Shrink or grow Logs pane
  Alt--                            Collapse focused non-editor pane
  Alt-=                            Reset focused pane size
  Ctrl-S                           Save current file
  Ctrl-W                           Close current editor tab
  F1 or ?                          Show or close this help
  q                                Quit

Files pane
  Up / k                           Move selection up
  Down / j                         Move selection down
  Enter / Right                    Expand directory or open file
  Left                             Collapse directory or move to parent
  r                                Refresh file tree

Editor
  Arrow keys                       Move cursor
  Type text                        Insert characters
  Enter                            Insert newline
  Backspace                        Delete previous character
  /                                Search current file
  [ / ]                            Previous or next editor tab
  Ctrl-S                           Save current file
  Ctrl-W                           Close current tab

Tools pane modes
  t                                Show Gradle Tasks
  d                                Show Android Devices
  p                                Show Problems

Tasks mode
  Up / k                           Move task selection up
  Down / j                         Move task selection down
  g / s                            Sync Gradle tasks, modules, variants
  f                                Filter task list
  v                                Cycle discovered build variants
  b                                Build selected variant
  i                                Install selected variant
  Enter                            Prepare selected task for confirmation

Task execution flow
  1. Select a task, or use b / i for the current variant.
  2. Zendroid shows the exact command first.
  3. Press y to run, or n / Esc to cancel.

Devices mode
  Up / k                           Move device selection up
  Down / j                         Move device selection down
  r                                Refresh devices with adb devices -l
  l / Enter                        Prepare Logcat stream for confirmation

Problems mode
  Up / k                           Move problem selection up
  Down / j                         Move problem selection down
  c                                Clear captured problems

Logs pane
  Up / k                           Scroll logs up
  Down / j                         Scroll logs down
  c                                Clear logs
  x                                Cancel active process

Common workflow: edit a file
  1. Run zendroid --project <project-path>.
  2. Press Alt-1 and open a file from Files.
  3. Press Alt-2, edit, then Ctrl-S to save.

Common workflow: sync and build
  1. Press Alt-3, then t for Tasks.
  2. Press s to sync Gradle.
  3. Press v to choose a variant.
  4. Press b, then y to confirm.

Common workflow: install
  1. Press Alt-3, then t.
  2. Press s if tasks are stale.
  3. Press v for the target variant.
  4. Press i, then y to confirm.

Common workflow: Logcat
  1. Press Alt-3, then d for Devices.
  2. Press r to refresh devices.
  3. Select a device.
  4. Press l or Enter, then y to confirm.

Safety model
  Selecting a task never runs it by itself.
  Build, install, custom task runs, and Logcat require confirmation.
  --read-only disables file edits and process execution.
  Only one foreground process can run at a time.
  Device discovery is explicit, not automatic.
";

fn pane_block<'a>(title: &'a str, active: bool) -> Block<'a> {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style(active))
}

fn border_style(active: bool) -> Style {
    if active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}
