use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::Sender;
use std::thread;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GradleTask {
    pub name: String,
    pub group: String,
    pub module: Option<String>,
    pub category: TaskCategory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskCategory {
    Build,
    Install,
    Test,
    Lint,
    Clean,
    Other,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GradleModel {
    pub tasks: Vec<GradleTask>,
    pub modules: Vec<String>,
    pub variants: Vec<String>,
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
    Finished(Result<GradleModel, String>),
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

    pub fn apply_discovery(&mut self, model: &GradleModel) {
        self.tasks = if model.tasks.is_empty() {
            fallback_tasks()
        } else {
            model.tasks.clone()
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

fn run_discovery(project_root: &Path) -> Result<GradleModel, String> {
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
    let tasks = parse_tasks(&stdout);
    let modules = discover_modules(project_root, &tasks);
    let variants = parse_variants(&tasks);
    Ok(GradleModel {
        tasks,
        modules,
        variants,
    })
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
            module: module_for_task(name.trim()),
            category: category_for_task(name.trim(), &group),
        });
    }

    tasks.sort_by(|left, right| left.name.cmp(&right.name));
    tasks.dedup_by(|left, right| left.name == right.name);
    tasks
}

pub fn parse_variants(tasks: &[GradleTask]) -> Vec<String> {
    let mut variants = tasks
        .iter()
        .filter_map(|task| {
            let leaf = task.name.rsplit(':').next().unwrap_or(&task.name);
            for prefix in ["assemble", "install", "bundle"] {
                if let Some(variant) = leaf.strip_prefix(prefix) {
                    if !variant.is_empty() && variant.chars().next().is_some_and(char::is_uppercase)
                    {
                        return Some(variant.to_string());
                    }
                }
            }
            None
        })
        .collect::<Vec<_>>();
    variants.sort();
    variants.dedup();
    variants
}

pub fn parse_settings_modules(raw: &str) -> Vec<String> {
    let mut modules = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("include") {
            continue;
        }
        for segment in trimmed.split(['"', '\'']) {
            if let Some(module) = segment.strip_prefix(':') {
                if !module.is_empty() {
                    modules.push(module.replace(':', "/"));
                }
            }
        }
    }
    modules.sort();
    modules.dedup();
    modules
}

fn discover_modules(project_root: &Path, tasks: &[GradleTask]) -> Vec<String> {
    let mut modules = Vec::new();
    for filename in ["settings.gradle.kts", "settings.gradle"] {
        let path = project_root.join(filename);
        if let Ok(raw) = std::fs::read_to_string(path) {
            modules.extend(parse_settings_modules(&raw));
        }
    }
    modules.extend(tasks.iter().filter_map(|task| task.module.clone()));
    modules.sort();
    modules.dedup();
    modules
}

fn module_for_task(name: &str) -> Option<String> {
    let trimmed = name.trim_start_matches(':');
    let mut parts = trimmed.split(':').collect::<Vec<_>>();
    if parts.len() <= 1 {
        return None;
    }
    parts.pop();
    Some(parts.join("/"))
}

fn category_for_task(name: &str, group: &str) -> TaskCategory {
    let leaf = name.rsplit(':').next().unwrap_or(name).to_lowercase();
    let group = group.to_lowercase();
    if leaf.contains("install") {
        TaskCategory::Install
    } else if leaf.contains("test") || group.contains("verification") {
        TaskCategory::Test
    } else if leaf.contains("lint") {
        TaskCategory::Lint
    } else if leaf == "clean" {
        TaskCategory::Clean
    } else if leaf.contains("assemble") || leaf.contains("build") || leaf.contains("bundle") {
        TaskCategory::Build
    } else {
        TaskCategory::Other
    }
}

fn fallback_tasks() -> Vec<GradleTask> {
    vec![
        GradleTask {
            name: ":app:assembleDebug".to_string(),
            group: "Build".to_string(),
            module: Some("app".to_string()),
            category: TaskCategory::Build,
        },
        GradleTask {
            name: ":app:installDebug".to_string(),
            group: "Install".to_string(),
            module: Some("app".to_string()),
            category: TaskCategory::Install,
        },
        GradleTask {
            name: ":app:test".to_string(),
            group: "Verification".to_string(),
            module: Some("app".to_string()),
            category: TaskCategory::Test,
        },
        GradleTask {
            name: ":app:connectedAndroidTest".to_string(),
            group: "Verification".to_string(),
            module: Some("app".to_string()),
            category: TaskCategory::Test,
        },
        GradleTask {
            name: ":app:lint".to_string(),
            group: "Verification".to_string(),
            module: Some("app".to_string()),
            category: TaskCategory::Lint,
        },
        GradleTask {
            name: "clean".to_string(),
            group: "Build".to_string(),
            module: None,
            category: TaskCategory::Clean,
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
        assert!(
            tasks
                .iter()
                .any(|task| task.module.as_deref() == Some("app"))
        );
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
        assert_eq!(
            tasks.iter().filter(|task| task.name == "app:clean").count(),
            1
        );
    }

    #[test]
    fn parses_variants_from_common_android_tasks() {
        let tasks = parse_tasks(
            "\
Build tasks
-----------
app:assembleDebug - Assemble.
app:assembleFreeRelease - Assemble.
app:bundleRelease - Bundle.
Install tasks
-------------
app:installDebug - Install.
",
        );

        let variants = parse_variants(&tasks);
        assert!(variants.contains(&"Debug".to_string()));
        assert!(variants.contains(&"FreeRelease".to_string()));
        assert!(variants.contains(&"Release".to_string()));
    }

    #[test]
    fn parses_settings_modules() {
        let raw = r#"
include(":app", ":core:data")
include ':feature:chat'
"#;

        let modules = parse_settings_modules(raw);
        assert!(modules.contains(&"app".to_string()));
        assert!(modules.contains(&"core/data".to_string()));
        assert!(modules.contains(&"feature/chat".to_string()));
    }
}
