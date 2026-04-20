use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ExplorerEntry {
    pub path: PathBuf,
    pub depth: usize,
    pub is_dir: bool,
    pub name: String,
}

#[derive(Debug)]
pub struct FileExplorer {
    pub root: PathBuf,
    pub entries: Vec<ExplorerEntry>,
    pub expanded: HashSet<PathBuf>,
    pub selected: usize,
    pub show_hidden: bool,
}

impl FileExplorer {
    pub fn new(root: PathBuf, show_hidden: bool, expanded_dirs: Vec<PathBuf>) -> Self {
        let mut expanded: HashSet<PathBuf> = expanded_dirs.into_iter().collect();
        expanded.insert(root.clone());
        let mut explorer = Self {
            root,
            entries: Vec::new(),
            expanded,
            selected: 0,
            show_hidden,
        };
        explorer.refresh();
        explorer
    }

    pub fn refresh(&mut self) {
        self.entries.clear();
        self.build_entries(self.root.clone(), 0);
        if self.selected >= self.entries.len() {
            self.selected = self.entries.len().saturating_sub(1);
        }
    }

    fn build_entries(&mut self, path: PathBuf, depth: usize) {
        let name = if depth == 0 {
            path.file_name()
                .map(|value| value.to_string_lossy().to_string())
                .unwrap_or_else(|| path.display().to_string())
        } else {
            path.file_name()
                .map(|value| value.to_string_lossy().to_string())
                .unwrap_or_else(|| path.display().to_string())
        };
        let is_dir = path.is_dir();
        self.entries.push(ExplorerEntry {
            path: path.clone(),
            depth,
            is_dir,
            name,
        });

        if !is_dir || !self.expanded.contains(&path) {
            return;
        }

        let Ok(read_dir) = fs::read_dir(&path) else {
            return;
        };
        let mut children = Vec::new();
        for entry in read_dir.flatten() {
            let child_path = entry.path();
            if !self.show_hidden && is_hidden(&child_path) {
                continue;
            }
            if child_path.is_dir() && ignored_dir(&child_path) {
                continue;
            }
            children.push(child_path);
        }
        children.sort_by(|left, right| {
            let left_dir = left.is_dir();
            let right_dir = right.is_dir();
            right_dir
                .cmp(&left_dir)
                .then_with(|| left.file_name().cmp(&right.file_name()))
        });

        for child in children {
            self.build_entries(child, depth + 1);
        }
    }

    pub fn move_down(&mut self) {
        if self.selected + 1 < self.entries.len() {
            self.selected += 1;
        }
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn selected_entry(&self) -> Option<&ExplorerEntry> {
        self.entries.get(self.selected)
    }

    pub fn toggle_selected(&mut self) -> Option<PathBuf> {
        let Some(entry) = self.selected_entry().cloned() else {
            return None;
        };
        if entry.is_dir {
            if self.expanded.contains(&entry.path) {
                self.expanded.remove(&entry.path);
            } else {
                self.expanded.insert(entry.path.clone());
            }
            self.refresh();
            None
        } else {
            Some(entry.path)
        }
    }

    pub fn collapse_selected(&mut self) {
        let Some(entry) = self.selected_entry().cloned() else {
            return;
        };
        if entry.is_dir && self.expanded.contains(&entry.path) && entry.path != self.root {
            self.expanded.remove(&entry.path);
            self.refresh();
        } else if let Some(parent) = entry.path.parent() {
            if let Some(index) = self.entries.iter().position(|value| value.path == parent) {
                self.selected = index;
            }
        }
    }

    pub fn expand_selected(&mut self) -> Option<PathBuf> {
        let Some(entry) = self.selected_entry().cloned() else {
            return None;
        };
        if entry.is_dir {
            self.expanded.insert(entry.path.clone());
            self.refresh();
            None
        } else {
            Some(entry.path)
        }
    }

    pub fn expanded_dirs(&self) -> Vec<PathBuf> {
        self.expanded.iter().cloned().collect()
    }

    pub fn select_path(&mut self, target: &Path) {
        if let Some(parent) = target.parent() {
            let mut current = Some(parent);
            while let Some(dir) = current {
                self.expanded.insert(dir.to_path_buf());
                if dir == self.root {
                    break;
                }
                current = dir.parent();
            }
            self.refresh();
        }
        if let Some(index) = self.entries.iter().position(|entry| entry.path == target) {
            self.selected = index;
        }
    }
}

fn ignored_dir(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|value| value.to_str()),
        Some(".git" | ".gradle" | ".idea" | "build" | "target")
    )
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|value| value.to_str())
        .map(|value| value.starts_with('.'))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignored_generated_dirs() {
        assert!(ignored_dir(Path::new("/tmp/build")));
        assert!(ignored_dir(Path::new("/tmp/.git")));
        assert!(!ignored_dir(Path::new("/tmp/app")));
    }
}
