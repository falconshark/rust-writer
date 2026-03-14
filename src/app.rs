// app.rs - Main application state and UI rendering

use crate::document::DocumentManager;
use crate::settings::{BgImageMode, Settings};
use crate::sounds::AudioPlayer;
use crate::theme::Theme;
use crate::toolbar::Toolbar;
use crate::word_count::WordCountBar;

use eframe::egui::{self, Color32, FontId, Key, Rect, RichText, TextEdit, Vec2};

#[allow(dead_code)]
pub struct RustWriterApp {
    // Core state
    doc_manager: DocumentManager,
    settings: Settings,

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
    show_theme_editor: bool,
    show_shortcuts_dialog: bool,

    // Theme — owns all visual configuration (background + paper + text)
    current_theme: Theme,

    // Writing area state
    scroll_offset: f32,
    target_scroll_y: Option<f32>, // set to request a programmatic scroll
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
    audio: Option<AudioPlayer>,

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
            toolbar: Toolbar::new(),
            word_count_bar: WordCountBar::new(),
            is_fullscreen: false,
            show_toolbar: true,
            show_statusbar: true,
            toolbar_visible_timer: 0.0,
            show_settings_dialog: false,
            show_about_dialog: false,
            show_theme_editor: false,
            show_shortcuts_dialog: false,
            current_theme,
            scroll_offset: 0.0,
            target_scroll_y: None,
            typewriter_mode: settings.typewriter_mode,
            auto_save_timer: 0.0,
            auto_save_interval: settings.auto_save_interval,
            daily_goal_words: settings.daily_goal_words,
            daily_goal_enabled: settings.daily_goal_enabled,
            session_words_typed: 0,
            typing_sounds_enabled: settings.typing_sounds,
            audio: AudioPlayer::new(),
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
        // Ctrl+Home - scroll to top
        if input.key_pressed(Key::Home) && input.modifiers.ctrl {
            self.target_scroll_y = Some(0.0);
        }
        // Ctrl+End - scroll to bottom
        if input.key_pressed(Key::End) && input.modifiers.ctrl {
            self.target_scroll_y = Some(f32::MAX);
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

                    // Theme (includes background + paper + text settings)
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
                    let ui_color = self.current_theme.ui_text_color();
                    ui.label(
                        RichText::new(display)
                            .color(ui_color)
                            .small(),
                    );

                    ui.separator();

                    // Word count
                    if let Some(doc) = doc {
                        let wc = doc.word_count();
                        ui.label(
                            RichText::new(format!("Words: {}", wc))
                                .color(ui_color)
                                .small(),
                        );

                        ui.separator();

                        // Character count
                        let cc = doc.char_count();
                        ui.label(
                            RichText::new(format!("Chars: {}", cc))
                                .color(ui_color)
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
                            .color(ui_color)
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
                    let ui_color = self.current_theme.ui_text_color();
                    for i in 0..count {
                        let title = self.doc_manager.document_title(i);
                        let is_active = i == current;
                        let color = if is_active {
                            ui_color
                        } else {
                            Color32::from_rgba_unmultiplied(
                                ui_color.r(), ui_color.g(), ui_color.b(), 140,
                            )
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

        // Build paper color with opacity
        let paper_color = Color32::from_rgba_unmultiplied(
            theme.paper_color.r(),
            theme.paper_color.g(),
            theme.paper_color.b(),
            (theme.paper_opacity * 255.0) as u8,
        );

        // Layer 1: background
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(theme.bg_color))
            .show(ctx, |ui| {
                let available_height = ui.available_height();
                // Paper outer width = text_width + left inner margin + right inner margin (20px each)
                let paper_outer_width = text_width + 2.0 * 20.0;

                // Gap between paper edges and background
                let bg_pad = 16.0;
                // Inner margin of the paper frame (padding inside paper)
                let v_margin = 20.0;
                // Inner height available to the scroll area inside the paper
                let inner_height = (available_height - 2.0 * bg_pad - 2.0 * v_margin).max(100.0);

                // Compute the paper rect centered in the panel with top/bottom gap
                let panel_rect = ui.available_rect_before_wrap();

                // Draw background image (if set) on top of bg_color, behind the paper
                if let Some(ref path) = self.settings.bg_image_path.clone() {
                    paint_bg_image(ui, ctx, path, &self.settings.bg_image_mode, panel_rect);
                }
                let center_x = panel_rect.center().x;
                let paper_x = (center_x - paper_outer_width / 2.0).max(panel_rect.left());
                let paper_rect = egui::Rect::from_min_size(
                    egui::pos2(paper_x, panel_rect.top() + bg_pad),
                    egui::vec2(
                        paper_outer_width.min(panel_rect.width()),
                        (panel_rect.height() - 2.0 * bg_pad).max(100.0),
                    ),
                );

                ui.allocate_ui_at_rect(paper_rect, |ui| {
                    // Layer 2: paper — fixed to viewport height, scroll happens inside
                    let paper_frame = egui::Frame::none()
                        .fill(paper_color)
                        .rounding(4.0)
                        .inner_margin(egui::Margin::symmetric(20.0, v_margin));

                    paper_frame.show(ui, |ui| {
                        ui.set_min_height(inner_height);
                        ui.set_max_height(inner_height);
                        ui.set_width(text_width);

                        let mut scroll_area = egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .scroll_bar_visibility(
                                egui::scroll_area::ScrollBarVisibility::AlwaysVisible,
                            );
                        if let Some(y) = self.target_scroll_y.take() {
                            scroll_area = scroll_area.scroll_offset(egui::Vec2::new(0.0, y));
                        }
                        scroll_area.show(ui, |ui| {
                            let text = self.doc_manager.current_text_mut();

                            let text_edit = TextEdit::multiline(text)
                                .font(FontId::new(font_size, font_family))
                                .text_color(theme.text_color)
                                .frame(false)
                                .desired_width(text_width)
                                .desired_rows(40)
                                .lock_focus(true);

                            let output = text_edit.show(ui);
                            let response = output.response;

                            // Update IME cursor area for CJK input methods.
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
                                if self.typing_sounds_enabled {
                                    if let Some(ref audio) = self.audio {
                                        let enter_pressed = ctx.input(|i| i.key_pressed(Key::Enter));
                                        if enter_pressed {
                                            audio.play_return();
                                        } else {
                                            audio.play_click();
                                        }
                                    }
                                }
                            }

                            ui.add_space(200.0);
                        });
                    });
                });
            });
    }

    // ─── Dialogs ─────────────────────────────────────────────────────────────

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
                // ── Preset themes ──────────────────────────────────────────
                ui.heading("Presets");
                ui.horizontal_wrapped(|ui| {
                    for preset in Theme::presets() {
                        if ui.button(&preset.name).clicked() {
                            self.current_theme = preset;
                            apply_theme_to_ctx(ctx, &self.current_theme);
                        }
                    }
                });

                ui.separator();

                // ── Background ─────────────────────────────────────────────
                ui.heading("Background");
                ui.horizontal(|ui| {
                    ui.label("Color:");
                    if ui.color_edit_button_srgba(&mut self.current_theme.bg_color).changed() {
                        apply_theme_to_ctx(ctx, &self.current_theme);
                    }
                });

                ui.add_space(4.0);
                ui.label("Image:");
                ui.horizontal(|ui| {
                    if ui.button("Pick Image…").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Images", &["png", "jpg", "jpeg", "webp", "bmp", "gif"])
                            .pick_file()
                        {
                            self.settings.bg_image_path =
                                Some(path.to_string_lossy().to_string());
                            let _ = self.settings.save();
                        }
                    }
                    if self.settings.bg_image_path.is_some() {
                        if ui.button("Clear").clicked() {
                            self.settings.bg_image_path = None;
                            let _ = self.settings.save();
                        }
                    }
                });
                if let Some(ref path) = self.settings.bg_image_path.clone() {
                    let name = std::path::Path::new(path)
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy();
                    ui.label(RichText::new(name.to_string()).small());
                    ui.horizontal(|ui| {
                        ui.label("Mode:");
                        egui::ComboBox::from_id_source("bg_image_mode")
                            .selected_text(bg_mode_label(&self.settings.bg_image_mode))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.settings.bg_image_mode,
                                    BgImageMode::Zoomed,
                                    "Zoomed (cover)",
                                );
                                ui.selectable_value(
                                    &mut self.settings.bg_image_mode,
                                    BgImageMode::Scaled,
                                    "Scaled (fit)",
                                );
                                ui.selectable_value(
                                    &mut self.settings.bg_image_mode,
                                    BgImageMode::Stretched,
                                    "Stretched",
                                );
                                ui.selectable_value(
                                    &mut self.settings.bg_image_mode,
                                    BgImageMode::Centered,
                                    "Centered",
                                );
                                ui.selectable_value(
                                    &mut self.settings.bg_image_mode,
                                    BgImageMode::Tiled,
                                    "Tiled",
                                );
                            });
                    });
                    if ui.button("Save").clicked() {
                        let _ = self.settings.save();
                    }
                }

                ui.separator();

                // ── Paper layer ────────────────────────────────────────────
                ui.heading("Paper");
                ui.horizontal(|ui| {
                    ui.label("Color:");
                    ui.color_edit_button_srgba(&mut self.current_theme.paper_color);
                });
                ui.horizontal(|ui| {
                    ui.label("Opacity:");
                    ui.add(
                        egui::Slider::new(&mut self.current_theme.paper_opacity, 0.0..=1.0)
                            .custom_formatter(|v, _| format!("{:.0}%", v * 100.0)),
                    );
                });

                ui.separator();

                // ── Text ───────────────────────────────────────────────────
                ui.heading("Text");
                ui.horizontal(|ui| {
                    ui.label("Color:");
                    if ui.color_edit_button_srgba(&mut self.current_theme.text_color).changed() {
                        apply_theme_to_ctx(ctx, &self.current_theme);
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Toolbar:");
                    if ui.color_edit_button_srgba(&mut self.current_theme.toolbar_bg).changed() {
                        apply_theme_to_ctx(ctx, &self.current_theme);
                    }
                });
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

fn bg_mode_label(mode: &BgImageMode) -> &'static str {
    match mode {
        BgImageMode::Zoomed => "Zoomed (cover)",
        BgImageMode::Scaled => "Scaled (fit)",
        BgImageMode::Stretched => "Stretched",
        BgImageMode::Centered => "Centered",
        BgImageMode::Tiled => "Tiled",
    }
}

fn paint_bg_image(
    ui: &egui::Ui,
    ctx: &egui::Context,
    path: &str,
    mode: &BgImageMode,
    rect: egui::Rect,
) {
    use egui::load::SizeHint;
    let uri = format!("file://{}", path);
    // Hint the loader to decode at screen resolution rather than full resolution.
    // This dramatically speeds up first-load for large wallpapers (e.g. 4K JPEGs).
    let hint = SizeHint::Size(rect.width() as u32, rect.height() as u32);
    match ctx.try_load_texture(&uri, egui::TextureOptions::LINEAR, hint) {
        Ok(egui::load::TexturePoll::Ready { texture }) => {
            let iw = texture.size.x;
            let ih = texture.size.y;
            let pw = rect.width();
            let ph = rect.height();
            let full_uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));

            match mode {
                BgImageMode::Stretched => {
                    ui.painter().image(texture.id, rect, full_uv, Color32::WHITE);
                }
                BgImageMode::Zoomed => {
                    // Cover: scale to fill, crop excess symmetrically
                    let scale = (pw / iw).max(ph / ih);
                    let sw = iw * scale;
                    let sh = ih * scale;
                    let u0 = (sw - pw) / (2.0 * sw);
                    let v0 = (sh - ph) / (2.0 * sh);
                    let uv = egui::Rect::from_min_max(
                        egui::pos2(u0, v0),
                        egui::pos2(1.0 - u0, 1.0 - v0),
                    );
                    ui.painter().image(texture.id, rect, uv, Color32::WHITE);
                }
                BgImageMode::Scaled => {
                    // Fit: letterbox, maintain aspect ratio
                    let scale = (pw / iw).min(ph / ih);
                    let dw = iw * scale;
                    let dh = ih * scale;
                    let draw_rect =
                        egui::Rect::from_center_size(rect.center(), egui::vec2(dw, dh));
                    ui.painter().image(texture.id, draw_rect, full_uv, Color32::WHITE);
                }
                BgImageMode::Centered => {
                    let draw_rect =
                        egui::Rect::from_center_size(rect.center(), egui::vec2(iw, ih));
                    ui.painter().image(texture.id, draw_rect, full_uv, Color32::WHITE);
                }
                BgImageMode::Tiled => {
                    let mut x = rect.left();
                    while x < rect.right() {
                        let mut y = rect.top();
                        while y < rect.bottom() {
                            let cw = (rect.right() - x).min(iw);
                            let ch = (rect.bottom() - y).min(ih);
                            let tile_rect = egui::Rect::from_min_size(
                                egui::pos2(x, y),
                                egui::vec2(cw, ch),
                            );
                            let uv = egui::Rect::from_min_max(
                                egui::pos2(0.0, 0.0),
                                egui::pos2(cw / iw, ch / ih),
                            );
                            ui.painter().image(texture.id, tile_rect, uv, Color32::WHITE);
                            y += ih;
                        }
                        x += iw;
                    }
                }
            }
        }
        Ok(egui::load::TexturePoll::Pending { .. }) => {
            ctx.request_repaint();
        }
        Err(_) => {}
    }
}

fn luma(c: Color32) -> f32 {
    0.299 * c.r() as f32 + 0.587 * c.g() as f32 + 0.114 * c.b() as f32
}

fn apply_theme_to_ctx(ctx: &egui::Context, theme: &Theme) {
    // Choose light or dark base visuals based on toolbar brightness so that
    // widget hover states, separators and button text all look correct
    // on both light and dark toolbars.
    let mut visuals = if luma(theme.toolbar_bg) > 128.0 {
        egui::Visuals::light()
    } else {
        egui::Visuals::dark()
    };

    // Toolbar and panel backgrounds
    visuals.panel_fill = theme.toolbar_bg;
    // Dialogs/windows use the toolbar color so they blend with the chrome,
    // NOT the writing-area bg_color.
    visuals.window_fill = theme.toolbar_bg;

    // Keep TextEdit backgrounds transparent — our paper Frame provides the fill.
    visuals.extreme_bg_color = Color32::TRANSPARENT;

    // Selection highlight
    visuals.selection.bg_fill = theme.selection_color;

    // Do NOT set override_text_color here: that would force the paper's dark
    // text_color onto toolbar buttons and dialog labels (dark-on-dark = unreadable).
    // The TextEdit uses .text_color(theme.text_color) explicitly.

    ctx.set_visuals(visuals);
}
