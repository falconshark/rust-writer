// settings.rs - Persistent application settings

use crate::theme::Theme;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;

/// How the background image is rendered, matching FocusWriter's 5 modes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum BgImageMode {
    Tiled,
    Stretched,
    Scaled,    // fit within bounds, letterbox (maintain aspect ratio)
    #[default]
    Zoomed,    // cover: fill completely, crop excess (maintain aspect ratio)
    Centered,  // original size, centered
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    // Font
    pub font_size: f32,
    pub font_family: String,

    // Writing area
    pub text_column_width: f32,
    pub line_spacing: f32,

    // Behavior
    pub typewriter_mode: bool,
    pub auto_save_interval: f32, // seconds
    pub typing_sounds: bool,
    #[serde(default = "Settings::default_sound_volume")]
    pub sound_volume: f32,

    // Daily goal
    pub daily_goal_enabled: bool,
    pub daily_goal_words: usize,

    // Theme (serialized name)
    pub theme_name: String,

    // Background image
    #[serde(default)]
    pub bg_image_path: Option<String>,
    #[serde(default)]
    pub bg_image_mode: BgImageMode,

    #[serde(default)]
    pub current_theme: Theme,

    // Session restore
    #[serde(default)]
    pub last_opened_files: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            font_size: 18.0,
            font_family: "serif".to_string(),
            text_column_width: 700.0,
            line_spacing: 1.6,
            typewriter_mode: true,
            auto_save_interval: 60.0,
            typing_sounds: false,
            sound_volume: Self::default_sound_volume(),
            daily_goal_enabled: false,
            daily_goal_words: 1000,
            theme_name: "Night Owl".to_string(),
            bg_image_path: None,
            bg_image_mode: BgImageMode::default(),
            current_theme: Theme::default(),
            last_opened_files: Vec::new(),
        }
    }
}

impl Settings {
    fn default_sound_volume() -> f32 { 2.0 }

    pub fn config_path() -> Option<PathBuf> {
        ProjectDirs::from("com", "focuswriter", "focuswriter-rs")
            .map(|dirs| dirs.config_dir().join("settings.toml"))
    }

    pub fn load() -> Self {
        if let Some(path) = Self::config_path() {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(mut s) = toml::from_str::<Settings>(&content) {
                        // For old config files that pre-date full theme serialization,
                        // current_theme will deserialize as Theme::default(). In that
                        // case fall back to looking up the preset by name.
                        if s.current_theme == Theme::default() && s.theme_name != Theme::default().name {
                            s.current_theme = Theme::by_name(&s.theme_name)
                                .unwrap_or_default();
                        }
                        return s;
                    }
                }
            }
        }
        Settings::default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(path) = Self::config_path() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let content = toml::to_string(self)?;
            fs::write(path, content)?;
        }
        Ok(())
    }
}
