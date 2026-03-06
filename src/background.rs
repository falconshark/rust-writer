// background.rs - Background rendering (solid, gradient, image)

use eframe::egui::{self, Color32, Painter, Rect, Vec2, Pos2};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub enum BackgroundKind {
    SolidColor(Color32),
    Gradient(Color32, Color32),
    Image(PathBuf),
}

pub struct Background {
    pub kind: BackgroundKind,
    pub overlay_opacity: f32,
    texture: Option<egui::TextureHandle>,
}

impl Background {
    pub fn new() -> Self {
        Self {
            kind: BackgroundKind::SolidColor(Color32::from_rgb(28, 32, 42)),
            overlay_opacity: 0.85,
            texture: None,
        }
    }

    pub fn set_solid(&mut self, color: Color32) {
        self.kind = BackgroundKind::SolidColor(color);
        self.texture = None;
    }

    pub fn set_gradient(&mut self, top: Color32, bottom: Color32) {
        self.kind = BackgroundKind::Gradient(top, bottom);
        self.texture = None;
    }

    pub fn load_image(&mut self, path: &Path, ctx: &egui::Context) {
        match image::open(path) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                let (w, h) = rgba.dimensions();
                let pixels: Vec<egui::Color32> = rgba
                    .chunks_exact(4)
                    .map(|c| egui::Color32::from_rgba_premultiplied(c[0], c[1], c[2], c[3]))
                    .collect();

                let color_image = egui::ColorImage {
                    size: [w as usize, h as usize],
                    pixels,
                };

                self.texture = Some(ctx.load_texture(
                    "bg_image",
                    color_image,
                    egui::TextureOptions::LINEAR,
                ));
                self.kind = BackgroundKind::Image(path.to_path_buf());
            }
            Err(e) => {
                log::error!("Failed to load background image: {}", e);
            }
        }
    }

    pub fn paint(&self, painter: &Painter, rect: Rect) {
        match &self.kind {
            BackgroundKind::SolidColor(color) => {
                painter.rect_filled(rect, 0.0, *color);
            }

            BackgroundKind::Gradient(top_color, bottom_color) => {
                // Paint gradient using mesh
                let top = *top_color;
                let bot = *bottom_color;

                // Use a simple two-color vertical gradient via a mesh
                let mut mesh = egui::Mesh::default();
                // top-left, top-right, bottom-left, bottom-right
                mesh.colored_vertex(rect.left_top(), top);
                mesh.colored_vertex(rect.right_top(), top);
                mesh.colored_vertex(rect.left_bottom(), bot);
                mesh.colored_vertex(rect.right_bottom(), bot);
                // Two triangles
                mesh.add_triangle(0, 1, 2);
                mesh.add_triangle(1, 3, 2);
                painter.add(egui::Shape::mesh(mesh));
            }

            BackgroundKind::Image(_) => {
                if let Some(tex) = &self.texture {
                    // Fill rect, maintaining aspect ratio (cover)
                    let img_size = tex.size_vec2();
                    let rect_ar = rect.width() / rect.height();
                    let img_ar = img_size.x / img_size.y;

                    let (draw_size, draw_offset) = if rect_ar > img_ar {
                        // Rect wider: fit width, crop height
                        let h = rect.width() / img_ar;
                        let offset_y = (rect.height() - h) / 2.0;
                        (Vec2::new(rect.width(), h), Vec2::new(0.0, offset_y))
                    } else {
                        // Rect taller: fit height, crop width
                        let w = rect.height() * img_ar;
                        let offset_x = (rect.width() - w) / 2.0;
                        (Vec2::new(w, rect.height()), Vec2::new(offset_x, 0.0))
                    };

                    let draw_rect = Rect::from_min_size(rect.min + draw_offset, draw_size);
                    painter.image(tex.id(), draw_rect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
                } else {
                    // Fallback: dark background if image not loaded
                    painter.rect_filled(rect, 0.0, Color32::from_rgb(20, 20, 30));
                }
            }
        }
    }
}
