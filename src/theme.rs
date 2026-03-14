// theme.rs - Color themes for the writing environment

use eframe::egui::Color32;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    // Background (full-screen layer)
    pub bg_color: Color32,
    // Paper layer (foreground, sits on top of background)
    pub paper_color: Color32,
    pub paper_opacity: f32, // 0.0..=1.0
    // Text
    pub text_color: Color32,
    pub selection_color: Color32,
    pub cursor_color: Color32,
    // UI chrome
    pub toolbar_bg: Color32,
}

impl Theme {
    pub fn night_owl() -> Self {
        Self {
            name: "Night Owl".to_string(),
            bg_color: Color32::from_rgb(17, 21, 28),
            paper_color: Color32::WHITE,
            paper_opacity: 0.92,
            text_color: Color32::from_rgb(30, 35, 48),
            toolbar_bg: Color32::from_rgb(15, 18, 25),
            selection_color: Color32::from_rgb(84, 109, 160),
            cursor_color: Color32::from_rgb(84, 109, 160),
        }
    }

    pub fn solarized_dark() -> Self {
        Self {
            name: "Solarized Dark".to_string(),
            bg_color: Color32::from_rgb(0, 43, 54),
            paper_color: Color32::WHITE,
            paper_opacity: 0.9,
            text_color: Color32::from_rgb(10, 55, 68),
            toolbar_bg: Color32::from_rgb(0, 33, 44),
            selection_color: Color32::from_rgb(42, 161, 152),
            cursor_color: Color32::from_rgb(181, 137, 0),
        }
    }

    pub fn solarized_light() -> Self {
        Self {
            name: "Solarized Light".to_string(),
            bg_color: Color32::from_rgb(253, 246, 227),
            paper_color: Color32::from_rgb(238, 232, 213),
            paper_opacity: 1.0,
            text_color: Color32::from_rgb(88, 110, 117),
            toolbar_bg: Color32::from_rgb(238, 232, 213),
            selection_color: Color32::from_rgb(147, 161, 161),
            cursor_color: Color32::from_rgb(88, 110, 117),
        }
    }

    pub fn typewriter() -> Self {
        Self {
            name: "Typewriter".to_string(),
            bg_color: Color32::from_rgb(101, 76, 40),
            paper_color: Color32::from_rgb(250, 245, 230),
            paper_opacity: 1.0,
            text_color: Color32::from_rgb(42, 30, 15),
            toolbar_bg: Color32::from_rgb(80, 58, 28),
            selection_color: Color32::from_rgb(160, 130, 80),
            cursor_color: Color32::from_rgb(42, 30, 15),
        }
    }

    pub fn forest() -> Self {
        Self {
            name: "Forest".to_string(),
            bg_color: Color32::from_rgb(20, 38, 25),
            paper_color: Color32::WHITE,
            paper_opacity: 0.93,
            text_color: Color32::from_rgb(25, 50, 30),
            toolbar_bg: Color32::from_rgb(15, 28, 18),
            selection_color: Color32::from_rgb(60, 130, 70),
            cursor_color: Color32::from_rgb(60, 130, 70),
        }
    }

    pub fn midnight_blue() -> Self {
        Self {
            name: "Midnight Blue".to_string(),
            bg_color: Color32::from_rgb(10, 12, 40),
            paper_color: Color32::WHITE,
            paper_opacity: 0.93,
            text_color: Color32::from_rgb(20, 25, 70),
            toolbar_bg: Color32::from_rgb(8, 10, 30),
            selection_color: Color32::from_rgb(60, 90, 160),
            cursor_color: Color32::from_rgb(80, 120, 220),
        }
    }

    pub fn paper_white() -> Self {
        Self {
            name: "Paper White".to_string(),
            bg_color: Color32::from_rgb(200, 198, 192),
            paper_color: Color32::from_rgb(255, 253, 245),
            paper_opacity: 1.0,
            text_color: Color32::from_rgb(35, 35, 35),
            toolbar_bg: Color32::from_rgb(215, 213, 208),
            selection_color: Color32::from_rgb(180, 200, 230),
            cursor_color: Color32::from_rgb(30, 30, 30),
        }
    }

    pub fn dracula() -> Self {
        Self {
            name: "Dracula".to_string(),
            bg_color: Color32::from_rgb(40, 42, 54),
            paper_color: Color32::WHITE,
            paper_opacity: 0.92,
            text_color: Color32::from_rgb(55, 57, 75),
            toolbar_bg: Color32::from_rgb(33, 34, 44),
            selection_color: Color32::from_rgb(139, 92, 246),
            cursor_color: Color32::from_rgb(80, 250, 123),
        }
    }

    /// All available theme presets
    pub fn presets() -> Vec<Theme> {
        vec![
            Self::night_owl(),
            Self::solarized_dark(),
            Self::solarized_light(),
            Self::typewriter(),
            Self::forest(),
            Self::midnight_blue(),
            Self::paper_white(),
            Self::dracula(),
        ]
    }

    pub fn by_name(name: &str) -> Option<Theme> {
        Self::presets().into_iter().find(|t| t.name == name)
    }

    /// Returns a readable text color for toolbar/dialog backgrounds —
    /// light text on dark toolbars, dark text on light toolbars.
    pub fn ui_text_color(&self) -> Color32 {
        let l = 0.299 * self.toolbar_bg.r() as f32
            + 0.587 * self.toolbar_bg.g() as f32
            + 0.114 * self.toolbar_bg.b() as f32;
        if l > 128.0 {
            Color32::from_rgb(40, 40, 40)
        } else {
            Color32::from_rgb(210, 210, 210)
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::night_owl()
    }
}
