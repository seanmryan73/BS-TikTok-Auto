pub mod app_settings;
pub use app_settings::AppSettings;

use std::fs;
use std::path::PathBuf;

pub struct SettingsFile {
    pub app: AppSettings,
}

impl SettingsFile {
    pub fn new(app: AppSettings) -> Self {
        Self { app }
    }
}

pub fn settings_path() -> PathBuf {
    let appdata = std::env::var("APPDATA").unwrap_or_else(|_| String::from("C:\\Users\\Default\\AppData\\Roaming"));
    PathBuf::from(appdata).join("BSTikTokAuto").join("settings.json")
}

pub fn ensure_dirs() {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let processed = settings_path()
        .parent()
        .map(|p| p.join("processed"))
        .unwrap_or_default();
    let _ = fs::create_dir_all(processed);
}

pub fn load() -> SettingsFile {
    let path = settings_path();
    let app = fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str::<AppSettings>(&s).ok())
        .unwrap_or_default();
    SettingsFile::new(app)
}

pub fn save(settings: &SettingsFile) {
    let path = settings_path();
    let tmp = path.with_extension("json.tmp");
    if let Ok(json) = serde_json::to_string_pretty(&settings.app) {
        if fs::write(&tmp, &json).is_ok() {
            let _ = fs::rename(&tmp, &path);
        }
    }
}
