use crate::config::{AppConfig, SessionState};
use crate::editor::EditorState;
use crate::explorer::FileExplorer;
use crate::gradle::{GradleTask, TaskDiscoveryState, TaskEvent, TaskPanel, discover_tasks};
use crate::process::{LogState, ProcessEvent, ProcessHandle, spawn_command};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPane {
    Explorer,
    Editor,
    Tasks,
    Logs,
}

#[derive(Debug, Clone)]
pub enum InputMode {
    Normal,
    Help,
    Search { query: String },
    TaskFilter { query: String },
    ConfirmRun { command: String },
}

#[derive(Debug, Clone, Copy)]
pub struct PaneLayout {
    pub explorer_width: u16,
    pub tasks_width: u16,
    pub logs_height: u16,
    pub explorer_collapsed: bool,
    pub tasks_collapsed: bool,
    pub logs_collapsed: bool,
}

impl Default for PaneLayout {
    fn default() -> Self {
        Self {
            explorer_width: 22,
            tasks_width: 25,
            logs_height: 10,
            explorer_collapsed: false,
            tasks_collapsed: false,
            logs_collapsed: false,
        }
    }
}

#[derive(Debug)]
pub struct App {
    pub project_root: PathBuf,
    pub config: AppConfig,
    pub read_only: bool,
    pub focus: FocusPane,
    pub input_mode: InputMode,
    pub explorer: FileExplorer,
    pub editor: EditorState,
    pub tasks: TaskPanel,
    pub logs: LogState,
    pub layout: PaneLayout,
    pub process: ProcessHandle,
    pub status: String,
    pub pending_task: Option<GradleTask>,
    pub task_tx: Sender<TaskEvent>,
    pub task_rx: Receiver<TaskEvent>,
    pub process_tx: Sender<ProcessEvent>,
    pub process_rx: Receiver<ProcessEvent>,
    pub should_quit: bool,
}

impl App {
    pub fn new(
        project_root: PathBuf,
        config: AppConfig,
        session: &SessionState,
        read_only: bool,
    ) -> Self {
        let (task_tx, task_rx) = std::sync::mpsc::channel();
        let (process_tx, process_rx) = std::sync::mpsc::channel();
        let focus = match session.selected_pane.as_deref() {
            Some("Explorer") => FocusPane::Explorer,
            Some("Tasks") => FocusPane::Tasks,
            Some("Logs") => FocusPane::Logs,
            _ => FocusPane::Editor,
        };

        let mut app = Self {
            project_root: project_root.clone(),
            config: config.clone(),
            read_only,
            focus,
            input_mode: InputMode::Normal,
            explorer: FileExplorer::new(
                project_root.clone(),
                config.show_hidden_files,
                session.explorer_open_dirs.clone(),
            ),
            editor: EditorState::default(),
            tasks: TaskPanel::new(),
            logs: LogState::default(),
            layout: PaneLayout::default(),
            process: ProcessHandle::default(),
            status: "Project loaded".to_string(),
            pending_task: None,
            task_tx,
            task_rx,
            process_tx,
            process_rx,
            should_quit: false,
        };

        for file in &session.last_open_files {
            let _ = app.editor.open_or_focus(file.clone());
        }
        if let Some(path) = &session.last_selected_file {
            let _ = app.editor.open_or_focus(path.clone());
            app.explorer.select_path(path);
        }
        if let Some(task_name) = &session.selected_task {
            if let Some(index) = app
                .tasks
                .tasks
                .iter()
                .position(|task| &task.name == task_name)
            {
                app.tasks.selected = index;
            }
        }
        app.refresh_tasks();
        app
    }

    pub fn session_state(&self) -> SessionState {
        SessionState {
            last_open_files: self
                .editor
                .buffers
                .iter()
                .map(|buffer| buffer.path.clone())
                .collect(),
            last_selected_file: self.editor.active_path().map(Path::to_path_buf),
            selected_task: self.tasks.selected_task().map(|task| task.name),
            explorer_open_dirs: self.explorer.expanded_dirs(),
            selected_pane: Some(format!("{:?}", self.focus)),
        }
    }

    pub fn handle_background_events(&mut self) {
        while let Ok(event) = self.task_rx.try_recv() {
            match event {
                TaskEvent::Started => {
                    self.tasks.state = TaskDiscoveryState::Discovering;
                    self.status = "Scanning Gradle tasks...".to_string();
                }
                TaskEvent::Finished(Ok(tasks)) => {
                    self.tasks.apply_discovery(tasks);
                    self.status = "Gradle tasks refreshed".to_string();
                }
                TaskEvent::Finished(Err(err)) => {
                    self.tasks.state = TaskDiscoveryState::Failed(err.clone());
                    self.status = err;
                }
            }
        }

        while let Ok(event) = self.process_rx.try_recv() {
            match event {
                ProcessEvent::Started { command } => {
                    self.logs.push(format!("$ {command}"));
                    self.process.command_display = Some(command);
                    self.status = "Task started".to_string();
                }
                ProcessEvent::Output(line) => self.logs.push(line),
                ProcessEvent::Finished { success, summary } => {
                    self.logs.push(summary.clone());
                    self.status = if success {
                        "Task completed successfully".to_string()
                    } else {
                        "Task failed".to_string()
                    };
                    self.process.clear();
                }
            }
        }
    }

    pub fn handle_event(&mut self, event: Event) {
        if let Event::Key(key) = event {
            self.handle_key_event(key);
        }
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        match std::mem::replace(&mut self.input_mode, InputMode::Normal) {
            InputMode::Help => {
                if !matches!(key.code, KeyCode::Esc | KeyCode::F(1) | KeyCode::Char('?')) {
                    self.input_mode = InputMode::Help;
                }
                return;
            }
            InputMode::Search { mut query } => {
                if self.handle_text_prompt(&mut query, key, PromptTarget::Search) {
                    self.input_mode = InputMode::Search { query };
                }
                return;
            }
            InputMode::TaskFilter { mut query } => {
                if self.handle_text_prompt(&mut query, key, PromptTarget::TaskFilter) {
                    self.input_mode = InputMode::TaskFilter { query };
                }
                return;
            }
            InputMode::ConfirmRun { command } => {
                match key.code {
                    KeyCode::Char('y') => {
                        if let Some(task) = self.pending_task.clone() {
                            self.run_task(task);
                        }
                    }
                    KeyCode::Char('n') | KeyCode::Esc => {
                        self.pending_task = None;
                        self.status = "Task run cancelled".to_string();
                    }
                    _ => {
                        self.input_mode = InputMode::ConfirmRun { command };
                    }
                }
                return;
            }
            InputMode::Normal => {}
        }

        if key.code == KeyCode::F(1) || key.code == KeyCode::Char('?') {
            self.input_mode = InputMode::Help;
            return;
        }
        if key.modifiers.contains(KeyModifiers::ALT) {
            if self.handle_alt_shortcut(key) {
                return;
            }
        }
        if key.code == KeyCode::Tab {
            self.focus = match self.focus {
                FocusPane::Explorer => FocusPane::Editor,
                FocusPane::Editor => FocusPane::Tasks,
                FocusPane::Tasks => FocusPane::Logs,
                FocusPane::Logs => FocusPane::Explorer,
            };
            return;
        }
        if key.code == KeyCode::BackTab {
            self.focus = match self.focus {
                FocusPane::Explorer => FocusPane::Logs,
                FocusPane::Editor => FocusPane::Explorer,
                FocusPane::Tasks => FocusPane::Editor,
                FocusPane::Logs => FocusPane::Tasks,
            };
            return;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('s') {
            if self.read_only {
                self.status = "Read-only mode is active".to_string();
            } else {
                self.status = match self.editor.save_current() {
                    Ok(_) => "File saved".to_string(),
                    Err(err) => err,
                };
            }
            return;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('w') {
            self.editor.close_current();
            return;
        }
        if key.code == KeyCode::Char('q') {
            self.should_quit = true;
            return;
        }

        match self.focus {
            FocusPane::Explorer => self.handle_explorer_key(key),
            FocusPane::Editor => self.handle_editor_key(key),
            FocusPane::Tasks => self.handle_tasks_key(key),
            FocusPane::Logs => self.handle_logs_key(key),
        }
    }

    fn handle_alt_shortcut(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('1') => {
                self.focus = FocusPane::Explorer;
                self.status = "Focused Files pane".to_string();
                true
            }
            KeyCode::Char('2') => {
                self.focus = FocusPane::Editor;
                self.status = "Focused Editor pane".to_string();
                true
            }
            KeyCode::Char('3') => {
                self.focus = FocusPane::Tasks;
                self.status = "Focused Tasks pane".to_string();
                true
            }
            KeyCode::Char('4') => {
                self.focus = FocusPane::Logs;
                self.status = "Focused Logs pane".to_string();
                true
            }
            KeyCode::Char('-') => {
                self.toggle_focused_pane();
                true
            }
            KeyCode::Char('=') | KeyCode::Char('+') => {
                self.expand_focused_pane();
                true
            }
            KeyCode::Char('h') => {
                self.resize_focused_pane(-3);
                true
            }
            KeyCode::Char('l') => {
                self.resize_focused_pane(3);
                true
            }
            KeyCode::Char('j') => {
                self.resize_logs(-2);
                true
            }
            KeyCode::Char('k') => {
                self.resize_logs(2);
                true
            }
            _ => false,
        }
    }

    fn handle_explorer_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => self.explorer.move_down(),
            KeyCode::Up | KeyCode::Char('k') => self.explorer.move_up(),
            KeyCode::Left => self.explorer.collapse_selected(),
            KeyCode::Right | KeyCode::Enter => {
                if let Some(path) = self
                    .explorer
                    .toggle_selected()
                    .or_else(|| self.explorer.expand_selected())
                {
                    self.open_file(path);
                }
            }
            KeyCode::Char('r') => self.explorer.refresh(),
            _ => {}
        }
    }

    fn handle_editor_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Left => self.editor.move_left(),
            KeyCode::Right => self.editor.move_right(),
            KeyCode::Up => self.editor.move_up(),
            KeyCode::Down => self.editor.move_down(),
            KeyCode::PageUp => self.editor.page_up(10),
            KeyCode::PageDown => self.editor.page_down(10),
            KeyCode::Backspace if !self.read_only => self.editor.backspace(),
            KeyCode::Enter if !self.read_only => self.editor.insert_newline(),
            KeyCode::Tab if !self.read_only => {
                for _ in 0..4 {
                    self.editor.insert_char(' ');
                }
            }
            KeyCode::Char('/') => {
                self.input_mode = InputMode::Search {
                    query: String::new(),
                };
            }
            KeyCode::Char(']') => self.editor.next_tab(),
            KeyCode::Char('[') => self.editor.previous_tab(),
            KeyCode::Char(ch) if !self.read_only && key.modifiers.is_empty() => {
                self.editor.insert_char(ch)
            }
            _ => {}
        }
    }

    fn handle_tasks_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => self.tasks.move_down(),
            KeyCode::Up | KeyCode::Char('k') => self.tasks.move_up(),
            KeyCode::Char('g') => self.refresh_tasks(),
            KeyCode::Char('f') => {
                self.input_mode = InputMode::TaskFilter {
                    query: self.tasks.filter.clone(),
                };
            }
            KeyCode::Enter => {
                if let Some(task) = self.tasks.selected_task() {
                    let command = format!("./gradlew {}", task.name);
                    self.pending_task = Some(task);
                    if self.read_only || self.config.confirm_before_run {
                        self.input_mode = InputMode::ConfirmRun { command };
                    } else if let Some(task) = self.pending_task.clone() {
                        self.run_task(task);
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_logs_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.logs.scroll = self.logs.scroll.saturating_sub(1);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.logs.scroll += 1;
            }
            KeyCode::Char('c') => self.logs.clear(),
            KeyCode::Char('x') => {
                self.status = match self.process.cancel() {
                    Ok(_) => "Cancelling active process...".to_string(),
                    Err(err) => err,
                };
            }
            _ => {}
        }
    }

    fn open_file(&mut self, path: PathBuf) {
        self.status = match self.editor.open_or_focus(path.clone()) {
            Ok(_) => {
                self.explorer.select_path(&path);
                format!("Opened {}", path.display())
            }
            Err(err) => err,
        };
    }

    fn refresh_tasks(&mut self) {
        discover_tasks(self.project_root.clone(), self.task_tx.clone());
    }

    fn run_task(&mut self, task: GradleTask) {
        if self.read_only {
            self.status = "Read-only mode prevents task execution".to_string();
            return;
        }
        if self.process.is_running() {
            self.status = "Another task is already running".to_string();
            return;
        }

        let gradlew = self.project_root.join("gradlew");
        let args = vec![task.name.clone()];
        match spawn_command(&gradlew, &args, &self.project_root, self.process_tx.clone()) {
            Ok(child) => {
                self.process.child = Some(child);
                self.pending_task = None;
                self.logs.push(format!("Preparing {}", task.name));
            }
            Err(err) => self.status = err,
        }
    }

    fn toggle_focused_pane(&mut self) {
        match self.focus {
            FocusPane::Explorer => {
                self.layout.explorer_collapsed = !self.layout.explorer_collapsed;
                self.status = if self.layout.explorer_collapsed {
                    "Collapsed Files pane".to_string()
                } else {
                    "Expanded Files pane".to_string()
                };
            }
            FocusPane::Tasks => {
                self.layout.tasks_collapsed = !self.layout.tasks_collapsed;
                self.status = if self.layout.tasks_collapsed {
                    "Collapsed Tasks pane".to_string()
                } else {
                    "Expanded Tasks pane".to_string()
                };
            }
            FocusPane::Logs => {
                self.layout.logs_collapsed = !self.layout.logs_collapsed;
                self.status = if self.layout.logs_collapsed {
                    "Collapsed Logs pane".to_string()
                } else {
                    "Expanded Logs pane".to_string()
                };
            }
            FocusPane::Editor => {
                self.status = "Editor pane cannot be collapsed".to_string();
            }
        }
    }

    fn expand_focused_pane(&mut self) {
        match self.focus {
            FocusPane::Explorer => {
                self.layout.explorer_collapsed = false;
                self.layout.explorer_width = 22;
                self.status = "Reset Files pane width".to_string();
            }
            FocusPane::Tasks => {
                self.layout.tasks_collapsed = false;
                self.layout.tasks_width = 25;
                self.status = "Reset Tasks pane width".to_string();
            }
            FocusPane::Logs => {
                self.layout.logs_collapsed = false;
                self.layout.logs_height = 10;
                self.status = "Reset Logs pane height".to_string();
            }
            FocusPane::Editor => {
                self.status = "Use Alt-1/3/4 to resize side panes".to_string();
            }
        }
    }

    fn resize_focused_pane(&mut self, delta: i16) {
        match self.focus {
            FocusPane::Explorer => {
                self.layout.explorer_collapsed = false;
                self.layout.explorer_width =
                    clamp_percent(self.layout.explorer_width, delta, 12, 40);
                self.status = format!("Files pane width: {}%", self.layout.explorer_width);
            }
            FocusPane::Tasks => {
                self.layout.tasks_collapsed = false;
                self.layout.tasks_width = clamp_percent(self.layout.tasks_width, delta, 14, 40);
                self.status = format!("Tasks pane width: {}%", self.layout.tasks_width);
            }
            FocusPane::Logs => {
                self.resize_logs(delta);
            }
            FocusPane::Editor => {
                self.status = "Focus Files or Tasks pane to resize width".to_string();
            }
        }
    }

    fn resize_logs(&mut self, delta: i16) {
        self.layout.logs_collapsed = false;
        self.layout.logs_height = clamp_percent(self.layout.logs_height, delta, 6, 18);
        self.status = format!("Logs pane height: {} rows", self.layout.logs_height);
    }

    fn handle_text_prompt(
        &mut self,
        query: &mut String,
        key: KeyEvent,
        target: PromptTarget,
    ) -> bool {
        match key.code {
            KeyCode::Esc => false,
            KeyCode::Backspace => {
                query.pop();
                true
            }
            KeyCode::Enter => {
                let value = query.clone();
                match target {
                    PromptTarget::Search => {
                        if self.editor.search(&value) {
                            self.status = format!("Found '{value}'");
                        } else {
                            self.status = format!("No results for '{value}'");
                        }
                    }
                    PromptTarget::TaskFilter => {
                        self.tasks.filter = value.clone();
                        self.tasks.selected = 0;
                        self.status = format!("Filtering tasks by '{value}'");
                    }
                }
                false
            }
            KeyCode::Char(ch)
                if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT =>
            {
                query.push(ch);
                true
            }
            _ => true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum PromptTarget {
    Search,
    TaskFilter,
}

fn clamp_percent(current: u16, delta: i16, min: u16, max: u16) -> u16 {
    let next = (current as i16 + delta).clamp(min as i16, max as i16);
    next as u16
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AppConfig, SessionState};
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn restores_all_tabs_and_focuses_last_selected_file() {
        let project_root = make_test_project();
        let first = project_root.join("first.txt");
        let second = project_root.join("second.txt");
        fs::write(&first, "first").unwrap();
        fs::write(&second, "second").unwrap();

        let session = SessionState {
            last_open_files: vec![first.clone(), second.clone()],
            last_selected_file: Some(second.clone()),
            selected_task: Some("clean".to_string()),
            explorer_open_dirs: vec![project_root.clone()],
            selected_pane: Some("Editor".to_string()),
        };

        let app = App::new(project_root.clone(), AppConfig::default(), &session, false);

        assert_eq!(app.editor.buffers.len(), 2);
        assert_eq!(app.editor.active_path(), Some(second.as_path()));
        assert!(app.editor.buffers.iter().any(|buffer| buffer.path == first));
        assert!(app.editor.buffers.iter().any(|buffer| buffer.path == second));
    }

    fn make_test_project() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("zendroid-test-{unique}"));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("settings.gradle.kts"), "rootProject.name = \"Test\"\n").unwrap();
        let gradlew = root.join("gradlew");
        fs::write(
            &gradlew,
            "#!/usr/bin/env bash\nprintf 'Build tasks\n-----------\nclean - Clean.\napp:assemble - Assemble.\n'",
        )
        .unwrap();
        let mut perms = fs::metadata(&gradlew).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&gradlew, perms).unwrap();
        root
    }
}
