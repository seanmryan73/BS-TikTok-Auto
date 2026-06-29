use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TokenData {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: u64,
    pub refresh_expires_at: u64,
    pub open_id: String,
}

impl TokenData {
    pub fn needs_refresh(&self) -> bool {
        self.expires_at <= unix_now() + 300
    }

    pub fn refresh_expired(&self) -> bool {
        self.refresh_expires_at <= unix_now()
    }

    pub fn seconds_until_expiry(&self) -> i64 {
        self.expires_at as i64 - unix_now() as i64
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn token_path() -> PathBuf {
    let appdata = std::env::var("APPDATA")
        .unwrap_or_else(|_| r"C:\Users\Default\AppData\Roaming".into());
    PathBuf::from(appdata).join("BSTikTokAuto").join("tokens.json")
}

pub fn load() -> Option<TokenData> {
    let data = std::fs::read_to_string(token_path()).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn save(token: &TokenData) {
    let path = token_path();
    let tmp = path.with_extension("json.tmp");
    if let Ok(json) = serde_json::to_string_pretty(token) {
        if std::fs::write(&tmp, json).is_ok() {
            let _ = std::fs::rename(&tmp, &path);
        }
    }
}

pub fn clear() {
    let _ = std::fs::remove_file(token_path());
}
