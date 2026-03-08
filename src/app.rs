// app.rs - Main application state and UI rendering

use crate::background::Background;
use crate::document::DocumentManager;
use crate::settings::Settings;
use crate::theme::Theme;
use crate::toolbar::Toolbar;
use crate::word_count::WordCountBar;

use eframe::egui::{self, Color32, FontId, Key, Rect, RichText, TextEdit, Vec2};

#[allow(dead_code)]
pub struct RustWriterApp {
    // Core state
    doc_manager: DocumentManager,
    settings: Settings,
    background: Background,

    // UI state
    toolbar: Toolbar,
    word_count_bar: WordCountBar,
    is_fullscreen: bool,
    show_toolbar: bool,
    show_statusbar: bool,
    toolbar_visible_timer: f32,

    // Dialogs
    show_settings_dialog: bool,
    show_about_dialog: bool,
    show_background_picker: bool,
    show_theme_editor: bool,
    show_shortcuts_dialog: bool,

    // Theme
    current_theme: Theme,

    // Writing area state
    scroll_offset: f32,
    typewriter_mode: bool, // Scroll to keep cursor centered

    // Auto-save
    auto_save_timer: f32,
    auto_save_interval: f32, // seconds

    // Daily goal
    daily_goal_words: usize,
    daily_goal_enabled: bool,
    session_words_typed: usize,

    // Sound effects
    typing_sounds_enabled: bool,

    // Status
    status_message: Option<(String, f32)>, // message + display timer
}

impl RustWriterApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let settings = Settings::load();
        let current_theme = settings.current_theme.clone();
        let doc_manager = DocumentManager::new();

        // Apply theme to egui context
        apply_theme_to_ctx(&cc.egui_ctx, &current_theme);

        Self {
            doc_manager,
            settings: settings.clone(),
            background: Background::new(),
            toolbar: Toolbar::new(),
            word_count_bar: WordCountBar::new(),
            is_fullscreen: false,
            show_toolbar: true,
            show_statusbar: true,
            toolbar_visible_timer: 0.0,
            show_settings_dialog: false,
            show_about_dialog: false,
            show_background_picker: false,
            show_theme_editor: false,
            show_shortcuts_dialog: false,
            current_theme,
            scroll_offset: 0.0,
            typewriter_mode: settings.typewriter_mode,
            auto_save_timer: 0.0,
            auto_save_interval: settings.auto_save_interval,
            daily_goal_words: settings.daily_goal_words,
            daily_goal_enabled: settings.daily_goal_enabled,
            session_words_typed: 0,
            typing_sounds_enabled: settings.typing_sounds,
            status_message: None,
        }
    }

    fn toggle_fullscreen(&mut self, ctx: &egui::Context) {
        self.is_fullscreen = !self.is_fullscreen;
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.is_fullscreen));
    }

    fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        let input = ctx.input(|i| i.clone());

        // Ctrl+N - New document
        if input.key_pressed(Key::N) && input.modifiers.ctrl {
            self.new_document();
        }
        // Ctrl+O - Open
        if input.key_pressed(Key::O) && input.modifiers.ctrl {
            self.open_document();
        }
        // Ctrl+S - Save
        if input.key_pressed(Key::S) && input.modifiers.ctrl && !input.modifiers.shift {
            self.save_current_document();
        }
        // Ctrl+Shift+S - Save As
        if input.key_pressed(Key::S) && input.modifiers.ctrl && input.modifiers.shift {
            self.save_as_current_document();
        }
        // F11 - Toggle fullscreen
        if input.key_pressed(Key::F11) {
            self.toggle_fullscreen(ctx);
        }
        // F5 - Typewriter mode
        if input.key_pressed(Key::F5) {
            self.typewriter_mode = !self.typewriter_mode;
        }
        // Escape - Exit fullscreen
        if input.key_pressed(Key::Escape) && self.is_fullscreen {
            self.toggle_fullscreen(ctx);
        }
        // Ctrl+, - Settings
        if input.key_pressed(Key::Comma) && input.modifiers.ctrl {
            self.show_settings_dialog = true;
        }
        // Ctrl+Tab - Next document
        if input.key_pressed(Key::Tab) && input.modifiers.ctrl {
            self.doc_manager.next_document();
        }
        // Ctrl+Shift+Tab - Prev document
        if input.key_pressed(Key::Tab) && input.modifiers.ctrl && input.modifiers.shift {
            self.doc_manager.prev_document();
        }
    }

    fn new_document(&mut self) {
        self.doc_manager.new_document();
        self.show_status("New document created");
    }

    fn open_document(&mut self) {
        let file = rfd::FileDialog::new()
            .add_filter("Text Files", &["txt", "md", "rst", "text"])
            .add_filter("All Files", &["*"])
            .pick_file();

        if let Some(path) = file {
            match self.doc_manager.open_document(&path) {
                Ok(_) => self.show_status(&format!(
                    "Opened: {}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                )),
                Err(e) => self.show_status(&format!("Error opening file: {}", e)),
            }
        }
    }

    fn save_current_document(&mut self) {
        if let Some(doc) = self.doc_manager.current_document_mut() {
            if doc.path.is_some() {
                match doc.save() {
                    Ok(_) => self.show_status("Saved"),
                    Err(e) => self.show_status(&format!("Save error: {}", e)),
                }
            } else {
                self.save_as_current_document();
            }
        }
    }

    fn save_as_current_document(&mut self) {
        let file = rfd::FileDialog::new()
            .add_filter("Text Files", &["txt", "md"])
            .add_filter("All Files", &["*"])
            .set_file_name("untitled.txt")
            .save_file();

        if let Some(path) = file {
            if let Some(doc) = self.doc_manager.current_document_mut() {
                doc.path = Some(path.clone());
                match doc.save() {
                    Ok(_) => self.show_status(&format!(
                        "Saved as: {}",
                        path.file_name().unwrap_or_default().to_string_lossy()
                    )),
                    Err(e) => self.show_status(&format!("Save error: {}", e)),
                }
            }
        }
    }

    fn show_status(&mut self, msg: &str) {
        self.status_message = Some((msg.to_string(), 3.0));
    }

    fn tick_timers(&mut self, dt: f32) {
        // Auto-save timer
        self.auto_save_timer += dt;
        if self.auto_save_timer >= self.auto_save_interval {
            self.auto_save_timer = 0.0;
            if let Some(doc) = self.doc_manager.current_document_mut() {
                if doc.is_modified() && doc.path.is_some() {
                    let _ = doc.save();
                }
            }
        }

        // Status message timer
        if let Some((_, ref mut timer)) = self.status_message {
            *timer -= dt;
            if *timer <= 0.0 {
                self.status_message = None;
            }
        }

        // Toolbar auto-hide timer (in fullscreen)
        if self.is_fullscreen && self.show_toolbar {
            self.toolbar_visible_timer += dt;
            if self.toolbar_visible_timer > 3.0 {
                self.show_toolbar = false;
                self.toolbar_visible_timer = 0.0;
            }
        }
    }

    // ─── UI rendering ──────────────────────────────────────────────────────

    fn render_toolbar(&mut self, ctx: &egui::Context) {
        if !self.show_toolbar {
            return;
        }

        egui::TopBottomPanel::top("toolbar")
            .frame(egui::Frame {
                fill: self.current_theme.toolbar_bg,
                inner_margin: egui::Margin::symmetric(8.0, 4.0),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // File operations
                    if ui.button("📄 New").clicked() {
                        self.new_document();
                    }
                    if ui.button("📂 Open").clicked() {
                        self.open_document();
                    }
                    if ui.button("💾 Save").clicked() {
                        self.save_current_document();
                    }

                    ui.separator();

                    // View toggles
                    let fs_label = if self.is_fullscreen {
                        "⊡ Exit FS"
                    } else {
                        "⛶ Fullscreen"
                    };
                    if ui.button(fs_label).clicked() {
                        self.toggle_fullscreen(ctx);
                    }

                    let tw_color = if self.typewriter_mode {
                        Color32::YELLOW
                    } else {
                        Color32::GRAY
                    };
                    if ui.colored_label(tw_color, "⌨ Typewriter").clicked() {
                        self.typewriter_mode = !self.typewriter_mode;
                    }

                    ui.separator();

                    // Background picker
                    if ui.button("🖼 Background").clicked() {
                        self.show_background_picker = true;
                    }

                    // Theme
                    if ui.button("🎨 Theme").clicked() {
                        self.show_theme_editor = true;
                    }

                    ui.separator();

                    if ui.button("⚙ Settings").clicked() {
                        self.show_settings_dialog = true;
                    }

                    if ui.button("? About").clicked() {
                        self.show_about_dialog = true;
                    }
                });
            });
    }

    fn render_status_bar(&self, ctx: &egui::Context) {
        if !self.show_statusbar {
            return;
        }

        let doc = self.doc_manager.current_document();

        egui::TopBottomPanel::bottom("statusbar")
            .frame(egui::Frame {
                fill: self.current_theme.toolbar_bg,
                inner_margin: egui::Margin::symmetric(8.0, 3.0),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Document title
                    let title = doc
                        .map(|d| d.title())
                        .unwrap_or_else(|| "No document".to_string());
                    let modified = doc.map(|d| d.is_modified()).unwrap_or(false);
                    let display = if modified {
                        format!("✏ {} *", title)
                    } else {
                        format!("📝 {}", title)
                    };
                    ui.label(
                        RichText::new(display)
                            .color(self.current_theme.text_color)
                            .small(),
                    );

                    ui.separator();

                    // Word count
                    if let Some(doc) = doc {
                        let wc = doc.word_count();
                        ui.label(
                            RichText::new(format!("Words: {}", wc))
                                .color(self.current_theme.text_color)
                                .small(),
                        );

                        ui.separator();

                        // Character count
                        let cc = doc.char_count();
                        ui.label(
                            RichText::new(format!("Chars: {}", cc))
                                .color(self.current_theme.text_color)
                                .small(),
                        );
                    }

                    // Daily goal progress
                    if self.daily_goal_enabled && self.daily_goal_words > 0 {
                        ui.separator();
                        let progress = (self.session_words_typed as f32
                            / self.daily_goal_words as f32)
                            .min(1.0);
                        let bar_width = 100.0;
                        let (rect, _) = ui
                            .allocate_exact_size(Vec2::new(bar_width, 14.0), egui::Sense::hover());
                        let painter = ui.painter();
                        painter.rect_filled(rect, 3.0, Color32::from_gray(60));
                        let filled =
                            Rect::from_min_size(rect.min, Vec2::new(bar_width * progress, 14.0));
                        let bar_color = if progress >= 1.0 {
                            Color32::GREEN
                        } else {
                            Color32::from_rgb(70, 130, 180)
                        };
                        painter.rect_filled(filled, 3.0, bar_color);
                        ui.label(
                            RichText::new(format!(
                                "{}/{}",
                                self.session_words_typed, self.daily_goal_words
                            ))
                            .color(self.current_theme.text_color)
                            .small(),
                        );
                    }

                    // Status message (right-aligned)
                    if let Some((msg, _)) = &self.status_message {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new(msg).color(Color32::GREEN).small());
                        });
                    }
                });
            });
    }

    fn render_document_tabs(&mut self, ctx: &egui::Context) {
        if self.doc_manager.document_count() <= 1 {
            return;
        }

        egui::TopBottomPanel::top("doc_tabs")
            .frame(egui::Frame {
                fill: self.current_theme.toolbar_bg.gamma_multiply(0.8),
                inner_margin: egui::Margin::symmetric(4.0, 2.0),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let count = self.doc_manager.document_count();
                    let current = self.doc_manager.current_index();
                    for i in 0..count {
                        let title = self.doc_manager.document_title(i);
                        let is_active = i == current;
                        let color = if is_active {
                            Color32::WHITE
                        } else {
                            Color32::GRAY
                        };
                        if ui
                            .selectable_label(is_active, RichText::new(&title).color(color))
                            .clicked()
                        {
                            self.doc_manager.set_current(i);
                        }
                        // Close button
                        if ui.small_button("x").clicked() {
                            self.doc_manager.close_document(i);
                            break;
                        }
                    }
                    if ui.button("+").clicked() {
                        self.new_document();
                    }
                });
            });
    }

    fn render_writing_area(&mut self, ctx: &egui::Context) {
        let text_width = self.settings.text_column_width;
        let font_size = self.settings.font_size;
        let font_family = egui::FontFamily::Proportional;
        let theme = self.current_theme.clone();

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                // 繪製背景
                self.background.paint(ui.painter(), ui.max_rect());

                let available_width = ui.available_width();
                let left_pad = ((available_width - text_width) / 2.0).max(20.0);

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.add_space(left_pad);
                            ui.vertical(|ui| {
                                ui.set_max_width(text_width);
                                ui.add_space(30.0);

                                // 繪製紙張背景
                                let paper_rect = ui.available_rect_before_wrap();
                                ui.painter().rect_filled(
                                    paper_rect.expand2(Vec2::new(20.0, 0.0)),
                                    4.0,
                                    theme.paper_color,
                                );

                                let text = self.doc_manager.current_text_mut();

                                let text_edit = TextEdit::multiline(text)
                                    .font(FontId::new(font_size, font_family))
                                    .text_color(theme.text_color)
                                    .frame(false)
                                    .desired_width(text_width)
                                    .desired_rows(40)
                                    .margin(egui::Margin::symmetric(20.0, 20.0))
                                    .lock_focus(true);

                                let output = text_edit.show(ui);
                                let response = output.response;

                                // Update IME cursor area so Chinese/Japanese/Korean input
                                // methods know where to display the composition window.
                                if response.has_focus() {
                                    if let Some(state) = TextEdit::load_state(ctx, response.id) {
                                        if let Some(cursor_range) = state.cursor.range(&output.galley) {
                                            let cursor_rect = output.galley.pos_from_cursor(&cursor_range.primary);
                                            let screen_pos = response.rect.min + cursor_rect.min.to_vec2();
                                            ctx.send_viewport_cmd(egui::ViewportCommand::IMERect(
                                                egui::Rect::from_min_size(
                                                    screen_pos,
                                                    egui::vec2(1.0, font_size),
                                                ),
                                            ));
                                        }
                                    }
                                }

                                if response.changed() {
                                    self.doc_manager.mark_modified();
                                }

                                ui.add_space(200.0);
                            });
                        });
                    });
            });
    }

    // ─── Dialogs ─────────────────────────────────────────────────────────────

    fn render_background_picker(&mut self, ctx: &egui::Context) {
        if !self.show_background_picker {
            return;
        }

        let mut open = true;
        egui::Window::new("🖼 Choose Background")
            .open(&mut open)
            .resizable(true)
            .default_size([500.0, 400.0])
            .show(ctx, |ui| {
                ui.heading("Background Type");
                ui.separator();

                ui.horizontal(|ui| {
                    if ui
                        .selectable_label(
                            matches!(
                                self.background.kind,
                                super::background::BackgroundKind::SolidColor(_)
                            ),
                            "Solid Color",
                        )
                        .clicked()
                    {
                        self.background.set_solid(Color32::from_rgb(30, 35, 45));
                    }

                    if ui
                        .selectable_label(
                            matches!(
                                self.background.kind,
                                super::background::BackgroundKind::Gradient(_, _)
                            ),
                            "Gradient",
                        )
                        .clicked()
                    {
                        self.background.set_gradient(
                            Color32::from_rgb(20, 20, 40),
                            Color32::from_rgb(40, 40, 80),
                        );
                    }

                    if ui
                        .selectable_label(
                            matches!(
                                self.background.kind,
                                super::background::BackgroundKind::Image(_)
                            ),
                            "Image File",
                        )
                        .clicked()
                    {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Images", &["png", "jpg", "jpeg", "bmp", "gif"])
                            .pick_file()
                        {
                            self.background.load_image(&path, ctx);
                        }
                    }
                });

                ui.separator();

                match &mut self.background.kind {
                    super::background::BackgroundKind::SolidColor(color) => {
                        ui.label("Pick a color:");
                        ui.color_edit_button_srgba(color);
                    }
                    super::background::BackgroundKind::Gradient(c1, c2) => {
                        ui.horizontal(|ui| {
                            ui.label("Start color:");
                            ui.color_edit_button_srgba(c1);
                            ui.label("End color:");
                            ui.color_edit_button_srgba(c2);
                        });
                    }
                    super::background::BackgroundKind::Image(path) => {
                        ui.label(format!("Image: {}", path.display()));
                        if ui.button("Change Image...").clicked() {
                            if let Some(new_path) = rfd::FileDialog::new()
                                .add_filter("Images", &["png", "jpg", "jpeg", "bmp"])
                                .pick_file()
                            {
                                self.background.load_image(&new_path, ctx);
                            }
                        }
                    }
                }

                ui.separator();
                ui.label("Overlay opacity (paper):");
                ui.add(egui::Slider::new(
                    &mut self.background.overlay_opacity,
                    0.0..=1.0,
                ));

                ui.separator();
                if ui.button("Close").clicked() {
                    self.show_background_picker = false;
                }
            });

        if !open {
            self.show_background_picker = false;
        }
    }

    fn render_settings_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_settings_dialog {
            return;
        }

        let mut open = true;
        egui::Window::new("⚙ Settings")
            .open(&mut open)
            .resizable(true)
            .default_size([450.0, 500.0])
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Font
                    ui.heading("Font");
                    ui.horizontal(|ui| {
                        ui.label("Size:");
                        ui.add(
                            egui::Slider::new(&mut self.settings.font_size, 10.0..=36.0)
                                .suffix("pt"),
                        );
                    });

                    ui.separator();

                    // Text column
                    ui.heading("Writing Area");
                    ui.horizontal(|ui| {
                        ui.label("Column width:");
                        ui.add(
                            egui::Slider::new(&mut self.settings.text_column_width, 400.0..=1200.0)
                                .suffix("px"),
                        );
                    });

                    ui.separator();

                    // Auto-save
                    ui.heading("Auto-Save");
                    ui.horizontal(|ui| {
                        ui.label("Interval:");
                        ui.add(
                            egui::Slider::new(&mut self.settings.auto_save_interval, 30.0..=600.0)
                                .suffix("s"),
                        );
                    });
                    self.auto_save_interval = self.settings.auto_save_interval;

                    ui.separator();

                    // Typewriter mode
                    ui.heading("Modes");
                    ui.checkbox(
                        &mut self.settings.typewriter_mode,
                        "Typewriter mode (keep cursor centered)",
                    );
                    self.typewriter_mode = self.settings.typewriter_mode;
                    ui.checkbox(&mut self.settings.typing_sounds, "Typing sound effects");
                    self.typing_sounds_enabled = self.settings.typing_sounds;

                    ui.separator();

                    // Daily goal
                    ui.heading("Daily Writing Goal");
                    ui.checkbox(
                        &mut self.settings.daily_goal_enabled,
                        "Enable daily word goal",
                    );
                    self.daily_goal_enabled = self.settings.daily_goal_enabled;
                    if self.settings.daily_goal_enabled {
                        ui.horizontal(|ui| {
                            ui.label("Words per day:");
                            ui.add(
                                egui::DragValue::new(&mut self.settings.daily_goal_words)
                                    .range(100..=10000),
                            );
                        });
                        self.daily_goal_words = self.settings.daily_goal_words;
                    }

                    ui.separator();

                    if ui.button("Save Settings").clicked() {
                        let _ = self.settings.save();
                        self.show_status("Settings saved");
                        self.show_settings_dialog = false;
                    }
                });
            });

        if !open {
            self.show_settings_dialog = false;
        }
    }

    fn render_theme_editor(&mut self, ctx: &egui::Context) {
        if !self.show_theme_editor {
            return;
        }

        let mut open = true;
        egui::Window::new("🎨 Theme Editor")
            .open(&mut open)
            .resizable(false)
            .default_size([350.0, 380.0])
            .show(ctx, |ui| {
                ui.heading("Built-in Themes");
                ui.horizontal_wrapped(|ui| {
                    for preset in Theme::presets() {
                        if ui.button(&preset.name).clicked() {
                            self.current_theme = preset;
                            apply_theme_to_ctx(ctx, &self.current_theme);
                        }
                    }
                });

                ui.separator();
                ui.heading("Custom Colors");

                ui.horizontal(|ui| {
                    ui.label("Background:  ");
                    ui.color_edit_button_srgba(&mut self.current_theme.bg_color);
                });
                ui.horizontal(|ui| {
                    ui.label("Paper color: ");
                    ui.color_edit_button_srgba(&mut self.current_theme.paper_color);
                });
                ui.horizontal(|ui| {
                    ui.label("Text color:  ");
                    ui.color_edit_button_srgba(&mut self.current_theme.text_color);
                });
                ui.horizontal(|ui| {
                    ui.label("Toolbar:     ");
                    ui.color_edit_button_srgba(&mut self.current_theme.toolbar_bg);
                });

                ui.separator();
                if ui.button("Apply Theme").clicked() {
                    apply_theme_to_ctx(ctx, &self.current_theme);
                    self.show_status("Theme applied");
                }
            });

        if !open {
            self.show_theme_editor = false;
        }
    }

    fn render_about_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_about_dialog {
            return;
        }

        let mut open = true;
        egui::Window::new("About Rust Writer")
            .open(&mut open)
            .resizable(false)
            .default_size([380.0, 280.0])
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    ui.heading(RichText::new("✍ Rust Writer").size(24.0));
                    ui.label(RichText::new("Version 0.1.0").color(Color32::GRAY));
                    ui.add_space(10.0);
                    ui.label("A Focus Writer like fullscreen distraction-free writing app,");
                    ui.label("inspired by FocusWriter, rewritten in Rust.");
                    ui.add_space(10.0);
                    ui.separator();
                    ui.label(RichText::new("Built with:").strong());
                    ui.label("• Rust 🦀");
                    ui.label("• egui / eframe (immediate-mode GUI)");
                    ui.label("• image-rs (background images)");
                    ui.label("• rfd (native file dialogs)");
                    ui.add_space(10.0);
                    ui.label(
                        RichText::new("Original FocusWriter by Graeme Gott")
                            .color(Color32::GRAY)
                            .small(),
                    );
                    ui.label(
                        RichText::new("https://github.com/gottcode/focuswriter")
                            .color(Color32::LIGHT_BLUE)
                            .small(),
                    );
                    ui.add_space(5.0);
                    if ui.button("Close").clicked() {
                        self.show_about_dialog = false;
                    }
                });
            });

        if !open {
            self.show_about_dialog = false;
        }
    }

    fn handle_mouse_for_toolbar(&mut self, ctx: &egui::Context) {
        if !self.is_fullscreen {
            return;
        }

        // Show toolbar when mouse near top
        let mouse_pos = ctx.input(|i| i.pointer.hover_pos());
        if let Some(pos) = mouse_pos {
            if pos.y < 60.0 {
                self.show_toolbar = true;
                self.toolbar_visible_timer = 0.0;
            }
        }
    }
}

impl eframe::App for RustWriterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let dt = ctx.input(|i| i.unstable_dt).min(0.1);

        // Enable IME for CJK input methods (GCIN, ibus, fcitx).
        // egui-winit calls window.set_ime_allowed() based on this command.
        // Must be sent every frame while a TextEdit is potentially focused.
        ctx.send_viewport_cmd(egui::ViewportCommand::IMEAllowed(true));

        // Timers and background logic
        self.tick_timers(dt);
        self.handle_keyboard_shortcuts(ctx);
        self.handle_mouse_for_toolbar(ctx);

        // Render panels (order matters!)
        self.render_toolbar(ctx);
        self.render_document_tabs(ctx);
        self.render_status_bar(ctx);
        self.render_writing_area(ctx);

        // Dialogs (rendered on top)
        self.render_background_picker(ctx);
        self.render_settings_dialog(ctx);
        self.render_theme_editor(ctx);
        self.render_about_dialog(ctx);

        // Request repaint for animations
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Auto-save on exit
        for doc in self.doc_manager.all_documents_mut() {
            if doc.is_modified() && doc.path.is_some() {
                let _ = doc.save();
            }
        }
        let _ = self.settings.save();
    }
}

fn apply_theme_to_ctx(ctx: &egui::Context, theme: &Theme) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = theme.toolbar_bg;
    visuals.window_fill = theme.bg_color;
    visuals.extreme_bg_color = theme.paper_color;
    visuals.override_text_color = Some(theme.text_color);
    ctx.set_visuals(visuals);
}
