use std::env;
use std::path::{Path, PathBuf};

pub fn discover_project_root(input: Option<PathBuf>) -> Result<PathBuf, String> {
    let start = match input {
        Some(path) => path,
        None => {
            env::current_dir().map_err(|err| format!("failed to read current directory: {err}"))?
        }
    };

    let canonical = start
        .canonicalize()
        .map_err(|err| format!("failed to resolve {}: {err}", start.display()))?;

    if canonical.is_file() {
        let parent = canonical
            .parent()
            .ok_or_else(|| "the provided file has no parent directory".to_string())?;
        return ascend(parent);
    }

    ascend(&canonical)
}

fn ascend(start: &Path) -> Result<PathBuf, String> {
    let mut current = Some(start);

    while let Some(path) = current {
        if is_android_project(path) {
            return Ok(path.to_path_buf());
        }
        current = path.parent();
    }

    Err(format!(
        "no Android project root found from {}",
        start.display()
    ))
}

fn is_android_project(path: &Path) -> bool {
    path.join("gradlew").exists()
        && (path.join("settings.gradle").exists() || path.join("settings.gradle.kts").exists())
}
