use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::Sender;
use std::thread;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GradleTask {
    pub name: String,
    pub group: String,
}

#[derive(Debug, Clone)]
pub enum TaskDiscoveryState {
    Idle,
    Discovering,
    Ready,
    Failed(String),
}

#[derive(Debug)]
pub enum TaskEvent {
    Started,
    Finished(Result<Vec<GradleTask>, String>),
}

#[derive(Debug)]
pub struct TaskPanel {
    pub tasks: Vec<GradleTask>,
    pub selected: usize,
    pub filter: String,
    pub state: TaskDiscoveryState,
}

impl TaskPanel {
    pub fn new() -> Self {
        Self {
            tasks: fallback_tasks(),
            selected: 0,
            filter: String::new(),
            state: TaskDiscoveryState::Idle,
        }
    }

    pub fn filtered_tasks(&self) -> Vec<&GradleTask> {
        let needle = self.filter.to_lowercase();
        self.tasks
            .iter()
            .filter(|task| {
                needle.is_empty()
                    || task.name.to_lowercase().contains(&needle)
                    || task.group.to_lowercase().contains(&needle)
            })
            .collect()
    }

    pub fn move_down(&mut self) {
        let len = self.filtered_tasks().len();
        if self.selected + 1 < len {
            self.selected += 1;
        }
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn selected_task(&self) -> Option<GradleTask> {
        self.filtered_tasks()
            .get(self.selected)
            .map(|task| (*task).clone())
    }

    pub fn apply_discovery(&mut self, tasks: Vec<GradleTask>) {
        self.tasks = if tasks.is_empty() {
            fallback_tasks()
        } else {
            tasks
        };
        self.selected = 0;
        self.state = TaskDiscoveryState::Ready;
    }
}

pub fn discover_tasks(project_root: PathBuf, tx: Sender<TaskEvent>) {
    thread::spawn(move || {
        let _ = tx.send(TaskEvent::Started);
        let result = run_discovery(&project_root);
        let _ = tx.send(TaskEvent::Finished(result));
    });
}

fn run_discovery(project_root: &Path) -> Result<Vec<GradleTask>, String> {
    let gradlew = project_root.join("gradlew");
    let output = Command::new(&gradlew)
        .arg("-q")
        .arg("tasks")
        .arg("--all")
        .current_dir(project_root)
        .output()
        .map_err(|err| format!("failed to run {}: {err}", gradlew.display()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("task discovery failed: {}", stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_tasks(&stdout))
}

pub fn parse_tasks(raw: &str) -> Vec<GradleTask> {
    let mut tasks = Vec::new();
    let mut group = "Other".to_string();

    for line in raw.lines() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() || trimmed.starts_with('-') || trimmed.starts_with("Tasks runnable") {
            continue;
        }
        if !line.starts_with(' ') && trimmed.ends_with("tasks") {
            group = trimmed.trim_end_matches("tasks").trim().to_string();
            if group.is_empty() {
                group = "Other".to_string();
            }
            continue;
        }

        let Some((name, _rest)) = trimmed.split_once(" - ") else {
            continue;
        };

        tasks.push(GradleTask {
            name: name.trim().to_string(),
            group: group.clone(),
        });
    }

    tasks.sort_by(|left, right| left.name.cmp(&right.name));
    tasks.dedup_by(|left, right| left.name == right.name);
    tasks
}

fn fallback_tasks() -> Vec<GradleTask> {
    vec![
        GradleTask {
            name: ":app:assembleDebug".to_string(),
            group: "Build".to_string(),
        },
        GradleTask {
            name: ":app:installDebug".to_string(),
            group: "Install".to_string(),
        },
        GradleTask {
            name: ":app:test".to_string(),
            group: "Verification".to_string(),
        },
        GradleTask {
            name: ":app:connectedAndroidTest".to_string(),
            group: "Verification".to_string(),
        },
        GradleTask {
            name: ":app:lint".to_string(),
            group: "Verification".to_string(),
        },
        GradleTask {
            name: "clean".to_string(),
            group: "Build".to_string(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_gradle_task_output() {
        let raw = "\
Build tasks
-----------
build - Assemble and test this project.
clean - Clean outputs.
app:assemble - Assemble outputs.
app:clean - Clean outputs.

Verification tasks
------------------
app:test - Runs tests.
";

        let tasks = parse_tasks(raw);
        assert_eq!(tasks.len(), 5);
        assert_eq!(tasks[0].group, "Build");
        assert!(tasks.iter().any(|task| task.name == "clean"));
        assert!(tasks.iter().any(|task| task.name == "build"));
        assert!(tasks.iter().any(|task| task.group == "Verification"));
    }

    #[test]
    fn deduplicates_root_and_module_tasks_by_name_only() {
        let raw = "\
Build tasks
-----------
clean - Clean root.
clean - Clean root again.
app:clean - Clean app.
";

        let tasks = parse_tasks(raw);
        assert_eq!(tasks.iter().filter(|task| task.name == "clean").count(), 1);
        assert_eq!(tasks.iter().filter(|task| task.name == "app:clean").count(), 1);
    }
}
