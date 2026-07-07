// FocusWriter-RS: A fullscreen distraction-free writing app in Rust
// Inspired by FocusWriter (https://github.com/gottcode/focuswriter)

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // Hide console on Windows release

mod app;
mod document;
mod search;
mod settings;
mod sounds;
mod theme;
mod toolbar;
mod updater;
mod word_count;

// library code lives in the package's lib crate rather than a submodule
use crate::app::RustWriterApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("Rust Writer")
            .with_icon(load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "Rust Writer",
        native_options,
        Box::new(|cc| {
            // Load fonts
            setup_fonts(&cc.egui_ctx);
            // Load images support
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(RustWriterApp::new(cc)))
        }),
    )
}

fn load_icon() -> egui::IconData {
    // Minimal embedded icon (32x32 RGBA)
    let icon_bytes = include_bytes!("../assets/icon.png");
    let image = image::load_from_memory(icon_bytes)
        .unwrap_or_else(|_| image::DynamicImage::new_rgba8(32, 32));
    let rgba = image.to_rgba8();
    let (w, h) = rgba.dimensions();
    egui::IconData {
        rgba: rgba.into_raw(),
        width: w,
        height: h,
    }
}

fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Add a multilingual font for writing
    fonts.font_data.insert(
        "writing_font".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/NotoSansTC-Regular.ttf")),
    );

    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "writing_font".to_owned());

    ctx.set_fonts(fonts);
}
