// theme.rs - Color themes for the writing environment

use eframe::egui::Color32;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub bg_color: Color32,
    pub paper_color: Color32,
    pub text_color: Color32,
    pub toolbar_bg: Color32,
    pub selection_color: Color32,
    pub cursor_color: Color32,
}

impl Theme {
    pub fn night_owl() -> Self {
        Self {
            name: "Night Owl".to_string(),
            bg_color: Color32::from_rgb(17, 21, 28),
            paper_color: Color32::from_rgba_premultiplied(30, 35, 48, 220),
            text_color: Color32::from_rgb(214, 222, 235),
            toolbar_bg: Color32::from_rgb(15, 18, 25),
            selection_color: Color32::from_rgb(84, 109, 160),
            cursor_color: Color32::WHITE,
        }
    }

    pub fn solarized_dark() -> Self {
        Self {
            name: "Solarized Dark".to_string(),
            bg_color: Color32::from_rgb(0, 43, 54),
            paper_color: Color32::from_rgba_premultiplied(7, 54, 66, 230),
            text_color: Color32::from_rgb(131, 148, 150),
            toolbar_bg: Color32::from_rgb(0, 33, 44),
            selection_color: Color32::from_rgb(42, 161, 152),
            cursor_color: Color32::from_rgb(181, 137, 0),
        }
    }

    pub fn solarized_light() -> Self {
        Self {
            name: "Solarized Light".to_string(),
            bg_color: Color32::from_rgb(253, 246, 227),
            paper_color: Color32::from_rgba_premultiplied(238, 232, 213, 240),
            text_color: Color32::from_rgb(101, 123, 131),
            toolbar_bg: Color32::from_rgb(238, 232, 213),
            selection_color: Color32::from_rgb(147, 161, 161),
            cursor_color: Color32::from_rgb(88, 110, 117),
        }
    }

    pub fn typewriter() -> Self {
        Self {
            name: "Typewriter".to_string(),
            bg_color: Color32::from_rgb(210, 195, 166),
            paper_color: Color32::from_rgba_premultiplied(225, 213, 185, 245),
            text_color: Color32::from_rgb(42, 30, 15),
            toolbar_bg: Color32::from_rgb(190, 175, 148),
            selection_color: Color32::from_rgb(160, 130, 80),
            cursor_color: Color32::from_rgb(42, 30, 15),
        }
    }

    pub fn forest() -> Self {
        Self {
            name: "Forest".to_string(),
            bg_color: Color32::from_rgb(20, 38, 25),
            paper_color: Color32::from_rgba_premultiplied(28, 50, 33, 225),
            text_color: Color32::from_rgb(168, 210, 148),
            toolbar_bg: Color32::from_rgb(15, 28, 18),
            selection_color: Color32::from_rgb(60, 130, 70),
            cursor_color: Color32::from_rgb(120, 200, 100),
        }
    }

    pub fn midnight_blue() -> Self {
        Self {
            name: "Midnight Blue".to_string(),
            bg_color: Color32::from_rgb(10, 12, 40),
            paper_color: Color32::from_rgba_premultiplied(18, 22, 60, 220),
            text_color: Color32::from_rgb(180, 200, 240),
            toolbar_bg: Color32::from_rgb(8, 10, 30),
            selection_color: Color32::from_rgb(60, 90, 160),
            cursor_color: Color32::from_rgb(120, 160, 240),
        }
    }

    pub fn paper_white() -> Self {
        Self {
            name: "Paper White".to_string(),
            bg_color: Color32::from_rgb(230, 228, 222),
            paper_color: Color32::from_rgba_premultiplied(255, 253, 245, 255),
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
            paper_color: Color32::from_rgba_premultiplied(50, 52, 70, 230),
            text_color: Color32::from_rgb(248, 248, 242),
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
}

impl Default for Theme {
    fn default() -> Self {
        Self::night_owl()
    }
}
