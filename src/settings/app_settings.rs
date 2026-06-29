use crate::theme::ThemeChoice;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum VideoQuality {
    High,
    Medium,
    Low,
}

impl Default for VideoQuality {
    fn default() -> Self {
        Self::High
    }
}

impl VideoQuality {
    pub fn label(self) -> &'static str {
        match self {
            VideoQuality::High => "High (CQ 23)",
            VideoQuality::Medium => "Medium (CQ 28)",
            VideoQuality::Low => "Low (CQ 35)",
        }
    }

    pub fn cq_value(self) -> u32 {
        match self {
            VideoQuality::High => 23,
            VideoQuality::Medium => 28,
            VideoQuality::Low => 35,
        }
    }

    pub const ALL: &'static [VideoQuality] = &[VideoQuality::High, VideoQuality::Medium, VideoQuality::Low];
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: ThemeChoice,
    pub clip_output_dir: String,
    pub quality: VideoQuality,
}

impl Default for AppSettings {
    fn default() -> Self {
        let clip_dir = std::env::var("USERPROFILE")
            .map(|p| format!(r"{}\Videos\BSClipper\", p))
            .unwrap_or_else(|_| String::from(r"C:\Videos\BSClipper\"));

        Self {
            theme: ThemeChoice::Lucky,
            clip_output_dir: clip_dir,
            quality: VideoQuality::High,
        }
    }
}
