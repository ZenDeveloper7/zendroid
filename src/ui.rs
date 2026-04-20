use crate::app::{App, FocusPane, InputMode};
use crate::editor::highlight_line;
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
        draw_tasks(frame, app, body[2]);
    }
    if !app.layout.logs_collapsed {
        draw_logs(frame, app, root[2]);
    }
    draw_status(frame, app, root[3]);

    if let Some((x, y)) = editor_cursor {
        frame.set_cursor_position(Position::new(x, y));
    }

    match &app.input_mode {
        InputMode::Help => draw_help(frame, area),
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
        .map(|task| ListItem::new(format!("{} [{}]", task.name, task.group)))
        .collect();
    let list = List::new(items)
        .block(pane_block(title, app.focus == FocusPane::Tasks))
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::White));
    let mut state = ListState::default().with_selected(Some(app.tasks.selected));
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
    let hint = "Alt-1..4 focus panes | Alt-h/l resize side pane | Alt-j/k resize logs | Alt-- collapse | Ctrl-S save | F1 help";
    let message = if app.status.is_empty() {
        hint
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

fn draw_help(frame: &mut Frame, area: Rect) {
    let popup = centered_rect(76, 70, area);
    frame.render_widget(Clear, popup);
    let help = "\
Global
  Alt-1..4         Focus Files, Editor, Tasks, Logs
  Tab / Shift-Tab  Cycle panes
  Alt-h / Alt-l    Resize focused side pane
  Alt-j / Alt-k    Resize logs pane
  Alt--            Collapse focused non-editor pane
  Alt-=            Reset focused pane size
  Ctrl-S           Save current file
  Ctrl-W           Close current tab
  q                Quit
  F1               Toggle help

Explorer
  Up/Down          Move selection
  Enter/Right      Expand directory or open file
  Left             Collapse directory

Editor
  Arrows           Move cursor
  Type             Insert text
  Enter            New line
  Backspace        Delete
  /                Search in file
  ] / [            Next/previous tab

Tasks
  Up/Down          Move selection
  Enter            Prepare task run confirmation
  g                Refresh Gradle tasks
  f                Filter tasks

Logs
  Up/Down          Scroll logs
  c                Clear logs
  x                Cancel active process
";
    frame.render_widget(
        Paragraph::new(help)
            .block(
                Block::default()
                    .title("Help")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .wrap(Wrap { trim: false }),
        popup,
    );
}

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
