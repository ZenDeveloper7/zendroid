use dirs::{config_dir, data_dir};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub theme: String,
    pub keymap_preset: String,
    pub default_log_source: String,
    pub show_hidden_files: bool,
    pub tasks_panel_expanded: bool,
    pub confirm_before_run: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: "amber".to_string(),
            keymap_preset: "default".to_string(),
            default_log_source: "process".to_string(),
            show_hidden_files: false,
            tasks_panel_expanded: true,
            confirm_before_run: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionState {
    #[serde(default)]
    pub last_open_files: Vec<PathBuf>,
    #[serde(default)]
    pub last_selected_file: Option<PathBuf>,
    #[serde(default)]
    pub selected_task: Option<String>,
    #[serde(default)]
    pub explorer_open_dirs: Vec<PathBuf>,
    #[serde(default)]
    pub selected_pane: Option<String>,
    #[serde(default)]
    pub selected_variant: Option<String>,
    #[serde(default)]
    pub right_pane: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ConfigStore {
    pub config: AppConfig,
    pub session: SessionState,
    session_path: PathBuf,
}

impl ConfigStore {
    pub fn load(custom_path: Option<PathBuf>) -> Self {
        let config_path = custom_path.unwrap_or_else(default_config_path);
        let session_path = default_session_path();

        let config = load_json::<AppConfig>(&config_path).unwrap_or_else(|| {
            let default = AppConfig::default();
            write_json(&config_path, &default);
            default
        });

        let session = load_json::<SessionState>(&session_path).unwrap_or_default();

        Self {
            config,
            session,
            session_path,
        }
    }

    pub fn save_session(&self, session: &SessionState) {
        write_json(&self.session_path, session);
    }
}

fn default_config_path() -> PathBuf {
    config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("zendroid")
        .join("config.json")
}

fn default_session_path() -> PathBuf {
    data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("zendroid")
        .join("session.json")
}

fn load_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Option<T> {
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

fn write_json<T: Serialize>(path: &Path, value: &T) {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(raw) = serde_json::to_string_pretty(value) {
        let _ = fs::write(path, raw);
    }
}
